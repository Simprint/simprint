use uuid::Uuid;

use crate::dto::WorkspaceDto;
use crate::entitys::{CreateTeamRequest, CreateWorkspaceRequest, UpdateWorkspaceRequest};
use crate::models;
use crate::svc_ctx::SvcCtx;

/// 创建工作空间
pub async fn create_workspace_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &CreateWorkspaceRequest,
) -> Result<Uuid, String> {
    // 获取用户信息，用于生成团队名称
    let user_info = models::user::fetch_user_info_by_uuid(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "用户不存在".to_string())?;

    // 生成团队名称
    let team_name = user_info
        .nickname
        .as_ref()
        .map(|n| format!("{} 的团队", n))
        .unwrap_or_else(|| {
            format!(
                "{} 的团队",
                user_info
                    .email
                    .split('@')
                    .next()
                    .unwrap_or("用户")
            )
        });

    // 创建工作空间
    let workspace_uuid = models::insert_workspace(&svc_ctx.db, user_uuid, payload)
        .await
        .map_err(|e| e.to_string())?;

    // 创建默认配额（从配置读取）
    let quota = &svc_ctx.config.workspace_quota.default;
    models::insert_or_update_workspace_quota(
        &svc_ctx.db,
        workspace_uuid,
        quota.max_environments,
        quota.max_team_members,
        quota.max_proxies,
        quota.max_rpa_tasks,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 创建默认团队（每个工作空间自动创建一个团队）
    let team_request = CreateTeamRequest {
        workspace_uuid,
        name: team_name,
        description: Some("默认团队".to_string()),
    };
    let team_uuid = models::insert_team(&svc_ctx.db, user_uuid, &team_request)
        .await
        .map_err(|e| e.to_string())?;

    // 设置用户当前工作空间和团队，确保上下文始终一致。
    models::user::set_user_current_workspace_and_team(&svc_ctx.db, user_uuid, workspace_uuid, team_uuid)
        .await
        .map_err(|e| e.to_string())?;

    Ok(workspace_uuid)
}

/// 获取工作空间详情
pub async fn get_workspace_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
) -> Result<WorkspaceDto, String> {
    models::fetch_workspace_by_uuid(&svc_ctx.db, workspace_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "工作空间不存在".to_string())
}

/// 获取用户所属的所有工作空间
pub async fn get_user_workspaces_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<Vec<WorkspaceDto>, String> {
    models::fetch_user_workspaces(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 更新工作空间
pub async fn update_workspace_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &UpdateWorkspaceRequest,
) -> Result<(), String> {
    // 检查权限（只有所有者可以更新）
    let workspace = models::fetch_workspace_by_uuid(&svc_ctx.db, payload.uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "工作空间不存在".to_string())?;

    if workspace.owner_uuid != user_uuid {
        return Err("只有工作空间所有者可以更新".to_string());
    }

    models::update_workspace(&svc_ctx.db, payload.uuid, payload.name.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// 删除工作空间（软删除）
pub async fn delete_workspace_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    workspace_uuid: Uuid,
) -> Result<(), String> {
    // 检查权限（只有所有者可以删除）
    let workspace = models::fetch_workspace_by_uuid(&svc_ctx.db, workspace_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "工作空间不存在".to_string())?;

    if workspace.owner_uuid != user_uuid {
        return Err("只有工作空间所有者可以删除".to_string());
    }

    let current_workspace_uuid = models::user::fetch_user_current_workspace(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    if current_workspace_uuid == Some(workspace_uuid) {
        return Err("不能删除当前正在使用的工作空间，请先切换到其他工作空间".to_string());
    }

    // 检查用户是否只有一个工作空间，如果是则不允许删除
    let user_workspaces = models::fetch_user_workspaces(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    if user_workspaces.len() <= 1 {
        return Err("至少需要保留一个工作空间，无法删除最后一个工作空间".to_string());
    }

    models::delete_workspace(&svc_ctx.db, workspace_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 检查用户是否是工作空间所有者
pub async fn check_workspace_owner_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<bool, String> {
    models::check_workspace_owner(&svc_ctx.db, workspace_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())
}

pub async fn switch_workspace_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<(), String> {
    let teams = models::fetch_user_teams(&svc_ctx.db, workspace_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    let current_team_uuid = models::fetch_user_current_team(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    let team_uuid = current_team_uuid
        .filter(|current| teams.iter().any(|team| team.uuid == *current))
        .or_else(|| teams.first().map(|team| team.uuid))
        .ok_or_else(|| "该工作空间下没有可用团队".to_string())?;

    models::user::set_user_current_workspace_and_team(&svc_ctx.db, user_uuid, workspace_uuid, team_uuid)
        .await
        .map_err(|e| e.to_string())
}
