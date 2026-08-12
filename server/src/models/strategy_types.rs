use crate::dto::strategy_types::StrategyType;
use sqlx::{Error, Pool, Postgres};

/// 根据ID查询策略类型
pub async fn query_strategy_type_by_id(
    pool: &Pool<Postgres>,
    id: i32,
) -> Result<StrategyType, Error> {
    let strategy_type: StrategyType = sqlx::query_as("SELECT * FROM strategy_types WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(strategy_type)
}

/// 根据code查询策略类型
pub async fn query_strategy_type_by_code(
    pool: &Pool<Postgres>,
    code: &str,
) -> Result<StrategyType, Error> {
    let strategy_type: StrategyType =
        sqlx::query_as("SELECT * FROM strategy_types WHERE code = $1")
            .bind(code)
            .fetch_one(pool)
            .await?;

    Ok(strategy_type)
}

/// 查询所有可用的策略类型（只查询激活的）
pub async fn query_available_strategy_types(
    pool: &Pool<Postgres>,
) -> Result<Vec<StrategyType>, Error> {
    let strategy_types: Vec<StrategyType> =
        sqlx::query_as("SELECT * FROM strategy_types WHERE is_active = true ORDER BY code")
            .fetch_all(pool)
            .await?;

    Ok(strategy_types)
}

/// 根据分类查询策略类型
pub async fn query_strategy_types_by_category(
    pool: &Pool<Postgres>,
    category: &str,
) -> Result<Vec<StrategyType>, Error> {
    let strategy_types: Vec<StrategyType> = sqlx::query_as(
        "SELECT * FROM strategy_types WHERE category = $1 AND is_active = true ORDER BY code",
    )
    .bind(category)
    .fetch_all(pool)
    .await?;

    Ok(strategy_types)
}
