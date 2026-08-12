use axum::routing::post;

use crate::handlers::local_api;
use crate::routes::route::{self, MetaRoute};

pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut local_api_route = route::RouteGroup::new("/local-api");

    local_api_route.add_route_item(route::RouteItem::post(
        "/get",
        post(local_api::get_local_api_config_handler),
    ));
    local_api_route.add_route_item(route::RouteItem::post(
        "/update",
        post(local_api::update_local_api_config_handler),
    ));
    local_api_route.add_route_item(route::RouteItem::post(
        "/reset-api-key",
        post(local_api::reset_local_api_key_handler),
    ));
    local_api_route.add_route_item(route::RouteItem::post(
        "/validate",
        post(local_api::validate_local_api_key_handler),
    ));

    meta_route.add_route_group(local_api_route);
}
