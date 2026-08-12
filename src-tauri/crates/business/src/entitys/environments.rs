use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;

/// 查询环境列表请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListEnvironmentsRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub filters: Option<EnvironmentFilters>,
}

/// 环境筛选条件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvironmentFilters {
    pub keyword: Option<String>,
    pub status: Option<String>,
    pub group_uuid: Option<Uuid>,
    pub tag_uuids: Option<Vec<Uuid>>,
    pub created_from: Option<String>,
    pub created_to: Option<String>,
}

/// 创建环境请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateEnvironmentRequest {
    pub name: String,
    pub description: Option<String>,
    pub group_uuid: Option<Uuid>,
    pub tag_uuids: Option<Vec<Uuid>>,
    pub account_uuids: Option<Vec<Uuid>>,
    pub proxy_uuid: Option<Uuid>,     // 代理 UUID（单个，可选）
    pub cookies: Option<Vec<CookieGroupInput>>,
    pub urls: Option<Vec<UrlInput>>,
    pub config: EnvironmentConfigRequest,
}

/// 环境配置请求（对应 WindowConfig）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvironmentConfigRequest {
    pub window_info: serde_json::Value,
    pub basic_settings: serde_json::Value,
    pub fingerprint_settings: serde_json::Value,
    pub device_settings: serde_json::Value,
    pub preference_settings: serde_json::Value,
    #[serde(default)]
    pub project_metadata: serde_json::Value,
}

/// 批量创建环境请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchCreateEnvironmentRequest {
    pub environments: Vec<CreateEnvironmentRequest>, // 环境创建请求数组
}

/// 更新环境请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateEnvironmentRequest {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub group_uuid: Option<Uuid>,
    pub cookies: Option<Vec<CookieGroupInput>>,
    pub urls: Option<Vec<UrlInput>>,
    pub config: Option<EnvironmentConfigRequest>,
}

/// 设置环境代理请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SetEnvironmentProxyRequest {
    pub uuid: Uuid,
    pub proxy_uuid: Option<Uuid>,
}

/// 设置环境账号请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SetEnvironmentAccountsRequest {
    pub uuid: Uuid,
    pub account_uuids: Vec<Uuid>,
}

/// 分配标签请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssignTagsRequest {
    pub uuid: Uuid,
    pub tag_uuids: Vec<Uuid>,
}

/// 批量分配标签请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchAssignTagRequest {
    pub env_uuids: Vec<Uuid>,
    pub tag_uuid: Uuid,
}

/// 批量移除标签请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchRemoveTagsRequest {
    pub env_uuids: Vec<Uuid>,
    pub tag_uuid: Option<Uuid>,
}

/// 移除标签请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoveTagRequest {
    pub uuid: Uuid,
    pub tag_uuid: Uuid,
}

/// 移动到分组请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MoveToGroupRequest {
    pub uuid: Uuid,
    pub group_uuid: Option<Uuid>,
}

/// 批量移动到分组请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchMoveToGroupRequest {
    pub env_uuids: Vec<Uuid>,
    pub group_uuid: Uuid,
}

// ============ Environment URLs ============

/// 添加环境 URL 请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddEnvironmentUrlRequest {
    pub environment_uuid: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub sort_order: Option<i32>,
}

/// 批量添加环境 URL 请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchAddEnvironmentUrlsRequest {
    pub environment_uuid: Uuid,
    pub urls: Vec<UrlInput>,
}

/// URL 输入
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UrlInput {
    pub url: String,
    pub title: Option<String>,
    pub sort_order: Option<i32>,
}

/// 删除环境 URL 请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteEnvironmentUrlRequest {
    pub id: i32,
}

/// 清空环境 URL 请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClearEnvironmentUrlsRequest {
    pub environment_uuid: Uuid,
}

// ============ Environment Cookies ============

/// 添加环境 Cookie 请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddEnvironmentCookieRequest {
    pub environment_uuid: Uuid,
    pub site: String,
    pub cookie_text: String,
}

/// 批量添加环境 Cookie 请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchAddEnvironmentCookiesRequest {
    pub environment_uuid: Uuid,
    pub cookies: Vec<CookieGroupInput>,
}

/// Cookie 分组输入
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CookieGroupInput {
    pub site: String,
    pub cookie_text: String,
}

/// 删除环境 Cookie 请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteEnvironmentCookieRequest {
    pub id: i32,
}

/// 清空环境 Cookie 请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClearEnvironmentCookiesRequest {
    pub environment_uuid: Uuid,
}

/// Cookie 输入结构（用于批量添加）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CookieInput {
    pub site_input: String,
    pub domain: String,
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub http_only: Option<bool>,
    pub secure: Option<bool>,
    pub same_site: Option<String>,
}

// ========== 响应结构体 ==========

/// 环境列表响应（使用与环境详情一致的数据结构）
#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentListResponse {
    pub items: Vec<EnvironmentDetailResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 环境详情响应（包含完整配置）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentDetailResponse {
    pub environment: crate::dto::EnvironmentDto,
    pub config: Option<crate::dto::EnvironmentConfigDto>,
    pub cookies: Vec<crate::dto::EnvironmentCookieGroupDto>,
    pub urls: Vec<crate::dto::EnvironmentUrlDto>,
    pub tags: Vec<crate::dto::TagDto>,
    pub accounts: Vec<crate::dto::PlatformAccountDto>,
    pub group: Option<crate::dto::GroupSummaryDto>, // 分组完整信息
    pub proxy: Option<crate::dto::ProxySummaryDto>, // 代理完整信息
    pub extensions: Vec<crate::dto::environments::ExtensionSummaryDto>, // 扩展列表
}
