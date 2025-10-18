use anyhow::Result;
use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Pool};
use tokio_postgres::NoTls;
use crate::types::Document;

pub type PgPool = Pool;

pub async fn init_pool(pg_url: &str) -> Result<PgPool> {
    let mut cfg = Config::new();
    cfg.pg.expose_pg_config = true;
    cfg.pg_config = Some(pg_url.parse()?);
    cfg.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
    cfg.create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls).map_err(Into::into)
}

pub async fn upsert_document(pool: &PgPool, d: &Document) -> Result<()> {
    let client = pool.get().await?;
    // Table assumed from your earlier migrations; create if missing:
    // documents(url text primary key, fetched_at timestamptz, title text, description text, body_text text, content_type text, http_status int)
    let stmt = client.prepare_cached(r#"
        INSERT INTO documents (url, fetched_at, title, description, body_text, content_type, http_status)
        VALUES ($1,$2,$3,$4,$5,$6,$7)
        ON CONFLICT (url) DO UPDATE SET
            fetched_at = EXCLUDED.fetched_at,
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            body_text = EXCLUDED.body_text,
            content_type = EXCLUDED.content_type,
            http_status = EXCLUDED.http_status
    "#).await?;

    client.execute(&stmt, &[
        &d.url, &d.fetched_at, &d.title, &d.description, &d.body_text, &d.content_type, &d.http_status
    ]).await?;
    Ok(())
}