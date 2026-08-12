use uuid::Uuid;

use crate::dto::GroupDto;
use crate::entitys::{CreateGroupRequest, UpdateGroupRequest};
use crate::models;
use crate::svc_ctx::SvcCtx;

/// 创建分组
pub async fn create_group_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    payload: &CreateGroupRequest,
) -> Result<Uuid, String> {
    // 1. 检查用户是否在当前工作空间的团队中
    let team_member = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "您不是该团队的成员".to_string())?;

    // 2. 检查用户是否是团队 Owner/Admin（只有 Owner/Admin 可以创建分组）
    let can_create = matches!(team_member.role.as_str(), "owner" | "admin");
    if !can_create {
        return Err("您没有创建分组的权限，需要 Owner 或 Admin 角色".to_string());
    }

    models::insert_group(
        &svc_ctx.db,
        workspace_uuid,
        team_uuid,
        &payload.name,
        payload.description.as_deref(),
        user_uuid,
    )
    .await
    .map_err(|e| e.to_string())
}

/// 获取分组列表
pub async fn get_groups_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    page: i64,
    page_size: i64,
) -> Result<Vec<GroupDto>, String> {
    let offset = (page - 1) * page_size;
    models::fetch_groups(&svc_ctx.db, workspace_uuid, team_uuid, offset, page_size)
        .await
        .map_err(|e| e.to_string())
}

/// 获取分组详情
pub async fn get_group_service(svc_ctx: &SvcCtx, group_uuid: Uuid) -> Result<GroupDto, String> {
    models::fetch_group_by_uuid(&svc_ctx.db, group_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "分组不存在".to_string())
}

/// 更新分组
pub async fn update_group_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    user_uuid: Uuid,
    payload: &UpdateGroupRequest,
) -> Result<(), String> {
    // 1. 检查用户是否在当前工作空间的团队中
    let team_member = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "您不是该团队的成员".to_string())?;

    // 2. 查询分组
    let group = models::fetch_group_by_uuid(&svc_ctx.db, payload.uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "分组不存在".to_string())?;

    // 3. 验证分组属于指定工作空间和团队
    if group.workspace_uuid != workspace_uuid || group.team_uuid != team_uuid {
        return Err("分组不属于指定工作空间或团队".to_string());
    }

    // 4. 检查用户是否是团队 Owner/Admin 或拥有分组的 manage 权限
    let is_owner_or_admin = matches!(team_member.role.as_str(), "owner" | "admin");
    let has_manage = if !is_owner_or_admin {
        models::check_group_permission(
            &svc_ctx.db,
            workspace_uuid,
            payload.uuid,
            user_uuid,
            "manage",
        )
        .await
        .map_err(|e| e.to_string())?
    } else {
        true
    };

    if !has_manage {
        return Err("您没有管理该分组的权限".to_string());
    }

    models::update_group(
        &svc_ctx.db,
        payload.uuid,
        payload.name.as_deref(),
        payload.description.as_deref(),
        payload.sort_order,
    )
    .await
    .map_err(|e| e.to_string())
}

/// 删除分组
pub async fn delete_group_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    user_uuid: Uuid,
    group_uuid: Uuid,
) -> Result<(), String> {
    // 1. 检查用户是否在当前工作空间的团队中
    let team_member = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "您不是该团队的成员".to_string())?;

    // 2. 查询分组
    let group = models::fetch_group_by_uuid(&svc_ctx.db, group_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "分组不存在".to_string())?;

    // 3. 验证分组属于指定工作空间和团队
    if group.workspace_uuid != workspace_uuid || group.team_uuid != team_uuid {
        return Err("分组不属于指定工作空间或团队".to_string());
    }

    // 4. 检查用户是否是团队 Owner/Admin 或拥有分组的 manage 权限
    let is_owner_or_admin = matches!(team_member.role.as_str(), "owner" | "admin");
    let has_manage = if !is_owner_or_admin {
        models::check_group_permission(&svc_ctx.db, workspace_uuid, group_uuid, user_uuid, "manage")
            .await
            .map_err(|e| e.to_string())?
    } else {
        true
    };

    if !has_manage {
        return Err("您没有删除该分组的权限".to_string());
    }

    models::delete_group(&svc_ctx.db, group_uuid).await.map_err(|e| e.to_string())
}
