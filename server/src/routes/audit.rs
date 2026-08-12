use axum::routing::post;

use crate::handlers::{
    export_audit_logs_handler, get_audit_log_detail_handler, get_audit_logs_handler,
    get_audit_stats_handler,
};
use crate::routes::route::{self, MetaRoute};

/// 注册审计日志相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut audit_route = route::RouteGroup::new("/audit");

    audit_route.add_route_item(route::RouteItem::post(
        "/logs",
        post(get_audit_logs_handler),
    ));
    audit_route.add_route_item(route::RouteItem::post(
        "/logs/detail",
        post(get_audit_log_detail_handler),
    ));
    audit_route.add_route_item(route::RouteItem::post(
        "/logs/export",
        post(export_audit_logs_handler),
    ));
    audit_route.add_route_item(route::RouteItem::post(
        "/stats",
        post(get_audit_stats_handler),
    ));

    meta_route.add_route_group(audit_route);
}
