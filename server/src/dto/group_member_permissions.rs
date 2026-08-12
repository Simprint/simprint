use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// 分组权限 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct GroupMemberPermissionDto {
    pub group_uuid: Uuid,
    pub workspace_uuid: Uuid,
    pub team_uuid: Uuid,
    pub user_uuid: Uuid,
    pub permission_type: String,
    pub granted_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 分组权限详情 DTO（包含用户信息）
#[derive(Debug, Clone, Serialize)]
pub struct GroupMemberPermissionDetailDto {
    pub group_uuid: Uuid,
    pub workspace_uuid: Uuid,
    pub team_uuid: Uuid,
    pub user_uuid: Uuid,
    pub permission_type: String,
    pub granted_by: Uuid,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
