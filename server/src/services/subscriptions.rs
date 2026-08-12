use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::dto::SubscriptionDto;
use crate::entitys::{SubscribePlanRequest, VerifyCouponRequest};
use crate::models;
use crate::svc_ctx::SvcCtx;

use super::coupons::validate_coupon_service;
use super::wallet::deduct_wallet_service;
use crate::models::workspace_quotas;

/// 获取当前订阅（按用户）
pub async fn get_current_subscription_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<Option<SubscriptionDto>, String> {
    models::billing::fetch_active_subscription(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 获取工作空间当前订阅
pub async fn get_workspace_subscription_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
) -> Result<Option<SubscriptionDto>, String> {
    models::billing::fetch_workspace_active_subscription(&svc_ctx.db, workspace_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 订阅套餐
pub async fn subscribe_plan_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    user_uuid: Uuid,
    payload: &SubscribePlanRequest,
) -> Result<Uuid, String> {
    // 1. 获取套餐信息
    let plan = models::billing::fetch_plan_by_uuid(&svc_ctx.db, payload.plan_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "套餐不存在".to_string())?;

    if plan.status != "active" {
        return Err("该套餐已下架".to_string());
    }

    // 2. 计算基础价格和套餐折扣
    let (base_price, plan_discount_percent, duration_days) = match payload.billing_period.as_str() {
        "monthly" => (
            plan.price_per_month,
            plan.discount_monthly.unwrap_or(Decimal::ZERO),
            30,
        ),
        "yearly" => (
            plan.price_per_year,
            plan.discount_yearly.unwrap_or(Decimal::ZERO),
            365,
        ),
        _ => return Err("无效的计费周期".to_string()),
    };

    // 应用套餐级折扣
    let price_after_plan_discount =
        base_price * (Decimal::from(100) - plan_discount_percent) / Decimal::from(100);

    // 3. 验证优惠券（如果有）
    let mut final_price = price_after_plan_discount;
    let mut coupon_uuid: Option<Uuid> = None;
    let mut coupon_discount_amount = Decimal::ZERO;

    if let Some(coupon_code) = &payload.coupon_code {
        let coupon_result = validate_coupon_service(
            svc_ctx,
            user_uuid,
            &VerifyCouponRequest {
                code: coupon_code.clone(),
                amount: price_after_plan_discount, // 在套餐折扣后的价格上应用优惠券
            },
        )
        .await?;

        coupon_uuid = Some(coupon_result.coupon.uuid);
        coupon_discount_amount = coupon_result.discount_amount;
        final_price = price_after_plan_discount - coupon_discount_amount;
    }

    // 4. 检查支付方式并验证钱包余额（仅钱包支付需要检查）
    let payment_method = payload.payment_method.as_deref().unwrap_or("wallet");
    let is_wallet_payment = payment_method == "wallet";

    if is_wallet_payment {
        let wallet = models::billing::fetch_user_wallet(&svc_ctx.db, user_uuid)
            .await
            .map_err(|e| e.to_string())?;

        let wallet_balance = wallet.map(|w| w.balance).unwrap_or(Decimal::ZERO);
        if wallet_balance < final_price {
            return Err("钱包余额不足，请先充值".to_string());
        }
    }

    // 5. 取消工作空间现有订阅
    if let Some(existing) = models::billing::fetch_workspace_active_subscription(&svc_ctx.db, workspace_uuid)
        .await
        .map_err(|e| e.to_string())?
    {
        models::billing::cancel_subscription(&svc_ctx.db, existing.uuid)
            .await
            .map_err(|e| e.to_string())?;
    }

    // 6. 计算到期时间
    let expires_at = Utc::now() + chrono::Duration::days(duration_days);
    let next_billing_date = expires_at.date_naive();

    // 7. 创建订阅
    let subscription_uuid = models::billing::insert_subscription(
        &svc_ctx.db,
        workspace_uuid,
        user_uuid,
        payload.plan_uuid,
        &payload.billing_period,
        final_price,
        &plan.currency,
        expires_at,
        next_billing_date,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 7.1. 记录优惠券使用（如果有）
    if let Some(coupon_uuid_val) = coupon_uuid {
        // 记录优惠券使用
        models::billing::insert_coupon_usage(
            &svc_ctx.db,
            coupon_uuid_val,
            user_uuid,
            Some(subscription_uuid),
            coupon_discount_amount,
        )
        .await
        .map_err(|e| e.to_string())?;

        // 更新用户优惠券状态为 used（如果是从 user_coupons 表发放的）
        models::billing::update_user_coupon_status(
            &svc_ctx.db,
            user_uuid,
            coupon_uuid_val,
            "used",
            Some(Utc::now()),
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    // 8. 扣减钱包余额（仅钱包支付需要扣减）
    if is_wallet_payment {
        deduct_wallet_service(
            svc_ctx,
            user_uuid,
            final_price,
            "订阅套餐",
            Some(subscription_uuid),
        )
        .await?;
    }

    // 9. 更新工作空间配额
    workspace_quotas::insert_or_update_workspace_quota(
        &svc_ctx.db,
        workspace_uuid,
        plan.max_environments,
        plan.max_team_members,
        plan.max_proxies,
        plan.max_rpa_tasks,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 10. 创建发票
    let invoice_number = format!("INV-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"));
    models::billing::insert_invoice(
        &svc_ctx.db,
        user_uuid,
        &invoice_number,
        final_price,
        &plan.currency,
        Some(subscription_uuid),
        None,
        "subscription",
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(subscription_uuid)
}

/// 取消订阅
pub async fn cancel_subscription_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    subscription_uuid: Uuid,
) -> Result<(), String> {
    // 验证订阅属于当前用户
    let subscription = models::billing::fetch_subscription_by_uuid(&svc_ctx.db, subscription_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "订阅不存在".to_string())?;

    if subscription.user_uuid != user_uuid {
        return Err("无权操作此订阅".to_string());
    }

    if subscription.status == "cancelled" {
        return Err("订阅已取消".to_string());
    }

    models::billing::cancel_subscription(&svc_ctx.db, subscription_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 恢复订阅
pub async fn resume_subscription_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    subscription_uuid: Uuid,
) -> Result<(), String> {
    let subscription = models::billing::fetch_subscription_by_uuid(&svc_ctx.db, subscription_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "订阅不存在".to_string())?;

    if subscription.user_uuid != user_uuid {
        return Err("无权操作此订阅".to_string());
    }

    if subscription.status != "cancelled" {
        return Err("只能恢复已取消的订阅".to_string());
    }

    // 检查是否过期
    if subscription.expires_at < Utc::now() {
        return Err("订阅已过期，请重新订阅".to_string());
    }

    models::billing::resume_subscription(&svc_ctx.db, subscription_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 切换自动续费
pub async fn toggle_auto_renew_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    subscription_uuid: Uuid,
    auto_renew: bool,
) -> Result<(), String> {
    let subscription = models::billing::fetch_subscription_by_uuid(&svc_ctx.db, subscription_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "订阅不存在".to_string())?;

    if subscription.user_uuid != user_uuid {
        return Err("无权操作此订阅".to_string());
    }

    models::billing::toggle_auto_renew(&svc_ctx.db, subscription_uuid, auto_renew)
        .await
        .map_err(|e| e.to_string())
}
