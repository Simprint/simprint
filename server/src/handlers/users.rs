use axum::{extract::Extension, extract::State};

use crate::audit_log;
use crate::entitys::{
    LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, ResetPasswordRequest,
    SendCodeRequest, UpdatePasswordRequest, UpdateUserRequest, UserResponse,
    VerifyPasswordRequest, VerifyPasswordResponse,
};
use crate::services::audit::log_audit_anonymous;
use crate::services::users::{
    get_current_user_service, login_service, refresh_token_service, register_service,
    reset_password_service, send_verification_code_service, update_password_service,
    update_user_service, verify_password_service,
};
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 用户注册处理
pub async fn register_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<RegisterRequest>,
) -> Result<RegisterResponse> {
    let result = register_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e.to_string())))?;

    // 记录审计日志（注册时使用返回的 user_uuid）
    if let Some(ref user_info) = result.user_info {
        log_audit_anonymous(
            &svc_ctx,
            &ctx,
            user_info.uuid,
            "register",
            "user",
            "用户注册",
        )
        .await;
    }

    Ok(Response::success(Some("注册成功"), Some(result)))
}

/// 用户登录处理
pub async fn login_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<LoginRequest>,
) -> Result<LoginResponse> {
    let result = login_service(&svc_ctx, payload)
        .await
        .map_err(|e| Response::fail(Some(&e.to_string())))?;

    // 记录审计日志（登录时使用返回的 user_uuid）
    if let Some(ref user_info) = result.user_info {
        log_audit_anonymous(&svc_ctx, &ctx, user_info.uuid, "login", "user", "用户登录").await;
    }

    Ok(Response::success(Some("登录成功"), Some(result)))
}

/// 刷新 Token 处理
pub async fn refresh_token_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<crate::entitys::RefreshTokenRequest>,
) -> Result<LoginResponse> {
    let result = refresh_token_service(&svc_ctx, &payload.refresh_token)
        .await
        .map_err(|e| Response::fail(Some(&e.to_string())))?;

    Ok(Response::success(Some("刷新成功"), Some(result)))
}

/// 获取当前用户信息处理
pub async fn get_current_user_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<UserResponse> {
    let user_info = get_current_user_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e.to_string())))?;

    Ok(Response::success(Some("获取成功"), Some(user_info)))
}

/// 更新用户信息处理
pub async fn update_user_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<()> {
    update_user_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
        .await
        .map_err(|e| Response::fail(Some(&e.to_string())))?;

    Ok(Response::success(Some("更新成功"), None))
}

/// 修改密码处理
pub async fn update_password_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdatePasswordRequest>,
) -> Result<()> {
    update_password_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
        .await
        .map_err(|e| Response::fail(Some(&e.to_string())))?;

    // 记录审计日志
    audit_log!(&svc_ctx, &ctx, "update_password", "user", "修改密码").await;

    Ok(Response::success(Some("修改成功"), None))
}

/// 校验当前用户密码处理
pub async fn verify_password_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<VerifyPasswordRequest>,
) -> Result<VerifyPasswordResponse> {
    let result = verify_password_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
        .await
        .map_err(|e| Response::fail(Some(&e.to_string())))?;

    Ok(Response::success(Some("校验成功"), Some(result)))
}

/// 重置密码处理
pub async fn reset_password_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<()> {
    reset_password_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e.to_string())))?;

    Ok(Response::success(Some("重置成功"), None))
}

/// 发送验证码处理
pub async fn send_code_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<SendCodeRequest>,
) -> Result<()> {
    send_verification_code_service(&svc_ctx, &payload.email, &payload.r#type)
        .await
        .map_err(|e| Response::fail(Some(&e.to_string())))?;

    Ok(Response::success(Some("发送成功"), None))
}
