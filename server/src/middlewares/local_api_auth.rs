use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use chrono::Utc;

use crate::{
    caches::{
        LocalApiKeyCache, LocalApiPermissionCache, get_local_api_key_cache,
        get_local_api_permission_cache, get_local_api_permission_definition_cache,
        get_local_api_rate_count, increment_local_api_rate_count, set_local_api_key_cache,
        set_local_api_permission_cache, set_local_api_permission_definition_cache,
    },
    middlewares::{fill_authenticated_request_context, get_request_path_and_method},
    models,
    svc_ctx::SvcCtx,
};

const LOCAL_API_AUTH_HEADER: &str = "x-local-api-auth";
const LOCAL_API_KEY_HEADER: &str = "x-local-api-key";
const LOCAL_API_PERMISSION_HEADER: &str = "x-local-api-permission";

pub fn has_local_api_auth_headers(req: &Request) -> bool {
    req.headers().contains_key(LOCAL_API_AUTH_HEADER)
        || (req.headers().contains_key(LOCAL_API_KEY_HEADER)
            && req.headers().contains_key(LOCAL_API_PERMISSION_HEADER))
}

fn is_local_api_management_path(path: &str) -> bool {
    let business_path = crate::middlewares::extract_resource_path(path);
    business_path.starts_with("/local-api")
}

fn parse_local_api_auth(req: &Request) -> Option<(String, String)> {
    if let Some(combined) = req
        .headers()
        .get(LOCAL_API_AUTH_HEADER)
        .and_then(|value| value.to_str().ok())
    {
        let (api_key, permission_code) = combined.split_once(':')?;
        let api_key = api_key.trim();
        let permission_code = permission_code.trim();

        if !api_key.is_empty() && !permission_code.is_empty() {
            return Some((api_key.to_string(), permission_code.to_string()));
        }
    }

    let api_key = req
        .headers()
        .get(LOCAL_API_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    let permission_code = req
        .headers()
        .get(LOCAL_API_PERMISSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())?;

    Some((api_key.to_string(), permission_code.to_string()))
}

pub async fn local_api_auth(
    State(state): State<SvcCtx>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let (resource_method, resource_path) = get_request_path_and_method(&req);
    let resource_method = resource_method.to_string();
    let resource_path = resource_path.to_string();

    if resource_method == "OPTIONS" || is_local_api_management_path(&resource_path) {
        return Ok(next.run(req).await);
    }

    let Some((api_key, permission_code)) = parse_local_api_auth(&req) else {
        return Ok(next.run(req).await);
    };

    let user_uuid = authorize_local_api_request(&state, api_key.as_str(), permission_code.as_str())
        .await?;

    fill_authenticated_request_context(
        &state,
        &mut req,
        user_uuid,
        &resource_method,
        &resource_path,
    )
    .await;

    Ok(next.run(req).await)
}

async fn authorize_local_api_request(
    state: &SvcCtx,
    api_key: &str,
    permission_code: &str,
) -> Result<uuid::Uuid, StatusCode> {
    let key_hash = models::local_api::hash_api_key(api_key);
    let api_key = if let Some(cache) = get_local_api_key_cache(state, &key_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        cache
    } else {
        let db_api_key = models::local_api::fetch_api_key_by_hash(&state.db, &key_hash)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let cache = LocalApiKeyCache {
            id: db_api_key.id,
            user_uuid: db_api_key.user_uuid,
            is_active: db_api_key.is_active,
            expires_at: db_api_key.expires_at,
            daily_limit: db_api_key.daily_limit,
        };
        set_local_api_key_cache(state, &key_hash, &cache)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        cache
    };

    if !api_key.is_active {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if let Some(expires_at) = api_key.expires_at {
        if expires_at < Utc::now() {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    if get_local_api_permission_definition_cache(state, permission_code)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_none()
    {
        let definition = models::local_api::fetch_permission_definition(&state.db, permission_code)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::FORBIDDEN)?;
        set_local_api_permission_definition_cache(state, permission_code, &definition)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let permission = if let Some(cache) =
        get_local_api_permission_cache(state, api_key.id, permission_code)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        cache
    } else {
        let db_permission =
            models::local_api::fetch_api_key_permission(&state.db, api_key.id, permission_code)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or(StatusCode::FORBIDDEN)?;
        let cache = LocalApiPermissionCache {
            is_enabled: db_permission.is_enabled,
            rate_limit_per_minute: db_permission.rate_limit_per_minute,
            rate_limit_per_hour: db_permission.rate_limit_per_hour,
        };
        set_local_api_permission_cache(state, api_key.id, permission_code, &cache)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        cache
    };

    if !permission.is_enabled {
        return Err(StatusCode::FORBIDDEN);
    }

    let now = Utc::now();
    let day_key = now.format("%Y%m%d").to_string();
    let hour_key = now.format("%Y%m%d%H").to_string();
    let minute_key = now.format("%Y%m%d%H%M").to_string();

    let day_count = get_local_api_rate_count(state, api_key.id, None, "day", &day_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if day_count >= i64::from(api_key.daily_limit) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let minute_count =
        get_local_api_rate_count(state, api_key.id, Some(permission_code), "minute", &minute_key)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if minute_count >= i64::from(permission.rate_limit_per_minute) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let hour_count =
        get_local_api_rate_count(state, api_key.id, Some(permission_code), "hour", &hour_key)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if hour_count >= i64::from(permission.rate_limit_per_hour) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    increment_local_api_rate_count(state, api_key.id, Some(permission_code), "minute", &minute_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    increment_local_api_rate_count(state, api_key.id, Some(permission_code), "hour", &hour_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    increment_local_api_rate_count(state, api_key.id, None, "day", &day_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(api_key.user_uuid)
}
