use rust_decimal::Decimal;
use uuid::Uuid;

use crate::dto::{CouponDto, PlanDto, PlanFeatureDto};
use crate::entitys::{GetPlanPriceRequest, PlanPriceResponse, VerifyCouponRequest};
use crate::models;
use crate::services::coupons::validate_coupon_service;
use crate::svc_ctx::SvcCtx;

/// 带特性的套餐结构
#[derive(Debug, Clone)]
pub struct PlanWithFeatures {
    pub plan: PlanDto,
    pub features: Vec<PlanFeatureDto>,
    pub calculated_price: Option<crate::entitys::PlanPriceInfo>,
}

/// 获取套餐列表（包含特性）
pub async fn get_plans_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    coupon_code: Option<&str>,
    billing_period: &str,
) -> Result<Vec<PlanWithFeatures>, String> {
    let plans = models::billing::fetch_plans(&svc_ctx.db).await.map_err(|e| e.to_string())?;

    let mut plans_with_features = Vec::new();

    for plan in plans {
        let features = models::billing::fetch_plan_features(&svc_ctx.db, plan.uuid)
            .await
            .map_err(|e| e.to_string())?;

        // 如果提供了优惠券代码，计算价格
        let calculated_price = if let Some(code) = coupon_code {
            let price_result = calculate_plan_price_service(
                svc_ctx,
                user_uuid,
                &GetPlanPriceRequest {
                    plan_uuid: plan.uuid,
                    billing_period: billing_period.to_string(),
                    coupon_code: Some(code.to_string()),
                },
            )
            .await
            .ok(); // 如果计算失败，返回 None，不影响套餐列表返回

            price_result.map(|price| crate::entitys::PlanPriceInfo {
                original_price: price.original_price,
                plan_discount: price.plan_discount,
                coupon_discount: price.coupon_discount,
                final_price: price.final_price,
                total_saved: price.total_saved,
                billing_period: billing_period.to_string(),
            })
        } else {
            None
        };

        plans_with_features.push(PlanWithFeatures {
            plan,
            features,
            calculated_price,
        });
    }

    Ok(plans_with_features)
}

/// 获取套餐详情（包含特性）
pub async fn get_plan_detail_service(
    svc_ctx: &SvcCtx,
    plan_uuid: Uuid,
) -> Result<(PlanDto, Vec<PlanFeatureDto>), String> {
    let plan = models::billing::fetch_plan_by_uuid(&svc_ctx.db, plan_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "套餐不存在".to_string())?;

    let features = models::billing::fetch_plan_features(&svc_ctx.db, plan_uuid)
        .await
        .map_err(|e| e.to_string())?;

    Ok((plan, features))
}

/// 计算套餐最终价格
pub async fn calculate_plan_price_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &GetPlanPriceRequest,
) -> Result<PlanPriceResponse, String> {
    // 1. 获取套餐信息
    let plan = models::billing::fetch_plan_by_uuid(&svc_ctx.db, payload.plan_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "套餐不存在".to_string())?;

    if plan.status != "active" {
        return Err("该套餐已下架".to_string());
    }

    // 2. 计算基础价格（考虑套餐级折扣）
    let (base_price, plan_discount_percent) = match payload.billing_period.as_str() {
        "monthly" => (
            plan.price_per_month,
            plan.discount_monthly.unwrap_or(Decimal::ZERO),
        ),
        "yearly" => (
            plan.price_per_year,
            plan.discount_yearly.unwrap_or(Decimal::ZERO),
        ),
        _ => return Err("无效的计费周期".to_string()),
    };

    // 3. 应用套餐级折扣
    let price_after_plan_discount =
        base_price * (Decimal::from(100) - plan_discount_percent) / Decimal::from(100);
    let plan_discount = base_price - price_after_plan_discount;

    // 4. 验证并应用优惠券（如果有）
    let mut coupon_discount = Decimal::ZERO;
    let mut coupon_info: Option<CouponDto> = None;

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

        coupon_discount = coupon_result.discount_amount;
        coupon_info = Some(coupon_result.coupon);
    }

    // 5. 计算最终价格
    let final_price = price_after_plan_discount - coupon_discount;
    let total_saved = plan_discount + coupon_discount;

    Ok(PlanPriceResponse {
        original_price: base_price,
        plan_discount,
        coupon_discount,
        final_price: final_price.max(Decimal::ZERO), // 确保价格不为负
        total_saved,
        coupon_info,
    })
}
