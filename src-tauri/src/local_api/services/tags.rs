use axum::http::StatusCode;
use serde_json::Value;

use crate::local_api::{
    client::business::dispatch_data_value_request, context::LocalApiRequestContext,
    services::forward_service, types::LocalApiRoute,
};

macro_rules! tag_service {
    ($fn_name:ident, $path:literal, $permission:literal, $response_ty:ty) => {
        pub async fn $fn_name(
            ctx: &LocalApiRequestContext,
            payload: Value,
        ) -> Result<Value, (StatusCode, String)> {
            dispatch_data_value_request::<$response_ty>(
                concat!("tags", $path),
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
                    local_path: concat!("/api/local/tags", $path),
                    server_path: concat!("tags", $path),
                    permission_code: $permission,
                },
            )
            .await
        }
    };
}

tag_service!(
    list_tags_service,
    "/list",
    "tags.list",
    Vec<crate::local_api::entitys::LocalApiTagItem>
);
tag_service!(create_tag_service, "/create", "tags.create");
tag_service!(update_tag_service, "/update", "tags.update");
tag_service!(delete_tag_service, "/delete", "tags.delete");
