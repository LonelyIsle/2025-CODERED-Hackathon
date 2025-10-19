use actix_web::{middleware, post, get, web, App, HttpResponse, HttpServer, Responder};
use actix_web::web::Query;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::util::SubscriberInitExt; // <- needed for .try_init()

mod scrape;
mod store;
mod types;

use crate::scrape::{ScrapeClient, scrape_one};
use crate::store::{PgPool, init_pool};
use crate::types::{IngestRequest};

#[get("/health")]
async fn health() -> impl Responder {
    web::Json(serde_json::json!({ "status": "ok" }))
}

/* ------------------------ /ingest/url ------------------------ */

#[post("/ingest/url")]
async fn ingest_url(
    payload: web::Json<IngestRequest>,
    pg: web::Data<PgPool>,
    sc: web::Data<ScrapeClient>,
) -> actix_web::Result<impl Responder> {
    let req = payload.into_inner();
    match scrape_one(&sc, &req.url).await {
        Ok(doc) => {
            use crate::store::DocumentRow;
            let row = DocumentRow {
                url: &doc.url,
                fetched_at: doc.fetched_at,
                title: doc.title.as_deref(),
                description: doc.description.as_deref(),
                body_text: &doc.body_text,
                content_type: doc.content_type.as_deref(),
                http_status: doc.http_status,
                content_hash: doc.content_hash.as_deref(),
                lang: doc.lang.as_deref(),
                etag: doc.etag.as_deref(),
            };
            if let Err(e) = store::upsert_document(&pg, &row).await {
                error!(error=?e, "failed to store document");
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "ok": false, "error": "store_failed"
                })));
            }
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "ok": true,
                "url": row.url,
                "title": row.title,
                "bytes": row.body_text.len()
            })))
        }
        Err(e) => {
            error!(error=?e, url=%req.url, "scrape failed");
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "ok": false, "error": e.to_string()
            })))
        }
    }
}

/* ------------------------ /crawl/seed ------------------------ */

#[post("/crawl/seed")]
async fn crawl_seed(pg: web::Data<PgPool>) -> actix_web::Result<impl Responder> {
    let seeds: &[(&str, i32)] = &[
        ("https://www.ipcc.ch/", 100),
        ("https://www.noaa.gov/climate", 90),
        ("https://www.climate.gov/news-features", 90),
        ("https://www.nature.com/subjects/climate-change", 80),
        ("https://www.nytimes.com/section/climate", 70),
        ("https://www.theguardian.com/environment/climate-crisis", 70),
        ("https://www.unep.org/resources", 70),
        ("https://www.iea.org/topics/climate-change", 70),
        ("https://www.epa.gov/climate-change", 60),
        ("https://www.carbonbrief.org/", 90),
        ("https://www.wri.org/insights", 70),
        ("https://www.edf.org/climate", 60),
        ("https://www.bbc.com/news/science_and_environment", 60),
        ("https://www.nasa.gov/climate/", 80),
    ];
    match store::enqueue_many(&pg, seeds).await {
        Ok(n) => Ok(HttpResponse::Ok().json(serde_json::json!({ "ok": true, "enqueued": n }))),
        Err(e) => {
            error!(error=?e, "seed enqueue failed");
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({ "ok": false })))
        }
    }
}

/* ------------------------ /crawl/tick ------------------------ */

#[derive(Debug, serde::Deserialize)]
struct TickQ { batch: Option<i64> }

#[post("/crawl/tick")]
async fn crawl_tick(
    q: Query<TickQ>,
    pg: web::Data<PgPool>,
    sc: web::Data<ScrapeClient>,
) -> actix_web::Result<impl Responder> {
    let batch = q.batch.unwrap_or(25).clamp(1, 200);
    let mut ok = 0usize;
    let mut failed = 0usize;

    let items = match store::dequeue_due(&pg, batch).await {
        Ok(v) => v,
        Err(e) => {
            error!(error=?e, "dequeue failed");
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "ok": false, "error": "dequeue_failed"
            })));
        }
    };

    for it in items {
        match scrape_one(&sc, &it.url).await {
            Ok(doc) => {
                use crate::store::DocumentRow;
                let row = DocumentRow {
                    url: &doc.url,
                    fetched_at: doc.fetched_at,
                    title: doc.title.as_deref(),
                    description: doc.description.as_deref(),
                    body_text: &doc.body_text,
                    content_type: doc.content_type.as_deref(),
                    http_status: doc.http_status,
                    content_hash: doc.content_hash.as_deref(),
                    lang: doc.lang.as_deref(),
                    etag: doc.etag.as_deref(),
                };
                if let Err(e) = store::upsert_document(&pg, &row).await {
                    error!(error=?e, url=%doc.url, "upsert failed");
                    let _ = store::reschedule_failure(&pg, it.id, "upsert_failed", 30).await;
                    failed += 1;
                    continue;
                }
                let _ = store::reschedule_success(&pg, it.id, doc.http_status).await;
                ok += 1;
            }
            Err(e) => {
                error!(error=?e, url=%it.url, "scrape failed");
                let _ = store::reschedule_failure(&pg, it.id, &format!("{e}"), 30).await;
                failed += 1;
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "ok": true,
        "processed_ok": ok,
        "failed": failed
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Logging
    let _ = fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .try_init();

    // Config
    let addr  = std::env::var("WORKER_BIND").unwrap_or_else(|_| "127.0.0.1:5002".into());
    let pg_url = std::env::var("PG_URL").expect("PG_URL not set");

    // Init subsystems
    let pool = init_pool(&pg_url).await.expect("pg pool init failed");
    info!("✅ connected to Postgres");

    let sc = ScrapeClient::new(
        "ClimateImpactBot/1.0 (+https://codered.plobethus.com)",
        2,
        std::time::Duration::from_millis(400),
    );

    info!("🌐 worker listening on {}", addr);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(sc.clone()))
            .wrap(middleware::Logger::default())
            .service(health)
            .service(ingest_url)   // <- now in scope
            .service(crawl_seed)
            .service(crawl_tick)
    })
    .bind(addr)?
    .workers(2)
    .run()
    .await
}