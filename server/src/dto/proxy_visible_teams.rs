use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// 代理可见团队 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ProxyVisibleTeamDto {
    pub proxy_uuid: Uuid,
    pub workspace_uuid: Uuid,
    pub team_uuid: Uuid,
    pub created_at: DateTime<Utc>,
}

/// 代理可见团队详情 DTO（包含团队信息）
#[derive(Debug, Clone, Serialize)]
pub struct ProxyVisibleTeamDetailDto {
    pub proxy_uuid: Uuid,
    pub workspace_uuid: Uuid,
    pub team_uuid: Uuid,
    pub team_name: Option<String>,
    pub created_at: DateTime<Utc>,
}
