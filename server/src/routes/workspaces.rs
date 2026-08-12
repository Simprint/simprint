use axum::routing::post;

use crate::handlers::workspaces;
use crate::routes::route::{self, MetaRoute};

/// 注册工作空间相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut workspace_route = route::RouteGroup::new("/workspaces");

    workspace_route.add_route_item(route::RouteItem::post(
        "/create",
        post(workspaces::create_workspace_handler),
    ));
    workspace_route.add_route_item(route::RouteItem::post(
        "/list",
        post(workspaces::get_my_workspaces_handler),
    ));
    workspace_route.add_route_item(route::RouteItem::post(
        "/get",
        post(workspaces::get_workspace_handler),
    ));
    workspace_route.add_route_item(route::RouteItem::post(
        "/update",
        post(workspaces::update_workspace_handler),
    ));
    workspace_route.add_route_item(route::RouteItem::post(
        "/delete",
        post(workspaces::delete_workspace_handler),
    ));
    workspace_route.add_route_item(route::RouteItem::post(
        "/switch",
        post(workspaces::switch_workspace_handler),
    ));

    meta_route.add_route_group(workspace_route);
}
