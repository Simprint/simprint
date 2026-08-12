use chrono::Utc;
use sqlx::{Pool, Postgres};

use crate::{dto::maintenance::Maintenance, entitys::maintenance::CreateMaintenanceRequest};

pub async fn create_maintenance(
    pool: &Pool<Postgres>,
    request: CreateMaintenanceRequest,
) -> Result<Maintenance, anyhow::Error> {
    let now = Utc::now();

    // 开始事务
    let mut tx = pool.begin().await?;

    // 关闭所有其他活跃的维护
    sqlx::query(
        r#"
        UPDATE maintenances
        SET status = 'inactive', updated_at = $1
        WHERE status = 'active'
        "#,
    )
    .bind(now)
    .execute(&mut *tx)
    .await?;

    // 创建新的维护记录
    let maintenance = sqlx::query_as::<_, Maintenance>(
        r#"
        INSERT INTO maintenances (name, description, status, start_time, end_time, maintenance_type, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, name, description, status, start_time, end_time, maintenance_type, created_at, updated_at
        "#
    )
    .bind(request.name)
    .bind(request.description)
    .bind(request.status)
    .bind(request.start_time)
    .bind(request.end_time)
    .bind(String::from(request.maintenance_type))
    .bind(now)
    .bind(now)
    .fetch_one(&mut *tx)
    .await?;

    // 提交事务
    tx.commit().await?;

    Ok(maintenance)
}

pub async fn list_maintenances(
    pool: &Pool<Postgres>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Maintenance>, sqlx::Error> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let maintenances = sqlx::query_as::<_, Maintenance>(
        r#"
        SELECT id, name, description, status, start_time, end_time, maintenance_type,
               created_at, updated_at
        FROM maintenances
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(maintenances)
}

/// 结束维护
pub async fn end_maintenance(pool: &Pool<Postgres>) -> Result<bool, anyhow::Error> {
    let now = Utc::now();

    // 关闭所有其他活跃的维护
    sqlx::query(
        r#"
        UPDATE maintenances
        SET status = 'inactive', updated_at = $1
        WHERE status = 'active'
        "#,
    )
    .bind(now)
    .execute(pool)
    .await?;

    Ok(true)
}

pub async fn update_maintenance_status(
    pool: &Pool<Postgres>,
    id: i64,
    status: &str,
) -> Result<bool, anyhow::Error> {
    let now = Utc::now();
    let result = sqlx::query(
        r#"
        UPDATE maintenances
        SET status = $1, updated_at = $2
        WHERE id = $3
        "#,
    )
    .bind(status)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_maintenance_by_id(
    pool: &Pool<Postgres>,
    id: i64,
) -> Result<Option<Maintenance>, anyhow::Error> {
    let maintenance = sqlx::query_as::<_, Maintenance>(
        r#"
        SELECT id, name, description, status, start_time, end_time, maintenance_type,
               created_at, updated_at
        FROM maintenances
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(maintenance)
}

pub async fn get_active_maintenances(
    pool: &Pool<Postgres>,
) -> Result<Option<Maintenance>, anyhow::Error> {
    let maintenance = sqlx::query_as::<_, Maintenance>(
        r#"
        SELECT id, name, description, status, start_time, end_time, maintenance_type,
               created_at, updated_at
        FROM maintenances
        WHERE status = 'active' AND start_time <= NOW() AND end_time >= NOW()
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?;

    Ok(maintenance)
}

pub async fn create_maintenances_table(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS maintenances (
            id BIGSERIAL PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            description TEXT,
            status VARCHAR(20) NOT NULL,
            start_time TIMESTAMPTZ NOT NULL,
            end_time TIMESTAMPTZ NOT NULL,
            maintenance_type VARCHAR(20) NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_maintenances_status ON maintenances(status);
        CREATE INDEX IF NOT EXISTS idx_maintenances_time ON maintenances(start_time, end_time);
        CREATE INDEX IF NOT EXISTS idx_maintenances_type ON maintenances(maintenance_type);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
