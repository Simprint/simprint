use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// 团队 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TeamDto {
    pub id: i32,
    pub uuid: Uuid,
    pub workspace_uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_uuid: Uuid,
    pub avatar_hash: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 团队摘要 DTO（用于登录响应等场景，只包含基本信息）
#[derive(Debug, Clone, Serialize)]
pub struct TeamSummaryDto {
    pub uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
}

/// 团队成员 DTO（包含用户信息）
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TeamMemberDto {
    pub id: i32,
    pub team_uuid: Uuid,
    pub workspace_uuid: Uuid,
    pub user_uuid: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    // 从 user_infos 表关联的用户信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
}

/// 团队邀请 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TeamInvitationDto {
    pub id: i32,
    pub uuid: Uuid,
    pub team_uuid: Uuid,
    pub email: String,
    pub role: String,
    pub invited_by: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub status: String,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 登录历史 DTO
#[derive(Debug, Clone, FromRow)]
pub struct LoginHistoryDto {
    pub id: i64,
    pub user_uuid: Uuid,
    pub ip_address: String,
    pub device_info: Option<String>,
    pub user_agent: Option<String>,
    pub location: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}
