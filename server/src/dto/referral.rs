use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// 推广链接层级 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ReferralLinkTierDto {
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub unlock_threshold: i32,
    pub reward_rate: Decimal,
    pub discount_rate: Decimal,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 推广链接 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ReferralLinkDto {
    pub id: i32,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub code: String,
    pub url: Option<String>,
    pub tier_uuid: Option<Uuid>,
    pub unlocked: Option<bool>,
    pub is_current: Option<bool>,
    pub reward_rate: Decimal,
    pub discount_rate: Decimal,
    pub registered_users: Option<i32>,
    pub paid_users: Option<i32>,
    pub total_consumption: Option<Decimal>,
    pub last_30_days_consumption: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户邀请关系 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserReferralDto {
    pub id: i32,
    pub inviter_uuid: Uuid,
    pub invitee_uuid: Uuid,
    pub link_uuid: Option<Uuid>,
    pub status: String,
    pub total_consumption: Option<Decimal>,
    pub last_30_days_consumption: Option<Decimal>,
    pub registered_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
    pub first_paid_at: Option<DateTime<Utc>>,
}

/// 被邀请用户列表查询行（用于 `referral/users` 的 JOIN 查询）
///
/// 注意：这是 SQL 行映射结构，属于 DTO/数据结构层（而不是 models 业务逻辑层）。
#[derive(Debug, Clone, FromRow)]
pub struct ReferredUserRow {
    pub id: i32,
    pub email: String,
    pub status: String,
    pub link_uuid: Option<Uuid>,
    pub total_consumption: Option<Decimal>,
    pub last_30_days_consumption: Option<Decimal>,
    pub registered_at: DateTime<Utc>,
}

/// 被邀请用户列表项（对齐前端 `ReferredUser`：camelCase + number）
///
/// 前端表格字段：
/// - id
/// - email
/// - registeredAt
/// - status
/// - totalConsumption
/// - last30DaysConsumption
/// - linkId
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferredUserItemDto {
    pub id: String,
    pub email: String,
    pub registered_at: DateTime<Utc>,
    pub status: String,
    pub total_consumption: f64,
    pub last_30_days_consumption: f64,
    pub link_id: String,
}

/// 推荐奖励 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ReferralRewardDto {
    pub id: i64,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub reward_type: String,
    pub points: i32,
    pub description: Option<String>,
    pub referred_user_uuid: Option<Uuid>,
    pub link_uuid: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// 用户推荐积分 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserReferralPointsDto {
    pub id: i32,
    pub user_uuid: Uuid,
    pub total_points: i32,
    pub available_points: i32,
    pub used_points: i32,
    pub pending_points: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 兑换选项 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RedeemOptionDto {
    pub id: i32,
    pub uuid: Uuid,
    pub redeem_type: String,
    pub name: String,
    pub description: Option<String>,
    pub points_required: i32,
    pub value: Decimal,
    pub currency: Option<String>,
    pub exchange_rate: i32,
    pub status: String,
    pub sort_order: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 兑换记录 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RedeemRecordDto {
    pub id: i64,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub option_uuid: Uuid,
    pub points_used: i32,
    pub value: Decimal,
    pub currency: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
