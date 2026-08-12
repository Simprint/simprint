use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// 用户基础信息 DTO
#[derive(Debug, Clone, FromRow)]
pub struct UserDto {
    pub uuid: Uuid,
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 用户详细信息 DTO
#[derive(Debug, Clone, FromRow)]
pub struct UserInfoDto {
    pub id: i32,
    pub user_uuid: Uuid,
    pub nickname: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub password: String,
    pub avatar_hash: Option<String>,
    pub status: String,
    pub current_team_uuid: Option<Uuid>,
    pub current_workspace_uuid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
