use chrono::{Timelike, Utc};
use uuid::Uuid;

use crate::dto::{LocalApiConfigDto, ResetLocalApiKeyDto, ValidateLocalApiKeyDto};
use crate::entitys::{UpdateLocalApiConfigRequest, ValidateLocalApiKeyRequest};
use crate::{models, svc_ctx::SvcCtx};

const DEFAULT_DAILY_LIMIT: i32 = 1000;

pub async fn get_local_api_config_service(
    context: &SvcCtx,
    user_uuid: Uuid,
) -> Result<LocalApiConfigDto, String> {
    init_local_api_for_user_service(context, user_uuid).await?;
    let settings = models::local_api::fetch_local_api_settings(&context.db, user_uuid)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Local API settings do not exist".to_string())?;
    let api_key = models::local_api::fetch_active_api_key(&context.db, user_uuid)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Local API key does not exist".to_string())?;

    Ok(LocalApiConfigDto {
        enabled: settings.enabled,
        api_key: api_key.api_key.ok_or_else(|| "Local API key is unavailable".to_string())?,
        port: settings.port,
        remote_access: settings.remote_access,
        cors_origins: models::local_api::parse_cors_origins(&settings.cors_origins),
        requests_today: api_key.requests_today,
        daily_limit: api_key.daily_limit,
    })
}

pub async fn update_local_api_config_service(
    context: &SvcCtx,
    user_uuid: Uuid,
    request: &UpdateLocalApiConfigRequest,
) -> Result<LocalApiConfigDto, String> {
    if let Some(port) = request.port {
        if !(1..=65535).contains(&port) {
            return Err("Local API port must be between 1 and 65535".to_string());
        }
    }

    init_local_api_for_user_service(context, user_uuid).await?;
    let cors_origins = request
        .cors_origins
        .as_ref()
        .map(|origins| models::local_api::build_cors_origins_value(origins));
    models::local_api::upsert_local_api_settings(
        &context.db,
        user_uuid,
        request.enabled,
        request.port,
        request.remote_access,
        cors_origins.as_ref(),
    )
    .await
    .map_err(|error| error.to_string())?;

    get_local_api_config_service(context, user_uuid).await
}

pub async fn reset_local_api_key_service(
    context: &SvcCtx,
    user_uuid: Uuid,
) -> Result<ResetLocalApiKeyDto, String> {
    init_local_api_for_user_service(context, user_uuid).await?;
    models::local_api::deactivate_api_keys_for_user(&context.db, user_uuid)
        .await
        .map_err(|error| error.to_string())?;
    let (api_key, key_id) = create_api_key(context, user_uuid).await?;
    ensure_default_permissions(context, key_id).await?;
    Ok(ResetLocalApiKeyDto { api_key })
}

