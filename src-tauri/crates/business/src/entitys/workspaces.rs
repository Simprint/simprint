use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;

/// 创建工作空间请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub workspace_type: Option<String>,
}

/// 更新工作空间请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateWorkspaceRequest {
    pub uuid: Uuid,
    pub name: Option<String>,
}

/// 切换工作空间请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SwitchWorkspaceRequest {
    pub workspace_uuid: Uuid,
}

/// 查询工作空间列表请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListWorkspacesRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub filters: Option<WorkspaceFilters>,
}

/// 工作空间筛选条件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceFilters {
    pub workspace_type: Option<String>,
    pub keyword: Option<String>,
}

// ========== 响应结构体 ==========

/// 工作空间列表响应
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceListResponse {
    pub current_workspace_uuid: Option<Uuid>,
    pub workspaces: Vec<WorkspaceItem>,
}

/// 工作空间列表项
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceItem {
    pub uuid: Uuid,
    pub name: String,
    pub workspace_type: String,
    pub is_current: bool,
}
