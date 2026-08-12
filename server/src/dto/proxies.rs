use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// 代理 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ProxyDto {
    pub id: i32,
    pub uuid: Uuid,
    pub workspace_uuid: Uuid,
    pub owner_uuid: Uuid,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub proxy_type: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssh_key_encrypted: Option<String>,
    pub ssh_passphrase_encrypted: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub status: String,
    pub latency: Option<i32>,
    pub last_check_ip: Option<String>,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub environments_count: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 代理健康检查 DTO
#[derive(Debug, Clone, FromRow)]
pub struct ProxyHealthCheckDto {
    pub id: i64,
    pub proxy_uuid: Uuid,
    pub status: String,
    pub latency: Option<i32>,
    pub ip_address: Option<String>,
    pub error_message: Option<String>,
    pub checked_at: DateTime<Utc>,
}
