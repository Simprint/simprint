use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::WorkspaceDto;
use crate::entitys::CreateWorkspaceRequest;

/// 创建工作空间
pub async fn insert_workspace(
    pool: &Pool<Postgres>,
    owner_uuid: Uuid,
    payload: &CreateWorkspaceRequest,
) -> Result<Uuid, Error> {
    let workspace_uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO workspaces (name, owner_uuid, workspace_type)
        VALUES ($1, $2, $3)
        RETURNING uuid;
        "#,
    )
    .bind(&payload.name)
    .bind(owner_uuid)
    .bind(payload.workspace_type.as_deref().unwrap_or("personal"))
    .fetch_one(pool)
    .await?;

    Ok(workspace_uuid)
}

/// 根据 UUID 查询工作空间
pub async fn fetch_workspace_by_uuid(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
) -> Result<Option<WorkspaceDto>, Error> {
    let rec = sqlx::query_as::<_, WorkspaceDto>(
        r#"
        SELECT uuid, name, owner_uuid, workspace_type, created_at, updated_at, deleted_at
        FROM workspaces
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(workspace_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 查询用户所属的所有工作空间
pub async fn fetch_user_workspaces(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<Vec<WorkspaceDto>, Error> {
    let recs = sqlx::query_as::<_, WorkspaceDto>(
        r#"
        SELECT uuid, name, owner_uuid, workspace_type, created_at, updated_at, deleted_at
        FROM workspaces
        WHERE owner_uuid = $1 AND deleted_at IS NULL
        ORDER BY created_at
        "#,
    )
    .bind(user_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 更新工作空间
pub async fn update_workspace(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    name: Option<&str>,
) -> Result<(), Error> {
    if let Some(name) = name {
        sqlx::query(
            r#"
            UPDATE workspaces SET name = $1 WHERE uuid = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(name)
        .bind(workspace_uuid)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// 删除工作空间（软删除）
pub async fn delete_workspace(pool: &Pool<Postgres>, workspace_uuid: Uuid) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE workspaces SET deleted_at = CURRENT_TIMESTAMP WHERE uuid = $1
        "#,
    )
    .bind(workspace_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 检查用户是否是工作空间所有者
pub async fn check_workspace_owner(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<bool, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM workspaces
        WHERE uuid = $1 AND owner_uuid = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(workspace_uuid)
    .bind(user_uuid)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}
