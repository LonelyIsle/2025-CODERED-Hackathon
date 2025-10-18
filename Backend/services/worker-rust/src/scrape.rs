use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use chrono::Utc;
use reqwest::{redirect::Policy, Client};
use robots_txt::RobotsTxt;
use scraper::{Html, Selector};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use url::Url;

use crate::types::Document;

#[derive(Clone)]
pub struct ScrapeClient {
    pub(crate) http: Client,
    pub(crate) user_agent: String,
    // simple polite throttling
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

    pub async fn fetch_bytes(&self, url: &Url) -> Result<(reqwest::StatusCode, String, Bytes)> {
        // Limit concurrency + add a small delay to be polite
        let _permit = self
            .domain_limit
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| anyhow!("semaphore closed"))?;
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

async fn allowed_by_robots(sc: &ScrapeClient, url: &Url) -> bool {
    let robots_url = match url.join("/robots.txt") {
        Ok(u) => u,
        Err(_) => return true, // be permissive if malformed join
    };

    let ua = sc.user_agent.clone();
    match sc.http.get(robots_url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.text().await {
            Ok(txt) => {
                if let Ok(robots) = RobotsTxt::parse(&txt) {
                    robots.allowed(url.as_str(), &ua).unwrap_or(true)
                } else {
                    true
                }
            }
            _ => true,
        },
        _ => true,
    }
}

/// Strip scripts/styles and turn visible text into a single string.
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

    // Gather visible text from body (scripts/styles/noscript ignored by not selecting them)
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

    let (status, ct, body) = sc.fetch_bytes(&url).await?;
    if !status.is_success() {
        bail!("http status {}", status.as_u16());
    }

    // Only HTML for now
    if !ct.to_lowercase().starts_with("text/html") {
        bail!("content-type not html: {ct}");
    }

    // Decode bytes (assume utf-8; extend with chardet if needed)
    let html = String::from_utf8_lossy(&body).to_string();

    let (title, description, text) = html_to_text(&html);
    let trimmed = text.chars().take(200_000).collect::<String>(); // guard against huge pages

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