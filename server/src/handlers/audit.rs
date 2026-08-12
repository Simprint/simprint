use axum::extract::{Extension, State};

use crate::dto::AuditLogDto;
use crate::entitys::{
    AuditLogsListResponse, AuditStatsRequest, AuditStatsResponse, ExportAuditLogsRequest,
    ExportResponse, ListAuditLogsRequest, UuidRequest,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 获取审计日志列表
pub async fn get_audit_logs_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListAuditLogsRequest>,
) -> Result<AuditLogsListResponse> {
    let current_user_uuid = ctx.user_uuid_unwrap();
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, current_user_uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let (items, total) =
        services::audit::get_audit_logs_service(&svc_ctx, current_user_uuid, team_uuid, &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(AuditLogsListResponse {
            items,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

/// 获取审计日志详情
pub async fn get_audit_log_detail_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<AuditLogDto> {
    let log = services::audit::get_audit_log_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(log)))
}

/// 导出审计日志
pub async fn export_audit_logs_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ExportAuditLogsRequest>,
) -> Result<ExportResponse> {
    let current_user_uuid = ctx.user_uuid_unwrap();
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, current_user_uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let (content, filename, mime_type) = services::audit::export_audit_logs_service(
        &svc_ctx,
        current_user_uuid,
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("导出成功"),
        Some(ExportResponse {
            content,
            filename,
            mime_type,
        }),
    ))
}

/// 获取审计统计
pub async fn get_audit_stats_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(_payload): Json<AuditStatsRequest>,
) -> Result<AuditStatsResponse> {
    let current_user_uuid = ctx.user_uuid_unwrap();
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, current_user_uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let stats = services::audit::get_audit_stats_service(&svc_ctx, current_user_uuid, team_uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(stats)))
}
