// src/main.rs
use actix_web::{middleware, post, get, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::time::Duration;
use tracing::{info};
use tracing_subscriber::{EnvFilter, util::SubscriberInitExt};

mod scrape;
mod store;
mod types;
mod crawl;

use crate::scrape::ScrapeClient;
use crate::store::{PgPool, init_pool};
use crate::types::{IngestRequest, Health};
use crate::crawl::{Crawler, CrawlConfig};

#[get("/health")]
async fn health() -> impl Responder {
    web::Json(Health { status: "ok".into() })
}

#[post("/ingest/url")]
async fn ingest_url(
    payload: web::Json<IngestRequest>,
    pg: web::Data<PgPool>,
    sc: web::Data<ScrapeClient>,
    robots: web::Data<scrape::RobotsCache>,
) -> actix_web::Result<impl Responder> {
    let req = payload.into_inner();
    match scrape::scrape_one(&sc, &robots, &req.url).await {
        Ok(doc) => {
            if let Err(_) = store::upsert_document(&pg, &doc).await {
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({"ok": false, "error": "store_failed"})));
            }
            Ok(HttpResponse::Ok().json(serde_json::json!({"ok": true, "url": doc.url, "title": doc.title, "bytes": doc.body_text.len()})))
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({"ok": false, "error": e.to_string()}))),
    }
}

#[post("/crawl/seed")]
async fn crawl_seed(crawler: web::Data<Crawler>) -> actix_web::Result<impl Responder> {
    if let Err(e) = crawler.seed_default_sources().await {
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({"ok": false, "error": e.to_string()})));
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"ok": true})))
}

#[post("/crawl/tick")]
async fn crawl_tick(
    crawler: web::Data<Crawler>,
    q: web::Query<std::collections::HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let batch: i64 = q.get("batch").and_then(|s| s.parse().ok()).unwrap_or(50);
    match crawler.crawl_tick(batch as usize).await {
        Ok(ok) => Ok(HttpResponse::Ok().json(serde_json::json!({"ok": true, "fetched": ok}))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({"ok": false, "error": e.to_string()}))),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    // Config from env
    let addr = std::env::var("WORKER_BIND").unwrap_or_else(|_| "127.0.0.1:5002".to_string());
    let pg_url = std::env::var("PG_URL").expect("PG_URL not set");

    // Init subsystems
    let pool = init_pool(&pg_url).await.expect("pg pool init failed");
    store::ensure_tables(&pool).await.expect("ensure_tables failed");
    info!("✅ connected to Postgres");

    // Shared scraper + robots cache
    let robots = scrape::RobotsCache::new(60 * 30); // 30 min TTL
    let sc = ScrapeClient::new("ClimateImpactBot/1.0 (+https://codered.plobethus.com)", 2, Duration::from_millis(350)).expect("client");

    // Crawler
    let crawler = Crawler::new(
        pool.clone(),
        CrawlConfig {
            user_agent: "ClimateImpactBot/1.0 (+https://codered.plobethus.com)".into(),
            per_domain_concurrency: 2,
            per_domain_delay_ms: 350,
            max_tasks: 64,
            robots_ttl_secs: 60 * 30,
        }
    ).await.expect("crawler init");

    info!("🌐 worker listening on {}", addr);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(sc.clone()))
            .app_data(web::Data::new(robots.clone()))
            .app_data(web::Data::new(crawler.clone()))
            .wrap(middleware::Logger::default())
            .service(health)
            .service(ingest_url)
            .service(crawl_seed)
            .service(crawl_tick)
    })
    .bind(addr)?
    .workers(2)
    .run()
    .await
}