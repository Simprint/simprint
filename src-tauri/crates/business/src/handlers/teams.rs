use axum::{extract::Extension, extract::State};

use crate::audit_log;
use crate::entitys::{
    AcceptInvitationRequest, AcceptInvitationResponse, CreateResponse, CreateTeamRequest,
    InviteMemberRequest, InviteResponse, ListTeamMembersRequest, MemberListResponse,
    RejectInvitationRequest, RemoveMemberRequest, SwitchTeamRequest, TeamItem, TeamListResponse,
    UpdateMemberRoleRequest, UpdateTeamRequest, UuidRequest,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 创建团队
pub async fn create_team_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateTeamRequest>,
) -> Result<CreateResponse> {
    let team_uuid =
        services::teams::create_team_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "create",
        "team",
        team_uuid,
        &payload.name,
        "创建团队"
    )
    .await;

    Ok(Response::success(
        Some("创建成功"),
        Some(CreateResponse { uuid: team_uuid }),
    ))
}

/// 获取用户的所有团队
pub async fn get_my_teams_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<TeamListResponse> {
    let workspace_uuid = ctx.current_workspace_uuid.ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let teams = services::teams::get_user_teams_service(&svc_ctx, workspace_uuid, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let current_team_uuid =
        services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    let mut team_items: Vec<TeamItem> = vec![];

    for t in &teams {
        // 获取用户在该团队的角色
        let role = if t.owner_uuid == ctx.user_uuid_unwrap() {
            "owner".to_string()
        } else {
            crate::models::fetch_team_member(&svc_ctx.db, workspace_uuid, t.uuid, ctx.user_uuid_unwrap())
                .await
                .ok()
                .flatten()
                .map(|m| m.role)
                .unwrap_or_else(|| "member".to_string())
        };

        // 获取成员数量（不使用筛选，统计所有活跃成员）
        let members_count =
            crate::models::fetch_team_member_count(&svc_ctx.db, t.uuid, None, None, None)
                .await
                .unwrap_or(0);

        team_items.push(TeamItem {
            uuid: t.uuid,
            name: t.name.clone(),
            description: t.description.clone(),
            role,
            members_count,
            is_current: current_team_uuid == Some(t.uuid),
        });
    }

    Ok(Response::success(
        Some("获取成功"),
        Some(TeamListResponse {
            current_team_uuid,
            teams: team_items,
        }),
    ))
}

/// 切换团队
pub async fn switch_team_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<SwitchTeamRequest>,
) -> Result<()> {
    let workspace_uuid = ctx.current_workspace_uuid.ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    services::teams::switch_team_service(&svc_ctx, workspace_uuid, ctx.user_uuid_unwrap(), &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("切换成功"), None))
}

/// 获取团队详情
pub async fn get_team_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<crate::dto::TeamDto> {
    let team = services::teams::get_team_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(team)))
}

/// 更新团队信息
pub async fn update_team_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdateTeamRequest>,
) -> Result<()> {
    let workspace_uuid = ctx.current_workspace_uuid.ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    services::teams::update_team_service(&svc_ctx, workspace_uuid, ctx.user_uuid_unwrap(), &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("更新成功"), None))
}

/// 获取团队成员列表
pub async fn get_team_members_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListTeamMembersRequest>,
) -> Result<MemberListResponse> {
    // 获取用户当前团队
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?
        .ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    let (members, total) = services::teams::get_team_members_service(&svc_ctx, team_uuid, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(MemberListResponse {
            items: members,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

/// 邀请成员
pub async fn invite_member_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<InviteMemberRequest>,
) -> Result<InviteResponse> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?
        .ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    let workspace_uuid = ctx.current_workspace_uuid.ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let invitation_uuid = services::teams::invite_member_service(
        &svc_ctx,
        workspace_uuid,
        team_uuid,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "invite",
        "team_member",
        &format!("邀请成员: {}", payload.email)
    )
    .await;

    Ok(Response::success(
        Some("邀请已发送"),
        Some(InviteResponse { invitation_uuid }),
    ))
}

/// 更新成员角色
pub async fn update_member_role_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdateMemberRoleRequest>,
) -> Result<crate::dto::TeamMemberDto> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?
        .ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    let workspace_uuid = ctx.current_workspace_uuid.ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let member = services::teams::update_member_role_service(
        &svc_ctx,
        workspace_uuid,
        team_uuid,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("更新成功"), Some(member)))
}

/// 移除成员
pub async fn remove_member_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<RemoveMemberRequest>,
) -> Result<()> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?
        .ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    let workspace_uuid = ctx.current_workspace_uuid.ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    services::teams::remove_member_service(
        &svc_ctx,
        workspace_uuid,
        team_uuid,
        ctx.user_uuid_unwrap(),
        payload.member_uuid,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "remove",
        "team_member",
        payload.member_uuid,
        "移除成员"
    )
    .await;

    Ok(Response::success(Some("移除成功"), None))
}

/// 获取待处理邀请列表
pub async fn get_pending_invitations_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<Vec<crate::dto::TeamInvitationDto>> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?
        .ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    let invitations = services::teams::get_pending_invitations_service(&svc_ctx, team_uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(invitations)))
}

/// 取消邀请
pub async fn cancel_invitation_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<()> {
    services::teams::cancel_invitation_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("邀请已取消"), None))
}

/// 接受邀请
pub async fn accept_invitation_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<AcceptInvitationRequest>,
) -> Result<AcceptInvitationResponse> {
    let team_uuid = services::teams::accept_invitation_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload.token,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "accept_invitation",
        "team",
        team_uuid,
        "接受团队邀请"
    )
    .await;

    Ok(Response::success(
        Some("已加入团队"),
        Some(AcceptInvitationResponse { team_uuid }),
    ))
}

/// 拒绝邀请
pub async fn reject_invitation_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<RejectInvitationRequest>,
) -> Result<()> {
    services::teams::reject_invitation_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload.token)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(&svc_ctx, &ctx, "reject_invitation", "team", "拒绝团队邀请").await;

    Ok(Response::success(Some("已拒绝邀请"), None))
}

/// 退出团队
pub async fn leave_team_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<()> {
    // 获取当前团队
    let current_team_uuid =
        services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
            .await
            .map_err(|e| Response::fail(Some(&e)))?
            .ok_or_else(|| Response::fail(Some("当前没有选择团队")))?;

    let workspace_uuid = ctx.current_workspace_uuid.ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    services::teams::leave_team_service(&svc_ctx, workspace_uuid, current_team_uuid, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "leave",
        "team",
        current_team_uuid,
        "退出团队"
    )
    .await;

    Ok(Response::success(Some("已退出团队"), None))
}
