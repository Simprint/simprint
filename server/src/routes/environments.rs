use axum::routing::post;

use crate::handlers::{
    add_environment_cookie_handler, add_environment_url_handler, assign_tags_handler,
    batch_assign_tags_handler, batch_create_environments_handler,
    batch_delete_environments_handler, batch_get_environments_handler,
    batch_move_to_group_handler, batch_permanent_delete_environments_handler,
    batch_remove_tags_handler, batch_restore_environments_handler,
    clear_environment_cookies_handler, clear_environment_urls_handler,
    create_environment_handler, create_group_handler, create_tag_handler,
    delete_environment_cookie_handler, delete_environment_handler,
    delete_environment_url_handler, delete_group_handler, delete_tag_handler,
    get_environment_cookies_handler, get_environment_handler, get_environment_urls_handler,
    get_environments_handler, get_groups_handler, get_recycle_bin_environments_handler,
    get_tags_handler, move_to_group_handler, permanent_delete_environment_handler,
    remove_tag_handler, restore_environment_handler, set_environment_accounts_handler,
    set_proxy_handler, update_environment_handler, update_group_handler, update_tag_handler,
};
use crate::routes::route::{self, MetaRoute};

/// 注册环境相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    // 分组路由
    let mut group_route = route::RouteGroup::new("/groups");
    group_route.add_route_item(route::RouteItem::post("/list", post(get_groups_handler)));
    group_route.add_route_item(route::RouteItem::post(
        "/create",
        post(create_group_handler),
    ));
    group_route.add_route_item(route::RouteItem::post(
        "/update",
        post(update_group_handler),
    ));
    group_route.add_route_item(route::RouteItem::post(
        "/delete",
        post(delete_group_handler),
    ));
    meta_route.add_route_group(group_route);

    // 标签路由
    let mut tag_route = route::RouteGroup::new("/tags");
    tag_route.add_route_item(route::RouteItem::post("/list", post(get_tags_handler)));
    tag_route.add_route_item(route::RouteItem::post("/create", post(create_tag_handler)));
    tag_route.add_route_item(route::RouteItem::post("/update", post(update_tag_handler)));
    tag_route.add_route_item(route::RouteItem::post("/delete", post(delete_tag_handler)));
    meta_route.add_route_group(tag_route);

    // 环境路由
    let mut env_route = route::RouteGroup::new("/environments");
    env_route.add_route_item(route::RouteItem::post(
        "/list",
        post(get_environments_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/detail",
        post(get_environment_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/batch-detail",
        post(batch_get_environments_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/create",
        post(create_environment_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/batch-create",
        post(batch_create_environments_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/update",
        post(update_environment_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/delete",
        post(delete_environment_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/batch-delete",
        post(batch_delete_environments_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/set-proxy",
        post(set_proxy_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/assign-tags",
        post(assign_tags_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/remove-tag",
        post(remove_tag_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/move-to-group",
        post(move_to_group_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/batch-move-to-group",
        post(batch_move_to_group_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/set-accounts",
        post(set_environment_accounts_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/batch-assign-tags",
        post(batch_assign_tags_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/batch-remove-tags",
        post(batch_remove_tags_handler),
    ));
    // URL 管理
    env_route.add_route_item(route::RouteItem::post(
        "/urls/list",
        post(get_environment_urls_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/urls/add",
        post(add_environment_url_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/urls/delete",
        post(delete_environment_url_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/urls/clear",
        post(clear_environment_urls_handler),
    ));
    // Cookie 管理
    env_route.add_route_item(route::RouteItem::post(
        "/cookies/list",
        post(get_environment_cookies_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/cookies/add",
        post(add_environment_cookie_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/cookies/delete",
        post(delete_environment_cookie_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/cookies/clear",
        post(clear_environment_cookies_handler),
    ));

    // 回收站路由（作为环境路由的子路由）
    env_route.add_route_item(route::RouteItem::post(
        "/recycle-bin/list",
        post(get_recycle_bin_environments_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/recycle-bin/restore",
        post(restore_environment_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/recycle-bin/batch-restore",
        post(batch_restore_environments_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/recycle-bin/permanent-delete",
        post(permanent_delete_environment_handler),
    ));
    env_route.add_route_item(route::RouteItem::post(
        "/recycle-bin/batch-permanent-delete",
        post(batch_permanent_delete_environments_handler),
    ));

    meta_route.add_route_group(env_route);
}
