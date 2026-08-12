use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 版本类型
#[derive(Debug, Deserialize, Serialize, FromRow, Clone, Default)]
pub struct VersionType {
    pub id: i32,
    pub type_code: String,
    pub type_name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub is_auto_download: bool,
}
