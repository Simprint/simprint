use uuid::Uuid;

use crate::dto::GroupMemberPermissionDto;
use crate::entitys::{
    CheckGroupPermissionRequest, GrantGroupPermissionRequest, ListUserGroupPermissionsRequest,
    RevokeGroupPermissionRequest,
};
use crate::models;
use crate::svc_ctx::SvcCtx;

/// 授予分组权限
pub async fn grant_group_permission_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &GrantGroupPermissionRequest,
) -> Result<(), String> {
    // 检查权限：只有团队 Owner/Admin 或拥有分组 manage 权限的用户可以授权
    let group = models::fetch_group_by_uuid(&svc_ctx.db, payload.group_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "分组不存在".to_string())?;

    // 检查用户是否是团队成员（工作空间级别）
    let team_member = models::fetch_team_member(
        &svc_ctx.db,
        group.workspace_uuid,
        group.team_uuid,
        user_uuid,
    )
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "您不是该团队的成员".to_string())?;

    // Owner/Admin 自动拥有所有权限
    let can_grant = team_member.role == "owner"
        || team_member.role == "admin"
        || models::check_group_permission(
            &svc_ctx.db,
            group.workspace_uuid,
            payload.group_uuid,
            user_uuid,
            "manage",
        )
        .await
        .map_err(|e| e.to_string())?;

    if !can_grant {
        return Err("您没有权限授予分组权限".to_string());
    }

    models::grant_group_permission(
        &svc_ctx.db,
        payload.group_uuid,
        group.workspace_uuid,
        group.team_uuid,
        payload.user_uuid,
        &payload.permission_type,
        user_uuid,
    )
    .await
    .map_err(|e| e.to_string())
}

/// 撤销分组权限
pub async fn revoke_group_permission_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &RevokeGroupPermissionRequest,
) -> Result<(), String> {
    // 检查权限：只有团队 Owner/Admin 或拥有分组 manage 权限的用户可以撤销权限
    let group = models::fetch_group_by_uuid(&svc_ctx.db, payload.group_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "分组不存在".to_string())?;

    // 检查用户是否是团队成员（工作空间级别）
    let team_member = models::fetch_team_member(
        &svc_ctx.db,
        group.workspace_uuid,
        group.team_uuid,
        user_uuid,
    )
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "您不是该团队的成员".to_string())?;

    // Owner/Admin 自动拥有所有权限
    let can_revoke = team_member.role == "owner"
        || team_member.role == "admin"
        || models::check_group_permission(
            &svc_ctx.db,
            group.workspace_uuid,
            payload.group_uuid,
            user_uuid,
            "manage",
        )
        .await
        .map_err(|e| e.to_string())?;

    if !can_revoke {
        return Err("您没有权限撤销分组权限".to_string());
    }

    models::revoke_group_permission(&svc_ctx.db, payload.group_uuid, payload.user_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 检查分组权限
pub async fn check_group_permission_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    payload: &CheckGroupPermissionRequest,
) -> Result<bool, String> {
    models::check_group_permission(
        &svc_ctx.db,
        workspace_uuid,
        payload.group_uuid,
        payload.user_uuid,
        &payload.permission_type,
    )
    .await
    .map_err(|e| e.to_string())
}

/// 查询用户的分组权限列表
pub async fn list_user_group_permissions_service(
    svc_ctx: &SvcCtx,
    payload: &ListUserGroupPermissionsRequest,
) -> Result<Vec<GroupMemberPermissionDto>, String> {
    models::fetch_user_group_permissions(
        &svc_ctx.db,
        payload.user_uuid,
        None, // workspace_uuid 可以从 payload 中获取，如果需要的话
        payload.group_uuid,
    )
    .await
    .map_err(|e| e.to_string())
}
