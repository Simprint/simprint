use axum::routing::post;

use crate::handlers::{
    get_redeem_options_handler, get_redeem_records_handler, get_referral_dashboard_handler,
    get_referral_links_handler, get_referral_plan_summary_handler, get_referral_rewards_handler,
    get_referral_stats_handler, get_referral_tiers_handler, get_referred_users_handler,
    get_user_points_handler, redeem_points_handler, switch_referral_link_handler,
};
use crate::routes::route::{self, MetaRoute};

/// 注册推荐计划相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut referral_route = route::RouteGroup::new("/referral");

    // 统计
    referral_route.add_route_item(route::RouteItem::post(
        "/stats",
        post(get_referral_stats_handler),
    ));

    // 统计看板聚合
    referral_route.add_route_item(route::RouteItem::post(
        "/dashboard",
        post(get_referral_dashboard_handler),
    ));

    // 套餐页推广摘要
    referral_route.add_route_item(route::RouteItem::post(
        "/summary-for-plan",
        post(get_referral_plan_summary_handler),
    ));

    // 层级配置
    referral_route.add_route_item(route::RouteItem::post(
        "/tiers",
        post(get_referral_tiers_handler),
    ));

    // 推广链接
    referral_route.add_route_item(route::RouteItem::post(
        "/links",
        post(get_referral_links_handler),
    ));
    referral_route.add_route_item(route::RouteItem::post(
        "/links/switch",
        post(switch_referral_link_handler),
    ));

    // 奖励记录
    referral_route.add_route_item(route::RouteItem::post(
        "/rewards",
        post(get_referral_rewards_handler),
    ));

    // 被邀请用户
    referral_route.add_route_item(route::RouteItem::post(
        "/users",
        post(get_referred_users_handler),
    ));

    // 积分
    referral_route.add_route_item(route::RouteItem::post(
        "/points",
        post(get_user_points_handler),
    ));

    // 兑换选项
    referral_route.add_route_item(route::RouteItem::post(
        "/redeem/options",
        post(get_redeem_options_handler),
    ));
    referral_route.add_route_item(route::RouteItem::post(
        "/redeem",
        post(redeem_points_handler),
    ));
    referral_route.add_route_item(route::RouteItem::post(
        "/redeem/records",
        post(get_redeem_records_handler),
    ));

    meta_route.add_route_group(referral_route);
}
