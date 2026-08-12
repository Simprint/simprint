use axum::{extract::Extension, extract::State};

use crate::entitys::{
    AccountListResponse, BatchImportAccountsRequest, BatchImportResponse, BatchUuidRequest,
    CreateAccountRequest, CreateResponse, ListAccountsRequest, UpdateAccountRequest, UuidRequest,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 获取账号列表
pub async fn get_accounts_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListAccountsRequest>,
) -> Result<AccountListResponse> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let (accounts, total) = services::accounts::get_accounts_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(AccountListResponse {
            items: accounts,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

/// 获取账号详情
pub async fn get_account_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<crate::dto::PlatformAccountDto> {
    let account = services::accounts::get_account_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(account)))
}

/// 创建账号
pub async fn create_account_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<CreateResponse> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let account_uuid = services::accounts::create_account_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("创建成功"),
        Some(CreateResponse { uuid: account_uuid }),
    ))
}

/// 更新账号
pub async fn update_account_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdateAccountRequest>,
) -> Result<()> {
    services::accounts::update_account_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("更新成功"), None))
}

/// 删除账号
pub async fn delete_account_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UuidRequest>,
) -> Result<()> {
    services::accounts::delete_account_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("删除成功"), None))
}

/// 批量删除账号
pub async fn batch_delete_accounts_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<BatchUuidRequest>,
) -> Result<u64> {
    let count = services::accounts::batch_delete_accounts_service(&svc_ctx, &payload.uuids)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("删除成功"), Some(count)))
}

/// 批量导入账号
pub async fn batch_import_accounts_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<BatchImportAccountsRequest>,
) -> Result<BatchImportResponse> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let result = services::accounts::batch_import_accounts_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("导入完成"),
        Some(BatchImportResponse {
            success_count: result.success_count,
            failed_count: result.failed_count,
            errors: result.errors,
        }),
    ))
}
