use axum::routing::post;

use crate::handlers::{get_preferences_handler, update_preferences_handler};
use crate::routes::route::{self, MetaRoute};

/// 注册用户偏好设置相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut pref_route = route::RouteGroup::new("/preferences");

    pref_route.add_route_item(route::RouteItem::post(
        "/get",
        post(get_preferences_handler),
    ));
    pref_route.add_route_item(route::RouteItem::post(
        "/update",
        post(update_preferences_handler),
    ));

    meta_route.add_route_group(pref_route);
}
