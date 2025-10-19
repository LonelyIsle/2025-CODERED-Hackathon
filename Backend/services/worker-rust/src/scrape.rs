use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use chrono::Utc;
use reqwest::{redirect::Policy, Client, StatusCode};
use scraper::{Html, Selector};
use std::{sync::Arc, time::Duration};
use tokio::sync::Semaphore;
use url::Url;

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

    pub async fn fetch_bytes(&self, url: &Url) -> Result<(StatusCode, String, Bytes)> {
        let _permit = self.domain_limit.acquire().await?;
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

/// Minimal robots.txt check:
/// - fetches /robots.txt (if present)
/// - chooses the first matching User-agent group (exact UA substring match, else `*`)
/// - applies Disallow prefixes (Allow with longer prefix overrides)
async fn allowed_by_robots(sc: &ScrapeClient, url: &Url) -> bool {
    // Build robots.txt URL (http[s]://host[:port]/robots.txt)
    let mut robots = url.clone();
    robots.set_path("/robots.txt");
    robots.set_query(None);
    robots.set_fragment(None);

    let txt = match sc.http.get(robots).send().await {
        Ok(resp) if resp.status().is_success() => resp.text().await.ok(),
        _ => None,
    };
    let txt = match txt {
        Some(t) => t,
        None => return true, // no robots -> allow
    };

    #[derive(Default)]
    struct Group {
        uas: Vec<String>,
        disallow: Vec<String>,
        allow: Vec<String>,
    }

    // Parse into groups
    let mut groups: Vec<Group> = Vec::new();
    let mut current = Group::default();
    let mut saw = false;

    for raw in txt.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let mut parts = line.splitn(2, ':');
        let k = parts.next().unwrap().trim().to_ascii_lowercase();
        let v = parts.next().unwrap_or("").trim().to_string();

        match k.as_str() {
            "user-agent" => {
                if saw && (!current.uas.is_empty() || !current.disallow.is_empty() || !current.allow.is_empty()) {
                    groups.push(current);
                    current = Group::default();
                }
                current.uas.push(v.to_ascii_lowercase());
                saw = true;
            }
            "disallow" => { current.disallow.push(v); saw = true; }
            "allow" => { current.allow.push(v); saw = true; }
            _ => {}
        }
    }
    if saw { groups.push(current); }

    // Choose group: exact UA substring match, else `*`
    let ua_lc = sc.user_agent.to_ascii_lowercase();
    let mut chosen: Option<&Group> = None;

    for g in &groups {
        if g.uas.iter().any(|u| !u.is_empty() && ua_lc.contains(u)) {
            chosen = Some(g);
            break;
        }
    }
    if chosen.is_none() {
        for g in &groups {
            if g.uas.iter().any(|u| u == "*") {
                chosen = Some(g);
                break;
            }
        }
    }
    let g = match chosen { Some(g) => g, None => return true };

    let path = url.path();
    // Disallow prefix blocks unless an Allow with a longer prefix overrides.
    for d in &g.disallow {
        if d.is_empty() { continue; }
        if path.starts_with(d) {
            let override_ok = g.allow.iter().any(|a| !a.is_empty() && path.starts_with(a) && a.len() > d.len());
            if !override_ok { return false; }
        }
    }
    true
}

/// Extract (title, description, body text) from HTML.
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

    // Visible text from <body>
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

use crate::types::Document;

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
    })
}