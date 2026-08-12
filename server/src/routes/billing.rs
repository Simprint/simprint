use axum::routing::post;

use crate::handlers::{
    cancel_subscription_handler, create_recharge_order_handler, get_account_info_handler,
    get_auto_renewal_services_handler, get_available_coupons_handler,
    get_current_subscription_handler, get_invoices_handler, get_order_status_handler,
    get_payment_orders_handler, get_plan_detail_handler, get_plan_price_handler,
    get_plans_handler, get_quota_handler, get_transactions_handler, get_user_coupons_handler,
    get_wallet_handler, resume_subscription_handler, subscribe_plan_handler,
    toggle_auto_renew_handler, verify_coupon_handler,
};
use crate::routes::route::{self, MetaRoute};

/// 注册计费相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut billing_route = route::RouteGroup::new("/billing");

    // 套餐
    billing_route.add_route_item(route::RouteItem::post("/plans", post(get_plans_handler)));
    billing_route.add_route_item(route::RouteItem::post(
        "/plans/detail",
        post(get_plan_detail_handler),
    ));
    billing_route.add_route_item(route::RouteItem::post(
        "/plans/price",
        post(get_plan_price_handler),
    ));

    // 订阅
    billing_route.add_route_item(route::RouteItem::post(
        "/subscription",
        post(get_current_subscription_handler),
    ));
    billing_route.add_route_item(route::RouteItem::post(
        "/subscription/subscribe",
        post(subscribe_plan_handler),
    ));
    billing_route.add_route_item(route::RouteItem::post(
        "/subscription/cancel",
        post(cancel_subscription_handler),
    ));
    billing_route.add_route_item(route::RouteItem::post(
        "/subscription/resume",
        post(resume_subscription_handler),
    ));
    billing_route.add_route_item(route::RouteItem::post(
        "/subscription/toggle-auto-renew",
        post(toggle_auto_renew_handler),
    ));

    // 钱包
    billing_route.add_route_item(route::RouteItem::post("/wallet", post(get_wallet_handler)));
    billing_route.add_route_item(route::RouteItem::post(
        "/wallet/transactions",
        post(get_transactions_handler),
    ));

    // 发票
    billing_route.add_route_item(route::RouteItem::post(
        "/invoices",
        post(get_invoices_handler),
    ));

    // 配额
    billing_route.add_route_item(route::RouteItem::post("/quota", post(get_quota_handler)));

    // 优惠券
    billing_route.add_route_item(route::RouteItem::post(
        "/coupon/verify",
        post(verify_coupon_handler),
    ));
    billing_route.add_route_item(route::RouteItem::post(
        "/coupons/my-coupons",
        post(get_user_coupons_handler),
    ));
    billing_route.add_route_item(route::RouteItem::post(
        "/coupons/available",
        post(get_available_coupons_handler),
    ));

    // 支付订单
    billing_route.add_route_item(route::RouteItem::post(
        "/orders",
        post(get_payment_orders_handler),
    ));
    billing_route.add_route_item(route::RouteItem::post(
        "/orders/create-recharge",
        post(create_recharge_order_handler),
    ));
    billing_route.add_route_item(route::RouteItem::post(
        "/orders/status",
        post(get_order_status_handler),
    ));

    // 自动续费服务
    billing_route.add_route_item(route::RouteItem::post(
        "/auto-renewal-services",
        post(get_auto_renewal_services_handler),
    ));

    // 账户信息
    billing_route.add_route_item(route::RouteItem::post(
        "/account-info",
        post(get_account_info_handler),
    ));

    meta_route.add_route_group(billing_route);
}
