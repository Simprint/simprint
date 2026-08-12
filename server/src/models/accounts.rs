use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::PlatformAccountDto;

/// 创建平台账号
pub async fn insert_platform_account(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    platform_url: &str,
    platform_name: Option<&str>,
    account: &str,
    password: Option<&str>,
    remark: Option<&str>,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO platform_accounts (user_uuid, team_uuid, platform_url, platform_name,
                                        account, password, remark)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING uuid;
        "#,
    )
    .bind(user_uuid)
    .bind(team_uuid)
    .bind(platform_url)
    .bind(platform_name)
    .bind(account)
    .bind(password)
    .bind(remark)
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

/// 查询平台账号列表
pub async fn fetch_platform_accounts(
    pool: &Pool<Postgres>,
    team_uuid: Option<Uuid>,
    user_uuid: Uuid,
    keyword: Option<&str>,
    platform_name: Option<&str>,
    status: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<PlatformAccountDto>, Error> {
    let keyword = keyword.map(|value| format!("%{}%", value.trim()));

    let recs = sqlx::query_as::<_, PlatformAccountDto>(
        r#"
        SELECT pa.id, pa.uuid, pa.user_uuid, pa.team_uuid, pa.platform_url, pa.platform_name,
               pa.account, pa.password, pa.status, pa.remark, pa.usage_count,
               (SELECT COUNT(*) FROM environment_accounts ea WHERE ea.account_uuid = pa.uuid) AS environments_count,
               pa.last_used_at, pa.created_at, pa.updated_at, pa.deleted_at
        FROM platform_accounts pa
        WHERE (pa.team_uuid = $1 OR (pa.team_uuid IS NULL AND pa.user_uuid = $2))
          AND (
            $3::text IS NULL
            OR pa.platform_url ILIKE $3
            OR COALESCE(pa.platform_name, '') ILIKE $3
            OR pa.account ILIKE $3
            OR COALESCE(pa.remark, '') ILIKE $3
          )
          AND ($4::varchar IS NULL OR pa.platform_name = $4)
          AND ($5::varchar IS NULL OR pa.status = $5)
          AND pa.deleted_at IS NULL
        ORDER BY pa.created_at DESC
        LIMIT $6 OFFSET $7
        "#,
    )
    .bind(team_uuid)
    .bind(user_uuid)
    .bind(keyword)
    .bind(platform_name)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询平台账号总数
pub async fn fetch_platform_accounts_count(
    pool: &Pool<Postgres>,
    team_uuid: Option<Uuid>,
    user_uuid: Uuid,
    keyword: Option<&str>,
    platform_name: Option<&str>,
    status: Option<&str>,
) -> Result<i64, Error> {
    let keyword = keyword.map(|value| format!("%{}%", value.trim()));

    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM platform_accounts
        WHERE (team_uuid = $1 OR (team_uuid IS NULL AND user_uuid = $2))
          AND (
            $3::text IS NULL
            OR platform_url ILIKE $3
            OR COALESCE(platform_name, '') ILIKE $3
            OR account ILIKE $3
            OR COALESCE(remark, '') ILIKE $3
          )
          AND ($4::varchar IS NULL OR platform_name = $4)
          AND ($5::varchar IS NULL OR status = $5)
          AND deleted_at IS NULL
        "#,
    )
    .bind(team_uuid)
    .bind(user_uuid)
    .bind(keyword)
    .bind(platform_name)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 根据 UUID 查询平台账号
pub async fn fetch_platform_account_by_uuid(
    pool: &Pool<Postgres>,
    account_uuid: Uuid,
) -> Result<Option<PlatformAccountDto>, Error> {
    let rec = sqlx::query_as::<_, PlatformAccountDto>(
        r#"
        SELECT pa.id, pa.uuid, pa.user_uuid, pa.team_uuid, pa.platform_url, pa.platform_name,
               pa.account, pa.password, pa.status, pa.remark, pa.usage_count,
               (SELECT COUNT(*) FROM environment_accounts ea WHERE ea.account_uuid = pa.uuid) AS environments_count,
               pa.last_used_at, pa.created_at, pa.updated_at, pa.deleted_at
        FROM platform_accounts pa
        WHERE pa.uuid = $1 AND pa.deleted_at IS NULL
        "#,
    )
    .bind(account_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 更新平台账号
pub async fn update_platform_account(
    pool: &Pool<Postgres>,
    account_uuid: Uuid,
    platform_url: Option<&str>,
    platform_name: Option<&str>,
    account: Option<&str>,
    password: Option<&str>,
    remark: Option<&str>,
    status: Option<&str>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE platform_accounts
        SET platform_url = COALESCE($1, platform_url),
            platform_name = COALESCE($2, platform_name),
            account = COALESCE($3, account),
            password = COALESCE($4, password),
            remark = COALESCE($5, remark),
            status = COALESCE($6, status)
        WHERE uuid = $7 AND deleted_at IS NULL
        "#,
    )
    .bind(platform_url)
    .bind(platform_name)
    .bind(account)
    .bind(password)
    .bind(remark)
    .bind(status)
    .bind(account_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 增加账号使用次数
pub async fn increment_account_usage(
    pool: &Pool<Postgres>,
    account_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE platform_accounts
        SET usage_count = usage_count + 1, last_used_at = CURRENT_TIMESTAMP
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(account_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 软删除平台账号
pub async fn delete_platform_account(
    pool: &Pool<Postgres>,
    account_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE platform_accounts SET deleted_at = CURRENT_TIMESTAMP
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(account_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 批量软删除平台账号
pub async fn batch_delete_platform_accounts(
    pool: &Pool<Postgres>,
    account_uuids: &[Uuid],
) -> Result<u64, Error> {
    let result = sqlx::query(
        r#"
        UPDATE platform_accounts SET deleted_at = CURRENT_TIMESTAMP
        WHERE uuid = ANY($1) AND deleted_at IS NULL
        "#,
    )
    .bind(account_uuids)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

// ============ Environment Accounts ============

/// 关联环境和账号
pub async fn insert_environment_account(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
    account_uuid: Uuid,
    sort_order: i32,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        INSERT INTO environment_accounts (environment_uuid, account_uuid, sort_order)
        VALUES ($1, $2, $3)
        ON CONFLICT (environment_uuid, account_uuid) DO UPDATE SET sort_order = $3;
        "#,
    )
    .bind(env_uuid)
    .bind(account_uuid)
    .bind(sort_order)
    .execute(pool)
    .await?;

    Ok(())
}

/// 移除环境和账号的关联
pub async fn remove_environment_account(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
    account_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        DELETE FROM environment_accounts
        WHERE environment_uuid = $1 AND account_uuid = $2;
        "#,
    )
    .bind(env_uuid)
    .bind(account_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 清空环境的所有账号关联
pub async fn clear_environment_accounts(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        DELETE FROM environment_accounts WHERE environment_uuid = $1;
        "#,
    )
    .bind(env_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 查询环境关联的所有账号
pub async fn fetch_environment_accounts(
    pool: &Pool<Postgres>,
    env_uuid: Uuid,
) -> Result<Vec<PlatformAccountDto>, Error> {
    let recs = sqlx::query_as::<_, PlatformAccountDto>(
        r#"
        SELECT pa.id, pa.uuid, pa.user_uuid, pa.team_uuid, pa.platform_url, pa.platform_name,
               pa.account, pa.password, pa.status, pa.remark, pa.usage_count,
               (SELECT COUNT(*) FROM environment_accounts ea2 WHERE ea2.account_uuid = pa.uuid) AS environments_count,
               pa.last_used_at, pa.created_at, pa.updated_at, pa.deleted_at
        FROM platform_accounts pa
        INNER JOIN environment_accounts ea ON pa.uuid = ea.account_uuid
        WHERE ea.environment_uuid = $1 AND pa.deleted_at IS NULL
        ORDER BY ea.sort_order
        "#,
    )
    .bind(env_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}
