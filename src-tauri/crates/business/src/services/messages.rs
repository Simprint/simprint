use uuid::Uuid;

// DTOs are used through entitys
use crate::entitys::{
    BatchMarkReadRequest, CreateMessageRequest, HandleMessageRequest, ListMessagesRequest,
    MarkMessageReadRequest, MessageListResponse, MessageStatsResponse,
};
use crate::models;
use crate::svc_ctx::SvcCtx;

/// 创建消息
pub async fn create_message_service(
    svc_ctx: &SvcCtx,
    sender_uuid: Option<Uuid>,
    payload: &CreateMessageRequest,
) -> Result<Uuid, String> {
    let priority = payload.priority.as_deref().unwrap_or("normal");

    // 创建消息
    let message_uuid = models::create_message(
        &svc_ctx.db,
        sender_uuid,
        &payload.message_type,
        &payload.title,
        payload.content.as_deref(),
        &payload.recipient_type,
        payload.related_type.as_deref(),
        payload.related_uuid,
        priority,
        payload.metadata.clone(),
    )
    .await
    .map_err(|e| e.to_string())?;

    // 根据接收者类型添加接收者
    match payload.recipient_type.as_str() {
        "single" | "multiple" => {
            if payload.recipient_uuids.is_empty() {
                return Err("接收者列表不能为空".to_string());
            }

            // 对于邀请类消息，设置 action_status 为 pending
            let action_status = if payload.message_type == "team_invitation" {
                Some("pending")
            } else {
                None
            };

            models::add_message_recipients(
                &svc_ctx.db,
                message_uuid,
                &payload.recipient_uuids,
                action_status,
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        "team" => {
            // 团队消息由数据库触发器自动分发，无需手动添加
            if payload.related_type.as_deref() != Some("team") || payload.related_uuid.is_none() {
                return Err("团队消息必须指定 related_type='team' 和 related_uuid".to_string());
            }
        }
        "all" => {
            // 系统广播消息，需要为所有用户创建关联记录
            // 这里可以通过应用层逻辑实现，或者使用数据库触发器
            // 暂时返回错误，提示需要特殊处理
            return Err("系统广播消息暂不支持".to_string());
        }
        _ => {
            return Err(format!("不支持的接收者类型: {}", payload.recipient_type));
        }
    }

    Ok(message_uuid)
}

/// 获取用户消息列表
pub async fn get_user_messages_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &ListMessagesRequest,
) -> Result<MessageListResponse, String> {
    let offset = (payload.pagination.page - 1) * payload.pagination.page_size;

    let filters = payload.filters.as_ref();

    let messages = models::fetch_user_messages(
        &svc_ctx.db,
        user_uuid,
        offset,
        payload.pagination.page_size,
        filters.and_then(|f| f.message_type.as_deref()),
        filters.and_then(|f| f.is_read),
        filters.and_then(|f| f.action_status.as_deref()),
        filters.and_then(|f| f.priority.as_deref()),
    )
    .await
    .map_err(|e| e.to_string())?;

    let total = models::fetch_user_messages_count(
        &svc_ctx.db,
        user_uuid,
        filters.and_then(|f| f.message_type.as_deref()),
        filters.and_then(|f| f.is_read),
        filters.and_then(|f| f.action_status.as_deref()),
        filters.and_then(|f| f.priority.as_deref()),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(MessageListResponse {
        items: messages,
        total,
        page: payload.pagination.page,
        page_size: payload.pagination.page_size,
    })
}

/// 标记消息为已读
pub async fn mark_message_read_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &MarkMessageReadRequest,
) -> Result<(), String> {
    models::mark_message_read(&svc_ctx.db, payload.message_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 批量标记消息为已读
pub async fn batch_mark_messages_read_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &BatchMarkReadRequest,
) -> Result<(), String> {
    if payload.message_uuids.is_empty() {
        return Err("消息列表不能为空".to_string());
    }

    models::batch_mark_messages_read(&svc_ctx.db, &payload.message_uuids, user_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 处理消息（接受/拒绝）
pub async fn handle_message_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &HandleMessageRequest,
) -> Result<(), String> {
    if payload.action != "accept" && payload.action != "reject" {
        return Err("操作类型必须是 'accept' 或 'reject'".to_string());
    }

    models::handle_message(
        &svc_ctx.db,
        payload.message_uuid,
        user_uuid,
        &payload.action,
    )
    .await
    .map_err(|e| e.to_string())
}

/// 获取用户消息统计
pub async fn get_user_message_stats_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<MessageStatsResponse, String> {
    let (total, unread, by_type) = models::fetch_user_message_stats(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    Ok(MessageStatsResponse {
        total,
        unread,
        by_type,
    })
}

/// 删除消息
pub async fn delete_message_service(svc_ctx: &SvcCtx, message_uuid: Uuid) -> Result<(), String> {
    models::delete_message(&svc_ctx.db, message_uuid)
        .await
        .map_err(|e| e.to_string())
}
