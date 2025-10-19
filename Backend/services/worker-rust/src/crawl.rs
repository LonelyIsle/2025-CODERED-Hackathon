// src/crawl.rs
use anyhow::{Result, anyhow};
use chrono::{Utc, Duration as ChronoDur};
use dashmap::DashMap;
use reqwest::redirect::Policy;
use sha2::{Sha256, Digest};
use sitemap::reader::SiteMapReader;
use std::{collections::HashSet, time::Duration, sync::Arc};
use tokio::{task, time::sleep};
use url::Url;
use whatlang::detect;

use crate::scrape::{ScrapeClient, scrape_one, RobotsCache};
use crate::store::{PgPool, upsert_document, enqueue_url, dequeue_batch, mark_backoff, ensure_tables};
use feed_rs::parser;

#[derive(Clone)]
pub struct CrawlConfig {
    pub user_agent: String,
    pub per_domain_concurrency: usize,
    pub per_domain_delay_ms: u64,
    pub max_tasks: usize,
    pub robots_ttl_secs: u64,
}

pub struct Crawler {
    sc: ScrapeClient,
    robots: RobotsCache,
    cfg: CrawlConfig,
    pool: PgPool,
}

impl Crawler {
    pub async fn new(pool: PgPool, cfg: CrawlConfig) -> Result<Self> {
        let sc = ScrapeClient::new(
            &cfg.user_agent,
            cfg.per_domain_concurrency,
            Duration::from_millis(cfg.per_domain_delay_ms),
        )?;
        Ok(Self {
            sc,
            robots: RobotsCache::new(cfg.robots_ttl_secs),
            cfg,
            pool,
        })
    }

    pub async fn seed_default_sources(&self) -> Result<()> {
        // Climate & environment sources (safe, public)
        let seeds = vec![
            ("NOAA Climate News", "https://www.noaa.gov/news"),
            ("NASA Climate", "https://climate.nasa.gov/news/"),
            ("EPA Climate Change", "https://www.epa.gov/climate-change"),
            ("IPCC", "https://www.ipcc.ch/"),
            ("UNFCCC", "https://unfccc.int/news"),
            ("CarbonBrief", "https://www.carbonbrief.org/"),
            ("Nature Climate", "https://www.nature.com/subjects/climate-science"),
            ("ScienceDaily Env", "https://www.sciencedaily.com/news/earth_climate/"),
            ("Yale Climate Connections", "https://yaleclimateconnections.org/"),
            ("Guardian Climate", "https://www.theguardian.com/environment/climate-crisis"),
        ];
        for (_, url) in seeds {
            let _ = enqueue_url(&self.pool, url, Some("seed")).await;
        }
        Ok(())
    }

    /// Pull RSS/Atom if present and enqueue links.
    pub async fn try_discover_feeds(&self, origin: &str) -> Result<usize> {
        let mut added = 0;
        let url = Url::parse(origin)?;
        // Try common feed paths quickly
        let guesses = [
            origin.to_string(),
            url.join("/feed/").ok().map(|u| u.to_string()).unwrap_or_default(),
            url.join("/rss.xml").ok().map(|u| u.to_string()).unwrap_or_default(),
            url.join("/atom.xml").ok().map(|u| u.to_string()).unwrap_or_default(),
        ];
        let client = &self.sc.http;

        for g in guesses.into_iter().filter(|s| !s.is_empty()) {
            if let Ok(resp) = client.get(&g).send().await {
                if resp.status().is_success() {
                    if let Ok(bytes) = resp.bytes().await {
                        if let Ok(feed) = parser::parse(&bytes[..]) {
                            for entry in feed.entries {
                                if let Some(link) = entry.links.first() {
                                    if let Some(href) = &link.href {
                                        if enqueue_url(&self.pool, href, Some("rss")).await.is_ok() {
                                            added += 1;
                                        }
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
        Ok(added)
    }

    /// Pull sitemap.xml and enqueue URLs (throttled).
    pub async fn try_discover_sitemap(&self, origin: &str) -> Result<usize> {
        let mut added = 0;
        let base = Url::parse(origin)?;
        let sm_url = base.join("/sitemap.xml")?;
        let resp = self.sc.http.get(sm_url).send().await?;
        if !resp.status().is_success() {
            return Ok(0);
        }
        let bytes = resp.bytes().await?;
        for entity in SiteMapReader::new(bytes.as_ref()) {
            if let Ok(sitemap::structs::SiteMapEntity::Url(url_entry)) = entity {
                let loc = url_entry.loc.get_url();
                if enqueue_url(&self.pool, loc, Some("sitemap")).await.is_ok() {
                    added += 1;
                }
            }
        }
        Ok(added)
    }

    /// Single crawl tick: pop a batch from queue, fetch concurrently, store.
    pub async fn crawl_tick(&self, batch_size: usize) -> Result<usize> {
        let tasks = dequeue_batch(&self.pool, batch_size).await?;
        if tasks.is_empty() { return Ok(0); }

        let mut handles = vec![];
        for item in tasks {
            let sc = self.sc.clone();
            let robots = self.robots.clone();
            let pool = self.pool.clone();
            let url = item.url.clone();

            handles.push(task::spawn(async move {
                // Fetch & parse
                match scrape_one(&sc, &robots, &url).await {
                    Ok(mut doc) => {
                        // content hash + lang detection
                        let mut hasher = Sha256::new();
                        hasher.update(doc.body_text.as_bytes());
                        doc.content_hash = Some(format!("{:x}", hasher.finalize()));
                        doc.lang = detect(&doc.body_text).map(|i| i.lang().code()).map(|s| s.to_string());

                        if let Err(e) = upsert_document(&pool, &doc).await {
                            // backoff on store failure
                            let _ = mark_backoff(&pool, &url, 10).await;
                            return Err(anyhow!("store failed: {e}"));
                        }
                        Ok::<(), anyhow::Error>(())
                    }
                    Err(e) => {
                        // backoff for transient or blocked cases
                        let _ = mark_backoff(&pool, &url, 60).await;
                        Err(anyhow!("scrape failed: {e}"))
                    }
                }
            }));
        }

        // Wait for all
        let mut ok = 0usize;
        for h in handles {
            if h.await.ok().and_then(|r| r.ok()).is_some() {
                ok += 1;
            }
        }
        Ok(ok)
    }
}