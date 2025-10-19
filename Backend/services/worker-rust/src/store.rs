// src/store.rs
use anyhow::{Context, Result};
use chrono::Utc;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;

use crate::types::Document;

pub type PgPool = Pool;

pub async fn init_pool(pg_url: &str) -> Result<PgPool> {
    let mgr = Manager::from_config(
        pg_url.parse().context("bad PG_URL")?,
        NoTls,
        ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        },
    );
    let pool = Pool::builder(mgr)
        .max_size(8)
        .build()
        .context("build pool")?;

    // Make sure tables exist
    ensure_tables(&pool).await?;
    Ok(pool)
}

async fn ensure_tables(pool: &PgPool) -> Result<()> {
    let client = pool.get().await.context("get conn for ensure_tables")?;

    // Only the tables we actually use now
    client
        .batch_execute(
            r#"
CREATE TABLE IF NOT EXISTS public.ingested_documents (
  id           bigserial PRIMARY KEY,
  url          text UNIQUE NOT NULL,
  fetched_at   timestamptz NOT NULL,
  title        text,
  description  text,
  body_text    text NOT NULL,
  content_type text,
  http_status  int NOT NULL,
  created_at   timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_ingested_fetched_at
  ON public.ingested_documents (fetched_at DESC);
"#,
        )
        .await
        .context("ensure ingested_documents")?;

    Ok(())
}

pub async fn upsert_document(pool: &PgPool, d: &Document) -> Result<()> {
    let client = pool.get().await.context("get conn for upsert")?;

    // Upsert only the columns that exist in your current table definition
    let stmt = client
        .prepare(
            r#"
INSERT INTO public.ingested_documents
  (url, fetched_at, title, description, body_text, content_type, http_status)
VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (url) DO UPDATE SET
  fetched_at   = EXCLUDED.fetched_at,
  title        = EXCLUDED.title,
  description  = EXCLUDED.description,
  body_text    = EXCLUDED.body_text,
  content_type = EXCLUDED.content_type,
  http_status  = EXCLUDED.http_status
"#,
        )
        .await
        .context("prepare upsert")?;

    client
        .execute(
            &stmt,
            &[
                &d.url,
                &d.fetched_at,
                &d.title,
                &d.description,
                &d.body_text,
                &d.content_type,
                &(d.http_status as i32),
            ],
        )
        .await
        .context("exec upsert")?;

    Ok(())
}