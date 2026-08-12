use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 获取工作空间配额请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetWorkspaceQuotaRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_uuid: Option<Uuid>,
}

/// 更新配额使用情况请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateQuotaUsageRequest {
    pub workspace_uuid: Uuid,
    pub quota_type: String, // 'environments', 'proxies', 'team_members', 'rpa_tasks'
    pub increment: bool,    // true 为增加，false 为减少
    pub amount: i32,        // 增加或减少的数量
}

// ========== 响应结构体 ==========

/// 工作空间配额响应
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceQuotaResponse {
    pub workspace_uuid: Uuid,
    pub max_environments: i32,
    pub used_environments: i32,
    pub max_team_members: i32,
    pub used_team_members: i32,
    pub max_proxies: i32,
    pub used_proxies: i32,
    pub max_rpa_tasks: i32,
    pub used_rpa_tasks: i32,
}
