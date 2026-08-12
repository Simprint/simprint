use axum::extract::{Extension, State};

use crate::entitys::{
    GetLocalApiConfigRequest, ResetLocalApiKeyRequest, UpdateLocalApiConfigRequest,
    ValidateLocalApiKeyRequest,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

pub async fn get_local_api_config_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(_payload): Json<GetLocalApiConfigRequest>,
) -> Result<crate::dto::LocalApiConfigDto> {
    let config =
        services::local_api::get_local_api_config_service(&svc_ctx, ctx.user_uuid_unwrap())
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(None, Some(config)))
}

pub async fn update_local_api_config_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdateLocalApiConfigRequest>,
) -> Result<crate::dto::LocalApiConfigDto> {
    let config = services::local_api::update_local_api_config_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("更新成功"), Some(config)))
}

pub async fn reset_local_api_key_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(_payload): Json<ResetLocalApiKeyRequest>,
) -> Result<crate::dto::ResetLocalApiKeyDto> {
    let result = services::local_api::reset_local_api_key_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("重置成功"), Some(result)))
}

pub async fn validate_local_api_key_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<ValidateLocalApiKeyRequest>,
) -> Result<crate::dto::ValidateLocalApiKeyDto> {
    let result = services::local_api::validate_local_api_key_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(None, Some(result)))
}
