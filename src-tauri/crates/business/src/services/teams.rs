use uuid::Uuid;

use crate::dto::{TeamDto, TeamMemberDto};
use crate::entitys::{
    AddMemberRequest, CreateTeamRequest, ListTeamMembersRequest, SwitchTeamRequest,
    UpdateMemberRoleRequest, UpdateTeamRequest,
};
use crate::models;
use crate::svc_ctx::SvcCtx;

/// 创建团队
pub async fn create_team_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &CreateTeamRequest,
) -> Result<Uuid, String> {
    models::insert_team(&svc_ctx.db, user_uuid, payload)
        .await
        .map_err(|e| e.to_string())
}

/// 获取团队详情
pub async fn get_team_service(svc_ctx: &SvcCtx, team_uuid: Uuid) -> Result<TeamDto, String> {
    models::fetch_team_by_uuid(&svc_ctx.db, team_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "团队不存在".to_string())
}

/// 获取用户在本机加入的所有团队。
pub async fn get_user_teams_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<Vec<TeamDto>, String> {
    models::fetch_all_user_teams(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 获取用户当前团队
pub async fn get_current_team_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<Option<Uuid>, String> {
    models::fetch_user_current_team(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 切换团队
pub async fn switch_team_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &SwitchTeamRequest,
) -> Result<(), String> {
    let team = models::fetch_team_by_uuid(&svc_ctx.db, payload.team_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "团队不存在".to_string())?;
    models::fetch_team_member(
        &svc_ctx.db,
        team.workspace_uuid,
        payload.team_uuid,
        user_uuid,
    )
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "您不是该团队的成员".to_string())?;

    models::user::set_user_current_workspace_and_team(
        &svc_ctx.db,
        user_uuid,
        team.workspace_uuid,
        payload.team_uuid,
    )
    .await
    .map_err(|e| e.to_string())
}

/// 更新团队信息
pub async fn update_team_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    user_uuid: Uuid,
    payload: &UpdateTeamRequest,
) -> Result<(), String> {
    // 检查权限（仅所有者和管理员可以更新）
    let member = models::fetch_team_member(&svc_ctx.db, workspace_uuid, payload.uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "您不是该团队的成员".to_string())?;

    if member.role != "owner" && member.role != "admin" {
        return Err("权限不足".to_string());
    }

    models::update_team(
        &svc_ctx.db,
        payload.uuid,
        payload.name.as_deref(),
        payload.description.as_deref(),
        payload.avatar_hash.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// 获取团队成员列表
pub async fn get_team_members_service(
    svc_ctx: &SvcCtx,
    team_uuid: Uuid,
    payload: &ListTeamMembersRequest,
) -> Result<(Vec<TeamMemberDto>, i64), String> {
    let offset = (payload.pagination.page - 1) * payload.pagination.page_size;

    // 提取筛选条件
    let keyword = payload.filters.as_ref().and_then(|f| f.keyword.as_deref());
    let role = payload.filters.as_ref().and_then(|f| f.role.as_deref());
    let status = payload.filters.as_ref().and_then(|f| f.status.as_deref());

    let members = models::fetch_team_members(
        &svc_ctx.db,
        team_uuid,
        offset,
        payload.pagination.page_size,
        keyword,
        role,
        status,
    )
    .await
    .map_err(|e| e.to_string())?;

    let total = models::fetch_team_member_count(&svc_ctx.db, team_uuid, keyword, role, status)
        .await
        .map_err(|e| e.to_string())?;

    Ok((members, total))
}

/// 将另一个本地用户直接加入团队。
pub async fn add_member_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    added_by_uuid: Uuid,
    payload: &AddMemberRequest,
) -> Result<Uuid, String> {
    // 检查操作者权限（工作空间级别）
    let member = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, added_by_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "您不是该团队的成员".to_string())?;

    if member.role != "owner" && member.role != "admin" {
        return Err("权限不足".to_string());
    }

    if !matches!(payload.role.as_str(), "admin" | "editor" | "viewer") {
        return Err("无效的团队角色".to_string());
    }
    let user_info = crate::models::user::fetch_user_info_by_uuid(&svc_ctx.db, payload.user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "本地用户不存在".to_string())?;
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM local_user_auth WHERE user_uuid = $1")
        .bind(payload.user_uuid)
        .fetch_one(&svc_ctx.db)
        .await
        .map_err(|e| e.to_string())?
        .gt(&0)
        .then_some(())
        .ok_or_else(|| "目标不是本地用户".to_string())?;

    // 检查用户是否已是团队成员
    if models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, user_info.user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Err("该用户已是团队成员".to_string());
    }

    models::insert_team_member(
        &svc_ctx.db,
        team_uuid,
        user_info.user_uuid,
        &payload.role,
        Some(added_by_uuid),
    )
    .await
    .map_err(|e| e.to_string())?;
    models::update_used_team_members(&svc_ctx.db, workspace_uuid)
        .await
        .map_err(|e| format!("更新配额失败: {e}"))?;

    Ok(user_info.user_uuid)
}

