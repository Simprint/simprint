use axum::{extract::Extension, extract::State};

use crate::entitys::{
    BatchMarkReadRequest, CreateMessageRequest, HandleMessageRequest, ListMessagesRequest,
    MarkMessageReadRequest, MessageListResponse, MessageStatsResponse,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};
use sqlx;

/// 创建消息
pub async fn create_message_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateMessageRequest>,
) -> Result<crate::entitys::CreateResponse> {
    let message_uuid = services::messages::create_message_service(
        &svc_ctx,
        Some(ctx.user_uuid_unwrap()),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("消息创建成功"),
        Some(crate::entitys::CreateResponse { uuid: message_uuid }),
    ))
}

/// 获取用户消息列表
pub async fn get_user_messages_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListMessagesRequest>,
) -> Result<MessageListResponse> {
    let response =
        services::messages::get_user_messages_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(response)))
}

/// 标记消息为已读
pub async fn mark_message_read_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<MarkMessageReadRequest>,
) -> Result<()> {
    services::messages::mark_message_read_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("标记成功"), None))
}

/// 批量标记消息为已读
pub async fn batch_mark_messages_read_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<BatchMarkReadRequest>,
) -> Result<()> {
    services::messages::batch_mark_messages_read_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("批量标记成功"), None))
}

/// 处理消息（接受/拒绝）
pub async fn handle_message_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<HandleMessageRequest>,
) -> Result<()> {
    services::messages::handle_message_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("处理成功"), None))
}

/// 获取用户消息统计
pub async fn get_user_message_stats_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<MessageStatsResponse> {
    let response =
        services::messages::get_user_message_stats_service(&svc_ctx, ctx.user_uuid_unwrap())
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(response)))
}

/// 删除消息
pub async fn delete_message_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<crate::entitys::UuidRequest>,
) -> Result<()> {
    // 验证消息是否属于当前用户（通过查询 user_messages 表）
    let user_message_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM user_messages
            WHERE message_uuid = $1 AND user_uuid = $2
        )
        "#,
    )
    .bind(payload.uuid)
    .bind(ctx.user_uuid_unwrap())
    .fetch_one(&svc_ctx.db)
    .await
    .map_err(|e| Response::fail(Some(&e.to_string())))?;

    if !user_message_exists {
        return Err(Response::fail(Some("消息不存在或无权限")));
    }

    services::messages::delete_message_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("删除成功"), None))
}
