use axum::routing::get;

use crate::handlers::get_public_key_handler;
use crate::routes::route::{self, MetaRoute};

/// 注册 secret 相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) -> () {
    let mut secret_route = route::RouteGroup::new("/secret");

    secret_route.add_route_item(route::RouteItem::get(
        "/public/key",
        get(get_public_key_handler),
    ));

    meta_route.add_route_group(secret_route);
}
