use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;

/// 查询标签列表请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListTagsRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub keyword: Option<String>,
}

/// 创建标签请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub color: Option<String>,
}

/// 更新标签请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateTagRequest {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}
