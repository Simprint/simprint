use axum::{extract::Extension, extract::State};

use crate::audit_log;
use crate::entitys::{
    BatchSetProxyVisibleRequest, ListProxyVisibleTeamsRequest, ListVisibleProxiesRequest,
    RemoveProxyVisibleRequest, SetProxyVisibleRequest,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 设置代理对团队可见
pub async fn set_proxy_visible_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<SetProxyVisibleRequest>,
) -> Result<()> {
    services::proxy_visibility::set_proxy_visible_to_team_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "update",
        "proxy_visibility",
        payload.proxy_uuid,
        "",
        "设置代理可见性"
    )
    .await;

    Ok(Response::success(Some("设置成功"), None))
}

/// 移除代理对团队的可见性
pub async fn remove_proxy_visible_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<RemoveProxyVisibleRequest>,
) -> Result<()> {
    services::proxy_visibility::remove_proxy_visible_from_team_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "update",
        "proxy_visibility",
        payload.proxy_uuid,
        "",
        "移除代理可见性"
    )
    .await;

    Ok(Response::success(Some("移除成功"), None))
}

/// 批量设置代理可见性
pub async fn batch_set_proxy_visible_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<BatchSetProxyVisibleRequest>,
) -> Result<()> {
    services::proxy_visibility::batch_set_proxy_visible_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "update",
        "proxy_visibility",
        payload.proxy_uuid,
        "",
        "批量设置代理可见性"
    )
    .await;

    Ok(Response::success(Some("设置成功"), None))
}

/// 获取可见的代理列表
pub async fn get_visible_proxies_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListVisibleProxiesRequest>,
) -> Result<crate::entitys::VisibleProxyListResponse> {
    let proxies = services::proxy_visibility::get_visible_proxies_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        None,
        Some(crate::entitys::VisibleProxyListResponse { items: proxies }),
    ))
}

/// 获取代理的可见团队列表
pub async fn get_proxy_visible_teams_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(_ctx): Extension<RequestContext>,
    Json(payload): Json<ListProxyVisibleTeamsRequest>,
) -> Result<crate::entitys::ProxyVisibleTeamListResponse> {
    let teams = crate::models::fetch_visible_teams_by_proxy(&svc_ctx.db, payload.proxy_uuid)
        .await
        .map_err(|e| Response::fail(Some(&e.to_string())))?;

    // 转换为详情 DTO（包含团队名称）
    let mut team_details = vec![];
    for team in teams {
        let team_info = crate::models::fetch_team_by_uuid(&svc_ctx.db, team.team_uuid)
            .await
            .ok()
            .flatten();

        team_details.push(crate::dto::ProxyVisibleTeamDetailDto {
            proxy_uuid: team.proxy_uuid,
            workspace_uuid: team.workspace_uuid,
            team_uuid: team.team_uuid,
            team_name: team_info.map(|t| t.name),
            created_at: team.created_at,
        });
    }

    Ok(Response::success(
        None,
        Some(crate::entitys::ProxyVisibleTeamListResponse {
            items: team_details,
        }),
    ))
}
