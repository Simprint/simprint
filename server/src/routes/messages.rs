use axum::routing::post;

use crate::handlers::{
    batch_mark_messages_read_handler, create_message_handler, delete_message_handler,
    get_user_message_stats_handler, get_user_messages_handler, handle_message_handler,
    mark_message_read_handler,
};
use crate::routes::route::{self, MetaRoute};

/// 注册消息相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut message_route = route::RouteGroup::new("/messages");

    // 消息管理
    message_route.add_route_item(route::RouteItem::post(
        "/create",
        post(create_message_handler),
    ));
    message_route.add_route_item(route::RouteItem::post(
        "/list",
        post(get_user_messages_handler),
    ));
    message_route.add_route_item(route::RouteItem::post(
        "/stats",
        post(get_user_message_stats_handler),
    ));
    message_route.add_route_item(route::RouteItem::post(
        "/read",
        post(mark_message_read_handler),
    ));
    message_route.add_route_item(route::RouteItem::post(
        "/batch-read",
        post(batch_mark_messages_read_handler),
    ));
    message_route.add_route_item(route::RouteItem::post(
        "/handle",
        post(handle_message_handler),
    ));
    message_route.add_route_item(route::RouteItem::post(
        "/delete",
        post(delete_message_handler),
    ));

    meta_route.add_route_group(message_route);
}
