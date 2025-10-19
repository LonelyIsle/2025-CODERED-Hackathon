use anyhow::Result;
use deadpool_postgres::{Manager, ManagerConfig, RecyclingMethod, Pool};
use tokio_postgres::NoTls;
use tokio_postgres::types::ToSql;

use crate::types::Document;

pub type PgPool = Pool;

pub async fn init_pool(pg_url: &str) -> Result<PgPool> {
    let cfg = pg_url.parse::<tokio_postgres::Config>()?;
    let mgr = Manager::from_config(cfg, NoTls, ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    let pool = Pool::builder(mgr).max_size(8).build()?;
    Ok(pool)
}

/// Create tables if they don't exist (idempotent).
pub async fn ensure_tables(pool: &PgPool) -> Result<()> {
    let client = pool.get().await?;
    // documents table
    client.batch_execute(r#"
        CREATE TABLE IF NOT EXISTS public.documents (
            url TEXT PRIMARY KEY,
            fetched_at TIMESTAMPTZ NOT NULL,
            title TEXT NULL,
            description TEXT NULL,
            body_text TEXT NOT NULL,
            content_type TEXT NULL,
            http_status INTEGER NOT NULL,
            content_hash TEXT NULL,
            etag TEXT NULL,
            lang TEXT NULL,
            last_modified TEXT NULL
        );
    "#).await?;

    // crawl_queue table used by crawler helpers
    client.batch_execute(r#"
        CREATE TABLE IF NOT EXISTS public.crawl_queue (
            id BIGSERIAL PRIMARY KEY,
            url TEXT NOT NULL UNIQUE,
            priority INTEGER NOT NULL DEFAULT 100,
            attempts INTEGER NOT NULL DEFAULT 0,
            next_fetch_at TIMESTAMPTZ NOT NULL DEFAULT now()
        );
    "#).await?;

    Ok(())
}

/// Upsert a document by URL.
pub async fn upsert_document(pool: &PgPool, d: &Document) -> Result<()> {
    let client = pool.get().await?;
    let params: [&(dyn ToSql + Sync); 11] = [
        &d.url,
        &d.fetched_at,
        &d.title,
        &d.description,
        &d.body_text,
        &d.content_type,
        &d.http_status,
        &d.content_hash,
        &d.etag,
        &d.lang,
        &d.last_modified,
    ];

    client.execute(
        r#"
        INSERT INTO public.documents
            (url, fetched_at, title, description, body_text, content_type, http_status, content_hash, etag, lang, last_modified)
        VALUES
            ($1,  $2,        $3,   $4,          $5,        $6,          $7,          $8,          $9,  $10, $11)
        ON CONFLICT (url) DO UPDATE SET
            fetched_at = EXCLUDED.fetched_at,
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            body_text = EXCLUDED.body_text,
            content_type = EXCLUDED.content_type,
            http_status = EXCLUDED.http_status,
            content_hash = EXCLUDED.content_hash,
            etag = EXCLUDED.etag,
            lang = EXCLUDED.lang,
            last_modified = EXCLUDED.last_modified
        "#,
        &params,
    ).await?;

    Ok(())
}

/// Enqueue a URL into the crawl queue (creates if missing).
pub async fn enqueue_url(pool: &PgPool, url: &str, priority: i32) -> Result<()> {
    let client = pool.get().await?;
    client.execute(
        r#"
        INSERT INTO public.crawl_queue (url, priority)
        VALUES ($1, $2)
        ON CONFLICT (url) DO NOTHING
        "#,
        &[&url, &priority],
    ).await?;
    Ok(())
}

/// Dequeue a batch of due URLs.
pub async fn dequeue_batch(pool: &PgPool, batch: i64) -> Result<Vec<(i64, String)>> {
    let client = pool.get().await?;
    let rows = client.query(
        r#"
        SELECT id, url FROM public.crawl_queue
        WHERE next_fetch_at <= now()
        ORDER BY priority ASC, next_fetch_at ASC
        LIMIT $1
        "#,
        &[&batch],
    ).await?;

    // mark attempt and push next_fetch_at a bit
    for r in &rows {
        let id: i64 = r.get(0);
        client.execute(
            r#"
            UPDATE public.crawl_queue
            SET attempts = attempts + 1,
                next_fetch_at = now() + interval '5 minutes'
            WHERE id = $1
            "#,
            &[&id],
        ).await?;
    }

    Ok(rows.into_iter().map(|r| (r.get(0), r.get(1))).collect())
}

/// Back off a URL for N minutes.
pub async fn mark_backoff(pool: &PgPool, url: &str, minutes: i64) -> Result<()> {
    let client = pool.get().await?;
    client.execute(
        r#"
        UPDATE public.crawl_queue
        SET next_fetch_at = now() + ($2 || ' minutes')::interval
        WHERE url = $1
        "#,
        &[&url, &minutes],
    ).await?;
    Ok(())
}

// -------- Optional helpers (compile-safe stubs) --------

#[derive(Debug)]
pub struct QueueItem {
    pub id: i64,
    pub url: String,
}

pub async fn dequeue_due(pool: &PgPool, batch: i64) -> Result<Vec<QueueItem>> {
    let client = pool.get().await?;
    let rows = client.query(
        r#"
        SELECT id, url FROM public.crawl_queue
        WHERE next_fetch_at <= now()
        ORDER BY priority ASC, next_fetch_at ASC
        LIMIT $1
        "#,
        &[&batch],
    ).await?;

    for r in &rows {
        let id: i64 = r.get(0);
        client.execute(
            r#"
            UPDATE public.crawl_queue
            SET attempts = attempts + 1,
                next_fetch_at = now() + interval '5 minutes'
            WHERE id = $1
            "#,
            &[&id],
        ).await?;
    }

    Ok(rows.into_iter().map(|r| QueueItem { id: r.get(0), url: r.get(1) }).collect())
}

pub async fn reschedule_success(pool: &PgPool, id: i64) -> Result<()> {
    let client = pool.get().await?;
    client.execute(
        r#"
        UPDATE public.crawl_queue
        SET next_fetch_at = now() + interval '1 day'
        WHERE id = $1
        "#,
        &[&id],
    ).await?;
    Ok(())
}

pub async fn reschedule_failure(pool: &PgPool, id: i64) -> Result<()> {
    let client = pool.get().await?;
    client.execute(
        r#"
        UPDATE public.crawl_queue
        SET next_fetch_at = now() + interval '6 hours'
        WHERE id = $1
        "#,
        &[&id],
    ).await?;
    Ok(())
}