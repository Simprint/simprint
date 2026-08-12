use axum::extract::{Extension, State};

use crate::dto::UserPreferenceDto;
use crate::entitys::settings::UpdatePreferencesRequest;
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 获取用户偏好设置
pub async fn get_preferences_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<UserPreferenceDto> {
    let preferences =
        services::preferences::get_preferences_service(&svc_ctx, ctx.user_uuid_unwrap())
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(preferences)))
}

/// 更新用户偏好设置
pub async fn update_preferences_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdatePreferencesRequest>,
) -> Result<UserPreferenceDto> {
    let preferences = services::preferences::update_preferences_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("更新成功"), Some(preferences)))
}
