use redis::aio::ConnectionManager;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::utils::{DatabaseConfig, IConfig, RedisConfig};

/// 服务上下文 - 管理共享资源
#[derive(Clone)]
pub struct SvcCtx {
    pub config: IConfig,
    pub db: Pool<Postgres>,
    pub redis: ConnectionManager,
}

impl SvcCtx {
    /// 创建服务上下文
    pub async fn new(config: &IConfig) -> Result<Self, anyhow::Error> {
        // 创建数据库连接池
        let db = Self::create_db(&config.database).await?;
        let redis = Self::create_redis(&config.redis).await?;

        Ok(Self {
            config: config.clone(),
            db,
            redis,
        })
    }

    /// 创建数据库连接
    pub async fn create_db(config: &DatabaseConfig) -> Result<Pool<Postgres>, anyhow::Error> {
        let pool = PgPoolOptions::new()
            .max_lifetime(std::time::Duration::from_secs(config.max_lifetime as u64)) // 连接最大生存时间, 5 分钟
            .idle_timeout(std::time::Duration::from_secs(config.idle_timeout as u64)) // 空闲超时时间, 30秒
            .acquire_timeout(std::time::Duration::from_secs(
                config.acquire_timeout as u64,
            )) // 添加获取连接的超时时间
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect(&config.url)
            .await?;

        Ok(pool)
    }

    /// 创建redis连接
    pub async fn create_redis(config: &RedisConfig) -> Result<ConnectionManager, anyhow::Error> {
        let client = redis::Client::open(config.url.as_str())?;

        let connection = ConnectionManager::new(client).await?;

        Ok(connection)
    }
}
