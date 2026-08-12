use axum::routing::post;

use crate::handlers::workspace_quotas;
use crate::routes::route::{self, MetaRoute};

/// 注册工作空间配额相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut quota_route = route::RouteGroup::new("/workspace-quotas");

    quota_route.add_route_item(route::RouteItem::post(
        "/get",
        post(workspace_quotas::get_workspace_quota_handler),
    ));
    quota_route.add_route_item(route::RouteItem::post(
        "/update",
        post(workspace_quotas::update_quota_usage_handler),
    ));

    meta_route.add_route_group(quota_route);
}
