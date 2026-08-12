use axum::routing::post;

use crate::handlers::{
    batch_update_extensions_handler, disable_extension_handler, enable_extension_handler,
    get_extension_categories_handler, get_extension_detail_handler, get_extensions_handler,
    get_installed_extensions_handler, install_extension_handler, uninstall_extension_handler,
    update_extension_handler,
};
use crate::routes::route::{self, MetaRoute};

/// 注册扩展管理相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut ext_route = route::RouteGroup::new("/extensions");

    // 扩展市场
    ext_route.add_route_item(route::RouteItem::post(
        "/list",
        post(get_extensions_handler),
    ));
    ext_route.add_route_item(route::RouteItem::post(
        "/detail",
        post(get_extension_detail_handler),
    ));
    ext_route.add_route_item(route::RouteItem::post(
        "/categories",
        post(get_extension_categories_handler),
    ));

    // 已安装扩展
    ext_route.add_route_item(route::RouteItem::post(
        "/installed",
        post(get_installed_extensions_handler),
    ));

    // 安装/卸载/更新
    ext_route.add_route_item(route::RouteItem::post(
        "/install",
        post(install_extension_handler),
    ));
    ext_route.add_route_item(route::RouteItem::post(
        "/uninstall",
        post(uninstall_extension_handler),
    ));
    ext_route.add_route_item(route::RouteItem::post(
        "/update",
        post(update_extension_handler),
    ));
    ext_route.add_route_item(route::RouteItem::post(
        "/batch-update",
        post(batch_update_extensions_handler),
    ));

    // 禁用/启用扩展
    ext_route.add_route_item(route::RouteItem::post(
        "/disable",
        post(disable_extension_handler),
    ));
    ext_route.add_route_item(route::RouteItem::post(
        "/enable",
        post(enable_extension_handler),
    ));

    meta_route.add_route_group(ext_route);
}
