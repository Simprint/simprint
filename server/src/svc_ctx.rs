use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::{
    caches::CacheStore,
    utils::{DatabaseConfig, IConfig},
};

/// Shared resources used by handlers and services.
#[derive(Clone)]
pub struct SvcCtx {
    pub config: IConfig,
    pub db: Pool<Postgres>,
    pub cache: CacheStore,
}

impl SvcCtx {
    pub async fn new(config: &IConfig) -> Result<Self, anyhow::Error> {
        let db = Self::create_db(&config.database).await?;
        let cache = if let Some(redis) = &config.redis {
            CacheStore::redis(&redis.url).await?
        } else {
            CacheStore::memory()
        };

        Ok(Self {
            config: config.clone(),
            db,
            cache,
        })
    }

    pub async fn create_db(config: &DatabaseConfig) -> Result<Pool<Postgres>, anyhow::Error> {
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
}
