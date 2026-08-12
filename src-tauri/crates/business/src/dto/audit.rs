use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// 审计日志 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AuditLogDto {
    pub id: i64,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub team_uuid: Option<Uuid>,
    pub action: String,
    pub target_type: String,
    pub target_uuid: Option<Uuid>,
    pub target_name: Option<String>,
    pub details: Option<String>,
    pub changes: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub created_at: DateTime<Utc>,
    // 用户信息（联表查询）
    pub user_name: Option<String>,
    pub user_email: Option<String>,
}
