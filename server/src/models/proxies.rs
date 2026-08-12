use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::{ProxyDto, ProxyHealthCheckDto};

/// 创建代理
pub async fn insert_proxy(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    owner_uuid: Uuid,
    name: &str,
    host: &str,
    port: i32,
    proxy_type: &str,
    username: Option<&str>,
    password: Option<&str>,
    country: Option<&str>,
    city: Option<&str>,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO proxies (workspace_uuid, owner_uuid, name, host, port, proxy_type,
                              username, password, country, city)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING uuid;
        "#,
    )
    .bind(workspace_uuid)
    .bind(owner_uuid)
    .bind(name)
    .bind(host)
    .bind(port)
    .bind(proxy_type)
    .bind(username)
    .bind(password)
    .bind(country)
    .bind(city)
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

/// 查询代理列表
pub async fn fetch_proxies(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    proxy_type: Option<&str>,
    status: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<ProxyDto>, Error> {
    let recs = sqlx::query_as::<_, ProxyDto>(
        r#"
        SELECT p.id, p.uuid, p.workspace_uuid, p.owner_uuid, p.name, p.host, p.port, p.proxy_type,
               p.username, p.password, p.ssh_key_encrypted, p.ssh_passphrase_encrypted,
               p.country, p.city, p.status, p.latency, p.last_check_ip, p.last_checked_at,
               (SELECT COUNT(*) FROM environments e WHERE e.proxy_uuid = p.uuid AND e.deleted_at IS NULL) AS environments_count,
               p.created_at, p.updated_at, p.deleted_at
        FROM proxies p
        WHERE p.workspace_uuid = $1
          AND ($2::varchar IS NULL OR p.proxy_type = $2)
          AND ($3::varchar IS NULL OR p.status = $3)
          AND p.deleted_at IS NULL
        ORDER BY p.created_at DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(workspace_uuid)
    .bind(proxy_type)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询代理总数
pub async fn fetch_proxies_count(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    proxy_type: Option<&str>,
    status: Option<&str>,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM proxies
        WHERE workspace_uuid = $1
          AND ($2::varchar IS NULL OR proxy_type = $2)
          AND ($3::varchar IS NULL OR status = $3)
          AND deleted_at IS NULL
        "#,
    )
    .bind(workspace_uuid)
    .bind(proxy_type)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 根据 UUID 查询代理
pub async fn fetch_proxy_by_uuid(
    pool: &Pool<Postgres>,
    proxy_uuid: Uuid,
) -> Result<Option<ProxyDto>, Error> {
    let rec = sqlx::query_as::<_, ProxyDto>(
        r#"
        SELECT p.id, p.uuid, p.workspace_uuid, p.owner_uuid, p.name, p.host, p.port, p.proxy_type,
               p.username, p.password, p.ssh_key_encrypted, p.ssh_passphrase_encrypted,
               p.country, p.city, p.status, p.latency, p.last_check_ip, p.last_checked_at,
               (SELECT COUNT(*) FROM environments e WHERE e.proxy_uuid = p.uuid AND e.deleted_at IS NULL) AS environments_count,
               p.created_at, p.updated_at, p.deleted_at
        FROM proxies p
        WHERE p.uuid = $1 AND p.deleted_at IS NULL
        "#,
    )
    .bind(proxy_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 更新代理
pub async fn update_proxy(
    pool: &Pool<Postgres>,
    proxy_uuid: Uuid,
    name: Option<&str>,
    host: Option<&str>,
    port: Option<i32>,
    proxy_type: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
    country: Option<&str>,
    city: Option<&str>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE proxies
        SET name = COALESCE($1, name),
            host = COALESCE($2, host),
            port = COALESCE($3, port),
            proxy_type = COALESCE($4, proxy_type),
            username = COALESCE($5, username),
            password = COALESCE($6, password),
            country = COALESCE($7, country),
            city = COALESCE($8, city)
        WHERE uuid = $9 AND deleted_at IS NULL
        "#,
    )
    .bind(name)
    .bind(host)
    .bind(port)
    .bind(proxy_type)
    .bind(username)
    .bind(password)
    .bind(country)
    .bind(city)
    .bind(proxy_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 更新代理检测结果
pub async fn update_proxy_check_result(
    pool: &Pool<Postgres>,
    proxy_uuid: Uuid,
    status: &str,
    latency: Option<i32>,
    ip_address: Option<&str>,
    country: Option<&str>,
    city: Option<&str>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE proxies
        SET status = $1,
            latency = $2,
            last_check_ip = $3,
            country = $4,
            city = $5,
            last_checked_at = CURRENT_TIMESTAMP
        WHERE uuid = $6 AND deleted_at IS NULL
        "#,
    )
    .bind(status)
    .bind(latency)
    .bind(ip_address)
    .bind(country)
    .bind(city)
    .bind(proxy_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 增加代理使用次数
pub async fn increment_proxy_usage(pool: &Pool<Postgres>, proxy_uuid: Uuid) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE proxies SET usage_count = usage_count + 1
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(proxy_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 软删除代理
pub async fn delete_proxy(pool: &Pool<Postgres>, proxy_uuid: Uuid) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE proxies SET deleted_at = CURRENT_TIMESTAMP
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(proxy_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 批量软删除代理
pub async fn batch_delete_proxies(
    pool: &Pool<Postgres>,
    proxy_uuids: &[Uuid],
) -> Result<u64, Error> {
    let result = sqlx::query(
        r#"
        UPDATE proxies SET deleted_at = CURRENT_TIMESTAMP
        WHERE uuid = ANY($1) AND deleted_at IS NULL
        "#,
    )
    .bind(proxy_uuids)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

// ============ Proxy Health Checks ============

/// 记录代理健康检查
pub async fn insert_proxy_health_check(
    pool: &Pool<Postgres>,
    proxy_uuid: Uuid,
    status: &str,
    latency: Option<i32>,
    ip_address: Option<&str>,
    error_message: Option<&str>,
) -> Result<i64, Error> {
    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO proxy_health_checks (proxy_uuid, status, latency, ip_address, error_message)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id;
        "#,
    )
    .bind(proxy_uuid)
    .bind(status)
    .bind(latency)
    .bind(ip_address)
    .bind(error_message)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// 查询代理健康检查历史
pub async fn fetch_proxy_health_checks(
    pool: &Pool<Postgres>,
    proxy_uuid: Uuid,
    limit: i64,
) -> Result<Vec<ProxyHealthCheckDto>, Error> {
    let recs = sqlx::query_as::<_, ProxyHealthCheckDto>(
        r#"
        SELECT id, proxy_uuid, status, latency, ip_address, error_message, checked_at
        FROM proxy_health_checks
        WHERE proxy_uuid = $1
        ORDER BY checked_at DESC
        LIMIT $2
        "#,
    )
    .bind(proxy_uuid)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}
