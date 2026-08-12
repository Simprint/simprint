use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 平台账号 DTO
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PlatformAccountDto {
    pub id: i32,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub team_uuid: Option<Uuid>,
    pub platform_url: String,
    pub platform_name: Option<String>,
    pub account: String,
    pub password: Option<String>,
    pub status: String,
    pub remark: Option<String>,
    pub usage_count: Option<i32>,
    pub environments_count: Option<i64>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 环境账号关联 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct EnvironmentAccountDto {
    pub id: i32,
    pub environment_uuid: Uuid,
    pub account_uuid: Uuid,
    pub sort_order: Option<i32>,
    pub created_at: DateTime<Utc>,
}
