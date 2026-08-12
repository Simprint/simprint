use axum::routing::post;

use crate::handlers::group_permissions;
use crate::routes::route::{self, MetaRoute};

/// 注册分组权限相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut group_permission_route = route::RouteGroup::new("/group-permissions");

    group_permission_route.add_route_item(route::RouteItem::post(
        "/grant",
        post(group_permissions::grant_group_permission_handler),
    ));
    group_permission_route.add_route_item(route::RouteItem::post(
        "/revoke",
        post(group_permissions::revoke_group_permission_handler),
    ));
    group_permission_route.add_route_item(route::RouteItem::post(
        "/check",
        post(group_permissions::check_group_permission_handler),
    ));
    group_permission_route.add_route_item(route::RouteItem::post(
        "/list",
        post(group_permissions::list_user_group_permissions_handler),
    ));

    meta_route.add_route_group(group_permission_route);
}
