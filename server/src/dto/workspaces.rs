use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// 工作空间 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct WorkspaceDto {
    pub uuid: Uuid,
    pub name: String,
    pub owner_uuid: Uuid,
    pub workspace_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 工作空间摘要 DTO（用于列表显示等场景）
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceSummaryDto {
    pub uuid: Uuid,
    pub name: String,
    pub workspace_type: String,
    pub owner_uuid: Uuid,
}
