use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::entitys::{
    BatchIssueCouponRequest, CouponValidationResult, GetUserCouponsRequest, IssueCouponRequest,
    UserCouponsListResponse, VerifyCouponRequest,
};
use crate::models;
use crate::svc_ctx::SvcCtx;

/// 验证优惠券
pub async fn validate_coupon_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &VerifyCouponRequest,
) -> Result<CouponValidationResult, String> {
    let coupon = models::billing::fetch_coupon_by_code(&svc_ctx.db, &payload.code)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "优惠券不存在".to_string())?;

    // 检查有效期
    let now = Utc::now();
    if coupon.valid_from > now {
        return Err("优惠券尚未生效".to_string());
    }
    if let Some(valid_until) = coupon.valid_until {
        if valid_until < now {
            return Err("优惠券已过期".to_string());
        }
    }

    // 检查使用次数限制
    if let Some(max_uses) = coupon.max_uses {
        if coupon.used_count >= max_uses {
            return Err("优惠券已达使用上限".to_string());
        }
    }

    // 检查单用户使用次数限制
    if let Some(max_uses_per_user) = coupon.max_uses_per_user {
        let user_usage =
            models::billing::fetch_coupon_user_usage_count(&svc_ctx.db, coupon.uuid, user_uuid)
                .await
                .map_err(|e| e.to_string())?;
        if user_usage >= max_uses_per_user {
            return Err("您已达该优惠券使用上限".to_string());
        }
    }

    // 检查最低消费
    if let Some(min_amount) = coupon.min_amount {
        if payload.amount < min_amount {
            return Err(format!("订单金额需满 {} 元才可使用此优惠券", min_amount));
        }
    }

    // 计算折扣金额
    let mut discount_amount = match coupon.discount_type.as_str() {
        "percentage" => payload.amount * coupon.discount_value / Decimal::from(100),
        "fixed" => coupon.discount_value,
        _ => Decimal::ZERO,
    };

    // 检查最大折扣限制
    if let Some(max_discount) = coupon.max_discount {
        if discount_amount > max_discount {
            discount_amount = max_discount;
        }
    }

    Ok(CouponValidationResult {
        coupon,
        discount_amount,
    })
}

/// 获取用户优惠券列表
pub async fn get_user_coupons_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &GetUserCouponsRequest,
) -> Result<UserCouponsListResponse, String> {
    let (items, total) = models::billing::fetch_user_coupons(
        &svc_ctx.db,
        user_uuid,
        payload.status.as_deref(),
        payload.pagination.page,
        payload.pagination.page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(UserCouponsListResponse {
        items,
        total,
        page: payload.pagination.page,
        page_size: payload.pagination.page_size,
    })
}

/// 给单个用户发放优惠券
pub async fn issue_coupon_to_user_service(
    svc_ctx: &SvcCtx,
    payload: &IssueCouponRequest,
) -> Result<i32, String> {
    // 验证优惠券存在且有效
    let coupon = models::billing::fetch_coupon_by_uuid(&svc_ctx.db, payload.coupon_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "优惠券不存在或已失效".to_string())?;

    // 如果提供了过期时间，使用它；否则从优惠券继承
    let expires_at = payload.expires_at.or(coupon.valid_until);

    let id = models::billing::insert_user_coupon(
        &svc_ctx.db,
        payload.user_uuid,
        payload.coupon_uuid,
        expires_at,
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("RowNotFound") {
            "该用户已拥有此优惠券".to_string()
        } else {
            e.to_string()
        }
    })?;

    Ok(id)
}

/// 批量发放优惠券
pub async fn batch_issue_coupons_service(
    svc_ctx: &SvcCtx,
    payload: &BatchIssueCouponRequest,
) -> Result<usize, String> {
    // 验证优惠券存在且有效
    let coupon = models::billing::fetch_coupon_by_uuid(&svc_ctx.db, payload.coupon_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "优惠券不存在或已失效".to_string())?;

    // 如果提供了过期时间，使用它；否则从优惠券继承
    let expires_at = payload.expires_at.or(coupon.valid_until);

    let count = models::billing::batch_insert_user_coupons(
        &svc_ctx.db,
        payload.coupon_uuid,
        &payload.user_uuids,
        expires_at,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(count)
}

/// 获取用户可用优惠券（自动过滤过期和已使用的）
pub async fn get_available_coupons_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<Vec<crate::dto::UserCouponWithDetailsDto>, String> {
    let items = models::billing::fetch_available_user_coupons(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    Ok(items)
}
