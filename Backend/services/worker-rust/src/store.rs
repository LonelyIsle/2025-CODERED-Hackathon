// src/store.rs
use anyhow::{Result, anyhow};
use deadpool_postgres::{ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::Config as PgConfig;

use crate::types::Document;

pub type PgPool = Pool;

pub async fn init_pool(pg_url: &str) -> Result<PgPool> {
    let mut cfg: PgConfig = pg_url.parse()?;
    let mgr = deadpool_postgres::Manager::from_config(
        cfg,
        tokio_postgres::NoTls,
        ManagerConfig { recycling_method: RecyclingMethod::Fast }
    );
    Ok(Pool::builder(mgr)
        .max_size(16)
        .runtime(Runtime::Tokio1)
        .build()
        .unwrap())
}

pub async fn ensure_tables(pool: &PgPool) -> Result<()> {
    let client = pool.get().await?;
    client.batch_execute(r#"
        CREATE TABLE IF NOT EXISTS public.crawl_queue (
          id bigserial PRIMARY KEY,
          url text NOT NULL UNIQUE,
          discovered_via text,
          priority int NOT NULL DEFAULT 100,
          next_fetch_at timestamptz NOT NULL DEFAULT now(),
          attempts int NOT NULL DEFAULT 0,
          created_at timestamptz NOT NULL DEFAULT now()
        );
    "#).await?;
    Ok(())
}

pub async fn enqueue_url(pool: &PgPool, url: &str, via: Option<&str>) -> Result<()> {
    let client = pool.get().await?;
    let _ = client.execute(
        "INSERT INTO public.crawl_queue (url, discovered_via) VALUES ($1, $2)
         ON CONFLICT (url) DO NOTHING",
        &[&url, &via.map(|s| s.to_string())]
    ).await?;
    Ok(())
}

#[derive(Debug)]
pub struct QueueItem { pub url: String }

pub async fn dequeue_batch(pool: &PgPool, n: i64) -> Result<Vec<QueueItem>> {
    // naive: pick due URLs and bump next_fetch_at to avoid contention
    let client = pool.get().await?;
    let rows = client.query(
        "UPDATE public.crawl_queue
           SET next_fetch_at = now() + interval '5 minutes',
               attempts = attempts + 1
         WHERE id IN (
            SELECT id FROM public.crawl_queue
             WHERE next_fetch_at <= now()
             ORDER BY priority ASC, id ASC
             LIMIT $1
         )
         RETURNING url", &[&n]
    ).await?;
    Ok(rows.into_iter().map(|r| QueueItem { url: r.get(0) }).collect())
}

pub async fn mark_backoff(pool: &PgPool, url: &str, seconds: i64) -> Result<()> {
    let client = pool.get().await?;
    let _ = client.execute(
        "UPDATE public.crawl_queue SET next_fetch_at = now() + make_interval(secs => $2) WHERE url = $1",
        &[&url, &seconds]
    ).await?;
    Ok(())
}

pub async fn upsert_document(pool: &PgPool, d: &Document) -> Result<()> {
    let client = pool.get().await?;
    client.batch_execute(r#"
        CREATE TABLE IF NOT EXISTS public.ingested_documents (
          id            bigserial PRIMARY KEY,
          url           text NOT NULL UNIQUE,
          fetched_at    timestamptz NOT NULL,
          title         text,
          description   text,
          body_text     text NOT NULL,
          content_type  text,
          http_status   int NOT NULL,
          etag          text,
          last_modified text,
          lang          text,
          content_hash  text,
          created_at    timestamptz NOT NULL DEFAULT now()
        );
    "#).await?;

    let _ = client.execute(
        r#"INSERT INTO public.ingested_documents
            (url, fetched_at, title, description, body_text, content_type, http_status, etag, last_modified, lang, content_hash)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
           ON CONFLICT (url) DO UPDATE
           SET fetched_at=$2, title=$3, description=$4, body_text=$5, content_type=$6,
               http_status=$7, etag=$8, last_modified=$9, lang=$10, content_hash=$11"#,
        &[
            &d.url, &d.fetched_at, &d.title, &d.description, &d.body_text,
            &d.content_type, &d.http_status, &d.etag, &d.last_modified, &d.lang, &d.content_hash
        ]
    ).await?;

    Ok(())
}