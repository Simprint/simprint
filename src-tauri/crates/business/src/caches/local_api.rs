use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::dto::LocalApiPermissionDefinitionDto;
use crate::svc_ctx::SvcCtx;

const LOCAL_API_KEY_CACHE_PREFIX: &str = "local_api:key";
const LOCAL_API_PERMISSION_CACHE_PREFIX: &str = "local_api:permission";
const LOCAL_API_PERMISSION_DEFINITION_CACHE_PREFIX: &str = "local_api:permission_definition";
const LOCAL_API_RATE_CACHE_PREFIX: &str = "local_api:rate";
const LOCAL_API_KEY_CACHE_TTL: u64 = 60 * 10;
const LOCAL_API_PERMISSION_CACHE_TTL: u64 = 60 * 10;
const MINUTE_WINDOW_SECONDS: i64 = 60;
const HOUR_WINDOW_SECONDS: i64 = 60 * 60;
const DAY_WINDOW_SECONDS: i64 = 60 * 60 * 24;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalApiKeyCache {
    pub id: i32,
    pub user_uuid: Uuid,
    pub is_active: bool,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub daily_limit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalApiPermissionCache {
    pub is_enabled: bool,
    pub rate_limit_per_minute: i32,
    pub rate_limit_per_hour: i32,
}

fn local_api_key_cache_key(key_hash: &str) -> String {
    format!("{}:{}", LOCAL_API_KEY_CACHE_PREFIX, key_hash)
}

fn local_api_permission_cache_key(api_key_id: i32, permission_code: &str) -> String {
    format!(
        "{}:{}:{}",
        LOCAL_API_PERMISSION_CACHE_PREFIX, api_key_id, permission_code
    )
}

fn local_api_permission_definition_cache_key(permission_code: &str) -> String {
    format!(
        "{}:{}",
        LOCAL_API_PERMISSION_DEFINITION_CACHE_PREFIX, permission_code
    )
}

fn local_api_rate_cache_key(
    api_key_id: i32,
    permission_code: Option<&str>,
    window_type: &str,
    window_key: &str,
) -> String {
    if let Some(permission_code) = permission_code {
        format!(
            "{}:{}:{}:{}:{}",
            LOCAL_API_RATE_CACHE_PREFIX, api_key_id, permission_code, window_type, window_key
        )
    } else {
        format!(
            "{}:{}:{}:{}",
            LOCAL_API_RATE_CACHE_PREFIX, api_key_id, window_type, window_key
        )
    }
}

pub async fn get_local_api_key_cache(
    svc_ctx: &SvcCtx,
    key_hash: &str,
) -> Result<Option<LocalApiKeyCache>, anyhow::Error> {
    let key = local_api_key_cache_key(key_hash);
    svc_ctx.cache.get_json(&key).await
}

pub async fn set_local_api_key_cache(
    svc_ctx: &SvcCtx,
    key_hash: &str,
    cache: &LocalApiKeyCache,
) -> Result<(), anyhow::Error> {
    let key = local_api_key_cache_key(key_hash);
    svc_ctx
        .cache
        .set_json(key, cache, Duration::from_secs(LOCAL_API_KEY_CACHE_TTL))
        .await
}

pub async fn delete_local_api_key_cache(
    svc_ctx: &SvcCtx,
    key_hash: &str,
) -> Result<(), anyhow::Error> {
    let key = local_api_key_cache_key(key_hash);
    svc_ctx.cache.delete(&key).await
}

pub async fn get_local_api_permission_cache(
    svc_ctx: &SvcCtx,
    api_key_id: i32,
    permission_code: &str,
) -> Result<Option<LocalApiPermissionCache>, anyhow::Error> {
    let key = local_api_permission_cache_key(api_key_id, permission_code);
    svc_ctx.cache.get_json(&key).await
}

pub async fn set_local_api_permission_cache(
    svc_ctx: &SvcCtx,
    api_key_id: i32,
    permission_code: &str,
    cache: &LocalApiPermissionCache,
) -> Result<(), anyhow::Error> {
    let key = local_api_permission_cache_key(api_key_id, permission_code);
    svc_ctx
        .cache
        .set_json(
            key,
            cache,
            Duration::from_secs(LOCAL_API_PERMISSION_CACHE_TTL),
        )
        .await
}

pub async fn delete_local_api_permission_cache(
    svc_ctx: &SvcCtx,
    api_key_id: i32,
    permission_code: &str,
) -> Result<(), anyhow::Error> {
    let key = local_api_permission_cache_key(api_key_id, permission_code);
    svc_ctx.cache.delete(&key).await
}

pub async fn delete_local_api_permission_caches_for_key(
    svc_ctx: &SvcCtx,
    api_key_id: i32,
) -> Result<(), anyhow::Error> {
    let prefix = format!("{}:{}:", LOCAL_API_PERMISSION_CACHE_PREFIX, api_key_id);
    svc_ctx.cache.delete_prefix(&prefix).await
}

pub async fn get_local_api_permission_definition_cache(
    svc_ctx: &SvcCtx,
    permission_code: &str,
) -> Result<Option<LocalApiPermissionDefinitionDto>, anyhow::Error> {
    let key = local_api_permission_definition_cache_key(permission_code);
    svc_ctx.cache.get_json(&key).await
}

pub async fn set_local_api_permission_definition_cache(
    svc_ctx: &SvcCtx,
    permission_code: &str,
    cache: &LocalApiPermissionDefinitionDto,
) -> Result<(), anyhow::Error> {
    let key = local_api_permission_definition_cache_key(permission_code);
    svc_ctx
        .cache
        .set_json(
            key,
            cache,
            Duration::from_secs(LOCAL_API_PERMISSION_CACHE_TTL),
        )
        .await
}

pub async fn delete_local_api_permission_definition_cache(
    svc_ctx: &SvcCtx,
    permission_code: &str,
) -> Result<(), anyhow::Error> {
    let key = local_api_permission_definition_cache_key(permission_code);
    svc_ctx.cache.delete(&key).await
}

pub async fn get_local_api_rate_count(
    svc_ctx: &SvcCtx,
    api_key_id: i32,
    permission_code: Option<&str>,
    window_type: &str,
    window_key: &str,
) -> Result<i64, anyhow::Error> {
    let key = local_api_rate_cache_key(api_key_id, permission_code, window_type, window_key);
    Ok(svc_ctx.cache.get_i64(&key).await?.unwrap_or(0))
}

pub async fn increment_local_api_rate_count(
    svc_ctx: &SvcCtx,
    api_key_id: i32,
    permission_code: Option<&str>,
    window_type: &str,
    window_key: &str,
) -> Result<i64, anyhow::Error> {
    let key = local_api_rate_cache_key(api_key_id, permission_code, window_type, window_key);
    let ttl = match window_type {
        "minute" => MINUTE_WINDOW_SECONDS,
        "hour" => HOUR_WINDOW_SECONDS,
        "day" => DAY_WINDOW_SECONDS,
        _ => DAY_WINDOW_SECONDS,
    };
    svc_ctx
        .cache
        .increment(&key, Duration::from_secs(ttl as u64))
        .await
}
