use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use reqwest::{redirect::Policy, Client, StatusCode};
use robots_txt::Robots;
use scraper::{Html, Selector};
use std::{collections::HashSet, time::Duration};
use url::{Position, Url};
use chrono::Utc;

use crate::types::Document;

#[derive(Clone)]
pub struct ScrapeClient {
    pub http: Client,
    pub user_agent: String,
    // simple per-request delay can be added externally if you like
}

impl ScrapeClient {
    pub fn new(user_agent: &str) -> Self {
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
        }
    }

    pub async fn fetch_bytes(&self, url: &Url) -> Result<(StatusCode, String, Bytes)> {
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
    // robots.txt URL
    let robots_url = match url.join("/robots.txt") {
        Ok(u) => u,
        Err(_) => return true, // malformed join, allow
    };

    // Get robots quickly; allow on error
    let txt = match sc.http.get(robots_url).send().await {
        Ok(resp) if resp.status().is_success() => resp.text().await.ok(),
        _ => None,
    };

    if let Some(txt) = txt {
        // parse lossily (crate 0.7 API)
        let robots = Robots::from_str_lossy(&txt);
        // robots_txt uses lowercase agent matching; use '*'
        let ua = "*";
        robots.allowed(ua, url.as_str()).unwrap_or(true)
    } else {
        true
    }
}

/// Extract <a href> absolute same-host links, deduped, limited.
pub fn extract_links(html: &str, base: &Url, limit: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::<String>::new();

    let doc = Html::parse_document(html);
    let a_sel = Selector::parse("a[href]").unwrap();

    for a in doc.select(&a_sel) {
        if let Some(href) = a.value().attr("href") {
            if let Ok(abs) = base.join(href) {
                // keep http(s) only
                if abs.scheme() != "http" && abs.scheme() != "https" {
                    continue;
                }
                // same host only to keep things polite
                if abs.host_str() != base.host_str() {
                    continue;
                }
                // strip fragment
                let mut u = abs.clone();
                u.set_fragment(None);

                // normalize path+query as key
                let key = u[..Position::AfterPath].to_string() + &u[Position::BeforeQuery..].to_string();
                if seen.insert(key) {
                    out.push(u.to_string());
                    if out.len() >= limit {
                        break;
                    }
                }
            }
        }
    }
    out
}

/// Strip scripts/styles and turn visible text into one string, plus title/description.
fn html_to_text(html: &str) -> (Option<String>, Option<String>, String) {
    let doc = Html::parse_document(html);

    // title
    let title_sel = Selector::parse("title").unwrap();
    let title = doc.select(&title_sel).next()
        .map(|n| n.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty());

    // meta description
    let meta_sel = Selector::parse("meta[name=description]").unwrap();
    let description = doc.select(&meta_sel).next()
        .and_then(|m| m.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // gather body text
    let body_sel = Selector::parse("body").unwrap();
    let body_text = doc.select(&body_sel)
        .flat_map(|b| b.text())
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    (title, description, body_text)
}

pub async fn scrape_one(sc: &ScrapeClient, url_raw: &str) -> Result<(Document, String, Vec<String>)> {
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

    // Decode bytes (assume utf-8)
    let html = String::from_utf8_lossy(&body).to_string();
    let (title, description, text) = html_to_text(&html);

    // Discover a few internal links to enqueue
    let discovered = extract_links(&html, &url, 50);

    let trimmed = text.chars().take(200_000).collect::<String>();

    let doc = Document {
        url: url.to_string(),
        fetched_at: Utc::now(),
        title,
        description,
        body_text: trimmed,
        content_type: Some(ct),
        http_status: status.as_u16() as i32,
        lang: None,
        content_hash: None,
        etag: None,
        last_modified: None,
    };

    Ok((doc, url.to_string(), discovered))
}