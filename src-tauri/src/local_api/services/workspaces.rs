use axum::http::StatusCode;
use serde_json::Value;

use crate::local_api::{
    client::business::dispatch_data_value_request, context::LocalApiRequestContext,
    services::forward_service, types::LocalApiRoute,
};

macro_rules! workspace_service {
    ($fn_name:ident, $path:literal, $permission:literal, $response_ty:ty) => {
        pub async fn $fn_name(
            ctx: &LocalApiRequestContext,
            payload: Value,
        ) -> Result<Value, (StatusCode, String)> {
            dispatch_data_value_request::<$response_ty>(
                concat!("workspaces", $path),
                $permission,
                &ctx.api_key,
                payload,
            )
            .await
        }
    };
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
                    local_path: concat!("/api/local/workspaces", $path),
                    server_path: concat!("workspaces", $path),
                    permission_code: $permission,
                },
            )
            .await
        }
    };
}

workspace_service!(
    list_workspaces_service,
    "/list",
    "workspaces.list",
    crate::local_api::entitys::LocalApiWorkspaceListResponse
);
workspace_service!(
    get_workspace_service,
    "/get",
    "workspaces.get",
    crate::local_api::entitys::LocalApiWorkspaceDetail
);
workspace_service!(switch_workspace_service, "/switch", "workspaces.switch");
