use crate::{
    caches::CacheStore,
    database::{self, DbPool},
    utils::{DatabaseConfig, IConfig},
};

/// Shared resources used by handlers and services.
#[derive(Clone)]
pub struct SvcCtx {
    pub config: IConfig,
    pub db: DbPool,
    pub cache: CacheStore,
}

impl SvcCtx {
    pub async fn new(config: &IConfig) -> Result<Self, anyhow::Error> {
        let db = Self::create_db(&config.database).await?;
        let cache = CacheStore::memory();

        Ok(Self {
            config: config.clone(),
            db,
            cache,
        })
    }

    pub async fn create_db(config: &DatabaseConfig) -> Result<DbPool, anyhow::Error> {
        database::connect(config).await
    }
}
