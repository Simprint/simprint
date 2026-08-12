use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;

/// 查询账号列表请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListAccountsRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub filters: Option<AccountFilters>,
}

/// 账号筛选条件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountFilters {
    pub keyword: Option<String>,
    pub platform_name: Option<String>,
    pub status: Option<String>,
}

/// 创建账号请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateAccountRequest {
    pub platform_url: String,
    pub platform_name: Option<String>,
    pub account: String,
    pub password: Option<String>,
    pub remark: Option<String>,
}

/// 更新账号请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateAccountRequest {
    pub uuid: Uuid,
    pub platform_url: Option<String>,
    pub platform_name: Option<String>,
    pub account: Option<String>,
    pub password: Option<String>,
    pub remark: Option<String>,
    pub status: Option<String>,
}

/// 批量导入账号项（客户端已解析好的结构化数据）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchImportAccountItem {
    pub platform_url: String,
    pub platform_name: Option<String>,
    pub account: String,
    pub password: Option<String>,
    pub remark: Option<String>,
}

/// 批量导入账号请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchImportAccountsRequest {
    pub accounts: Vec<BatchImportAccountItem>,
}

/// 导出账号请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportAccountsRequest {
    pub uuids: Option<Vec<Uuid>>,
    pub format: String,
    pub include_password: bool,
}

// ========== 响应结构体 ==========

/// 账号列表响应
#[derive(Debug, Clone, Serialize)]
pub struct AccountListResponse {
    pub items: Vec<crate::dto::PlatformAccountDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}
