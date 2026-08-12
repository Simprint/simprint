use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ========== 请求参数 ==========

/// 分页请求参数
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_order: Option<String>,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

/// UUID 请求参数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UuidRequest {
    pub uuid: Uuid,
}

/// 批量 UUID 请求参数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchUuidRequest {
    pub uuids: Vec<Uuid>,
}

// ========== 响应结构体 ==========

/// 创建资源响应（返回新创建资源的 UUID）
#[derive(Debug, Clone, Serialize)]
pub struct CreateResponse {
    pub uuid: Uuid,
}

/// 创建资源响应（返回新创建资源的数字 ID）
#[derive(Debug, Clone, Serialize)]
pub struct IdResponse {
    pub id: i32,
}

/// 邀请响应
#[derive(Debug, Clone, Serialize)]
pub struct InviteResponse {
    pub invitation_uuid: Uuid,
}

/// 批量导入响应
#[derive(Debug, Clone, Serialize)]
pub struct BatchImportResponse {
    pub success_count: i32,
    pub failed_count: i32,
    pub errors: Vec<String>,
}
