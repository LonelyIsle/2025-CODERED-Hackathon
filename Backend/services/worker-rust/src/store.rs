use anyhow::Result;
use deadpool_postgres::{Manager, ManagerConfig, RecyclingMethod, Pool};
use tokio_postgres::NoTls; // keep around if you later want a direct client
use tokio_postgres::types::ToSql;
use chrono::{Duration, Utc};

use crate::types::Document;

pub type PgPool = Pool;

pub async fn init_pool(pg_url: &str) -> Result<PgPool> {
    let cfg = pg_url.parse::<tokio_postgres::Config>()?;
    let mgr = Manager::from_config(cfg, NoTls, ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    let pool = Pool::builder(mgr).max_size(8).build().unwrap();

    // ensure tables we depend on (idempotent)
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
      lang          text,
      content_hash  text,
      etag          text,
      last_modified text,
      created_at    timestamptz NOT NULL DEFAULT now()
    );

    CREATE UNIQUE INDEX IF NOT EXISTS uq_ingested_url ON public.ingested_documents(url);
    CREATE INDEX IF NOT EXISTS idx_ingested_fetched_at ON public.ingested_documents (fetched_at DESC);

    CREATE TABLE IF NOT EXISTS public.crawl_seeds (
      id          bigserial PRIMARY KEY,
      label       text NOT NULL,
      url         text NOT NULL UNIQUE,
      created_at  timestamptz NOT NULL DEFAULT now()
    );

    CREATE TABLE IF NOT EXISTS public.crawl_queue (
      id            bigserial PRIMARY KEY,
      url           text NOT NULL UNIQUE,
      discovered_via text,
      priority      int NOT NULL DEFAULT 100,
      next_fetch_at timestamptz NOT NULL DEFAULT now(),
      attempts      int NOT NULL DEFAULT 0,
      created_at    timestamptz NOT NULL DEFAULT now()
    );
    "#).await?;

    Ok(pool)
}

pub async fn upsert_document(pool: &PgPool, d: &Document) -> Result<()> {
    let client = pool.get().await?;
    client.execute(r#"
      INSERT INTO public.ingested_documents
        (url, fetched_at, title, description, body_text, content_type, http_status, lang, content_hash, etag, last_modified)
      VALUES
        ($1,  $2,        $3,    $4,          $5,        $6,           $7,          $8,   $9,           $10, $11)
      ON CONFLICT (url) DO UPDATE SET
        fetched_at    = EXCLUDED.fetched_at,
        title         = EXCLUDED.title,
        description   = EXCLUDED.description,
        body_text     = EXCLUDED.body_text,
        content_type  = EXCLUDED.content_type,
        http_status   = EXCLUDED.http_status,
        lang          = COALESCE(EXCLUDED.lang, public.ingested_documents.lang),
        content_hash  = COALESCE(EXCLUDED.content_hash, public.ingested_documents.content_hash),
        etag          = COALESCE(EXCLUDED.etag, public.ingested_documents.etag),
        last_modified = COALESCE(EXCLUDED.last_modified, public.ingested_documents.last_modified)
    "#,
    &[
        &d.url,
        &d.fetched_at,
        &d.title,
        &d.description,
        &d.body_text,
        &d.content_type,
        &d.http_status,
        &d.lang,
        &d.content_hash,
        &d.etag,
        &d.last_modified,
    ]).await?;
    Ok(())
}

pub async fn insert_seed_if_absent(pool: &PgPool, label: &str, url: &str) -> Result<()> {
    let client = pool.get().await?;
    client.execute(
        "INSERT INTO public.crawl_seeds(label, url) VALUES ($1,$2)
         ON CONFLICT (url) DO NOTHING",
        &[&label, &url],
    ).await?;
    Ok(())
}

pub async fn enqueue_if_absent(pool: &PgPool, url: &str, discovered_via: Option<&str>, priority: i32) -> Result<()> {
    let client = pool.get().await?;
    client.execute(
        "INSERT INTO public.crawl_queue(url, discovered_via, priority, next_fetch_at)
         VALUES ($1,$2,$3, now())
         ON CONFLICT (url) DO NOTHING",
        &[&url, &discovered_via, &priority],
    ).await?;
    Ok(())
}

#[derive(Debug)]
pub struct QueueItem {
    pub id: i64,
    pub url: String,
}

/// Pop up to `batch` due URLs and mark attempts + schedule a backoff.
/// Returns (id,url) list; caller should process and then optionally reschedule again.
pub async fn dequeue_due(pool: &PgPool, batch: i64) -> Result<Vec<QueueItem>> {
    let client = pool.get().await?;

    // select due
    let rows = client.query(
        "SELECT id, url FROM public.crawl_queue
         WHERE next_fetch_at <= now()
         ORDER BY priority ASC, next_fetch_at ASC
         LIMIT $1",
        &[&batch]
    ).await?;

    // mark attempt and push next_fetch_at a bit to avoid dogpile
    for r in &rows {
        let id: i64 = r.get(0);
        client.execute(
            "UPDATE public.crawl_queue
             SET attempts = attempts + 1,
                 next_fetch_at = now() + interval '5 minutes'
             WHERE id = $1",
            &[&id]
        ).await?;
    }

    let out = rows.into_iter().map(|r| QueueItem {
        id: r.get(0),
        url: r.get(1),
    }).collect();

    Ok(out)
}

/// After successful crawl, set a longer next_fetch_at (e.g., 1 day).
pub async fn reschedule_success(pool: &PgPool, id: i64) -> Result<()> {
    let client = pool.get().await?;
    client.execute(
        "UPDATE public.crawl_queue
         SET next_fetch_at = now() + interval '1 day'
         WHERE id = $1",
        &[&id]
    ).await?;
    Ok(())
}

/// On hard failure, back off more aggressively (e.g., 6 hours).
pub async fn reschedule_failure(pool: &PgPool, id: i64) -> Result<()> {
    let client = pool.get().await?;
    client.execute(
        "UPDATE public.crawl_queue
         SET next_fetch_at = now() + interval '6 hours'
         WHERE id = $1",
        &[&id]
    ).await?;
    Ok(())
}