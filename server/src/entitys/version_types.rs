use serde::{Deserialize, Serialize};

/// 创建版本类型请求
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateVersionTypeRequest {
    pub type_code: String,
    pub type_name: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

/// 更新版本类型请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateVersionTypeRequest {
    pub type_name: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
    pub is_auto_download: Option<bool>,
}

/// 根据ID获取版本类型请求
#[derive(Debug, Deserialize, Serialize)]
pub struct GetVersionTypeByIdRequest {
    pub id: i32,
}

/// 根据代码获取版本类型请求
#[derive(Debug, Deserialize, Serialize)]
pub struct GetVersionTypeByCodeRequest {
    pub type_code: String,
}

/// 查询所有版本类型请求（可为空 body）
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ListAllVersionTypesRequest {}

/// 查询激活的版本类型请求（可为空 body）
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ListActiveVersionTypesRequest {}

/// 更新版本类型请求（含 id）
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateVersionTypeHandleRequest {
    pub id: i32,
    pub type_name: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

/// 删除版本类型请求
#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteVersionTypeRequest {
    pub id: i32,
}

/// 切换版本类型状态请求
#[derive(Debug, Deserialize, Serialize)]
pub struct ToggleVersionTypeStatusRequest {
    pub id: i32,
    pub is_active: bool,
}

/// 版本类型列表响应
#[derive(Debug, Deserialize, Serialize)]
pub struct VersionTypeListResponse {
    pub list: Vec<crate::dto::version_types::VersionType>,
}
