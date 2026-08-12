use axum::routing::get;

use crate::handlers::now_handle;
use crate::routes::route::{self, MetaRoute};

pub fn register_routes(meta_route: &mut MetaRoute) -> () {
    let mut time_route = route::RouteGroup::new("/time");

    time_route.add_route_item(route::RouteItem::get("/now", get(now_handle)));

    meta_route.add_route_group(time_route);
}
