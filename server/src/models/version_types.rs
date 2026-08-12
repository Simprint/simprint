use crate::dto::version_types::VersionType;
use crate::entitys::version_types::{CreateVersionTypeRequest, UpdateVersionTypeRequest};
use sqlx::{Error, Pool, Postgres};

/// 插入新版本类型
pub async fn insert_version_type(
    pool: &Pool<Postgres>,
    request: &CreateVersionTypeRequest,
) -> Result<i32, Error> {
    let sql = "
        INSERT INTO version_types (
            type_code, type_name, description, sort_order, is_active
        ) VALUES (
            $1, $2, $3, $4, true
        ) RETURNING id
    ";

    let result: (i32,) = sqlx::query_as(sql)
        .bind(&request.type_code)
        .bind(&request.type_name)
        .bind(&request.description)
        .bind(request.sort_order.unwrap_or(0))
        .fetch_one(pool)
        .await?;

    Ok(result.0)
}

/// 根据ID查询版本类型
pub async fn query_version_type_by_id(
    pool: &Pool<Postgres>,
    id: i32,
) -> Result<VersionType, Error> {
    let version_type: VersionType = sqlx::query_as("SELECT * FROM version_types WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(version_type)
}

/// 根据代码查询版本类型
pub async fn query_version_type_by_code(
    pool: &Pool<Postgres>,
    type_code: &str,
) -> Result<VersionType, Error> {
    let version_type: VersionType =
        sqlx::query_as("SELECT * FROM version_types WHERE type_code = $1")
            .bind(type_code)
            .fetch_one(pool)
            .await?;

    Ok(version_type)
}

/// 查询所有版本类型
pub async fn query_all_version_types(pool: &Pool<Postgres>) -> Result<Vec<VersionType>, Error> {
    let version_types: Vec<VersionType> =
        sqlx::query_as("SELECT * FROM version_types ORDER BY sort_order ASC, id ASC")
            .fetch_all(pool)
            .await?;

    Ok(version_types)
}

/// 查询激活的版本类型
pub async fn query_active_version_types(pool: &Pool<Postgres>) -> Result<Vec<VersionType>, Error> {
    let version_types: Vec<VersionType> = sqlx::query_as(
        "SELECT * FROM version_types WHERE is_active = true ORDER BY sort_order ASC, id ASC",
    )
    .fetch_all(pool)
    .await?;

    Ok(version_types)
}

/// 更新版本类型
pub async fn update_version_type(
    pool: &Pool<Postgres>,
    id: i32,
    request: &UpdateVersionTypeRequest,
) -> Result<bool, Error> {
    let sql = "
        UPDATE version_types SET
            type_name = COALESCE($1, type_name),
            description = COALESCE($2, description),
            sort_order = COALESCE($3, sort_order),
            is_active = COALESCE($4, is_active)
        WHERE id = $5
    ";

    let row = sqlx::query(sql)
        .bind(&request.type_name)
        .bind(&request.description)
        .bind(request.sort_order)
        .bind(request.is_active)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(row.rows_affected() == 1)
}

/// 删除版本类型
pub async fn delete_version_type(pool: &Pool<Postgres>, id: i32) -> Result<bool, Error> {
    let sql = "DELETE FROM version_types WHERE id = $1";
    let row = sqlx::query(sql).bind(id).execute(pool).await?;
    Ok(row.rows_affected() == 1)
}

/// 激活/停用版本类型
pub async fn toggle_version_type_status(
    pool: &Pool<Postgres>,
    id: i32,
    is_active: bool,
) -> Result<bool, Error> {
    let sql = "UPDATE version_types SET is_active = $1 WHERE id = $2";
    let row = sqlx::query(sql)
        .bind(is_active)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(row.rows_affected() == 1)
}
