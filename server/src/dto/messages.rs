use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// 消息 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct MessageDto {
    pub id: i32,
    pub uuid: Uuid,
    pub message_type: String,
    pub title: String,
    pub content: Option<String>,
    pub sender_uuid: Option<Uuid>,
    pub recipient_type: String,
    pub related_type: Option<String>,
    pub related_uuid: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub status: String,
    pub priority: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 用户消息关联 DTO（包含消息详情和用户状态）
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserMessageDto {
    // 消息基本信息
    pub message_uuid: Uuid,
    pub message_type: String,
    pub title: String,
    pub content: Option<String>,
    pub sender_uuid: Option<Uuid>,
    pub related_type: Option<String>,
    pub related_uuid: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub priority: String,
    pub message_created_at: DateTime<Utc>,

    // 用户消息状态
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub action_status: Option<String>,
    pub action_at: Option<DateTime<Utc>>,

    // 发送者信息（可选，通过 JOIN 获取）
    #[sqlx(default)]
    pub sender_name: Option<String>,
    #[sqlx(default)]
    pub sender_email: Option<String>,
}

/// 消息统计 DTO
#[derive(Debug, Clone, Serialize)]
pub struct MessageStatsDto {
    pub total: i64,
    pub unread: i64,
    pub by_type: std::collections::HashMap<String, i64>,
}
