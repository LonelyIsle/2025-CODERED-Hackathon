// src/scrape.rs
use anyhow::{Result, anyhow, bail};
use bytes::Bytes;
use chrono::Utc;
use dashmap::DashMap;
use reqwest::{Client, StatusCode, header};
use robots_txt::Robots;
use scraper::{Html, Selector};
use std::{time::{Duration, Instant}, sync::Arc};
use tokio::sync::Semaphore;
use url::Url;

use crate::types::Document;

#[derive(Clone)]
pub struct ScrapeClient {
    pub http: Client,
    pub user_agent: String,
    // polite throttling (global cap per domain executed by caller)
    domain_limit: Arc<Semaphore>,
    delay: Duration,
}

#[derive(Clone)]
pub struct RobotsCache {
    // host => (fetched_at_secs, robots)
    inner: Arc<DashMap<String, (u64, Robots<'static>)>>,
    ttl_secs: u64,
}

impl RobotsCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
            ttl_secs,
        }
    }
}

impl ScrapeClient {
    pub fn new(user_agent: &str, concurrent_per_domain: usize, delay: Duration) -> Result<Self> {
        let http = Client::builder()
            .user_agent(user_agent)
            .gzip(true).brotli(true).deflate(true)
            .redirect(reqwest::redirect::Policy::limited(8))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .http2_prior_knowledge() // try h2 where possible
            .pool_max_idle_per_host(8)
            .timeout(Duration::from_secs(25))
            .build()?;

        Ok(Self {
            http,
            user_agent: user_agent.to_string(),
            domain_limit: Arc::new(Semaphore::new(concurrent_per_domain)),
            delay,
        })
    }

    async fn fetch_robots(&self, url: &Url) -> Result<Option<Robots<'static>>> {
        let robots_url = url.join("/robots.txt")?;
        let res = self.http.get(robots_url).send().await?;
        if !res.status().is_success() {
            return Ok(None);
        }
        let txt = res.text().await.unwrap_or_default();
        let robots = Robots::from_str_lossy(Box::leak(txt.into_boxed_str()));
        Ok(Some(robots))
    }

    async fn allowed_by_robots(&self, robots: &RobotsCache, url: &Url) -> bool {
        let host = match url.host_str() { Some(h) => h.to_string(), None => return true };
        let now = now_secs();
        if let Some((when, r)) = robots.inner.get(&host).map(|e| *e.value()) {
            if now.saturating_sub(when) <= robots.ttl_secs {
                return r.allowed(url.as_str(), &self.user_agent).unwrap_or(true);
            }
        }
        if let Ok(Some(r)) = self.fetch_robots(url).await {
            robots.inner.insert(host, (now, r));
            return robots.inner.get(url.host_str().unwrap()).map(|e| e.value().1.allowed(url.as_str(), &self.user_agent).unwrap_or(true)).unwrap_or(true);
        }
        true // missing robots.txt -> allow
    }

    pub async fn fetch_bytes_conditional(
        &self,
        url: &Url,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<(StatusCode, String, Bytes, Option<String>, Option<String>)> {
        let _permit = self.domain_limit.acquire().await?;
        tokio::time::sleep(self.delay).await;

        let mut req = self.http.get(url.clone());
        if let Some(e) = etag {
            req = req.header(header::IF_NONE_MATCH, e);
        }
        if let Some(lm) = last_modified {
            req = req.header(header::IF_MODIFIED_SINCE, lm);
        }

        let res = req.send().await?;
        let status = res.status();
        let ct = res
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let etag_new = res.headers().get(header::ETAG).and_then(|v| v.to_str().ok()).map(|s| s.to_string());
        let lm_new   = res.headers().get(header::LAST_MODIFIED).and_then(|v| v.to_str().ok()).map(|s| s.to_string());

        let body = if status == StatusCode::NOT_MODIFIED {
            Bytes::new()
        } else {
            res.bytes().await?
        };

        Ok((status, ct, body, etag_new, lm_new))
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
}

/// Strip scripts/styles and return (title, description, text)
fn html_to_text(html: &str) -> (Option<String>, Option<String>, String) {
    let doc = Html::parse_document(html);

    let title = Selector::parse("title").ok()
        .and_then(|s| doc.select(&s).next())
        .map(|n| n.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty());

    let description = Selector::parse("meta[name=description],meta[property='og:description']").ok()
        .and_then(|s| doc.select(&s).next())
        .and_then(|m| m.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // body text
    let body_sel = Selector::parse("body").unwrap();
    let body_text = doc.select(&body_sel)
        .flat_map(|b| b.text())
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    (title, description, body_text)
}

pub async fn scrape_one(sc: &ScrapeClient, robots: &RobotsCache, url_raw: &str) -> Result<Document> {
    let url = Url::parse(url_raw).map_err(|e| anyhow!("bad url: {e}"))?;
    if !(url.scheme() == "https" || url.scheme() == "http") {
        bail!("unsupported scheme");
    }
    if !sc.allowed_by_robots(robots, &url).await {
        bail!("blocked by robots.txt");
    }

    // Basic conditional GETs: look up any existing metadata?
    let (etag, last_modified) = (None, None); // TODO: you can read from DB to fill these
    let (status, ct, body, etag_new, lm_new) = sc.fetch_bytes_conditional(&url, etag, last_modified).await?;

    if status == reqwest::StatusCode::NOT_MODIFIED {
        bail!("not_modified");
    }
    if !status.is_success() {
        bail!("http status {}", status.as_u16());
    }
    if !ct.to_lowercase().starts_with("text/html") {
        bail!("content-type not html: {ct}");
    }

    let html = String::from_utf8_lossy(&body).to_string();
    let (title, description, text) = html_to_text(&html);
    let trimmed = text.chars().take(200_000).collect::<String>();

    Ok(Document {
        url: url.to_string(),
        fetched_at: Utc::now(),
        title,
        description,
        body_text: trimmed,
        content_type: Some(ct),
        http_status: status.as_u16() as i32,
        // new fields—store.rs will upsert them
        etag: etag_new,
        last_modified: lm_new,
        lang: None,
        content_hash: None,
    })
}