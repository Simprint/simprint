use chrono::Utc;
use uuid::Uuid;

use crate::caches::{
    LocalApiKeyCache, LocalApiPermissionCache, delete_local_api_key_cache,
    delete_local_api_permission_caches_for_key,
    get_local_api_key_cache, get_local_api_permission_cache, get_local_api_permission_definition_cache,
    get_local_api_rate_count, increment_local_api_rate_count, set_local_api_key_cache,
    set_local_api_permission_cache, set_local_api_permission_definition_cache,
};
use crate::dto::{LocalApiConfigDto, ResetLocalApiKeyDto, ValidateLocalApiKeyDto};
use crate::entitys::{UpdateLocalApiConfigRequest, ValidateLocalApiKeyRequest};
use crate::models;
use crate::svc_ctx::SvcCtx;

const DEFAULT_DAILY_LIMIT: i32 = 1000;

pub async fn get_local_api_config_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<LocalApiConfigDto, String> {
    init_local_api_for_user_service(svc_ctx, user_uuid).await?;

    let settings = models::local_api::fetch_local_api_settings(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "API 服务配置不存在".to_string())?;

    let mut api_key = models::local_api::fetch_active_api_key(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "API 密钥不存在".to_string())?;

    if api_key.api_key.is_none() {
        rotate_local_api_key_for_user(svc_ctx, user_uuid).await?;
        api_key = models::local_api::fetch_active_api_key(&svc_ctx.db, user_uuid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "API 密钥不存在".to_string())?;
    }

    Ok(LocalApiConfigDto {
        enabled: settings.enabled,
        api_key: api_key
            .api_key
            .clone()
            .ok_or_else(|| "API 密钥不存在".to_string())?,
        port: settings.port,
        remote_access: settings.remote_access,
        cors_origins: models::local_api::parse_cors_origins(&settings.cors_origins),
        requests_today: get_local_api_rate_count(
            svc_ctx,
            api_key.id,
            None,
            "day",
            &Utc::now().format("%Y%m%d").to_string(),
        )
        .await
        .ok()
        .map(|count| count as i32)
        .unwrap_or(api_key.requests_today),
        daily_limit: api_key.daily_limit,
    })
}

