use axum::routing::post;

use crate::handlers::{
    get_current_user_handler, login_handler, refresh_token_handler, register_handler,
    reset_password_handler, send_code_handler, update_password_handler, update_user_handler,
    verify_password_handler,
};
use crate::routes::route::{self, MetaRoute};

/// 注册用户相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) -> () {
    let mut user_route = route::RouteGroup::new("/users");

    user_route.add_route_item(route::RouteItem::post("/register", post(register_handler)));

    user_route.add_route_item(route::RouteItem::post("/login", post(login_handler)));

    user_route.add_route_item(route::RouteItem::post(
        "/refresh-credentials",
        post(refresh_token_handler),
    ));

    user_route.add_route_item(route::RouteItem::post(
        "/me",
        post(get_current_user_handler),
    ));

    user_route.add_route_item(route::RouteItem::post("/update", post(update_user_handler)));

    user_route.add_route_item(route::RouteItem::post(
        "/password",
        post(update_password_handler),
    ));

    user_route.add_route_item(route::RouteItem::post(
        "/verify-password",
        post(verify_password_handler),
    ));

    user_route.add_route_item(route::RouteItem::post(
        "/reset-password",
        post(reset_password_handler),
    ));

    user_route.add_route_item(route::RouteItem::post(
        "/register-send-code",
        post(send_code_handler),
    ));

    user_route.add_route_item(route::RouteItem::post(
        "/reset-password-send-code",
        post(send_code_handler),
    ));

    meta_route.add_route_group(user_route);
}
