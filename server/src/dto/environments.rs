use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 分组 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct GroupDto {
    pub id: i32,
    pub uuid: Uuid,
    pub workspace_uuid: Uuid,
    pub team_uuid: Uuid,
    pub team_name: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub created_by: Option<Uuid>,
    pub created_by_name: Option<String>,
    pub environments_count: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 标签 DTO
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TagDto {
    pub id: i32,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub team_uuid: Option<Uuid>,
    pub name: String,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub environments_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 环境 DTO
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EnvironmentDto {
    pub id: i32,
    pub uuid: Uuid,
    pub workspace_uuid: Uuid,
    pub user_uuid: Uuid,
    pub team_uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub group_uuid: Option<Uuid>,
    pub proxy_uuid: Option<Uuid>,
    pub system_info: Option<String>,
    pub kernel_info: Option<String>,
    pub fingerprint_summary: Option<String>,
    pub last_opened_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 环境列表行（基础查询结果）
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EnvironmentRowDto {
    pub id: i32,
    pub uuid: Uuid,
    pub workspace_uuid: Uuid,
    pub user_uuid: Uuid,
    pub team_uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub system_info: Option<String>,
    pub kernel_info: Option<String>,
    pub fingerprint_summary: Option<String>,
    pub group_uuid: Option<Uuid>,
    pub proxy_uuid: Option<Uuid>,
    pub last_opened_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 环境标签关联行（用于批量查询）
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EnvironmentTagRowDto {
    pub environment_uuid: Uuid,
    pub tag_id: i32,
    pub tag_uuid: Uuid,
    pub tag_name: String,
    pub tag_color: Option<String>,
    pub tag_sort_order: Option<i32>,
    pub tag_user_uuid: Uuid,
    pub tag_team_uuid: Option<Uuid>,
    pub tag_environments_count: Option<i32>,
    pub tag_created_at: chrono::DateTime<chrono::Utc>,
    pub tag_updated_at: chrono::DateTime<chrono::Utc>,
    pub tag_deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 环境账号关联行（用于批量查询）
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EnvironmentAccountRowDto {
    pub environment_uuid: Uuid,
    pub account_id: i32,
    pub account_uuid: Uuid,
    pub platform_url: String,
    pub platform_name: Option<String>,
    pub account: String,
    pub account_status: String,
    pub remark: Option<String>,
}

/// 分组查询行（用于批量查询）
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GroupRowDto {
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

/// 代理查询行（用于批量查询，排除敏感数据）
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProxyRowDto {
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub proxy_type: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub status: String,
    pub latency: Option<i32>,
    pub last_check_ip: Option<String>,
}

/// 代理摘要 DTO（用于环境列表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySummaryDto {
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub proxy_type: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub status: String,
    pub latency: Option<i32>,
    pub last_check_ip: Option<String>,
}

/// 分组摘要 DTO（用于环境列表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSummaryDto {
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

/// 标签摘要 DTO（用于环境列表）
#[derive(Debug, Clone, Serialize)]
pub struct TagSummaryDto {
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}

/// 账号摘要 DTO（用于环境列表，排除敏感数据）
#[derive(Debug, Clone, Serialize)]
pub struct AccountSummaryDto {
    pub id: i32,
    pub uuid: Uuid,
    pub platform_url: String,
    pub platform_name: Option<String>,
    pub account: String,
    pub status: String,
    pub remark: Option<String>,
}

/// 环境列表项 DTO（包含完整关联数据）
#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentListItemDto {
    // 基础信息
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub system_info: Option<String>,
    pub kernel_info: Option<String>,
    pub fingerprint_summary: Option<String>,
    pub last_opened_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // 分组详情（完整对象）
    pub group: Option<GroupSummaryDto>,
    // 代理详情（完整对象）
    pub proxy: Option<ProxySummaryDto>,
    // 标签列表（完整对象列表，与环境详情接口保持一致）
    pub tags: Vec<TagDto>,
    // 账号列表（完整对象列表）
    pub accounts: Vec<AccountSummaryDto>,
    // 扩展列表（插件列表）
    pub extensions: Vec<ExtensionSummaryDto>,
}

/// 扩展摘要 DTO（用于环境列表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionSummaryDto {
    pub extension_id: String,
    pub name: String,
    pub version: String,
    pub icon_url: Option<String>,
    pub download_url: Option<String>,
    pub hash: Option<String>,
    pub scope: String, // user, team, group-personal, group-team
}

/// 环境配置 DTO
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EnvironmentConfigDto {
    pub id: i32,
    pub environment_uuid: Uuid,
    pub window_info: serde_json::Value,
    pub basic_settings: serde_json::Value,
    pub fingerprint_settings: serde_json::Value,
    pub device_settings: serde_json::Value,
    pub preference_settings: serde_json::Value,
    pub project_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 环境标签关联 DTO
#[derive(Debug, Clone, FromRow)]
pub struct EnvironmentTagDto {
    pub id: i32,
    pub environment_uuid: Uuid,
    pub tag_uuid: Uuid,
    pub created_at: DateTime<Utc>,
}

/// 环境 URL DTO
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EnvironmentUrlDto {
    pub id: i32,
    pub environment_uuid: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub sort_order: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// 环境 Cookie DTO
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EnvironmentCookieDto {
    pub id: i32,
    pub environment_uuid: Uuid,
    pub site_input: String,
    pub domain: String,
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub http_only: Option<bool>,
    pub secure: Option<bool>,
    pub same_site: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 环境 Cookie 分组 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentCookieGroupDto {
    pub site: String,
    pub cookie_text: String,
}

/// 模板 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TemplateDto {
    pub id: i32,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub team_uuid: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub system_info: Option<String>,
    pub kernel_info: Option<String>,
    pub config_json: serde_json::Value,
    pub usage_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
