use axum::routing::post;

use crate::handlers::proxy_visibility;
use crate::routes::route::{self, MetaRoute};

/// 注册代理可见性相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut proxy_visibility_route = route::RouteGroup::new("/proxy-visibility");

    proxy_visibility_route.add_route_item(route::RouteItem::post(
        "/set",
        post(proxy_visibility::set_proxy_visible_handler),
    ));
    proxy_visibility_route.add_route_item(route::RouteItem::post(
        "/remove",
        post(proxy_visibility::remove_proxy_visible_handler),
    ));
    proxy_visibility_route.add_route_item(route::RouteItem::post(
        "/batch-set",
        post(proxy_visibility::batch_set_proxy_visible_handler),
    ));
    proxy_visibility_route.add_route_item(route::RouteItem::post(
        "/list-visible",
        post(proxy_visibility::get_visible_proxies_handler),
    ));
    proxy_visibility_route.add_route_item(route::RouteItem::post(
        "/list-teams",
        post(proxy_visibility::get_proxy_visible_teams_handler),
    ));

    meta_route.add_route_group(proxy_visibility_route);
}
