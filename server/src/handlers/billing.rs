use axum::extract::{Extension, State};

use crate::dto::{
    AutoRenewalServiceDto, PaymentOrderDto, SubscriptionDto, UserWalletDto,
};
use crate::entitys::{
    AccountInfoResponse, CancelSubscriptionRequest, CreateOrderResponse, CreateRechargeOrderRequest,
    GetPlanPriceRequest, GetPlansRequest, GetUserCouponsRequest, InvoicesListResponse, ListInvoicesRequest,
    ListPaymentOrdersRequest, ListTransactionsRequest, PaymentOrdersListResponse,
    PlanDetailResponse, PlanPriceResponse, PlansResponse, ResumeSubscriptionRequest,
    SubscribePlanRequest, SubscribeResponse, ToggleAutoRenewRequest, TransactionsListResponse,
    UserCouponsListResponse, UuidRequest, VerifyCouponRequest, VerifyCouponResponse,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

// ============ Plans ============

/// 获取套餐列表（包含特性）
pub async fn get_plans_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<GetPlansRequest>,
) -> Result<PlansResponse> {
    let user_uuid = ctx.user_uuid_unwrap();
    let plans_with_features = services::plans::get_plans_service(
        &svc_ctx,
        user_uuid,
        payload.coupon_code.as_deref(),
        &payload.billing_period,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    let plans = plans_with_features
        .into_iter()
        .map(|pwf| crate::entitys::PlanWithFeatures {
            plan: pwf.plan,
            features: pwf.features,
            calculated_price: pwf.calculated_price,
        })
        .collect();

    Ok(Response::success(
        Some("获取成功"),
        Some(PlansResponse { plans }),
    ))
}

/// 获取套餐详情
pub async fn get_plan_detail_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<PlanDetailResponse> {
    let (plan, features) = services::plans::get_plan_detail_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(PlanDetailResponse { plan, features }),
    ))
}

// ============ Subscriptions ============

/// 获取当前订阅
pub async fn get_current_subscription_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<Option<SubscriptionDto>> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;

    let subscription =
        services::subscriptions::get_workspace_subscription_service(&svc_ctx, workspace_uuid)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(subscription)))
}

/// 订阅套餐
pub async fn subscribe_plan_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<SubscribePlanRequest>,
) -> Result<SubscribeResponse> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;

    let subscription_uuid = services::subscriptions::subscribe_plan_service(
        &svc_ctx,
        workspace_uuid,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("订阅成功"),
        Some(SubscribeResponse { subscription_uuid }),
    ))
}

/// 取消订阅
pub async fn cancel_subscription_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CancelSubscriptionRequest>,
) -> Result<()> {
    services::subscriptions::cancel_subscription_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        payload.subscription_uuid,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("取消成功"), None))
}

/// 恢复订阅
pub async fn resume_subscription_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ResumeSubscriptionRequest>,
) -> Result<()> {
    services::subscriptions::resume_subscription_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        payload.subscription_uuid,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("恢复成功"), None))
}

/// 切换自动续费
pub async fn toggle_auto_renew_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ToggleAutoRenewRequest>,
) -> Result<()> {
    services::subscriptions::toggle_auto_renew_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        payload.subscription_uuid,
        payload.auto_renew,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("设置成功"), None))
}

// ============ Wallet ============

/// 获取钱包信息
pub async fn get_wallet_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<UserWalletDto> {
    let wallet = services::wallet::get_wallet_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(wallet)))
}

/// 获取交易记录
pub async fn get_transactions_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListTransactionsRequest>,
) -> Result<TransactionsListResponse> {
    let (items, total) =
        services::wallet::get_transactions_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(TransactionsListResponse {
            items,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

// ============ Invoices ============

/// 获取发票列表
pub async fn get_invoices_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListInvoicesRequest>,
) -> Result<InvoicesListResponse> {
    let (items, total) =
        services::billing::get_invoices_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(InvoicesListResponse {
            items,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

// ============ Quotas ============

/// 获取工作空间配额
pub async fn get_quota_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<crate::dto::WorkspaceQuotaDto> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;

    let quota = services::workspace_quotas::get_workspace_quota_service(
        &svc_ctx,
        &crate::entitys::GetWorkspaceQuotaRequest {
            workspace_uuid: Some(workspace_uuid),
        },
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(quota)))
}

// ============ Coupons ============

/// 验证优惠券
pub async fn verify_coupon_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<VerifyCouponRequest>,
) -> Result<VerifyCouponResponse> {
    let result =
        services::coupons::validate_coupon_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("优惠券有效"),
        Some(VerifyCouponResponse {
            coupon_uuid: result.coupon.uuid,
            discount_type: result.coupon.discount_type,
            discount_value: result.coupon.discount_value,
            discount_amount: result.discount_amount,
        }),
    ))
}

/// 获取用户优惠券列表
pub async fn get_user_coupons_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<GetUserCouponsRequest>,
) -> Result<UserCouponsListResponse> {
    let result = services::coupons::get_user_coupons_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(result)))
}

/// 获取用户可用优惠券
pub async fn get_available_coupons_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<Vec<crate::dto::UserCouponWithDetailsDto>> {
    let result = services::coupons::get_available_coupons_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(result)))
}

/// 获取套餐价格（包含折扣计算）
pub async fn get_plan_price_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<GetPlanPriceRequest>,
) -> Result<PlanPriceResponse> {
    let result = services::plans::calculate_plan_price_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("计算成功"), Some(result)))
}

// ============ Payment Orders ============

/// 创建充值订单
pub async fn create_recharge_order_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateRechargeOrderRequest>,
) -> Result<CreateOrderResponse> {
    let (order_uuid, order_no) =
        services::orders::create_recharge_order_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("订单创建成功"),
        Some(CreateOrderResponse {
            order_uuid,
            order_no,
        }),
    ))
}

/// 获取支付订单列表
pub async fn get_payment_orders_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListPaymentOrdersRequest>,
) -> Result<PaymentOrdersListResponse> {
    let (items, total) =
        services::orders::get_payment_orders_service(&svc_ctx, ctx.user_uuid_unwrap(), &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(PaymentOrdersListResponse {
            items,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

/// 查询订单状态
pub async fn get_order_status_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UuidRequest>,
) -> Result<PaymentOrderDto> {
    let order =
        services::orders::get_order_status_service(&svc_ctx, ctx.user_uuid_unwrap(), payload.uuid)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(order)))
}

// ============ Auto Renewal Services ============

/// 获取自动续费服务列表
pub async fn get_auto_renewal_services_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<Vec<AutoRenewalServiceDto>> {
    let services =
        services::billing::get_auto_renewal_services_service(&svc_ctx, ctx.user_uuid_unwrap())
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(services)))
}

// ============ Account Info ============

/// 获取账户信息
pub async fn get_account_info_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<AccountInfoResponse> {
    let workspace_uuid = ctx
        .current_workspace_uuid
        .ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;

    let account_info = services::billing::get_account_info_service(
        &svc_ctx,
        workspace_uuid,
        ctx.user_uuid_unwrap(),
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(account_info)))
}
