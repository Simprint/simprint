use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;

/// 创建消息请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateMessageRequest {
    pub message_type: String, // team_invitation, system_notification, etc.
    pub title: String,
    pub content: Option<String>,    // JSON 格式的扩展数据
    pub recipient_uuids: Vec<Uuid>, // 接收者列表（recipient_type='single' 或 'multiple' 时使用）
    pub recipient_type: String,     // single, multiple, team, all
    pub related_type: Option<String>,
    pub related_uuid: Option<Uuid>,
    pub priority: Option<String>,            // low, normal, high, urgent
    pub metadata: Option<serde_json::Value>, // JSON 格式的扩展信息
}

/// 查询消息列表请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListMessagesRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub filters: Option<MessageFilters>,
}

/// 消息筛选条件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageFilters {
    pub message_type: Option<String>,
    pub is_read: Option<bool>,
    pub action_status: Option<String>,
    pub priority: Option<String>,
}

/// 标记消息为已读请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarkMessageReadRequest {
    pub message_uuid: Uuid,
}

/// 批量标记已读请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchMarkReadRequest {
    pub message_uuids: Vec<Uuid>,
}

/// 处理消息请求（用于邀请类消息）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HandleMessageRequest {
    pub message_uuid: Uuid,
    pub action: String, // accept, reject
}

/// 消息列表响应
#[derive(Debug, Clone, Serialize)]
pub struct MessageListResponse {
    pub items: Vec<crate::dto::UserMessageDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 消息统计响应
#[derive(Debug, Clone, Serialize)]
pub struct MessageStatsResponse {
    pub total: i64,
    pub unread: i64,
    pub by_type: std::collections::HashMap<String, i64>,
}
