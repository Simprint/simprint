use axum::http::StatusCode;
use serde_json::Value;

use crate::local_api::{
    client::business::dispatch_data_value_request, context::LocalApiRequestContext,
    services::forward_service, types::LocalApiRoute,
};

macro_rules! proxy_service {
    ($fn_name:ident, $path:literal, $permission:literal, $response_ty:ty) => {
        pub async fn $fn_name(
            ctx: &LocalApiRequestContext,
            payload: Value,
        ) -> Result<Value, (StatusCode, String)> {
            dispatch_data_value_request::<$response_ty>(
                concat!("proxies", $path),
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
                    local_path: concat!("/api/local/proxies", $path),
                    server_path: concat!("proxies", $path),
                    permission_code: $permission,
                },
            )
            .await
        }
    };
}

proxy_service!(
    list_proxies_service,
    "/list",
    "proxies.list",
    crate::local_api::entitys::LocalApiProxyListResponse
);
proxy_service!(
    get_proxy_service,
    "/detail",
    "proxies.detail",
    crate::local_api::entitys::LocalApiProxyDetail
);
proxy_service!(create_proxy_service, "/create", "proxies.create");
proxy_service!(update_proxy_service, "/update", "proxies.update");
proxy_service!(delete_proxy_service, "/delete", "proxies.delete");
proxy_service!(
    batch_delete_proxies_service,
    "/batch-delete",
    "proxies.batch-delete"
);
proxy_service!(
    batch_import_proxies_service,
    "/batch-import",
    "proxies.batch-import"
);
