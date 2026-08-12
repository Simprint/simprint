use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::utils::storage::get_objects;

/// 扩展 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ExtensionDto {
    pub id: i32,
    pub uuid: Uuid,
    pub extension_id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub category: String,
    pub browser: String,
    pub developer: Option<String>,
    pub homepage: Option<String>,
    pub icon_url: Option<String>,
    pub download_url: Option<String>,
    pub file_size: Option<i64>,
    pub downloads_count: Option<i64>,
    pub rating: Option<Decimal>,
    pub permissions: Option<serde_json::Value>,
    pub status: String,
    pub changelog: Option<serde_json::Value>,
    pub published_at: Option<DateTime<Utc>>,
    pub hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ExtensionDto {
    /// 将 object path 转换为完整 URL
    ///
    /// 数据库中存储的是对象存储 object path（如 `ext_id/version/hash.crx`），
    /// 返回给客户端时需要组装为完整 URL。
    ///
    /// 如果 URL 已经是完整的 http(s) 地址，则不进行转换。
    pub fn transform_urls(&mut self, public_base_url: &str, extension_root: &str) {
        // 转换图标 URL
        if let Some(path) = &self.icon_url {
            if !path.is_empty() && !path.starts_with("http") {
                self.icon_url = Some(get_objects::get_extension_icon_url(
                    public_base_url,
                    extension_root,
                    path,
                ));
            }
        }

        // 转换下载 URL
        if let Some(path) = &self.download_url {
            if !path.is_empty() && !path.starts_with("http") {
                self.download_url = Some(get_objects::get_extension_crx_url(
                    public_base_url,
                    extension_root,
                    path,
                ));
            }
        }
    }

    /// 批量转换 ExtensionDto 列表中的 object path 为完整 URL
    pub fn transform_urls_batch(
        extensions: &mut [ExtensionDto],
        public_base_url: &str,
        extension_root: &str,
    ) {
        for ext in extensions.iter_mut() {
            ext.transform_urls(public_base_url, extension_root);
        }
    }
}

/// 用户扩展 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserExtensionDto {
    pub id: i32,
    pub user_uuid: Uuid,
    pub extension_id: String,
    pub installed_version: String,
    pub status: String,
    pub installed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 团队扩展 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TeamExtensionDto {
    pub id: i32,
    pub team_uuid: Uuid,
    pub extension_id: String,
    pub installed_version: String,
    pub installed_by: Uuid,
    pub status: String,
    pub installed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 分组扩展 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct GroupExtensionDto {
    pub id: i32,
    pub group_uuid: Uuid,
    pub extension_id: String,
    pub installed_version: String,
    pub installed_by: Uuid,
    pub status: String,
    pub is_team_shared: bool,
    pub installed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户团队插件偏好设置 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserTeamExtensionPreferenceDto {
    pub id: i32,
    pub user_uuid: Uuid,
    pub team_uuid: Uuid,
    pub extension_id: String,
    pub is_disabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
