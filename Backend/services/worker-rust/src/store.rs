use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use deadpool_postgres::{Manager, ManagerConfig, RecyclingMethod, Pool};
use tokio_postgres::NoTls;

pub type PgPool = Pool;

pub async fn init_pool(pg_url: &str) -> Result<PgPool> {
    let cfg = tokio_postgres::Config::from_str(pg_url)
        .context("parse PG_URL")?;
    let mgr = Manager::from_config(cfg, NoTls, ManagerConfig {
        recycling_method: RecyclingMethod::Fast
    });
    let pool = Pool::builder(mgr).max_size(8).build().unwrap();

    // Ensure tables exist
    ensure_tables(&pool).await
        .context("ensure tables")?;

    Ok(pool)
}

pub async fn ensure_tables(pool: &PgPool) -> Result<()> {
    let conn = pool.get().await?;
    // 1) Ingested documents (content store)
    conn.batch_execute(r#"
    CREATE TABLE IF NOT EXISTS public.ingested_documents (
      id            bigserial PRIMARY KEY,
      url           text NOT NULL UNIQUE,
      fetched_at    timestamptz NOT NULL,
      title         text,
      description   text,
      body_text     text NOT NULL,
      content_type  text,
      http_status   int NOT NULL,
      content_hash  text,
      lang          text,
      etag          text,
      created_at    timestamptz NOT NULL DEFAULT now()
    );
    CREATE INDEX IF NOT EXISTS idx_ingested_fetched_at
      ON public.ingested_documents (fetched_at DESC);
    "#).await.context("ensure ingested_documents")?;

    // 2) Crawl queue
    conn.batch_execute(r#"
    CREATE TABLE IF NOT EXISTS public.crawl_queue (
      id            bigserial PRIMARY KEY,
      url           text NOT NULL UNIQUE,
      priority      int  NOT NULL DEFAULT 0,
      next_fetch_at timestamptz NOT NULL DEFAULT now(),
      last_status   int,
      last_error    text,
      tries         int  NOT NULL DEFAULT 0,
      created_at    timestamptz NOT NULL DEFAULT now(),
      updated_at    timestamptz NOT NULL DEFAULT now()
    );
    CREATE INDEX IF NOT EXISTS idx_crawl_next
      ON public.crawl_queue (next_fetch_at, priority DESC);
    "#).await.context("ensure crawl_queue")?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct DocumentRow<'a> {
    pub url: &'a str,
    pub fetched_at: DateTime<Utc>,
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub body_text: &'a str,
    pub content_type: Option<&'a str>,
    pub http_status: i32,
    pub content_hash: Option<&'a str>,
    pub lang: Option<&'a str>,
    pub etag: Option<&'a str>,
}

pub async fn upsert_document(pool: &PgPool, d: &DocumentRow<'_>) -> Result<()> {
    let client = pool.get().await?;
    client.execute(
        r#"
        INSERT INTO public.ingested_documents
          (url, fetched_at, title, description, body_text, content_type, http_status, content_hash, lang, etag)
        VALUES
          ($1,  $2,        $3,    $4,         $5,        $6,           $7,          $8,          $9,   $10)
        ON CONFLICT (url) DO UPDATE SET
          fetched_at   = EXCLUDED.fetched_at,
          title        = EXCLUDED.title,
          description  = EXCLUDED.description,
          body_text    = EXCLUDED.body_text,
          content_type = EXCLUDED.content_type,
          http_status  = EXCLUDED.http_status,
          content_hash = EXCLUDED.content_hash,
          lang         = EXCLUDED.lang,
          etag         = EXCLUDED.etag,
          updated_at   = now()
        "#,
        &[
            &d.url,
            &d.fetched_at,
            &d.title,
            &d.description,
            &d.body_text,
            &d.content_type,
            &d.http_status,
            &d.content_hash,
            &d.lang,
            &d.etag,
        ],
    ).await?;
    Ok(())
}

/* --------------------- Crawl queue helpers --------------------- */

#[derive(Debug, Clone)]
pub struct QueueItem {
    pub id: i64,
    pub url: String,
    pub priority: i32,
}

pub async fn enqueue_many(pool: &PgPool, items: &[(&str, i32)]) -> Result<usize> {
    if items.is_empty() { return Ok(0); }
    let client = pool.get().await?;
    let mut n = 0;
    for (url, prio) in items {
        let _ = client.execute(
            r#"
            INSERT INTO public.crawl_queue (url, priority)
            VALUES ($1, $2)
            ON CONFLICT (url) DO UPDATE
            SET priority = GREATEST(crawl_queue.priority, EXCLUDED.priority)
            "#,
            &[url, prio],
        ).await?;
        n += 1;
    }
    Ok(n)
}

pub async fn dequeue_due(pool: &PgPool, batch: i64) -> Result<Vec<QueueItem>> {
    let client = pool.get().await?;
    let tx = client.build_transaction().start().await?;
    let rows = tx.query(
        r#"
        SELECT id, url, priority
        FROM public.crawl_queue
        WHERE next_fetch_at <= now()
        ORDER BY priority DESC, id
        LIMIT $1
        FOR UPDATE SKIP LOCKED
        "#,
        &[&batch],
    ).await?;
    let items = rows.iter().map(|r| QueueItem {
        id: r.get(0),
        url: r.get(1),
        priority: r.get(2),
    }).collect();
    tx.commit().await?;
    Ok(items)
}

pub async fn reschedule_success(pool: &PgPool, id: i64, http_status: i32) -> Result<()> {
    let client = pool.get().await?;
    client.execute(
        r#"
        UPDATE public.crawl_queue
        SET last_status = $2,
            last_error  = NULL,
            tries       = 0,
            next_fetch_at = now() + interval '6 hours',
            updated_at  = now()
        WHERE id = $1
        "#,
        &[&id, &http_status],
    ).await?;
    Ok(())
}

pub async fn reschedule_failure(pool: &PgPool, id: i64, err: &str, backoff_minutes: i64) -> Result<()> {
    let client = pool.get().await?;
    client.execute(
        r#"
        UPDATE public.crawl_queue
        SET last_status = NULL,
            last_error  = $2,
            tries       = tries + 1,
            next_fetch_at = now() + make_interval(mins := $3),
            updated_at  = now()
        WHERE id = $1
        "#,
        &[&id, &err, &backoff_minutes],
    ).await?;
    Ok(())
}