pub async fn validate_local_api_key_service(
    context: &SvcCtx,
    request: &ValidateLocalApiKeyRequest,
) -> Result<ValidateLocalApiKeyDto, String> {
    let key_hash = models::local_api::hash_api_key(&request.api_key);
    let mut api_key = models::local_api::fetch_api_key_by_hash(&context.db, &key_hash)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Invalid Local API key".to_string())?;
    let now = Utc::now();
    if api_key.expires_at.is_some_and(|expires_at| expires_at < now) {
        return Err("Local API key has expired".to_string());
    }
    if api_key.last_reset_date != now.date_naive() {
        models::local_api::reset_api_key_daily_usage(&context.db, api_key.id, now.date_naive())
            .await
            .map_err(|error| error.to_string())?;
        api_key.requests_today = 0;
    }
    if api_key.requests_today >= api_key.daily_limit {
        return Err("Local API daily limit reached".to_string());
    }

    let definition =
        models::local_api::fetch_permission_definition(&context.db, &request.permission_code)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "Unknown Local API permission".to_string())?;
    let permission = models::local_api::fetch_api_key_permission(
        &context.db,
        api_key.id,
        &request.permission_code,
    )
    .await
    .map_err(|error| error.to_string())?
    .ok_or_else(|| "Local API key cannot access this route".to_string())?;
    if !permission.is_enabled {
        return Err("Local API permission is disabled".to_string());
    }

    let minute_start = now.with_second(0).and_then(|value| value.with_nanosecond(0)).unwrap_or(now);
    let hour_start = minute_start.with_minute(0).unwrap_or(minute_start);
    let minute_count = models::local_api::fetch_request_count(
        &context.db,
        api_key.id,
        &request.permission_code,
        "minute",
        minute_start,
    )
    .await
    .map_err(|error| error.to_string())?;
    let hour_count = models::local_api::fetch_request_count(
        &context.db,
        api_key.id,
        &request.permission_code,
        "hour",
        hour_start,
    )
    .await
    .map_err(|error| error.to_string())?;
    if minute_count >= permission.rate_limit_per_minute {
        return Err("Local API minute rate limit reached".to_string());
    }
    if hour_count >= permission.rate_limit_per_hour {
        return Err("Local API hourly rate limit reached".to_string());
    }

    models::local_api::increment_request_counter(
        &context.db,
        api_key.id,
        &request.permission_code,
        "minute",
        minute_start,
    )
    .await
    .map_err(|error| error.to_string())?;
    models::local_api::increment_request_counter(
        &context.db,
        api_key.id,
        &request.permission_code,
        "hour",
        hour_start,
    )
    .await
    .map_err(|error| error.to_string())?;
    models::local_api::increment_api_key_usage(&context.db, api_key.id, now)
        .await
        .map_err(|error| error.to_string())?;

    Ok(ValidateLocalApiKeyDto {
        valid: true,
        user_uuid: api_key.user_uuid,
        permission_code: definition.permission_code,
        requests_today: api_key.requests_today + 1,
        daily_limit: api_key.daily_limit,
        rate_limit_per_minute: permission.rate_limit_per_minute,
        rate_limit_per_hour: permission.rate_limit_per_hour,
    })
}

pub async fn init_local_api_for_user_service(
    context: &SvcCtx,
    user_uuid: Uuid,
) -> Result<(), String> {
    models::local_api::upsert_local_api_settings(&context.db, user_uuid, None, None, None, None)
        .await
        .map_err(|error| error.to_string())?;
    if let Some(key) = models::local_api::fetch_active_api_key(&context.db, user_uuid)
        .await
        .map_err(|error| error.to_string())?
    {
        if key.api_key.is_some() {
            return ensure_default_permissions(context, key.id).await;
        }
        models::local_api::deactivate_api_keys_for_user(&context.db, user_uuid)
            .await
            .map_err(|error| error.to_string())?;
    }
    let (_, key_id) = create_api_key(context, user_uuid).await?;
    ensure_default_permissions(context, key_id).await
}

async fn create_api_key(context: &SvcCtx, user_uuid: Uuid) -> Result<(String, i32), String> {
    let api_key = format!("sk_local_{}", Uuid::new_v4().simple());
    let key_hash = models::local_api::hash_api_key(&api_key);
    let key_prefix = api_key.chars().take(16).collect::<String>();
    let created = models::local_api::insert_api_key(
        &context.db,
        user_uuid,
        &key_prefix,
        &key_hash,
        &api_key,
        DEFAULT_DAILY_LIMIT,
    )
    .await
    .map_err(|error| error.to_string())?;
    Ok((api_key, created.id))
}

async fn ensure_default_permissions(context: &SvcCtx, api_key_id: i32) -> Result<(), String> {
    for definition in models::local_api::fetch_permission_definitions(&context.db)
        .await
        .map_err(|error| error.to_string())?
    {
        models::local_api::insert_api_key_permission(
            &context.db,
            api_key_id,
            &definition.permission_code,
            definition.default_rate_limit_per_minute,
            definition.default_rate_limit_per_hour,
        )
        .await
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}
