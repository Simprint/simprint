use sqlx::{Postgres, postgres::PgPoolOptions};

use crate::utils::DatabaseConfig;

/// Database engine currently used by the imported service models.
///
/// Keeping this alias in one module gives the SQLite migration a single
/// boundary while the PostgreSQL-specific queries are converted domain by
/// domain.
pub type Db = Postgres;
pub type Pool<T = Db> = sqlx::Pool<T>;
pub type DbPool = Pool<Db>;

pub async fn connect(config: &DatabaseConfig) -> anyhow::Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_lifetime(std::time::Duration::from_secs(config.max_lifetime))
        .idle_timeout(std::time::Duration::from_secs(config.idle_timeout))
        .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout))
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect(&config.url)
        .await?;

    Ok(pool)
}
