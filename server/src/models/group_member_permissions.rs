use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::GroupMemberPermissionDto;

/// 授予分组权限
pub async fn grant_group_permission(
    pool: &Pool<Postgres>,
    group_uuid: Uuid,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    user_uuid: Uuid,
    permission_type: &str,
    granted_by: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        INSERT INTO group_member_permissions (
            group_uuid, workspace_uuid, team_uuid, user_uuid, permission_type, granted_by
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (group_uuid, user_uuid) DO UPDATE SET
            permission_type = EXCLUDED.permission_type,
            granted_by = EXCLUDED.granted_by,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(group_uuid)
    .bind(workspace_uuid)
    .bind(team_uuid)
    .bind(user_uuid)
    .bind(permission_type)
    .bind(granted_by)
    .execute(pool)
    .await?;

    Ok(())
}

/// 撤销分组权限
pub async fn revoke_group_permission(
    pool: &Pool<Postgres>,
    group_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        DELETE FROM group_member_permissions
        WHERE group_uuid = $1 AND user_uuid = $2
        "#,
    )
    .bind(group_uuid)
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 查询用户的分组权限列表
pub async fn fetch_user_group_permissions(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    workspace_uuid: Option<Uuid>,
    group_uuid: Option<Uuid>,
) -> Result<Vec<GroupMemberPermissionDto>, Error> {
    let recs = sqlx::query_as::<_, GroupMemberPermissionDto>(
        r#"
        SELECT group_uuid, workspace_uuid, team_uuid, user_uuid, permission_type, granted_by,
               created_at, updated_at
        FROM group_member_permissions
        WHERE user_uuid = $1
          AND ($2::uuid IS NULL OR workspace_uuid = $2)
          AND ($3::uuid IS NULL OR group_uuid = $3)
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_uuid)
    .bind(workspace_uuid)
    .bind(group_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 检查用户是否有分组权限（工作空间级别）
pub async fn check_group_permission(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    group_uuid: Uuid,
    user_uuid: Uuid,
    permission_type: &str,
) -> Result<bool, Error> {
    // 首先检查用户是否是团队成员，以及是否是 Owner/Admin（自动拥有所有权限）
    let is_owner_or_admin: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM team_members tm
            INNER JOIN groups g ON tm.team_uuid = g.team_uuid AND tm.workspace_uuid = g.workspace_uuid
            WHERE g.uuid = $1
              AND g.workspace_uuid = $2
              AND tm.workspace_uuid = $2
              AND tm.user_uuid = $3
              AND tm.role IN ('owner', 'admin')
              AND tm.deleted_at IS NULL
        )
        "#,
    )
    .bind(group_uuid)
    .bind(workspace_uuid)
    .bind(user_uuid)
    .fetch_one(pool)
    .await?;

    if is_owner_or_admin {
        return Ok(true);
    }

    // 检查显式权限（工作空间级别）
    let has_permission = match permission_type {
        "read" => {
            // read 权限：检查是否有 read/write/manage 任一权限
            sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS (
                    SELECT 1 FROM group_member_permissions
                    WHERE group_uuid = $1 AND workspace_uuid = $2 AND user_uuid = $3
                )
                "#,
            )
            .bind(group_uuid)
            .bind(workspace_uuid)
            .bind(user_uuid)
            .fetch_one(pool)
            .await?
        }
        "write" => {
            // write 权限：检查是否有 write/manage 权限
            sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS (
                    SELECT 1 FROM group_member_permissions
                    WHERE group_uuid = $1 AND workspace_uuid = $2 AND user_uuid = $3
                      AND permission_type IN ('write', 'manage')
                )
                "#,
            )
            .bind(group_uuid)
            .bind(workspace_uuid)
            .bind(user_uuid)
            .fetch_one(pool)
            .await?
        }
        "manage" => {
            // manage 权限：检查是否有 manage 权限
            sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS (
                    SELECT 1 FROM group_member_permissions
                    WHERE group_uuid = $1 AND workspace_uuid = $2 AND user_uuid = $3
                      AND permission_type = 'manage'
                )
                "#,
            )
            .bind(group_uuid)
            .bind(workspace_uuid)
            .bind(user_uuid)
            .fetch_one(pool)
            .await?
        }
        _ => false,
    };

    Ok(has_permission)
}

/// 查询分组的所有权限
pub async fn fetch_group_permissions(
    pool: &Pool<Postgres>,
    group_uuid: Uuid,
) -> Result<Vec<GroupMemberPermissionDto>, Error> {
    let recs = sqlx::query_as::<_, GroupMemberPermissionDto>(
        r#"
        SELECT group_uuid, workspace_uuid, team_uuid, user_uuid, permission_type, granted_by,
               created_at, updated_at
        FROM group_member_permissions
        WHERE group_uuid = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(group_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}