pub async fn update_local_api_config_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &UpdateLocalApiConfigRequest,
) -> Result<LocalApiConfigDto, String> {
    if let Some(port) = payload.port {
        if !(1..=65535).contains(&port) {
            return Err("端口范围无效".to_string());
        }
    }

    init_local_api_for_user_service(svc_ctx, user_uuid).await?;

    let cors_origins = payload
        .cors_origins
        .as_ref()
        .map(|origins| models::local_api::build_cors_origins_value(origins));

    models::local_api::upsert_local_api_settings(
        &svc_ctx.db,
        user_uuid,
        payload.enabled,
        payload.port,
        payload.remote_access,
        cors_origins.as_ref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    get_local_api_config_service(svc_ctx, user_uuid).await
}

pub async fn reset_local_api_key_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<ResetLocalApiKeyDto, String> {
    init_local_api_for_user_service(svc_ctx, user_uuid).await?;

    if let Some(current_key) = models::local_api::fetch_active_api_key(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?
    {
        delete_local_api_key_cache(svc_ctx, &current_key.key_hash)
            .await
            .map_err(|e| e.to_string())?;
        delete_local_api_permission_caches_for_key(svc_ctx, current_key.id)
            .await
            .map_err(|e| e.to_string())?;
    }

    models::local_api::deactivate_api_keys_for_user(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    let api_key = generate_api_key();
    let key_hash = models::local_api::hash_api_key(&api_key);
    let key_prefix = api_key.chars().take(16).collect::<String>();
    let created_key = models::local_api::insert_api_key(
        &svc_ctx.db,
        user_uuid,
        &key_prefix,
        &key_hash,
        &api_key,
        DEFAULT_DAILY_LIMIT,
    )
    .await
    .map_err(|e| e.to_string())?;
    set_local_api_key_cache(
        svc_ctx,
        &key_hash,
        &LocalApiKeyCache {
            id: created_key.id,
            user_uuid: created_key.user_uuid,
            is_active: created_key.is_active,
            expires_at: created_key.expires_at,
            daily_limit: created_key.daily_limit,
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    ensure_default_permissions(svc_ctx, created_key.id).await?;

    Ok(ResetLocalApiKeyDto {
        api_key,
    })
}

pub async fn validate_local_api_key_service(
    svc_ctx: &SvcCtx,
    payload: &ValidateLocalApiKeyRequest,
) -> Result<ValidateLocalApiKeyDto, String> {
    let key_hash = models::local_api::hash_api_key(payload.api_key.as_str());
    let api_key = if let Some(cache) = get_local_api_key_cache(svc_ctx, &key_hash)
        .await
        .map_err(|e| e.to_string())?
    {
        cache
    } else {
        let db_api_key = models::local_api::fetch_api_key_by_hash(&svc_ctx.db, &key_hash)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "API 密钥无效".to_string())?;
        let cache = LocalApiKeyCache {
            id: db_api_key.id,
            user_uuid: db_api_key.user_uuid,
            is_active: db_api_key.is_active,
            expires_at: db_api_key.expires_at,
            daily_limit: db_api_key.daily_limit,
        };
        set_local_api_key_cache(svc_ctx, &key_hash, &cache)
            .await
            .map_err(|e| e.to_string())?;
        cache
    };

    if !api_key.is_active {
        return Err("API 密钥已停用".to_string());
    }

    if let Some(expires_at) = api_key.expires_at {
        if expires_at < Utc::now() {
            return Err("API 密钥已过期".to_string());
        }
    }

    let day_key = Utc::now().format("%Y%m%d").to_string();
    let day_count = get_local_api_rate_count(svc_ctx, api_key.id, None, "day", &day_key)
        .await
        .map_err(|e| e.to_string())?;
    if day_count >= i64::from(api_key.daily_limit) {
        return Err("API 密钥已达到今日调用上限".to_string());
    }

    let definition = if let Some(cache) =
        get_local_api_permission_definition_cache(svc_ctx, &payload.permission_code)
            .await
            .map_err(|e| e.to_string())?
    {
        cache
    } else {
        let definition = models::local_api::fetch_permission_definition(
            &svc_ctx.db,
            &payload.permission_code,
        )
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "未知的 permissionCode".to_string())?;
        set_local_api_permission_definition_cache(svc_ctx, &payload.permission_code, &definition)
            .await
            .map_err(|e| e.to_string())?;
        definition
    };

    let permission = if let Some(cache) =
        get_local_api_permission_cache(svc_ctx, api_key.id, &payload.permission_code)
            .await
            .map_err(|e| e.to_string())?
    {
        cache
    } else {
        let db_permission = fetch_effective_permission(svc_ctx, api_key.id, &payload.permission_code)
            .await?
            .ok_or_else(|| "当前密钥无权访问该接口".to_string())?;
        let cache = LocalApiPermissionCache {
            is_enabled: db_permission.is_enabled,
            rate_limit_per_minute: db_permission.rate_limit_per_minute,
            rate_limit_per_hour: db_permission.rate_limit_per_hour,
        };
        set_local_api_permission_cache(svc_ctx, api_key.id, &payload.permission_code, &cache)
            .await
            .map_err(|e| e.to_string())?;
        cache
    };

    if !permission.is_enabled {
        return Err("当前密钥已被禁止访问该接口".to_string());
    }

    let now = Utc::now();
    let minute_key = now.format("%Y%m%d%H%M").to_string();
    let hour_key = now.format("%Y%m%d%H").to_string();

    let minute_count = get_local_api_rate_count(
        svc_ctx,
        api_key.id,
        Some(payload.permission_code.as_str()),
        "minute",
        &minute_key,
    )
    .await
    .map_err(|e| e.to_string())?;
    if minute_count >= i64::from(permission.rate_limit_per_minute) {
        return Err("接口每分钟调用次数已达上限".to_string());
    }

    let hour_count = get_local_api_rate_count(
        svc_ctx,
        api_key.id,
        Some(payload.permission_code.as_str()),
        "hour",
        &hour_key,
    )
    .await
    .map_err(|e| e.to_string())?;
    if hour_count >= i64::from(permission.rate_limit_per_hour) {
        return Err("接口每小时调用次数已达上限".to_string());
    }

    increment_local_api_rate_count(
        svc_ctx,
        api_key.id,
        Some(payload.permission_code.as_str()),
        "minute",
        &minute_key,
    )
    .await
    .map_err(|e| e.to_string())?;
    increment_local_api_rate_count(
        svc_ctx,
        api_key.id,
        Some(payload.permission_code.as_str()),
        "hour",
        &hour_key,
    )
    .await
    .map_err(|e| e.to_string())?;
    increment_local_api_rate_count(svc_ctx, api_key.id, None, "day", &day_key)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ValidateLocalApiKeyDto {
        valid: true,
        permission_code: definition.permission_code,
        requests_today: (day_count + 1) as i32,
        daily_limit: api_key.daily_limit,
        rate_limit_per_minute: permission.rate_limit_per_minute,
        rate_limit_per_hour: permission.rate_limit_per_hour,
    })
}

pub async fn init_local_api_for_user_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<(), String> {
    models::local_api::upsert_local_api_settings(
        &svc_ctx.db,
        user_uuid,
        None,
        None,
        None,
        None,
    )
    .await
    .map_err(|e| e.to_string())?;

    let current_key = models::local_api::fetch_active_api_key(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(api_key) = current_key {
        if api_key.api_key.is_none() {
            rotate_local_api_key_for_user(svc_ctx, user_uuid).await?;
            return Ok(());
        }
        ensure_default_permissions(svc_ctx, api_key.id).await?;
        return Ok(());
    }

    let api_key = generate_api_key();
    let key_hash = models::local_api::hash_api_key(&api_key);
    let key_prefix = api_key.chars().take(16).collect::<String>();

    let created_key = models::local_api::insert_api_key(
        &svc_ctx.db,
        user_uuid,
        &key_prefix,
        &key_hash,
        &api_key,
        DEFAULT_DAILY_LIMIT,
    )
    .await
    .map_err(|e| e.to_string())?;
    set_local_api_key_cache(
        svc_ctx,
        &key_hash,
        &LocalApiKeyCache {
            id: created_key.id,
            user_uuid: created_key.user_uuid,
            is_active: created_key.is_active,
            expires_at: created_key.expires_at,
            daily_limit: created_key.daily_limit,
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    ensure_default_permissions(svc_ctx, created_key.id).await
}

async fn ensure_default_permissions(svc_ctx: &SvcCtx, api_key_id: i32) -> Result<(), String> {
    let definitions = models::local_api::fetch_permission_definitions(&svc_ctx.db)
        .await
        .map_err(|e| e.to_string())?;

    for definition in definitions {
        models::local_api::insert_api_key_permission(
            &svc_ctx.db,
            api_key_id,
            definition.permission_code.as_str(),
            definition.default_rate_limit_per_minute,
            definition.default_rate_limit_per_hour,
        )
        .await
        .map_err(|e| e.to_string())?;
        set_local_api_permission_definition_cache(
            svc_ctx,
            definition.permission_code.as_str(),
            &definition,
        )
        .await
        .map_err(|e| e.to_string())?;
        set_local_api_permission_cache(
            svc_ctx,
            api_key_id,
            definition.permission_code.as_str(),
            &LocalApiPermissionCache {
                is_enabled: true,
                rate_limit_per_minute: definition.default_rate_limit_per_minute,
                rate_limit_per_hour: definition.default_rate_limit_per_hour,
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

async fn fetch_effective_permission(
    svc_ctx: &SvcCtx,
    api_key_id: i32,
    permission_code: &str,
) -> Result<Option<crate::dto::LocalApiKeyPermissionDto>, String> {
    models::local_api::fetch_api_key_permission(&svc_ctx.db, api_key_id, permission_code)
        .await
        .map_err(|e| e.to_string())
}

fn generate_api_key() -> String {
    let raw = Uuid::new_v4().simple().to_string();
    format!("sk_local_{}", raw)
}

async fn rotate_local_api_key_for_user(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<ResetLocalApiKeyDto, String> {
    models::local_api::deactivate_api_keys_for_user(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    let api_key = generate_api_key();
    let key_hash = models::local_api::hash_api_key(&api_key);
    let key_prefix = api_key.chars().take(16).collect::<String>();

    let created_key = models::local_api::insert_api_key(
        &svc_ctx.db,
        user_uuid,
        &key_prefix,
        &key_hash,
        &api_key,
        DEFAULT_DAILY_LIMIT,
    )
    .await
    .map_err(|e| e.to_string())?;
    set_local_api_key_cache(
        svc_ctx,
        &key_hash,
        &LocalApiKeyCache {
            id: created_key.id,
            user_uuid: created_key.user_uuid,
            is_active: created_key.is_active,
            expires_at: created_key.expires_at,
            daily_limit: created_key.daily_limit,
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    ensure_default_permissions(svc_ctx, created_key.id).await?;

    Ok(ResetLocalApiKeyDto { api_key })
}
