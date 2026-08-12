use axum::{extract::Extension, extract::State};

use crate::audit_log;
use crate::entitys::{
    BatchImportProxiesRequest, BatchImportResponse, BatchUuidRequest, CreateProxyRequest,
    CreateResponse, ListProxiesRequest, ProxyListResponse, UpdateProxyRequest, UuidRequest,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 获取代理列表
pub async fn get_proxies_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListProxiesRequest>,
) -> Result<ProxyListResponse> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    
    let (proxies, total) =
        services::proxies::get_proxies_service(
            &svc_ctx,
            ctx.user_uuid_unwrap(),
            workspace_uuid,
            ctx.current_team_uuid,
            &payload,
        )
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(ProxyListResponse {
            items: proxies,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

/// 获取代理详情
pub async fn get_proxy_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<crate::dto::ProxyDto> {
    let proxy = services::proxies::get_proxy_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(proxy)))
}

/// 创建代理
pub async fn create_proxy_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateProxyRequest>,
) -> Result<CreateResponse> {
    let workspace_uuid = ctx.current_workspace_uuid.ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let proxy_uuid =
        services::proxies::create_proxy_service(&svc_ctx, ctx.user_uuid_unwrap(), workspace_uuid, &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "create",
        "proxy",
        proxy_uuid,
        &payload.name,
        "创建代理"
    )
    .await;

    Ok(Response::success(
        Some("创建成功"),
        Some(CreateResponse { uuid: proxy_uuid }),
    ))
}

/// 更新代理
pub async fn update_proxy_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdateProxyRequest>,
) -> Result<()> {
    services::proxies::update_proxy_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("更新成功"), None))
}

/// 删除代理
pub async fn delete_proxy_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UuidRequest>,
) -> Result<()> {
    services::proxies::delete_proxy_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(&svc_ctx, &ctx, "delete", "proxy", payload.uuid, "删除代理").await;

    Ok(Response::success(Some("删除成功"), None))
}

/// 批量删除代理
pub async fn batch_delete_proxies_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<BatchUuidRequest>,
) -> Result<u64> {
    let count = services::proxies::batch_delete_proxies_service(&svc_ctx, &payload.uuids)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "batch_delete",
        "proxy",
        &format!("批量删除 {} 个代理", count)
    )
    .await;

    Ok(Response::success(Some("删除成功"), Some(count)))
}

/// 批量导入代理
pub async fn batch_import_proxies_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<BatchImportProxiesRequest>,
) -> Result<BatchImportResponse> {
    // 获取工作空间 UUID（从请求或当前工作空间）
    let workspace_uuid = ctx.current_workspace_uuid.ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;

    let result = services::proxies::batch_import_proxies_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        workspace_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "batch_import",
        "proxy",
        &format!(
            "批量导入代理: 成功 {} 个, 失败 {} 个",
            result.success_count, result.failed_count
        )
    )
    .await;

    Ok(Response::success(
        Some("导入完成"),
        Some(BatchImportResponse {
            success_count: result.success_count,
            failed_count: result.failed_count,
            errors: result.errors,
        }),
    ))
}
