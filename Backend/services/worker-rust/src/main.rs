use actix_web::{middleware, post, get, web, App, HttpResponse, HttpServer, Responder};
use std::time::Duration;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt::Subscriber};
use tracing_subscriber::util::SubscriberInitExt;

mod scrape;
mod store;
mod types;

use crate::scrape::{ScrapeClient, scrape_one};
use crate::store::{PgPool, init_pool};
use crate::types::{IngestRequest, Health};

#[get("/health")]
async fn health() -> impl Responder {
    web::Json(Health { status: "ok".into() })
}

#[post("/ingest/url")]
async fn ingest_url(
    payload: web::Json<IngestRequest>,
    pg: web::Data<PgPool>,
    sc: web::Data<ScrapeClient>,
) -> actix_web::Result<impl Responder> {
    let req = payload.into_inner();

    let result = scrape_one(&sc, &req.url).await;
    match result {
        Ok((doc, _canonical, _links)) => {
            if let Err(e) = store::upsert_document(&pg, &doc).await {
                error!(error = ?e, "failed to store document");
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "ok": false, "error": "store_failed"
                })));
            }
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "ok": true,
                "url": doc.url,
                "title": doc.title,
                "bytes": doc.body_text.len()
            })))
        }
        Err(e) => {
            error!(error = ?e, "scrape failed");
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "ok": false, "error": e.to_string()
            })))
        }
    }
}

/// POST /crawl/seed  — idempotent inserts of a small curated list.
#[post("/crawl/seed")]
async fn crawl_seed(pg: web::Data<PgPool>) -> actix_web::Result<impl Responder> {
    let seeds: &[(&str, &str)] = &[
        ("NOAA Climate News", "https://www.climate.gov/news-features"),
        ("NASA Climate", "https://climate.nasa.gov/"),
        ("UNFCCC News", "https://unfccc.int/news"),
        ("Nature Climate", "https://www.nature.com/subjects/climate-change"),
        ("IPCC", "https://www.ipcc.ch/"),
        ("EPA Climate", "https://www.epa.gov/climate-change"),
        ("Guardian Climate", "https://www.theguardian.com/environment/climate-crisis"),
        ("NYTimes Climate", "https://www.nytimes.com/section/climate"),
    ];

    let mut inserted = 0usize;
    let mut enqueued = 0usize;

    for (label, url) in seeds {
        if let Err(e) = store::insert_seed_if_absent(&pg, label, url).await {
            error!(?e, "seed insert failed");
        } else {
            inserted += 1;
        }
        if let Err(e) = store::enqueue_if_absent(&pg, url, None, 50).await {
            error!(?e, "enqueue seed failed");
        } else {
            enqueued += 1;
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "ok": true,
        "seed_inserted": inserted,
        "seed_enqueued": enqueued
    })))
}

/// POST /crawl/tick?batch=50 — process due URLs; discover new same-host links.
#[post("/crawl/tick")]
async fn crawl_tick(
    pg: web::Data<PgPool>,
    sc: web::Data<ScrapeClient>,
    q: web::Query<std::collections::HashMap<String, String>>,
) -> actix_web::Result<impl Responder> {
    let batch: i64 = q.get("batch").and_then(|s| s.parse().ok()).unwrap_or(20);

    let items = match store::dequeue_due(&pg, batch).await {
        Ok(v) => v,
        Err(e) => {
            error!(?e, "dequeue_due failed");
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({"ok": false, "error":"dequeue_failed"})));
        }
    };

    let mut ok_count = 0usize;
    let mut fail_count = 0usize;
    let mut enqueued_new = 0usize;

    for it in items {
        match scrape_one(&sc, &it.url).await {
            Ok((doc, canonical, links)) => {
                if let Err(e) = store::upsert_document(&pg, &doc).await {
                    error!(?e, "upsert_document failed");
                    let _ = store::reschedule_failure(&pg, it.id).await;
                    fail_count += 1;
                    continue;
                }
                // enqueue discovered links (best-effort)
                for l in links.iter().take(200) {
                    if let Err(e) = store::enqueue_if_absent(&pg, l, Some(&canonical), 100).await {
                        let _ = e; // ignore duplicate/unique errors
                    } else {
                        enqueued_new += 1;
                    }
                }
                let _ = store::reschedule_success(&pg, it.id).await;
                ok_count += 1;
            }
            Err(e) => {
                error!(?e, url = %it.url, "scrape failed");
                let _ = store::reschedule_failure(&pg, it.id).await;
                fail_count += 1;
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "ok": true,
        "processed": ok_count + fail_count,
        "ok": ok_count,
        "failed": fail_count,
        "enqueued_new": enqueued_new
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Logging
    Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Config from env
    let addr = std::env::var("WORKER_BIND").unwrap_or_else(|_| "127.0.0.1:5002".to_string());
    let pg_url = std::env::var("PG_URL").expect("PG_URL not set");

    // Init subsystems
    let pool = init_pool(&pg_url).await.expect("pg pool init failed");
    info!("✅ connected to Postgres");

    let sc = ScrapeClient::new("ClimateImpactBot/1.0 (+https://codered.plobethus.com)");

    info!("🌐 worker listening on {}", addr);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(sc.clone()))
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