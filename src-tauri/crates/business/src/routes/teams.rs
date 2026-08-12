use axum::routing::post;

use crate::handlers::{
    accept_invitation_handler, cancel_invitation_handler, create_team_handler,
    get_my_teams_handler, get_pending_invitations_handler, get_team_handler,
    get_team_members_handler, invite_member_handler, leave_team_handler, reject_invitation_handler,
    remove_member_handler, switch_team_handler, update_member_role_handler, update_team_handler,
};
use crate::routes::route::{self, MetaRoute};

/// 注册团队相关的路由
pub fn register_routes(meta_route: &mut MetaRoute) {
    let mut team_route = route::RouteGroup::new("/teams");

    // 团队管理
    team_route.add_route_item(route::RouteItem::post("/create", post(create_team_handler)));
    team_route.add_route_item(route::RouteItem::post(
        "/my-teams",
        post(get_my_teams_handler),
    ));
    team_route.add_route_item(route::RouteItem::post("/switch", post(switch_team_handler)));
    team_route.add_route_item(route::RouteItem::post("/detail", post(get_team_handler)));
    team_route.add_route_item(route::RouteItem::post("/update", post(update_team_handler)));
    team_route.add_route_item(route::RouteItem::post("/leave", post(leave_team_handler)));

    // 成员管理
    team_route.add_route_item(route::RouteItem::post(
        "/members",
        post(get_team_members_handler),
    ));
    team_route.add_route_item(route::RouteItem::post(
        "/invite",
        post(invite_member_handler),
    ));
    team_route.add_route_item(route::RouteItem::post(
        "/invitations",
        post(get_pending_invitations_handler),
    ));
    team_route.add_route_item(route::RouteItem::post(
        "/invitation/cancel",
        post(cancel_invitation_handler),
    ));
    team_route.add_route_item(route::RouteItem::post(
        "/invitation/accept",
        post(accept_invitation_handler),
    ));
    team_route.add_route_item(route::RouteItem::post(
        "/invitation/reject",
        post(reject_invitation_handler),
    ));
    team_route.add_route_item(route::RouteItem::post(
        "/member/role",
        post(update_member_role_handler),
    ));
    team_route.add_route_item(route::RouteItem::post(
        "/member/remove",
        post(remove_member_handler),
    ));

    meta_route.add_route_group(team_route);
}
