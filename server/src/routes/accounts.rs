use axum::routing::post;

use crate::handlers::{
    batch_delete_accounts_handler, batch_import_accounts_handler, create_account_handler,
    delete_account_handler, get_account_handler, get_accounts_handler, update_account_handler,
};
use crate::routes::route::{self, MetaRoute};

/// 注册账号相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut account_route = route::RouteGroup::new("/accounts");

    account_route.add_route_item(route::RouteItem::post("/list", post(get_accounts_handler)));
    account_route.add_route_item(route::RouteItem::post("/detail", post(get_account_handler)));
    account_route.add_route_item(route::RouteItem::post(
        "/create",
        post(create_account_handler),
    ));
    account_route.add_route_item(route::RouteItem::post(
        "/update",
        post(update_account_handler),
    ));
    account_route.add_route_item(route::RouteItem::post(
        "/delete",
        post(delete_account_handler),
    ));
    account_route.add_route_item(route::RouteItem::post(
        "/batch-delete",
        post(batch_delete_accounts_handler),
    ));
    account_route.add_route_item(route::RouteItem::post(
        "/batch-import",
        post(batch_import_accounts_handler),
    ));

    meta_route.add_route_group(account_route);
}
