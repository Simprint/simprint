use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// 用户偏好设置 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserPreferenceDto {
    pub id: i32,
    pub user_uuid: Uuid,
    pub theme: String,
    pub language: String,
    pub notifications_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 系统配置 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct SystemConfigDto {
    pub id: i32,
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
