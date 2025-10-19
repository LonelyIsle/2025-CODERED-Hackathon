use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use chrono::Utc;
use reqwest::{header, redirect::Policy, Client, StatusCode};
use scraper::{Html, Selector};
use sha2::{Digest, Sha256};
use std::{sync::Arc, time::Duration};
use tokio::sync::Semaphore;
use url::Url;
use whatlang::detect;

use crate::types::Document;

#[derive(Clone)]
pub struct ScrapeClient {
    pub http: Client,
    pub user_agent: String,
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
            .unwrap();

        Self {
            http,
            user_agent: user_agent.to_string(),
            domain_limit: Arc::new(Semaphore::new(concurrent_per_domain)),
            delay,
        }
    }

    pub async fn fetch_bytes(&self, url: &Url) -> Result<(StatusCode, String, Option<String>, Option<String>, Bytes)> {
        let _permit = self.domain_limit.acquire().await?;
        tokio::time::sleep(self.delay).await;

        let res = self.http.get(url.clone()).send().await?;
        let status = res.status();

        let ct = res
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let etag = res
            .headers()
            .get(header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let last_modified = res
            .headers()
            .get(header::LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let body = res.bytes().await?;
        Ok((status, ct, etag, last_modified, body))
    }
}

/// Minimal robots.txt check
async fn allowed_by_robots(sc: &ScrapeClient, url: &Url) -> bool {
    let mut robots_url = url.clone();
    robots_url.set_path("/robots.txt");
    robots_url.set_query(None);
    robots_url.set_fragment(None);

    let text = match sc.http.get(robots_url).send().await {
        Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
        _ => return true,
    };

    let mut allows: Vec<String> = Vec::new();
    let mut disallows: Vec<String> = Vec::new();
    let ua_l = sc.user_agent.to_lowercase();
    let mut in_group = false;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let lower = line.to_lowercase();
        if let Some(rest) = lower.strip_prefix("user-agent:") {
            let agent = rest.trim();
            in_group = agent == "*" || ua_l.contains(agent);
            continue;
        }
        if !in_group { continue; }
        if let Some(path) = lower.strip_prefix("allow:") {
            allows.push(path.trim().to_string());
        } else if let Some(path) = lower.strip_prefix("disallow:") {
            disallows.push(path.trim().to_string());
        }
    }

    let p = url.path();
    let blocked = disallows.iter().any(|d| p.starts_with(d)) &&
                  !allows.iter().any(|a| p.starts_with(a) && !a.is_empty());
    !blocked
}

fn html_to_text(html: &str) -> (Option<String>, Option<String>, String) {
    let doc = Html::parse_document(html);

    let title_sel = Selector::parse("title").unwrap();
    let title = doc.select(&title_sel).next().map(|n| n.text().collect::<String>().trim().to_string()).filter(|s| !s.is_empty());

    let meta_desc_sel = Selector::parse("meta[name=\"description\"]").unwrap();
    let description = doc.select(&meta_desc_sel).next()
        .and_then(|n| n.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

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
    if !(url.scheme() == "https" || url.scheme() == "http") {
        bail!("unsupported scheme");
    }

    if !allowed_by_robots(sc, &url).await {
        bail!("blocked by robots.txt");
    }

    let (status, ct, etag, last_modified, body) = sc.fetch_bytes(&url).await?;
    if !status.is_success() {
        bail!("http status {}", status.as_u16());
    }

    if !ct.to_lowercase().starts_with("text/html") {
        bail!("content-type not html: {ct}");
    }

    let html = String::from_utf8_lossy(&body).to_string();
    let (title, description, text) = html_to_text(&html);
    let trimmed = text.chars().take(200_000).collect::<String>();

    let lang = detect(&trimmed).map(|i| i.lang().code().to_string());

    let mut hasher = Sha256::new();
    hasher.update(&body);
    let hash_hex = format!("{:x}", hasher.finalize());

    Ok(Document {
        url: url.to_string(),
        fetched_at: Utc::now(),
        title,
        description,
        body_text: trimmed,
        content_type: Some(ct),
        http_status: status.as_u16() as i32,
        content_hash: Some(hash_hex),
        etag,
        lang,
        last_modified,
    })
}