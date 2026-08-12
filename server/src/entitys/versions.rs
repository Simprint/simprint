use chrono::DateTime;
use serde::{Deserialize, Serialize};

/// 创建版本请求
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct CreateVersionRequest {
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
    pub pub_date: Option<DateTime<chrono::Utc>>,
    pub arch: Option<String>,
    pub package_format: Option<String>,
    pub requires_extract: Option<bool>,
    pub entrypoint_template: Option<String>,
    pub extract_root: Option<String>,
}

/// 更新版本请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateVersionRequest {
    pub name: Option<String>,
    pub notes: Option<String>,
    pub platform: Option<String>,
    pub url: Option<String>,
    pub hash: Option<String>,
    pub signature: Option<String>,
    pub install_path: Option<String>,
    pub file_size: Option<i32>,
    pub status: Option<String>,
    pub is_latest: Option<bool>,
    pub arch: Option<String>,
    pub package_format: Option<String>,
    pub requires_extract: Option<bool>,
    pub entrypoint_template: Option<String>,
    pub extract_root: Option<String>,
}

/// 版本列表响应
#[derive(Debug, Deserialize, Serialize)]
pub struct VersionListResponse {
    pub total: i64,
    pub list: Vec<crate::dto::versions::Version>,
}

/// 查询版本参数
#[derive(Debug, Deserialize, Serialize)]
pub struct QueryVersionParams {
    pub resource_name: Option<String>,
    pub platform: Option<String>,
    pub status: Option<String>,
}

/// 根据ID获取版本请求
#[derive(Debug, Deserialize, Serialize)]
pub struct GetVersionByIdRequest {
    pub id: i32,
}

/// 根据资源名称和版本号获取版本请求
#[derive(Debug, Deserialize, Serialize)]
pub struct GetVersionByNameAndVersionRequest {
    pub resource_name: String,
    pub version: String,
}

/// 获取最新版本请求
#[derive(Debug, Deserialize, Serialize)]
pub struct GetLatestVersionRequest {
    pub resource_name: String,
    pub platform: String,
}

/// 查询版本列表请求
#[derive(Debug, Deserialize, Serialize)]
pub struct ListVersionsRequest {
    pub params: QueryVersionParams,
    pub page_num: Option<i32>,
    pub page_size: Option<i32>,
}

/// 更新版本请求（含 id）
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateVersionHandleRequest {
    pub id: i32,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub platform: Option<String>,
    pub url: Option<String>,
    pub hash: Option<String>,
    pub signature: Option<String>,
    pub install_path: Option<String>,
    pub file_size: Option<i32>,
    pub status: Option<String>,
    pub is_latest: Option<bool>,
}

/// 删除版本请求
#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteVersionRequest {
    pub id: i32,
}

/// 设置最新版本请求
#[derive(Debug, Deserialize, Serialize)]
pub struct SetLatestVersionRequest {
    pub type_id: i32,
    pub resource_name: String,
    pub version_id: i32,
}

/// 版本回退请求
#[derive(Debug, Deserialize, Serialize)]
pub struct RollbackVersionRequest {
    pub type_id: i32,
    pub resource_name: String,
    pub target_version_id: i32,
}

/// 版本对比请求
#[derive(Debug, Deserialize, Serialize)]
pub struct CompareVersionsRequest {
    pub version_id_1: i32,
    pub version_id_2: i32,
}
