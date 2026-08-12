use axum::routing::post;

use crate::handlers::browser_kernel::list_browser_kernels_handler;
use crate::routes::route::{self, MetaRoute};

/// 注册浏览器内核相关路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut browser_kernel_route = route::RouteGroup::new("/browser-kernels");

    browser_kernel_route.add_route_item(route::RouteItem::post(
        "/list",
        post(list_browser_kernels_handler),
    ));

    meta_route.add_route_group(browser_kernel_route);
}
