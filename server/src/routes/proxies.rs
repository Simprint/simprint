use axum::routing::post;

use crate::handlers::{
    batch_delete_proxies_handler, batch_import_proxies_handler, create_proxy_handler,
    delete_proxy_handler, get_proxies_handler, get_proxy_handler, update_proxy_handler,
};
use crate::routes::route::{self, MetaRoute};

/// 注册代理相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut proxy_route = route::RouteGroup::new("/proxies");

    proxy_route.add_route_item(route::RouteItem::post("/list", post(get_proxies_handler)));
    proxy_route.add_route_item(route::RouteItem::post("/detail", post(get_proxy_handler)));
    proxy_route.add_route_item(route::RouteItem::post(
        "/create",
        post(create_proxy_handler),
    ));
    proxy_route.add_route_item(route::RouteItem::post(
        "/update",
        post(update_proxy_handler),
    ));
    proxy_route.add_route_item(route::RouteItem::post(
        "/delete",
        post(delete_proxy_handler),
    ));
    proxy_route.add_route_item(route::RouteItem::post(
        "/batch-delete",
        post(batch_delete_proxies_handler),
    ));
    proxy_route.add_route_item(route::RouteItem::post(
        "/batch-import",
        post(batch_import_proxies_handler),
    ));

    meta_route.add_route_group(proxy_route);
}
