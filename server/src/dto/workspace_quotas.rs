use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// 工作空间配额 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct WorkspaceQuotaDto {
    pub workspace_uuid: Uuid,
    pub max_environments: i32,
    pub used_environments: i32,
    pub max_team_members: i32,
    pub used_team_members: i32,
    pub max_proxies: i32,
    pub used_proxies: i32,
    pub max_rpa_tasks: i32,
    pub used_rpa_tasks: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
