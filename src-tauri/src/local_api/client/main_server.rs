use axum::http::StatusCode;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use tauri::Manager;

use crate::{
    app::{context::AppContext, handle::get_app_handle},
    infrastructure::main_server::error::ResponseError,
};

use super::headers::build_local_api_auth_headers;

pub async fn proxy_request(
    server_path: &str,
    permission_code: &str,
    api_key: &str,
    payload: Value,
) -> Result<Value, (StatusCode, String)> {
    let app =
        get_app_handle().map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    let business_context = app.state::<business::svc_ctx::SvcCtx>();
    business::services::local_api::validate_local_api_key_service(
        &business_context,
        &business::entitys::ValidateLocalApiKeyRequest {
            api_key: api_key.to_string(),
            permission_code: permission_code.to_string(),
        },
    )
    .await
    .map_err(|message| (StatusCode::UNAUTHORIZED, message))?;

    if let Some(result) =
        business::dispatcher::dispatch_post(&business_context, server_path, &payload).await
    {
        return match result {
            Ok(data) => Ok(json!({ "code": 1, "message": "OK", "data": data })),
            Err(message) => Err((StatusCode::BAD_REQUEST, message)),
        };
    }

    let ctx = AppContext::get();
    let headers = build_local_api_auth_headers(api_key, permission_code)
        .map_err(|error| (StatusCode::BAD_REQUEST, error))?;

    let response = ctx
        .main_server_client
        .post_with_headers(server_path, &payload, headers)
        .await
        .map_err(map_main_server_error)?;

    serde_json::to_value(response).map_err(|_| {
        (
            StatusCode::BAD_GATEWAY,
            json!({
                "code": -1,
                "message": "failed to serialize response"
            })
            .to_string(),
        )
    })
}

pub async fn proxy_data_request<T>(
    server_path: &str,
    permission_code: &str,
    api_key: &str,
    payload: Value,
) -> Result<T, (StatusCode, String)>
where
    T: DeserializeOwned,
{
    let value = proxy_request(server_path, permission_code, api_key, payload).await?;
    let response: crate::infrastructure::http::client::JsonRespnse = serde_json::from_value(value)
        .map_err(|error| {
            (
                StatusCode::BAD_GATEWAY,
                format!("failed to parse server response: {error}"),
            )
        })?;

    let data = response.data.ok_or_else(|| {
        (
            StatusCode::BAD_GATEWAY,
            "missing server response data".to_string(),
        )
    })?;

    serde_json::from_value(data).map_err(|error| {
        (
            StatusCode::BAD_GATEWAY,
            format!("failed to parse response data: {error}"),
        )
    })
}

pub async fn proxy_data_value_request<T>(
    server_path: &str,
    permission_code: &str,
    api_key: &str,
    payload: Value,
) -> Result<Value, (StatusCode, String)>
where
    T: DeserializeOwned + Serialize,
{
    let data: T = proxy_data_request(server_path, permission_code, api_key, payload).await?;
    serde_json::to_value(data).map_err(|error| {
        (
            StatusCode::BAD_GATEWAY,
            format!("failed to serialize response data: {error}"),
        )
    })
}

fn map_main_server_error(error: anyhow::Error) -> (StatusCode, String) {
    if let Some(response_error) = error.downcast_ref::<ResponseError>() {
        return match response_error {
            ResponseError::Unauthorized { message } => (StatusCode::UNAUTHORIZED, message.clone()),
            ResponseError::BadRequest { message } => (StatusCode::BAD_REQUEST, message.clone()),
            ResponseError::PublicKeyExpired => (
                StatusCode::BAD_GATEWAY,
                "server public key expired".to_string(),
            ),
            other => (StatusCode::BAD_GATEWAY, other.to_string()),
        };
    }

    (StatusCode::BAD_GATEWAY, error.to_string())
}
