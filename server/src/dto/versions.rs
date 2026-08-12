use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 统一版本表
#[derive(Debug, Deserialize, Serialize, FromRow, Clone, Default)]
pub struct Version {
    pub id: i32,
    pub type_id: i32,
    pub resource_name: String,
    pub version: String,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub platform: Option<String>,
    pub url: Option<String>,
    pub hash: Option<String>,
    pub signature: Option<String>,
    pub install_path: Option<String>,
    pub file_size: Option<i32>,
    pub is_latest: bool,
    pub status: String,
    pub pub_date: Option<DateTime<chrono::Utc>>,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: Option<DateTime<chrono::Utc>>,
    pub deleted_at: Option<DateTime<chrono::Utc>>,
    pub arch: Option<String>,
    pub package_format: Option<String>,
    pub requires_extract: bool,
    pub entrypoint_template: Option<String>,
    pub extract_root: Option<String>,
}
