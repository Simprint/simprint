use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;

/// 切换推广链接请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SwitchReferralLinkRequest {
    pub link_uuid: Uuid,
}

/// 查询奖励记录请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListReferralRewardsRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub keyword: Option<String>,
    pub reward_type: Option<String>,
    pub status: Option<String>,
}

/// 查询被邀请用户请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListReferredUsersRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub keyword: Option<String>,
    pub status: Option<String>,
}

/// 兑换积分请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedeemPointsRequest {
    pub option_uuid: Uuid,
    pub points: i32,
}

// ========== 响应结构体 ==========

use crate::dto::{
    RedeemOptionDto, RedeemRecordDto, ReferralLinkDto, ReferralLinkTierDto, ReferralRewardDto,
    ReferredUserItemDto, UserReferralPointsDto,
};
use rust_decimal::Decimal;

/// 推广统计响应
#[derive(Debug, Clone, Serialize)]
pub struct ReferralStatsResponse {
    pub total_referrals: i32,
    pub paid_referrals: i32,
    pub total_consumption: Decimal,
    pub last_30_days_consumption: Decimal,
    pub total_rewards: i32,
    pub available_points: i32,
    pub current_tier: Option<ReferralLinkTierDto>,
    pub next_tier: Option<ReferralLinkTierDto>,
    pub upgrade_progress: i32,
}

/// 推广积分摘要（用于看板聚合）
#[derive(Debug, Clone, Serialize)]
pub struct ReferralPointsSummary {
    pub available_points: i32,
    pub pending_points: i32,
    pub total_rewards: i32,
}

/// 推广看板聚合响应
#[derive(Debug, Clone, Serialize)]
pub struct ReferralDashboardResponse {
    pub stats: ReferralStatsResponse,
    pub links: Vec<ReferralLinkDto>,
    pub current_link: Option<ReferralLinkDto>,
    pub tiers: Vec<ReferralLinkTierDto>,
    pub points: ReferralPointsSummary,
}

/// 套餐页推广摘要响应
#[derive(Debug, Clone, Serialize)]
pub struct ReferralPlanSummaryResponse {
    /// 最近 30 天推广产生的消费金额对应的预估收益（按照当前层级 reward_rate 估算）
    pub referral_value_last_30_days: Decimal,
    /// 当前订阅套餐的月度价格（如果有订阅）
    pub current_plan_monthly_price: Option<Decimal>,
    /// 推广收益覆盖当前套餐费用的比例（0-1）
    pub coverage_ratio: Option<Decimal>,
}

/// 推广链接列表响应
#[derive(Debug, Clone, Serialize)]
pub struct ReferralLinksListResponse {
    pub items: Vec<ReferralLinkDto>,
    pub current_link: Option<ReferralLinkDto>,
}

/// 奖励记录列表响应
#[derive(Debug, Clone, Serialize)]
pub struct ReferralRewardsListResponse {
    pub items: Vec<ReferralRewardDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 被邀请用户列表响应
#[derive(Debug, Clone, Serialize)]
pub struct ReferredUsersListResponse {
    pub items: Vec<ReferredUserItemDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 兑换选项列表响应
#[derive(Debug, Clone, Serialize)]
pub struct RedeemOptionsListResponse {
    pub items: Vec<RedeemOptionDto>,
}

/// 兑换响应
#[derive(Debug, Clone, Serialize)]
pub struct RedeemResponse {
    pub record_uuid: Uuid,
    pub points_used: i32,
    pub value: Decimal,
}

/// 用户积分响应
#[derive(Debug, Clone, Serialize)]
pub struct UserPointsResponse {
    pub points: UserReferralPointsDto,
}

/// 兑换记录列表响应
#[derive(Debug, Clone, Serialize)]
pub struct RedeemRecordsListResponse {
    pub items: Vec<RedeemRecordDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}
