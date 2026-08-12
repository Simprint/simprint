use axum::http::StatusCode;
use serde_json::Value;

use crate::local_api::{
    client::business::dispatch_data_value_request, context::LocalApiRequestContext,
};

pub async fn list_browser_kernels_service(
    ctx: &LocalApiRequestContext,
    payload: Value,
) -> Result<Value, (StatusCode, String)> {
    dispatch_data_value_request::<crate::local_api::entitys::LocalApiBrowserKernelListResponse>(
        "browser-kernels/list",
        "browser-kernels.list",
        &ctx.api_key,
        payload,
    )
    .await
}
