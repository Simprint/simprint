use axum::http::StatusCode;
use serde_json::Value;

use crate::local_api::{
    client::business::dispatch_data_request, context::LocalApiRequestContext,
    services::forward_service, types::LocalApiRoute,
};

macro_rules! group_service {
    ($fn_name:ident, $path:literal, $permission:literal) => {
        pub async fn $fn_name(
            ctx: &LocalApiRequestContext,
            payload: Value,
        ) -> Result<Value, (StatusCode, String)> {
            forward_service(
                ctx,
                payload,
                LocalApiRoute {
                    method: "POST",
                    local_path: concat!("/api/local/groups", $path),
                    server_path: concat!("groups", $path),
                    permission_code: $permission,
                },
            )
            .await
        }
    };
}

pub async fn list_groups_service(
    ctx: &LocalApiRequestContext,
    payload: Value,
) -> Result<Value, (StatusCode, String)> {
    let items =
        dispatch_data_request::<Vec<Value>>("groups/list", "groups.list", &ctx.api_key, payload)
            .await?;
    Ok(serde_json::json!({ "items": items }))
}
group_service!(create_group_service, "/create", "groups.create");
group_service!(update_group_service, "/update", "groups.update");
group_service!(delete_group_service, "/delete", "groups.delete");
