use axum::extract::{Extension, State};

use crate::dto::{RedeemOptionDto, ReferralLinkTierDto, UserReferralPointsDto};
use crate::entitys::{
    ListReferralRewardsRequest, ListReferredUsersRequest, Pagination, RedeemPointsRequest,
    RedeemRecordsListResponse, RedeemResponse, ReferralDashboardResponse,
    ReferralLinksListResponse, ReferralPlanSummaryResponse, ReferralRewardsListResponse,
    ReferralStatsResponse, ReferredUsersListResponse, SwitchReferralLinkRequest,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 获取推广统计
pub async fn get_referral_stats_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<ReferralStatsResponse> {
    let stats = services::referral::get_referral_stats_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(stats)))
}

/// 获取推广看板聚合数据
pub async fn get_referral_dashboard_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<ReferralDashboardResponse> {
    let dashboard = services::referral::get_referral_dashboard_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(dashboard)))
}

/// 获取套餐页推广摘要信息
pub async fn get_referral_plan_summary_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<ReferralPlanSummaryResponse> {
    let summary =
        services::referral::get_referral_plan_summary_service(&svc_ctx, ctx.user_uuid_unwrap())
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(summary)))
}

/// 获取推广链接层级配置
pub async fn get_referral_tiers_handler(
    State(svc_ctx): State<SvcCtx>,
) -> Result<Vec<ReferralLinkTierDto>> {
    let tiers = services::referral::get_referral_tiers_service(&svc_ctx)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(tiers)))
}

/// 获取推广链接列表
pub async fn get_referral_links_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<ReferralLinksListResponse> {
    let (items, current_link) =
        services::referral::get_referral_links_service(&svc_ctx, ctx.user_uuid_unwrap())
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(ReferralLinksListResponse {
            items,
            current_link,
        }),
    ))
}

/// 切换当前推广链接
pub async fn switch_referral_link_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<SwitchReferralLinkRequest>,
) -> Result<()> {
    services::referral::switch_referral_link_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        payload.link_uuid,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("切换成功"), None))
}

/// 获取奖励记录列表
pub async fn get_referral_rewards_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListReferralRewardsRequest>,
) -> Result<ReferralRewardsListResponse> {
    let (items, total) = services::referral::get_referral_rewards_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(ReferralRewardsListResponse {
            items,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

/// 获取被邀请用户列表
pub async fn get_referred_users_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListReferredUsersRequest>,
) -> Result<ReferredUsersListResponse> {
    let (items, total) =
        services::referral::get_referred_users_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(ReferredUsersListResponse {
            items,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

/// 获取用户积分
pub async fn get_user_points_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<UserReferralPointsDto> {
    let points = services::referral::get_user_points_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(points)))
}

/// 获取兑换选项
pub async fn get_redeem_options_handler(
    State(svc_ctx): State<SvcCtx>,
) -> Result<Vec<RedeemOptionDto>> {
    let options = services::referral::get_redeem_options_service(&svc_ctx)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(options)))
}

/// 执行积分兑换
pub async fn redeem_points_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<RedeemPointsRequest>,
) -> Result<RedeemResponse> {
    let (record_uuid, points_used, value) =
        services::referral::redeem_points_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("兑换成功"),
        Some(RedeemResponse {
            record_uuid,
            points_used,
            value,
        }),
    ))
}

/// 获取兑换记录
pub async fn get_redeem_records_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<Pagination>,
) -> Result<RedeemRecordsListResponse> {
    let (items, total) =
        services::referral::get_redeem_records_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(RedeemRecordsListResponse {
            items,
            total,
            page: payload.page,
            page_size: payload.page_size,
        }),
    ))
}
