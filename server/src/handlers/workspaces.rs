use axum::{extract::Extension, extract::State};

use crate::audit_log;
use crate::entitys::{
    CreateResponse, CreateWorkspaceRequest, SwitchWorkspaceRequest, UpdateWorkspaceRequest,
    WorkspaceItem, WorkspaceListResponse,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 创建工作空间
pub async fn create_workspace_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateWorkspaceRequest>,
) -> Result<CreateResponse> {
    let workspace_uuid =
        services::workspaces::create_workspace_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "create",
        "workspace",
        workspace_uuid,
        &payload.name,
        "创建工作空间"
    )
    .await;

    Ok(Response::success(
        Some("创建成功"),
        Some(CreateResponse {
            uuid: workspace_uuid,
        }),
    ))
}

/// 获取用户的所有工作空间
pub async fn get_my_workspaces_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<WorkspaceListResponse> {
    let user_uuid = ctx.user_uuid_unwrap();
    let workspaces = services::workspaces::get_user_workspaces_service(&svc_ctx, user_uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    // 从 RequestContext 获取当前工作空间 UUID（已在 auth 中间件中设置）
    let mut current_workspace_uuid = ctx.current_workspace_uuid;

    // 如果用户没有设置当前工作空间，但有工作空间列表，自动设置为第一个工作空间
    if current_workspace_uuid.is_none() && !workspaces.is_empty() {
        let first_workspace_uuid = workspaces[0].uuid;
        // 更新数据库
        if let Err(e) = crate::models::user::set_user_current_workspace(
            &svc_ctx.db,
            user_uuid,
            first_workspace_uuid,
        )
        .await
        {
            tracing::warn!("Failed to set default workspace: {}", e);
        } else {
            current_workspace_uuid = Some(first_workspace_uuid);
        }
    }

    let workspace_items: Vec<WorkspaceItem> = workspaces
        .iter()
        .map(|w| WorkspaceItem {
            uuid: w.uuid,
            name: w.name.clone(),
            workspace_type: w.workspace_type.clone(),
            is_current: current_workspace_uuid == Some(w.uuid),
        })
        .collect();

    Ok(Response::success(
        None,
        Some(WorkspaceListResponse {
            current_workspace_uuid,
            workspaces: workspace_items,
        }),
    ))
}

/// 获取工作空间详情
pub async fn get_workspace_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(_ctx): Extension<RequestContext>,
    Json(payload): Json<crate::entitys::UuidRequest>,
) -> Result<crate::dto::WorkspaceDto> {
    let workspace = services::workspaces::get_workspace_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(None, Some(workspace)))
}

/// 更新工作空间
pub async fn update_workspace_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdateWorkspaceRequest>,
) -> Result<()> {
    services::workspaces::update_workspace_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "update",
        "workspace",
        payload.uuid,
        &payload.name.as_deref().unwrap_or(""),
        "更新工作空间"
    )
    .await;

    Ok(Response::success(Some("更新成功"), None))
}

/// 删除工作空间
pub async fn delete_workspace_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<crate::entitys::UuidRequest>,
) -> Result<()> {
    services::workspaces::delete_workspace_service(&svc_ctx, ctx.user_uuid_unwrap(), payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "delete",
        "workspace",
        payload.uuid,
        "",
        "删除工作空间"
    )
    .await;

    Ok(Response::success(Some("删除成功"), None))
}

/// 切换工作空间
pub async fn switch_workspace_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<SwitchWorkspaceRequest>,
) -> Result<()> {
    let user_uuid = ctx.user_uuid_unwrap();

    // 检查用户是否是工作空间所有者
    let is_owner = services::workspaces::check_workspace_owner_service(
        &svc_ctx,
        payload.workspace_uuid,
        user_uuid,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    if !is_owner {
        return Err(Response::fail(Some("您不是该工作空间的所有者")));
    }

    services::workspaces::switch_workspace_service(&svc_ctx, payload.workspace_uuid, user_uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("切换成功"), None))
}
