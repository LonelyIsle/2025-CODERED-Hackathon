use anyhow::Result;
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

use crate::types::Document;

pub type PgPool = Pool;

pub async fn init_pool(pg_url: &str) -> Result<PgPool> {
    // Build a deadpool-postgres pool from a connection URL.
    let mut cfg = Config::new();
    cfg.url = Some(pg_url.to_string());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

    // Ensure the target table + indexes exist (idempotent).
    let client = pool.get().await?;
    client
        .batch_execute(
            r#"
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

            -- make url unique so we can upsert on it
            CREATE UNIQUE INDEX IF NOT EXISTS uq_ingested_url ON public.ingested_documents (url);
            CREATE INDEX IF NOT EXISTS idx_ingested_fetched_at ON public.ingested_documents (fetched_at DESC);
            "#,
        )
        .await?;

    Ok(pool)
}

pub async fn upsert_document(pool: &PgPool, doc: &Document) -> Result<()> {
    let client = pool.get().await?;

    // Upsert into the correct table (ingested_documents), not documents
    let _ = client
        .execute(
            r#"
            INSERT INTO public.ingested_documents
                (url, fetched_at, title, description, body_text, content_type, http_status)
            VALUES
                ($1,  $2,         $3,   $4,          $5,        $6,           $7)
            ON CONFLICT (url) DO UPDATE
               SET fetched_at  = EXCLUDED.fetched_at,
                   title       = EXCLUDED.title,
                   description = EXCLUDED.description,
                   body_text   = EXCLUDED.body_text,
                   content_type= EXCLUDED.content_type,
                   http_status = EXCLUDED.http_status
            "#,
            &[
                &doc.url,
                &doc.fetched_at,
                &doc.title,
                &doc.description,
                &doc.body_text,
                &doc.content_type,
                &(doc.http_status as i32),
            ],
        )
        .await?;

    Ok(())
}