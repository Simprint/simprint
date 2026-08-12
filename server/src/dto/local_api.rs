use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct LocalApiSettingsDto {
    pub id: i32,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub enabled: bool,
    pub port: i32,
    pub remote_access: bool,
    pub cors_origins: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct LocalApiKeyDto {
    pub id: i32,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub key_prefix: String,
    pub key_hash: String,
    pub api_key: Option<String>,
    pub is_active: bool,
    pub requests_today: i32,
    pub daily_limit: i32,
    pub last_reset_date: NaiveDate,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct LocalApiKeyPermissionDto {
    pub id: i32,
    pub uuid: Uuid,
    pub api_key_id: i32,
    pub permission_code: String,
    pub is_enabled: bool,
    pub rate_limit_per_minute: i32,
    pub rate_limit_per_hour: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LocalApiPermissionDefinitionDto {
    pub id: i32,
    pub uuid: Uuid,
    pub permission_code: String,
    pub name: String,
    pub description: Option<String>,
    pub default_enabled: bool,
    pub default_rate_limit_per_minute: i32,
    pub default_rate_limit_per_hour: i32,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalApiConfigDto {
    pub enabled: bool,
    pub api_key: String,
    pub port: i32,
    pub remote_access: bool,
    pub cors_origins: Vec<String>,
    pub requests_today: i32,
    pub daily_limit: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetLocalApiKeyDto {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateLocalApiKeyDto {
    pub valid: bool,
    pub permission_code: String,
    pub requests_today: i32,
    pub daily_limit: i32,
    pub rate_limit_per_minute: i32,
    pub rate_limit_per_hour: i32,
}
