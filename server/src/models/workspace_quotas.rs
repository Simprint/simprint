use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::WorkspaceQuotaDto;

/// 创建或更新工作空间配额
pub async fn insert_or_update_workspace_quota(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    max_environments: i32,
    max_team_members: i32,
    max_proxies: i32,
    max_rpa_tasks: i32,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        INSERT INTO workspace_quotas (
            workspace_uuid, max_environments, max_team_members, max_proxies, max_rpa_tasks
        )
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (workspace_uuid) DO UPDATE SET
            max_environments = EXCLUDED.max_environments,
            max_team_members = EXCLUDED.max_team_members,
            max_proxies = EXCLUDED.max_proxies,
            max_rpa_tasks = EXCLUDED.max_rpa_tasks,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(workspace_uuid)
    .bind(max_environments)
    .bind(max_team_members)
    .bind(max_proxies)
    .bind(max_rpa_tasks)
    .execute(pool)
    .await?;

    Ok(())
}

/// 查询工作空间配额
pub async fn fetch_workspace_quota(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
) -> Result<Option<WorkspaceQuotaDto>, Error> {
    let rec = sqlx::query_as::<_, WorkspaceQuotaDto>(
        r#"
        SELECT workspace_uuid, max_environments, used_environments,
               max_team_members, used_team_members,
               max_proxies, used_proxies,
               max_rpa_tasks, used_rpa_tasks,
               created_at, updated_at
        FROM workspace_quotas
        WHERE workspace_uuid = $1
        "#,
    )
    .bind(workspace_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 增加环境使用数
pub async fn increment_used_environments(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    amount: i32,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE workspace_quotas
        SET used_environments = used_environments + $1,
            updated_at = CURRENT_TIMESTAMP
        WHERE workspace_uuid = $2
        "#,
    )
    .bind(amount)
    .bind(workspace_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 减少环境使用数
pub async fn decrement_used_environments(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    amount: i32,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE workspace_quotas
        SET used_environments = GREATEST(0, used_environments - $1),
            updated_at = CURRENT_TIMESTAMP
        WHERE workspace_uuid = $2
        "#,
    )
    .bind(amount)
    .bind(workspace_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 增加代理使用数
pub async fn increment_used_proxies(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    amount: i32,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE workspace_quotas
        SET used_proxies = used_proxies + $1,
            updated_at = CURRENT_TIMESTAMP
        WHERE workspace_uuid = $2
        "#,
    )
    .bind(amount)
    .bind(workspace_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 减少代理使用数
pub async fn decrement_used_proxies(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    amount: i32,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE workspace_quotas
        SET used_proxies = GREATEST(0, used_proxies - $1),
            updated_at = CURRENT_TIMESTAMP
        WHERE workspace_uuid = $2
        "#,
    )
    .bind(amount)
    .bind(workspace_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 更新团队成员使用数（统计所有团队的活跃成员）
pub async fn update_used_team_members(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE workspace_quotas wq
        SET used_team_members = (
            SELECT COUNT(DISTINCT tm.user_uuid)
            FROM team_members tm
            INNER JOIN teams t ON tm.team_uuid = t.uuid
            WHERE t.workspace_uuid = wq.workspace_uuid
              AND tm.deleted_at IS NULL
              AND t.deleted_at IS NULL
        ),
        updated_at = CURRENT_TIMESTAMP
        WHERE wq.workspace_uuid = $1
        "#,
    )
    .bind(workspace_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 检查配额是否充足
pub async fn check_quota(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    quota_type: &str,
) -> Result<bool, Error> {
    let result = match quota_type {
        "environments" => sqlx::query_scalar::<_, bool>(
            r#"
                SELECT used_environments < max_environments
                FROM workspace_quotas
                WHERE workspace_uuid = $1
                "#,
        )
        .bind(workspace_uuid)
        .fetch_optional(pool)
        .await?
        .unwrap_or(false),
        "proxies" => sqlx::query_scalar::<_, bool>(
            r#"
                SELECT used_proxies < max_proxies
                FROM workspace_quotas
                WHERE workspace_uuid = $1
                "#,
        )
        .bind(workspace_uuid)
        .fetch_optional(pool)
        .await?
        .unwrap_or(false),
        "team_members" => sqlx::query_scalar::<_, bool>(
            r#"
                SELECT used_team_members < max_team_members
                FROM workspace_quotas
                WHERE workspace_uuid = $1
                "#,
        )
        .bind(workspace_uuid)
        .fetch_optional(pool)
        .await?
        .unwrap_or(false),
        "rpa_tasks" => sqlx::query_scalar::<_, bool>(
            r#"
                SELECT used_rpa_tasks < max_rpa_tasks
                FROM workspace_quotas
                WHERE workspace_uuid = $1
                "#,
        )
        .bind(workspace_uuid)
        .fetch_optional(pool)
        .await?
        .unwrap_or(false),
        _ => false,
    };

    Ok(result)
}
