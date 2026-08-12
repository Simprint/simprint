use axum::http::StatusCode;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use tauri::Manager;

use crate::app::handle::get_app_handle;

pub async fn dispatch_request(
    route: &str,
    permission_code: &str,
    api_key: &str,
    payload: Value,
) -> Result<Value, (StatusCode, String)> {
    let app =
        get_app_handle().map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    let business_context = app.state::<business::svc_ctx::SvcCtx>();
    let validation = business::services::local_api::validate_local_api_key_service(
        &business_context,
        &business::entitys::ValidateLocalApiKeyRequest {
            api_key: api_key.to_string(),
            permission_code: permission_code.to_string(),
        },
    )
    .await
    .map_err(|message| (StatusCode::UNAUTHORIZED, message))?;
    let request_context = business_context.for_user(validation.user_uuid);

    if let Some(result) =
        business::dispatcher::dispatch_post(&request_context, route, &payload).await
    {
        return match result {
            Ok(data) => Ok(json!({ "code": 1, "message": "OK", "data": data })),
            Err(message) => Err((StatusCode::BAD_REQUEST, message)),
        };
    }

    Err((
        StatusCode::NOT_FOUND,
        format!("Local business route is not available: {route}"),
    ))
}

pub async fn dispatch_data_request<T>(
    route: &str,
    permission_code: &str,
    api_key: &str,
    payload: Value,
) -> Result<T, (StatusCode, String)>
where
    T: DeserializeOwned,
{
    let value = dispatch_request(route, permission_code, api_key, payload).await?;
    let response: crate::infrastructure::http::client::JsonRespnse = serde_json::from_value(value)
        .map_err(|error| {
            (
                StatusCode::BAD_GATEWAY,
                format!("failed to parse local response: {error}"),
            )
        })?;

    let data = response.data.ok_or_else(|| {
        (
            StatusCode::BAD_GATEWAY,
            "missing local response data".to_string(),
        )
    })?;

    serde_json::from_value(data).map_err(|error| {
        (
            StatusCode::BAD_GATEWAY,
            format!("failed to parse local response data: {error}"),
        )
    })
}

pub async fn dispatch_data_value_request<T>(
    route: &str,
    permission_code: &str,
    api_key: &str,
    payload: Value,
) -> Result<Value, (StatusCode, String)>
where
    T: DeserializeOwned + Serialize,
{
    let data: T = dispatch_data_request(route, permission_code, api_key, payload).await?;
    serde_json::to_value(data).map_err(|error| {
        (
            StatusCode::BAD_GATEWAY,
            format!("failed to serialize local response data: {error}"),
        )
    })
}
