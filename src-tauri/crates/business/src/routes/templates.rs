use axum::routing::post;

use crate::handlers::{
    apply_template_handler, create_from_template_handler, create_template_handler,
    delete_template_handler, get_template_handler, get_templates_handler, update_template_handler,
};
use crate::routes::route::{self, MetaRoute};

/// 注册模板相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut template_route = route::RouteGroup::new("/templates");

    template_route.add_route_item(route::RouteItem::post("/list", post(get_templates_handler)));
    template_route.add_route_item(route::RouteItem::post(
        "/detail",
        post(get_template_handler),
    ));
    template_route.add_route_item(route::RouteItem::post(
        "/create",
        post(create_template_handler),
    ));
    template_route.add_route_item(route::RouteItem::post(
        "/update",
        post(update_template_handler),
    ));
    template_route.add_route_item(route::RouteItem::post(
        "/delete",
        post(delete_template_handler),
    ));
    template_route.add_route_item(route::RouteItem::post(
        "/apply",
        post(apply_template_handler),
    ));
    template_route.add_route_item(route::RouteItem::post(
        "/create-from",
        post(create_from_template_handler),
    ));

    meta_route.add_route_group(template_route);
}
