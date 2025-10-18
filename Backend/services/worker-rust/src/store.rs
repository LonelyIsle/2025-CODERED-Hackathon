use anyhow::Result;
use deadpool_postgres::{Config as DpConfig, ManagerConfig, RecyclingMethod, Pool, Runtime};
use postgres_native_tls::MakeTlsConnector;
use tokio_postgres::NoTls; // kept for possible future use
use crate::types::Document;

pub type PgPool = Pool;

pub async fn init_pool(pg_url: &str) -> Result<PgPool> {
    let mut cfg = DpConfig::new();
    cfg.url = Some(pg_url.to_string());
    cfg.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });

    // TLS (safe to use even for local sslmode=disable; server decides)
    let tls_connector = native_tls::TlsConnector::builder().build()?;
    let tls = MakeTlsConnector::new(tls_connector);

    let pool = cfg.create_pool(Some(Runtime::Tokio1), tls)?;
    Ok(pool)
}

pub async fn upsert_document(pool: &PgPool, doc: &Document) -> Result<()> {
    let client = pool.get().await?;

    // Ensure table (idempotent, low-traffic path; for prod, use migrations)
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS documents (
            url TEXT PRIMARY KEY,
            fetched_at TIMESTAMPTZ NOT NULL,
            title TEXT,
            description TEXT,
            body_text TEXT NOT NULL,
            content_type TEXT,
            http_status INT NOT NULL
        );"
    ).await?;

    // Upsert
    let q = r#"
        INSERT INTO documents
            (url, fetched_at, title, description, body_text, content_type, http_status)
        VALUES
            ($1,  $2,        $3,    $4,         $5,        $6,           $7)
        ON CONFLICT (url) DO UPDATE SET
            fetched_at = EXCLUDED.fetched_at,
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            body_text = EXCLUDED.body_text,
            content_type = EXCLUDED.content_type,
            http_status = EXCLUDED.http_status
    "#;

    client.execute(
        q,
        &[
            &doc.url,
            &doc.fetched_at,
            &doc.title,
            &doc.description,
            &doc.body_text,
            &doc.content_type,
            &doc.http_status,
        ],
    ).await?;

    Ok(())
}