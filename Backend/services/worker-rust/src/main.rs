// src/main.rs
use actix_web::{middleware, post, get, web, App, HttpResponse, HttpServer, Responder};
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

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

    match scrape_one(&sc, &req.url).await {
        Ok(doc) => {
            if let Err(e) = store::upsert_document(&pg, &doc).await {
                error!(error=?e, "failed to store document");
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
            error!(error=?e, "scrape failed");
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "ok": false, "error": e.to_string()
            })))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Logging
    let _ = fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .finish()
        .init();

    // Config
    let addr = std::env::var("WORKER_BIND").unwrap_or_else(|_| "127.0.0.1:5002".to_string());
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
            .service(ingest_url)
    })
    .bind(addr)?
    .workers(2)
    .run()
    .await
}