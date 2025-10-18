use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use chrono::Utc;
use reqwest::{redirect::Policy, Client};
use scraper::{Html, Selector};
use std::{sync::Arc, time::Duration};
use tokio::sync::Semaphore;
use url::Url;

use crate::types::Document;

// Optional robots.txt check using robots_txt 0.7 (best-effort)
#[inline]
async fn robots_allows(http: &Client, user_agent: &str, url: &Url) -> bool {
    use robots_txt::Robots;

    // Build robots.txt URL
    let mut robots_base = url.clone();
    robots_base.set_path("/robots.txt");
    robots_base.set_query(None);
    robots_base.set_fragment(None);

    // Try fetch robots.txt quickly; if anything fails, allow by default
    let Ok(resp) = http.get(robots_base).send().await else {
        return true;
    };
    if !resp.status().is_success() {
        return true;
    }
    let Ok(txt) = resp.text().await else {
        return true;
    };

    // Parse robots and check path
    // robots_txt 0.7: Robots::from_str returns Robots (no Result)
    let robots = Robots::from_str(&txt);
    // Some implementations expect just the PATH, not the full URL
    let path = url.path().to_string();
    robots.is_allowed(user_agent, &path)
}

#[derive(Clone)]
pub struct ScrapeClient {
    http: Client,
    user_agent: String,
    // polite throttling
    domain_limit: Arc<Semaphore>,
    delay: Duration,
}

impl ScrapeClient {
    pub fn new(user_agent: &str, concurrent_per_domain: usize, delay: Duration) -> Self {
        let http = Client::builder()
            .user_agent(user_agent)
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .redirect(Policy::limited(8))
            .timeout(Duration::from_secs(20))
            .build()
            .expect("reqwest client");

        Self {
            http,
            user_agent: user_agent.to_string(),
            domain_limit: Arc::new(Semaphore::new(concurrent_per_domain)),
            delay,
        }
    }

    pub async fn fetch_bytes(&self, url: &Url) -> Result<(reqwest::StatusCode, String, Bytes)> {
        let _permit = self.domain_limit.acquire().await?;
        // fixed delay to be polite
        tokio::time::sleep(self.delay).await;

        let res = self.http.get(url.clone()).send().await?;
        let status = res.status();
        let ct = res
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let body = res.bytes().await?;
        Ok((status, ct, body))
    }
}

/// Extract title, description, and visible text
fn html_to_text(html: &str) -> (Option<String>, Option<String>, String) {
    let doc = Html::parse_document(html);

    // Title
    let title_sel = Selector::parse("title").unwrap();
    let title = doc
        .select(&title_sel)
        .next()
        .map(|n| n.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty());

    // Meta description
    let meta_sel = Selector::parse("meta[name=description]").unwrap();
    let description = doc
        .select(&meta_sel)
        .next()
        .and_then(|m| m.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // Body visible text (ignore scripts/styles by selecting only body text)
    let body_sel = Selector::parse("body").unwrap();
    let body_text = doc
        .select(&body_sel)
        .flat_map(|b| b.text())
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    (title, description, body_text)
}

pub async fn scrape_one(sc: &ScrapeClient, url_raw: &str) -> Result<Document> {
    let url = Url::parse(url_raw).map_err(|e| anyhow!("bad url: {e}"))?;
    if !(url.scheme() == "http" || url.scheme() == "https") {
        bail!("unsupported scheme");
    }

    // Best-effort robots.txt allow check
    if !robots_allows(&sc.http, &sc.user_agent, &url).await {
        bail!("blocked by robots.txt");
    }

    let (status, ct, body) = sc.fetch_bytes(&url).await?;
    if !status.is_success() {
        bail!("http status {}", status.as_u16());
    }

    // Only HTML for now
    if !ct.to_lowercase().starts_with("text/html") {
        bail!("content-type not html: {ct}");
    }

    // Decode bytes
    let html = String::from_utf8_lossy(&body).to_string();

    let (title, description, text) = html_to_text(&html);
    let trimmed = text.chars().take(200_000).collect::<String>(); // guard huge pages

    Ok(Document {
        url: url.to_string(),
        fetched_at: Utc::now(),
        title,
        description,
        body_text: trimmed,
        content_type: Some(ct),
        http_status: status.as_u16() as i32,
    })
}