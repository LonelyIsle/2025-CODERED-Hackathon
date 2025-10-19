// Minimal stub; expand as needed. Not used by main currently.
#![allow(dead_code)]
use anyhow::Result;
use crate::store::PgPool;
use crate::scrape::ScrapeClient;

pub struct CrawlConfig {
    pub user_agent: String,
    pub per_domain_concurrency: usize,
    pub per_domain_delay_ms: u64,
    pub max_tasks: usize,
    pub robots_ttl_secs: u64,
}

pub struct Crawler {
    sc: ScrapeClient,
    cfg: CrawlConfig,
    pool: PgPool,
}

impl Crawler {
    pub fn new(sc: ScrapeClient, cfg: CrawlConfig, pool: PgPool) -> Self {
        Self { sc, cfg, pool }
    }

    pub async fn run_once(&self) -> Result<usize> {
        // Placeholder: dequeue URLs and scrape them, then upsert docs.
        Ok(0)
    }
}