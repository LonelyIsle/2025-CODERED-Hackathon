use anyhow::Result;
use deadpool_postgres::{Config as PgConfig, Pool, Runtime};
use tokio_postgres::NoTls;

use crate::types::Document;

pub type PgPool = Pool;

pub async fn init_pool(pg_url: &str) -> Result<PgPool> {
    let mut cfg = PgConfig::new();
    cfg.url = Some(pg_url.to_string());

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    ensure_table(&pool).await?;
    Ok(pool)
}

async fn ensure_table(pool: &PgPool) -> Result<()> {
    // Safe to run on every boot
    const SQL: &str = r#"
    CREATE TABLE IF NOT EXISTS public.ingested_documents (
      id            bigserial PRIMARY KEY,
      url           text NOT NULL,
      fetched_at    timestamptz NOT NULL,
      title         text,
      description   text,
      body_text     text NOT NULL,
      content_type  text,
      http_status   int NOT NULL,
      created_at    timestamptz NOT NULL DEFAULT now()
    );
    CREATE UNIQUE INDEX IF NOT EXISTS uq_ingested_url ON public.ingested_documents (url);
    CREATE INDEX IF NOT EXISTS idx_ingested_fetched_at ON public.ingested_documents (fetched_at DESC);
    "#;

    let conn = pool.get().await?;
    conn.batch_execute(SQL).await?;
    Ok(())
}

pub async fn upsert_document(pool: &PgPool, d: &Document) -> Result<()> {
    // Upsert into the *ingested_documents* table (NOT public.documents)
    const SQL: &str = r#"
    INSERT INTO public.ingested_documents
      (url, fetched_at, title, description, body_text, content_type, http_status)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
    ON CONFLICT (url) DO UPDATE
      SET fetched_at   = EXCLUDED.fetched_at,
          title        = EXCLUDED.title,
          description  = EXCLUDED.description,
          body_text    = EXCLUDED.body_text,
          content_type = EXCLUDED.content_type,
          http_status  = EXCLUDED.http_status;
    "#;

    let conn = pool.get().await?;
    conn.execute(
        SQL,
        &[
            &d.url,
            &d.fetched_at,
            &d.title,
            &d.description,
            &d.body_text,
            &d.content_type,
            &d.http_status,
        ],
    )
    .await?;
    Ok(())
}