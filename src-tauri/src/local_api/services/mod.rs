pub mod browser_kernels;
pub mod environments;
pub mod groups;
pub mod proxies;
pub mod tags;
pub mod workspaces;

use axum::http::StatusCode;
use serde_json::Value;

use crate::local_api::{
    client::business::dispatch_request, context::LocalApiRequestContext, types::LocalApiRoute,
};

pub async fn forward_service(
    ctx: &LocalApiRequestContext,
    payload: Value,
    route: LocalApiRoute,
) -> Result<Value, (StatusCode, String)> {
    let response = dispatch_request(
        route.server_path,
        route.permission_code,
        &ctx.api_key,
        payload,
    )
    .await?;
    let response: crate::infrastructure::http::client::JsonRespnse =
        serde_json::from_value(response).map_err(|error| {
            (
                StatusCode::BAD_GATEWAY,
                format!("failed to parse business response: {error}"),
            )
        })?;
    Ok(response.data.unwrap_or(Value::Null))
}
