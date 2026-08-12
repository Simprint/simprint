use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 设置代理可见性请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SetProxyVisibleRequest {
    pub proxy_uuid: Uuid,
    pub team_uuid: Uuid,
}

/// 移除代理可见性请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoveProxyVisibleRequest {
    pub proxy_uuid: Uuid,
    pub team_uuid: Uuid,
}

/// 批量设置代理可见性请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchSetProxyVisibleRequest {
    pub proxy_uuid: Uuid,
    pub team_uuids: Vec<Uuid>,
}

/// 查询代理可见团队请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListProxyVisibleTeamsRequest {
    pub proxy_uuid: Uuid,
}

/// 查询可见代理请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListVisibleProxiesRequest {
    pub workspace_uuid: Uuid,
    pub team_uuid: Option<Uuid>,
}

// ========== 响应结构体 ==========

/// 代理可见团队列表响应
#[derive(Debug, Clone, Serialize)]
pub struct ProxyVisibleTeamListResponse {
    pub items: Vec<crate::dto::ProxyVisibleTeamDetailDto>,
}

/// 可见代理列表响应
#[derive(Debug, Clone, Serialize)]
pub struct VisibleProxyListResponse {
    pub items: Vec<crate::dto::ProxyDto>,
}
