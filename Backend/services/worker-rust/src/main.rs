use actix_web::{middleware, post, get, web, App, HttpResponse, HttpServer, Responder};
use std::time::{Duration, Instant};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt::Subscriber};

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
    let started = Instant::now();
    let req = payload.into_inner();

    let result = scrape_one(&sc, &req.url).await;
    match result {
        Ok(doc) => {
            if let Err(e) = store::upsert_document(&pg, &doc).await {
                error!(error = ?e, "failed to store document");
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "ok": false, "error": "store_failed"
                })));
            }
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "ok": true,
                "elapsed_ms": started.elapsed().as_millis(),
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ---- Logging (RUST_LOG=info by default) ----
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();

    // ---- Config from env ----
    let addr = std::env::var("WORKER_BIND").unwrap_or_else(|_| "127.0.0.1:5002".to_string());
    let pg_url = std::env::var("PG_URL").expect("PG_URL not set");

    // ---- Init subsystems ----
    let pool = init_pool(&pg_url).await.expect("pg pool init failed");
    info!("✅ connected to Postgres");

    let sc = ScrapeClient::new(
        "ClimateImpactBot/1.0 (+https://codered.plobethus.com)",
        2,                          // concurrent per domain
        Duration::from_millis(400), // politeness delay
    );

    info!("🌐 worker listening on {}", &addr);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(sc.clone()))
            .wrap(middleware::Logger::default())
            .service(health)
            .service(ingest_url)
    })
    .bind(addr)?
    .workers(2)
    .run()
    .await
}