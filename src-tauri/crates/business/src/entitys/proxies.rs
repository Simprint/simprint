use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;

/// 查询代理列表请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListProxiesRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub filters: Option<ProxyFilters>,
}

/// 代理筛选条件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProxyFilters {
    pub keyword: Option<String>,
    pub proxy_type: Option<String>,
    pub status: Option<String>,
    pub country: Option<String>,
}

/// 创建代理请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateProxyRequest {
    pub name: String,
    pub host: String,
    pub port: i32,
    pub proxy_type: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub ssh_key: Option<String>,
    pub ssh_passphrase: Option<String>,
}

/// 更新代理请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateProxyRequest {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub proxy_type: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
}

/// 批量导入代理项（客户端已解析好的结构化数据）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchImportProxyItem {
    pub name: String,
    pub host: String,
    pub port: i32,
    pub proxy_type: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
}

/// 批量导入代理请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchImportProxiesRequest {
    pub proxies: Vec<BatchImportProxyItem>,
}

// ========== 响应结构体 ==========

/// 代理列表响应
#[derive(Debug, Clone, Serialize)]
pub struct ProxyListResponse {
    pub items: Vec<crate::dto::ProxyDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}
