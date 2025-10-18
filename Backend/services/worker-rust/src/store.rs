use anyhow::{Context, Result};
use deadpool_postgres::{ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;

pub fn pool_from_env() -> Result<Pool> {
    let pg_url = std::env::var("PG_URL").context("PG_URL not set")?;

    let mut cfg = deadpool_postgres::Config::new();
    cfg.url = Some(pg_url);
    // (optional) tweak manager
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = cfg.create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls)?;
    Ok(pool)
}