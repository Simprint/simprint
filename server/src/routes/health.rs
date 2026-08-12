use axum::routing::get;

use crate::handlers::health_check_handler;
use crate::routes::route::{self, MetaRoute};

/// 注册健康检查路由
pub fn register_routes(meta_route: &mut MetaRoute) -> () {
    let mut health_route = route::RouteGroup::new("/health");

    health_route.add_route_item(route::RouteItem::get("/", get(health_check_handler)));

    meta_route.add_route_group(health_route);
}
