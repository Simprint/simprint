use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::{ProxyDto, ProxyVisibleTeamDto};

/// 添加代理可见团队
pub async fn insert_proxy_visible_team(
    pool: &Pool<Postgres>,
    proxy_uuid: Uuid,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        INSERT INTO proxy_visible_teams (proxy_uuid, workspace_uuid, team_uuid)
        VALUES ($1, $2, $3)
        ON CONFLICT (proxy_uuid, team_uuid) DO NOTHING
        "#,
    )
    .bind(proxy_uuid)
    .bind(workspace_uuid)
    .bind(team_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 移除代理可见团队
pub async fn remove_proxy_visible_team(
    pool: &Pool<Postgres>,
    proxy_uuid: Uuid,
    team_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        DELETE FROM proxy_visible_teams
        WHERE proxy_uuid = $1 AND team_uuid = $2
        "#,
    )
    .bind(proxy_uuid)
    .bind(team_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 查询代理的可见团队列表
pub async fn fetch_visible_teams_by_proxy(
    pool: &Pool<Postgres>,
    proxy_uuid: Uuid,
) -> Result<Vec<ProxyVisibleTeamDto>, Error> {
    let recs = sqlx::query_as::<_, ProxyVisibleTeamDto>(
        r#"
        SELECT proxy_uuid, workspace_uuid, team_uuid, created_at
        FROM proxy_visible_teams
        WHERE proxy_uuid = $1
        ORDER BY created_at
        "#,
    )
    .bind(proxy_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询团队可见的代理列表
pub async fn fetch_visible_proxies_by_team(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
) -> Result<Vec<ProxyDto>, Error> {
    let recs = sqlx::query_as::<_, ProxyDto>(
        r#"
        SELECT p.id, p.uuid, p.workspace_uuid, p.owner_uuid, p.name, p.host, p.port, p.proxy_type,
               p.username, p.password, p.ssh_key_encrypted, p.ssh_passphrase_encrypted,
               p.country, p.city, p.status, p.latency, p.last_check_ip, p.last_checked_at,
               (SELECT COUNT(*) FROM environments e WHERE e.proxy_uuid = p.uuid AND e.deleted_at IS NULL) AS environments_count,
               p.created_at, p.updated_at, p.deleted_at
        FROM proxies p
        INNER JOIN proxy_visible_teams pvt ON p.uuid = pvt.proxy_uuid
        WHERE pvt.workspace_uuid = $1 AND pvt.team_uuid = $2
          AND p.deleted_at IS NULL
        ORDER BY p.created_at DESC
        "#,
    )
    .bind(workspace_uuid)
    .bind(team_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 检查代理对团队是否可见
pub async fn check_proxy_visibility(
    pool: &Pool<Postgres>,
    proxy_uuid: Uuid,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
) -> Result<bool, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM proxy_visible_teams
        WHERE proxy_uuid = $1 AND workspace_uuid = $2 AND team_uuid = $3
        "#,
    )
    .bind(proxy_uuid)
    .bind(workspace_uuid)
    .bind(team_uuid)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

/// 查询工作空间所有可见的代理（包括工作空间 Owner 和代理所有者的代理）
pub async fn fetch_visible_proxies_for_user(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
) -> Result<Vec<ProxyDto>, Error> {
    // 工作空间 Owner 可以看到所有代理
    // 代理所有者可以看到自己的代理
    // 团队成员可以看到 proxy_visible_teams 中包含其团队的代理
    let recs = sqlx::query_as::<_, ProxyDto>(
        r#"
        SELECT DISTINCT p.id, p.uuid, p.workspace_uuid, p.owner_uuid, p.name, p.host, p.port, p.proxy_type,
               p.username, p.password, p.ssh_key_encrypted, p.ssh_passphrase_encrypted,
               p.country, p.city, p.status, p.latency, p.last_check_ip, p.last_checked_at,
               (SELECT COUNT(*) FROM environments e WHERE e.proxy_uuid = p.uuid AND e.deleted_at IS NULL) AS environments_count,
               p.created_at, p.updated_at, p.deleted_at
        FROM proxies p
        WHERE p.workspace_uuid = $1
          AND p.deleted_at IS NULL
          AND (
            -- 工作空间 Owner
            EXISTS (
                SELECT 1 FROM workspaces w
                WHERE w.uuid = $1 AND w.owner_uuid = $2 AND w.deleted_at IS NULL
            )
            -- 代理所有者
            OR p.owner_uuid = $2
            -- 团队成员可见的代理
            OR (
                $3 IS NOT NULL
                AND EXISTS (
                    SELECT 1 FROM proxy_visible_teams pvt
                    WHERE pvt.proxy_uuid = p.uuid
                      AND pvt.workspace_uuid = $1
                      AND pvt.team_uuid = $3
                )
            )
          )
        ORDER BY p.created_at DESC
        "#,
    )
    .bind(workspace_uuid)
    .bind(user_uuid)
    .bind(team_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 分页查询用户可见的代理列表，并支持名称搜索和筛选
pub async fn fetch_visible_proxies_for_user_paginated(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    name_keyword: Option<&str>,
    proxy_type: Option<&str>,
    status: Option<&str>,
    country: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<ProxyDto>, Error> {
    let name_keyword = name_keyword.map(|keyword| format!("%{}%", keyword.trim()));

    let recs = sqlx::query_as::<_, ProxyDto>(
        r#"
        SELECT DISTINCT p.id, p.uuid, p.workspace_uuid, p.owner_uuid, p.name, p.host, p.port, p.proxy_type,
               p.username, p.password, p.ssh_key_encrypted, p.ssh_passphrase_encrypted,
               p.country, p.city, p.status, p.latency, p.last_check_ip, p.last_checked_at,
               (SELECT COUNT(*) FROM environments e WHERE e.proxy_uuid = p.uuid AND e.deleted_at IS NULL) AS environments_count,
               p.created_at, p.updated_at, p.deleted_at
        FROM proxies p
        WHERE p.workspace_uuid = $1
          AND p.deleted_at IS NULL
          AND ($4::text IS NULL OR p.name ILIKE $4)
          AND ($5::text IS NULL OR p.proxy_type = $5)
          AND ($6::text IS NULL OR p.status = $6)
          AND ($7::text IS NULL OR p.country = $7)
          AND (
            EXISTS (
                SELECT 1 FROM workspaces w
                WHERE w.uuid = $1 AND w.owner_uuid = $2 AND w.deleted_at IS NULL
            )
            OR p.owner_uuid = $2
            OR (
                $3 IS NOT NULL
                AND EXISTS (
                    SELECT 1 FROM proxy_visible_teams pvt
                    WHERE pvt.proxy_uuid = p.uuid
                      AND pvt.workspace_uuid = $1
                      AND pvt.team_uuid = $3
                )
            )
          )
        ORDER BY p.created_at DESC
        OFFSET $8 LIMIT $9
        "#,
    )
    .bind(workspace_uuid)
    .bind(user_uuid)
    .bind(team_uuid)
    .bind(name_keyword)
    .bind(proxy_type)
    .bind(status)
    .bind(country)
    .bind(offset)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询用户可见代理总数，并支持名称搜索和筛选
pub async fn fetch_visible_proxies_for_user_count(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    name_keyword: Option<&str>,
    proxy_type: Option<&str>,
    status: Option<&str>,
    country: Option<&str>,
) -> Result<i64, Error> {
    let name_keyword = name_keyword.map(|keyword| format!("%{}%", keyword.trim()));

    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(DISTINCT p.uuid)
        FROM proxies p
        WHERE p.workspace_uuid = $1
          AND p.deleted_at IS NULL
          AND ($4::text IS NULL OR p.name ILIKE $4)
          AND ($5::text IS NULL OR p.proxy_type = $5)
          AND ($6::text IS NULL OR p.status = $6)
          AND ($7::text IS NULL OR p.country = $7)
          AND (
            EXISTS (
                SELECT 1 FROM workspaces w
                WHERE w.uuid = $1 AND w.owner_uuid = $2 AND w.deleted_at IS NULL
            )
            OR p.owner_uuid = $2
            OR (
                $3 IS NOT NULL
                AND EXISTS (
                    SELECT 1 FROM proxy_visible_teams pvt
                    WHERE pvt.proxy_uuid = p.uuid
                      AND pvt.workspace_uuid = $1
                      AND pvt.team_uuid = $3
                )
            )
          )
        "#,
    )
    .bind(workspace_uuid)
    .bind(user_uuid)
    .bind(team_uuid)
    .bind(name_keyword)
    .bind(proxy_type)
    .bind(status)
    .bind(country)
    .fetch_one(pool)
    .await?;

    Ok(count)
}
