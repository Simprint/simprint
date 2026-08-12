use axum::{extract::Extension, extract::State};

use crate::entitys::{GetWorkspaceQuotaRequest, UpdateQuotaUsageRequest};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 获取工作空间配额
pub async fn get_workspace_quota_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(mut payload): Json<GetWorkspaceQuotaRequest>,
) -> Result<crate::dto::WorkspaceQuotaDto> {
    // 如果没有传递 workspace_uuid，则从 RequestContext 中获取
    if payload.workspace_uuid.is_none() {
        payload.workspace_uuid = ctx.current_workspace_uuid;
    }

    // 验证 workspace_uuid 是否存在
    if payload.workspace_uuid.is_none() {
        return Err(Response::fail(Some("未提供工作空间 UUID")));
    }

    let quota = services::workspace_quotas::get_workspace_quota_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(None, Some(quota)))
}

/// 更新配额使用情况
pub async fn update_quota_usage_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(_ctx): Extension<RequestContext>,
    Json(payload): Json<UpdateQuotaUsageRequest>,
) -> Result<()> {
    services::workspace_quotas::update_quota_usage_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("更新成功"), None))
}
