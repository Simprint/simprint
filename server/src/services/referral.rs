use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::dto::{
    RedeemOptionDto, RedeemRecordDto, ReferralLinkDto, ReferralLinkTierDto, ReferralRewardDto,
    ReferredUserItemDto, UserReferralPointsDto,
};
use crate::entitys::{
    ListReferralRewardsRequest, ListReferredUsersRequest, Pagination, RedeemPointsRequest,
    ReferralDashboardResponse, ReferralPointsSummary, ReferralStatsResponse,
};
use crate::models;
use crate::svc_ctx::SvcCtx;
use crate::services::subscriptions::get_current_subscription_service;

/// 获取推广统计
pub async fn get_referral_stats_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<ReferralStatsResponse, String> {
    // 获取基本统计数据
    let (total_referrals, paid_referrals, total_consumption, last_30_days_consumption) =
        models::referral::fetch_referral_stats(&svc_ctx.db, user_uuid)
            .await
            .map_err(|e| e.to_string())?;

    // 获取总奖励积分
    let total_rewards = models::referral::fetch_total_reward_points(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 获取用户积分
    let points = models::referral::fetch_user_points(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;
    let available_points = points.map(|p| p.available_points).unwrap_or(0);

    // 获取当前层级和下一层级
    let current_tier = models::referral::fetch_tier_by_threshold(&svc_ctx.db, paid_referrals)
        .await
        .map_err(|e| e.to_string())?;

    let current_threshold = current_tier.as_ref().map(|t| t.unlock_threshold).unwrap_or(0);
    let next_tier = models::referral::fetch_next_tier(&svc_ctx.db, current_threshold)
        .await
        .map_err(|e| e.to_string())?;

    // 计算升级进度
    let upgrade_progress = if let Some(ref next) = next_tier {
        let progress = (paid_referrals - current_threshold) as f64
            / (next.unlock_threshold - current_threshold) as f64
            * 100.0;
        progress as i32
    } else {
        100
    };

    Ok(ReferralStatsResponse {
        total_referrals,
        paid_referrals,
        total_consumption,
        last_30_days_consumption,
        total_rewards,
        available_points,
        current_tier,
        next_tier,
        upgrade_progress,
    })
}

/// 获取推广链接层级配置
pub async fn get_referral_tiers_service(
    svc_ctx: &SvcCtx,
) -> Result<Vec<ReferralLinkTierDto>, String> {
    models::referral::fetch_referral_tiers(&svc_ctx.db)
        .await
        .map_err(|e| e.to_string())
}

/// 获取推广链接列表
pub async fn get_referral_links_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<(Vec<ReferralLinkDto>, Option<ReferralLinkDto>), String> {
    // 根据当前统计数据计算已解锁层级，并同步更新该用户所有推广链接的 unlocked 状态
    //
    // 解锁规则：
    // - 对于 unlock_threshold <= paid_referrals 的层级视为已解锁
    // - 这些层级对应的推广链接（按 tier_uuid 关联）统一标记为 unlocked = TRUE
    // - 其余链接统一标记为 unlocked = FALSE
    let (_total_referrals, paid_referrals, _total_consumption, _last_30_days_consumption) =
        models::referral::fetch_referral_stats(&svc_ctx.db, user_uuid)
            .await
            .map_err(|e| e.to_string())?;

    let tiers = models::referral::fetch_referral_tiers(&svc_ctx.db)
        .await
        .map_err(|e| e.to_string())?;

    let unlocked_tier_uuids: Vec<Uuid> = tiers
        .into_iter()
        .filter(|t| t.unlock_threshold <= paid_referrals)
        .map(|t| t.uuid)
        .collect();

    models::referral::update_referral_links_unlock_status_for_user(
        &svc_ctx.db,
        user_uuid,
        &unlocked_tier_uuids,
    )
    .await
    .map_err(|e| e.to_string())?;

    let mut links = models::referral::fetch_user_referral_links(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    let mut current_link = models::referral::fetch_current_referral_link(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 如果配置了推广链接前缀，则在返回前为每个链接拼接完整 URL
    if let Some(prefix) = svc_ctx.config.app.referral_link_prefix.as_deref() {
        let prefix = prefix.trim_end_matches('/');

        for link in &mut links {
            link.url = Some(format!("{prefix}?referral_code={}", link.code));
        }

        if let Some(ref mut cl) = current_link {
            cl.url = Some(format!("{prefix}?referral_code={}", cl.code));
        }
    }

    Ok((links, current_link))
}

/// 获取推广看板聚合数据
pub async fn get_referral_dashboard_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<ReferralDashboardResponse, String> {
    // 基础统计（含 available_points 和 total_rewards）
    let stats = get_referral_stats_service(svc_ctx, user_uuid).await?;

    // 推广链接及当前链接
    let (links, current_link) = get_referral_links_service(svc_ctx, user_uuid).await?;

    // 层级配置
    let tiers = get_referral_tiers_service(svc_ctx).await?;

    // 读取 pending_points，用于前端展示“审核中”积分
    let pending_points = models::referral::fetch_user_points(&svc_ctx.db, user_uuid)
        .await
        .ok()
        .flatten()
        .map(|p| p.pending_points)
        .unwrap_or(0);

    let points = ReferralPointsSummary {
        available_points: stats.available_points,
        pending_points,
        total_rewards: stats.total_rewards,
    };

    Ok(ReferralDashboardResponse {
        stats,
        links,
        current_link,
        tiers,
        points,
    })
}

/// 计算最近 30 天推广带来的预估收益，以及与当前套餐价格的覆盖比例
pub async fn get_referral_plan_summary_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<crate::entitys::ReferralPlanSummaryResponse, String> {
    // 基于推广统计获取最近 30 天消费金额和当前层级
    let stats = get_referral_stats_service(svc_ctx, user_uuid).await?;

    let last_30_days_consumption = stats.last_30_days_consumption;

    // 使用当前层级的 reward_rate 估算可得收益（如果没有当前层级，则按 0 估算）
    let reward_rate = stats
        .current_tier
        .as_ref()
        .map(|tier| tier.reward_rate)
        .unwrap_or(Decimal::ZERO);

    // reward_rate 以百分比存储，例如 10 表示 10%
    let referral_value_last_30_days =
        last_30_days_consumption * reward_rate / Decimal::from(100u32);

    // 获取当前订阅，估算月度套餐价格
    let current_subscription =
        get_current_subscription_service(svc_ctx, user_uuid).await.unwrap_or(None);

    let current_plan_monthly_price = current_subscription.as_ref().map(|sub| sub.price);

    // 计算覆盖比例
    let coverage_ratio = current_plan_monthly_price.map(|price| {
        if price.is_zero() {
            Decimal::ZERO
        } else {
            (referral_value_last_30_days / price).max(Decimal::ZERO)
        }
    });

    Ok(crate::entitys::ReferralPlanSummaryResponse {
        referral_value_last_30_days,
        current_plan_monthly_price,
        coverage_ratio,
    })
}

/// 切换当前推广链接
pub async fn switch_referral_link_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    link_uuid: Uuid,
) -> Result<(), String> {
    models::referral::switch_current_referral_link(&svc_ctx.db, user_uuid, link_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 获取奖励记录列表
pub async fn get_referral_rewards_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &ListReferralRewardsRequest,
) -> Result<(Vec<ReferralRewardDto>, i64), String> {
    let offset = (payload.pagination.page - 1) * payload.pagination.page_size;

    let rewards = models::referral::fetch_referral_rewards(
        &svc_ctx.db,
        user_uuid,
        payload.keyword.as_deref(),
        payload.reward_type.as_deref(),
        payload.status.as_deref(),
        offset,
        payload.pagination.page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    let total = models::referral::fetch_referral_rewards_count(
        &svc_ctx.db,
        user_uuid,
        payload.keyword.as_deref(),
        payload.reward_type.as_deref(),
        payload.status.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok((rewards, total))
}

/// 获取被邀请用户列表
pub async fn get_referred_users_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &ListReferredUsersRequest,
) -> Result<(Vec<ReferredUserItemDto>, i64), String> {
    let offset = (payload.pagination.page - 1) * payload.pagination.page_size;

    let rows = models::referral::fetch_referred_users(
        &svc_ctx.db,
        user_uuid,
        payload.keyword.as_deref(),
        payload.status.as_deref(),
        offset,
        payload.pagination.page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    let users: Vec<ReferredUserItemDto> = rows
        .into_iter()
        .map(|r| ReferredUserItemDto {
            id: r.id.to_string(),
            email: r.email,
            registered_at: r.registered_at,
            status: r.status,
            total_consumption: r.total_consumption.and_then(|d| d.to_f64()).unwrap_or(0.0),
            last_30_days_consumption: r
                .last_30_days_consumption
                .and_then(|d| d.to_f64())
                .unwrap_or(0.0),
            link_id: r.link_uuid.map(|u| u.to_string()).unwrap_or_default(),
        })
        .collect();

    let total = models::referral::fetch_referred_users_count(
        &svc_ctx.db,
        user_uuid,
        payload.keyword.as_deref(),
        payload.status.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok((users, total))
}

/// 获取用户积分
pub async fn get_user_points_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<UserReferralPointsDto, String> {
    let points = models::referral::fetch_user_points(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 如果不存在则创建
    if points.is_none() {
        models::referral::upsert_user_points(&svc_ctx.db, user_uuid, 0, 0, 0, 0)
            .await
            .map_err(|e| e.to_string())?;

        return models::referral::fetch_user_points(&svc_ctx.db, user_uuid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "创建积分记录失败".to_string());
    }

    points.ok_or_else(|| "积分记录不存在".to_string())
}

/// 获取兑换选项
pub async fn get_redeem_options_service(svc_ctx: &SvcCtx) -> Result<Vec<RedeemOptionDto>, String> {
    models::referral::fetch_redeem_options(&svc_ctx.db)
        .await
        .map_err(|e| e.to_string())
}

/// 执行积分兑换
pub async fn redeem_points_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &RedeemPointsRequest,
) -> Result<(Uuid, i32, Decimal), String> {
    // 获取兑换选项
    let option = models::referral::fetch_redeem_option_by_uuid(&svc_ctx.db, payload.option_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "兑换选项不存在".to_string())?;

    if option.status != "active" {
        return Err("该兑换选项已下架".to_string());
    }

    // 检查积分是否满足最低要求
    if payload.points < option.points_required {
        return Err(format!(
            "积分不足，最低需要 {} 积分",
            option.points_required
        ));
    }

    // 获取用户积分
    let points = models::referral::fetch_user_points(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "积分记录不存在".to_string())?;

    if points.available_points < payload.points {
        return Err("可用积分不足".to_string());
    }

    // 计算兑换价值
    let value = Decimal::from(payload.points) / Decimal::from(option.exchange_rate);

    // 扣减积分
    models::referral::deduct_user_points(&svc_ctx.db, user_uuid, payload.points)
        .await
        .map_err(|e| e.to_string())?;

    // 创建兑换记录
    let record_uuid = models::referral::insert_redeem_record(
        &svc_ctx.db,
        user_uuid,
        payload.option_uuid,
        payload.points,
        value,
        option.currency.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok((record_uuid, payload.points, value))
}

/// 获取兑换记录
pub async fn get_redeem_records_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    pagination: &Pagination,
) -> Result<(Vec<RedeemRecordDto>, i64), String> {
    let offset = (pagination.page - 1) * pagination.page_size;

    let records = models::referral::fetch_redeem_records(
        &svc_ctx.db,
        user_uuid,
        offset,
        pagination.page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    let total = models::referral::fetch_redeem_records_count(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    Ok((records, total))
}
