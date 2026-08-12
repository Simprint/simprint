use std::str::FromStr;

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{
    middlewares::{extract_resource_path, get_request_path_and_method},
    services,
    state::{CurrentUser, RequestContext},
    svc_ctx::SvcCtx,
};

pub async fn auth(
    State(state): State<SvcCtx>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let (resource_method, resource_path) = get_request_path_and_method(&req);

    // 转换为拥有所有权的 String，避免借用冲突
    let resource_method = resource_method.to_string();
    let resource_path = resource_path.to_string();

    if resource_method.eq("OPTIONS") {
        return Ok(next.run(req).await);
    }

    if req
        .extensions()
        .get::<RequestContext>()
        .and_then(|ctx| ctx.current_user.as_ref())
        .is_some()
    {
        return Ok(next.run(req).await);
    }

    if crate::middlewares::has_local_api_auth_headers(&req)
        && !extract_resource_path(&resource_path).starts_with("/local-api")
    {
        return Ok(next.run(req).await);
    }

    // 判断请求的是否在白名单，如果是白名单就直接允许访问。
    {
        let combine = format!("{}+{}", resource_method, resource_path);
        let whitelists = &state.config.app.route_whitelists;

        if whitelists.contains(&combine) {
            return Ok(next.run(req).await);
        }
    }

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| StatusCode::UNAUTHORIZED)?;

    let token_str = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // 先判断token是否正常，如果正常再去与远程的匹配。匹配成功后返回uuid.
    let secret = state.config.app.secret.as_bytes();
    let verify_token_response = services::verify_token_service(token_str, secret);

    if let Err(_e) = verify_token_response {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let verify_token_response = verify_token_response.unwrap();

    // TODO: 根据获取到的uuid可以获取到用户并调用检查权限的方法。
    // 2025-08-27 10:05:00: 权限检查可能移除, 该网关服务只面向用户
    let uuid = if let Ok(uuid) = Uuid::from_str(&verify_token_response) {
        uuid
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    fill_authenticated_request_context(&state, &mut req, uuid, &resource_method, &resource_path)
        .await;

    Ok(next.run(req).await)
}

pub async fn fill_authenticated_request_context(
    state: &SvcCtx,
    req: &mut Request,
    user_uuid: Uuid,
    resource_method: &str,
    resource_path: &str,
) {
    if req.extensions().get::<RequestContext>().is_none() {
        req.extensions_mut().insert(RequestContext::default());
    }

    if let Some(ctx) = req.extensions_mut().get_mut::<RequestContext>() {
        let ip = ctx.ip_or_unknown().to_string();
        tracing::info!("auth user uuid: {}, ip: {}", user_uuid, ip);

        ctx.current_user = Some(CurrentUser { user_uuid });
        ctx.current_team_uuid = None;
        ctx.current_workspace_uuid = None;

        let business_path = extract_resource_path(resource_path);
        ctx.resource_identifier = Some(format!("{}+{}", resource_method, business_path));
    }

    let user_info = crate::models::user::fetch_user_info_by_uuid(&state.db, user_uuid)
        .await
        .ok()
        .flatten();

    if let Some(ctx) = req.extensions_mut().get_mut::<RequestContext>() {
        if let Some(user_info) = user_info {
            ctx.current_team_uuid = user_info.current_team_uuid;
            ctx.current_workspace_uuid = if let Some(ws_uuid) = user_info.current_workspace_uuid {
                Some(ws_uuid)
            } else if let Some(team_uuid) = user_info.current_team_uuid {
                crate::models::teams::fetch_team_by_uuid(&state.db, team_uuid)
                    .await
                    .ok()
                    .flatten()
                    .map(|team| team.workspace_uuid)
            } else {
                None
            };
        }
    }
}
