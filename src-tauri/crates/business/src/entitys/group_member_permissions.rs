use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;

/// 授予分组权限请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GrantGroupPermissionRequest {
    pub group_uuid: Uuid,
    pub user_uuid: Uuid,
    pub permission_type: String, // 'read', 'write', 'manage'
}

/// 撤销分组权限请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RevokeGroupPermissionRequest {
    pub group_uuid: Uuid,
    pub user_uuid: Uuid,
}

/// 查询用户分组权限请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListUserGroupPermissionsRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub user_uuid: Uuid,
    pub group_uuid: Option<Uuid>,
}

/// 检查分组权限请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckGroupPermissionRequest {
    pub group_uuid: Uuid,
    pub user_uuid: Uuid,
    pub permission_type: String, // 'read', 'write', 'manage'
}

// ========== 响应结构体 ==========

/// 分组权限列表响应
#[derive(Debug, Clone, Serialize)]
pub struct GroupPermissionListResponse {
    pub items: Vec<crate::dto::GroupMemberPermissionDetailDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 检查权限响应
#[derive(Debug, Clone, Serialize)]
pub struct CheckPermissionResponse {
    pub has_permission: bool,
    pub permission_type: Option<String>,
}
