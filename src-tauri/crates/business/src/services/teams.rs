use uuid::Uuid;

use crate::dto::{TeamDto, TeamInvitationDto, TeamMemberDto};
use crate::entitys::{
    CreateTeamRequest, InviteMemberRequest, ListTeamMembersRequest, SwitchTeamRequest,
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

/// 获取用户所属的所有团队（工作空间级别）
pub async fn get_user_teams_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<Vec<TeamDto>, String> {
    models::fetch_user_teams(&svc_ctx.db, workspace_uuid, user_uuid)
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
    workspace_uuid: Uuid,
    user_uuid: Uuid,
    payload: &SwitchTeamRequest,
) -> Result<(), String> {
    // 验证用户是否是团队成员（工作空间级别）
    let member = models::fetch_team_member(&svc_ctx.db, workspace_uuid, payload.team_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    if member.is_none() {
        return Err("您不是该团队的成员".to_string());
    }

    models::set_user_current_team(&svc_ctx.db, user_uuid, payload.team_uuid)
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

/// 邀请成员
pub async fn invite_member_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    inviter_uuid: Uuid,
    payload: &InviteMemberRequest,
) -> Result<Uuid, String> {
    // 检查邀请者权限（工作空间级别）
    let member = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, inviter_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "您不是该团队的成员".to_string())?;

    if member.role != "owner" && member.role != "admin" {
        return Err("权限不足".to_string());
    }

    // 检查是否已有待处理的邀请
    if models::has_pending_invitation(&svc_ctx.db, team_uuid, &payload.email)
        .await
        .map_err(|e| e.to_string())?
    {
        return Err("该邮箱已有待处理的邀请".to_string());
    }

    // 检查用户是否已是成员
    let invited_user_info =
        crate::models::user::fetch_user_info_by_email(&svc_ctx.db, &payload.email)
            .await
            .map_err(|e| e.to_string())?;

    // 检查用户是否存在
    let user_info = invited_user_info.ok_or_else(|| "该邮箱未注册，请先注册账号".to_string())?;

    // 检查用户是否已是团队成员
    if models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, user_info.user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Err("该用户已是团队成员".to_string());
    }

    // 获取团队信息
    let team = models::fetch_team_by_uuid(&svc_ctx.db, team_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "团队不存在".to_string())?;

    // 获取邀请者信息
    let inviter_info = crate::models::user::fetch_user_info_by_uuid(&svc_ctx.db, inviter_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "邀请者信息不存在".to_string())?;

    // 生成邀请 token
    let token = uuid::Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    // 插入邀请记录
    let invitation_uuid = models::insert_team_invitation(
        &svc_ctx.db,
        team_uuid,
        &payload.email,
        &payload.role,
        inviter_uuid,
        &token,
        expires_at,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 1. 发送邮件通知
    let smtp_config = svc_ctx.config.clone().smtp.ok_or_else(|| "邮箱配置不存在".to_string())?;

    let inviter_name: &str =
        inviter_info.nickname.as_deref().unwrap_or_else(|| inviter_info.email.as_str());
    let inviter_email: &str = inviter_info.email.as_str();

    let email_title = format!("团队邀请：{} 邀请您加入团队", inviter_name);
    let email_body = format!(
        "<html><body style=\"font-family: Arial, sans-serif; line-height: 1.6; color: #333;\"><div style=\"max-width: 600px; margin: 0 auto; padding: 20px;\"><h2 style=\"color: rgb(37, 99, 235);\">团队邀请</h2><p>您好，</p><p><strong>{}</strong>（{}）邀请您加入团队 <strong>{}</strong>。</p><p>邀请角色：<strong>{}</strong></p><p>邀请有效期：7 天</p><p style=\"margin-top: 30px;\"><a href=\"#\" style=\"background-color: rgb(37, 99, 235); color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; display: inline-block;\">接受邀请</a></p><p style=\"margin-top: 20px; color: #666; font-size: 12px;\">如果您不想接受此邀请，可以忽略此邮件。</p></div></body></html>",
        inviter_name, inviter_email, team.name, payload.role
    );

    // 发送邮件（失败不影响邀请流程）
    let _ = crate::utils::send_email(
        &smtp_config.smtp_username,
        &smtp_config.smtp_password,
        &smtp_config.smtp_server,
        &payload.email,
        &email_title,
        &email_body,
    );

    // 2. 如果用户已注册，发送团队邀请消息通知
    let _ = crate::services::messages::create_message_service(
        svc_ctx,
        Some(inviter_uuid),
        &crate::entitys::CreateMessageRequest {
            message_type: "team_invitation".to_string(),
            title: format!("{} 邀请您加入团队 {}", inviter_name, team.name),
            content: Some(format!(
                "{}（{}）邀请您加入团队 {}，角色：{}",
                inviter_name, inviter_email, team.name, payload.role
            )),
            recipient_uuids: vec![user_info.user_uuid],
            recipient_type: "single".to_string(),
            related_type: Some("team".to_string()),
            related_uuid: Some(team_uuid),
            priority: Some("normal".to_string()),
            metadata: Some(serde_json::json!({
                "invitation_uuid": invitation_uuid.to_string(),
                "team_name": team.name,
                "role": payload.role,
                "inviter_name": inviter_name,
                "inviter_email": inviter_email,
                "token": token,
            })),
        },
    )
    .await;
    // 消息创建失败不影响邀请流程，使用 _ 忽略错误

    Ok(invitation_uuid)
}

/// 取消邀请
pub async fn cancel_invitation_service(
    svc_ctx: &SvcCtx,
    invitation_uuid: Uuid,
) -> Result<(), String> {
    models::cancel_team_invitation(&svc_ctx.db, invitation_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 接受邀请
pub async fn accept_invitation_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    token: &str,
) -> Result<Uuid, String> {
    // 查找邀请
    let invitation = models::fetch_team_invitation_by_token(&svc_ctx.db, token)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "邀请不存在或已过期".to_string())?;

    // 获取被邀请用户的当前工作空间
    let user_workspace_uuid = crate::models::user::fetch_user_current_workspace(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "请先选择工作空间".to_string())?;

    // 检查用户是否已经是该工作空间的团队成员（如果是，则不需要检查配额）
    let existing_member = models::fetch_team_member(
        &svc_ctx.db,
        user_workspace_uuid,
        invitation.team_uuid,
        user_uuid,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 如果用户还不是该工作空间的任何团队的成员，需要检查配额
    if existing_member.is_none() {
        // 检查工作空间成员配额是否充足
        let quota_available = models::check_quota(&svc_ctx.db, user_workspace_uuid, "team_members")
            .await
            .map_err(|e| e.to_string())?;
        if !quota_available {
            return Err("工作空间成员配额不足，无法接受邀请".to_string());
        }
    }

    // 使用 accept_team_invitation 函数处理整个流程
    let team_uuid = models::accept_team_invitation(&svc_ctx.db, invitation.uuid, user_uuid, user_workspace_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 更新成员配额（重新计算所有团队的活跃成员总数）
    // 注意：只有在用户之前不是该工作空间的成员时才需要更新配额
    if existing_member.is_none() {
        models::update_used_team_members(&svc_ctx.db, user_workspace_uuid)
            .await
            .map_err(|e| format!("更新配额失败: {}", e))?;
    }

    Ok(team_uuid)
}

/// 拒绝邀请
pub async fn reject_invitation_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    token: &str,
) -> Result<(), String> {
    // 查找邀请
    let invitation = models::fetch_team_invitation_by_token(&svc_ctx.db, token)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "邀请不存在或已过期".to_string())?;

    // 使用 reject_team_invitation 函数处理整个流程
    models::reject_team_invitation(&svc_ctx.db, invitation.uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())
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
    let target = models::fetch_team_member(&svc_ctx.db, workspace_uuid, team_uuid, payload.member_uuid)
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

/// 获取待处理邀请列表
pub async fn get_pending_invitations_service(
    svc_ctx: &SvcCtx,
    team_uuid: Uuid,
) -> Result<Vec<TeamInvitationDto>, String> {
    models::fetch_pending_invitations(&svc_ctx.db, team_uuid)
        .await
        .map_err(|e| e.to_string())
}