/// 更新成员角色
pub async fn update_member_role_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    operator_uuid: Uuid,
    payload: &UpdateMemberRoleRequest,
) -> Result<crate::dto::TeamMemberDto, String> {
    // 检查操作者权限（工作空间级别）
    let operator = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, operator_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "您不是该团队的成员".to_string())?;

    if operator.role != "owner" && operator.role != "admin" {
        return Err("权限不足".to_string());
    }

    // 不能修改所有者角色
    let target =
        models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, payload.member_uuid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "成员不存在".to_string())?;

    if target.role == "owner" {
        return Err("不能修改所有者角色".to_string());
    }

    // 管理员不能设置其他管理员
    if operator.role == "admin" && payload.role == "admin" {
        return Err("管理员不能设置其他管理员".to_string());
    }

    models::update_member_role(&svc_ctx.db, team_uuid, payload.member_uuid, &payload.role)
        .await
        .map_err(|e| e.to_string())?;

    // 返回更新后的成员信息
    models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, payload.member_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "获取更新后的成员信息失败".to_string())
}

/// 移除成员
pub async fn remove_member_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    operator_uuid: Uuid,
    member_uuid: Uuid,
) -> Result<(), String> {
    // 检查操作者权限（工作空间级别）
    let operator = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, operator_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "您不是该团队的成员".to_string())?;

    if operator.role != "owner" && operator.role != "admin" {
        return Err("权限不足".to_string());
    }

    // 不能移除所有者
    let target = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, member_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "成员不存在".to_string())?;

    if target.role == "owner" {
        return Err("不能移除所有者".to_string());
    }

    // 管理员不能移除其他管理员
    if operator.role == "admin" && target.role == "admin" {
        return Err("管理员不能移除其他管理员".to_string());
    }

    models::remove_team_member(&svc_ctx.db, team_uuid, member_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 更新成员配额（重新计算所有团队的活跃成员总数）
    models::update_used_team_members(&svc_ctx.db, workspace_uuid)
        .await
        .map_err(|e| format!("更新配额失败: {}", e))?;

    Ok(())
}

/// 退出团队
pub async fn leave_team_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<(), String> {
    // 检查用户是否是团队成员（工作空间级别）
    let member = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "您不是该团队的成员".to_string())?;

    // 所有者不能退出团队
    if member.role == "owner" {
        return Err("团队所有者不能退出团队，请先转移所有权或解散团队".to_string());
    }

    // 移除成员
    models::remove_team_member(&svc_ctx.db, team_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 更新成员配额（重新计算所有团队的活跃成员总数）
    models::update_used_team_members(&svc_ctx.db, workspace_uuid)
        .await
        .map_err(|e| format!("更新配额失败: {}", e))?;

    // 如果当前团队是用户的活跃团队，则需要切换到其他团队
    let current_team = models::fetch_user_current_team(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    if current_team == Some(team_uuid) {
        // 获取用户的其他团队（工作空间级别）
        let teams = models::fetch_user_teams(&svc_ctx.db, workspace_uuid, user_uuid)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(first_team) = teams.first() {
            // 切换到第一个可用团队
            models::set_user_current_team(&svc_ctx.db, user_uuid, first_team.uuid)
                .await
                .map_err(|e| e.to_string())?;
        } else {
            // 没有其他团队，清除当前团队
            models::clear_user_current_team(&svc_ctx.db, user_uuid)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
