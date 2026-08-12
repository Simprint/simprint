use axum::http::StatusCode;
use serde_json::Value;

use crate::{
    app::handle::get_app_handle,
    local_api::{
        client::business::dispatch_data_value_request,
        context::LocalApiRequestContext,
        entitys::{
            LocalApiBatchStartEnvironmentsRequest, LocalApiEnvironmentActionResponse,
            LocalApiStartEnvironmentRequest,
        },
        services::forward_service,
        types::LocalApiRoute,
    },
    services::environment::{EnvironmentLaunchRuntimeService, KernelService},
};

macro_rules! environment_service {
    ($fn_name:ident, $path:literal, $permission:literal, $response_ty:ty) => {
        pub async fn $fn_name(
            ctx: &LocalApiRequestContext,
            payload: Value,
        ) -> Result<Value, (StatusCode, String)> {
            dispatch_data_value_request::<$response_ty>(
                concat!("environments", $path),
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
                    local_path: concat!("/api/local/environments", $path),
                    server_path: concat!("environments", $path),
                    permission_code: $permission,
                },
            )
            .await
        }
    };
}

environment_service!(
    list_environments_service,
    "/list",
    "environments.list",
    crate::local_api::entitys::LocalApiEnvironmentListResponse
);
environment_service!(
    get_environment_service,
    "/detail",
    "environments.detail",
    crate::local_api::entitys::LocalApiEnvironmentDetailResponse
);
environment_service!(
    batch_get_environments_service,
    "/batch-detail",
    "environments.batch-detail",
    crate::local_api::entitys::LocalApiBatchEnvironmentDetailResponse
);
environment_service!(create_environment_service, "/create", "environments.create");
environment_service!(
    batch_create_environments_service,
    "/batch-create",
    "environments.batch-create"
);
environment_service!(update_environment_service, "/update", "environments.update");
environment_service!(delete_environment_service, "/delete", "environments.delete");
environment_service!(
    batch_delete_environments_service,
    "/batch-delete",
    "environments.batch-delete"
);
environment_service!(
    set_environment_proxy_service,
    "/set-proxy",
    "environments.set-proxy"
);
environment_service!(
    assign_tags_service,
    "/assign-tags",
    "environments.assign-tags"
);
environment_service!(remove_tag_service, "/remove-tag", "environments.remove-tag");
environment_service!(
    move_to_group_service,
    "/move-to-group",
    "environments.move-to-group"
);
environment_service!(
    batch_move_to_group_service,
    "/batch-move-to-group",
    "environments.batch-move-to-group"
);
environment_service!(
    set_environment_accounts_service,
    "/set-accounts",
    "environments.set-accounts"
);
environment_service!(
    batch_assign_tags_service,
    "/batch-assign-tags",
    "environments.batch-assign-tags"
);
environment_service!(
    batch_remove_tags_service,
    "/batch-remove-tags",
    "environments.batch-remove-tags"
);
environment_service!(
    list_environment_urls_service,
    "/urls/list",
    "environments.urls.list",
    Vec<crate::local_api::entitys::LocalApiEnvironmentUrlItem>
);
environment_service!(
    add_environment_url_service,
    "/urls/add",
    "environments.urls.add"
);
environment_service!(
    delete_environment_url_service,
    "/urls/delete",
    "environments.urls.delete"
);
environment_service!(
    clear_environment_urls_service,
    "/urls/clear",
    "environments.urls.clear"
);
environment_service!(
    list_environment_cookies_service,
    "/cookies/list",
    "environments.cookies.list",
    Vec<crate::local_api::entitys::LocalApiEnvironmentCookieItem>
);
environment_service!(
    add_environment_cookie_service,
    "/cookies/add",
    "environments.cookies.add"
);
environment_service!(
    delete_environment_cookie_service,
    "/cookies/delete",
    "environments.cookies.delete"
);
environment_service!(
    clear_environment_cookies_service,
    "/cookies/clear",
    "environments.cookies.clear"
);
environment_service!(
    list_recycle_bin_environments_service,
    "/recycle-bin/list",
    "environments.recycle-bin.list",
    crate::local_api::entitys::LocalApiRecycleBinEnvironmentListResponse
);
environment_service!(
    restore_environment_service,
    "/recycle-bin/restore",
    "environments.recycle-bin.restore"
);
environment_service!(
    batch_restore_environments_service,
    "/recycle-bin/batch-restore",
    "environments.recycle-bin.batch-restore"
);
environment_service!(
    permanent_delete_environment_service,
    "/recycle-bin/permanent-delete",
    "environments.recycle-bin.permanent-delete"
);
environment_service!(
    batch_permanent_delete_environments_service,
    "/recycle-bin/batch-permanent-delete",
    "environments.recycle-bin.batch-permanent-delete"
);

pub async fn start_environment_service(
    _ctx: &LocalApiRequestContext,
    payload: LocalApiStartEnvironmentRequest,
) -> Result<Value, (StatusCode, String)> {
    let app =
        get_app_handle().map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    let launch_paths = EnvironmentLaunchRuntimeService::resolve_launch_paths(&app)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    let env_uuid = payload.env_uuid;

    EnvironmentLaunchRuntimeService::start_environment_by_uuid(
        env_uuid.clone(),
        launch_paths,
        None,
        None,
    )
    .await
    .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))?;

    serde_json::to_value(LocalApiEnvironmentActionResponse {
        env_uuid,
        success: true,
    })
    .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))
}

pub async fn stop_environment_service(
    _ctx: &LocalApiRequestContext,
    payload: LocalApiStartEnvironmentRequest,
) -> Result<Value, (StatusCode, String)> {
    let env_uuid = payload.env_uuid;
    KernelService::stop_environment(env_uuid.clone())
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))?;

    serde_json::to_value(LocalApiEnvironmentActionResponse {
        env_uuid,
        success: true,
    })
    .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))
}

pub async fn batch_start_environments_local_service(
    _ctx: &LocalApiRequestContext,
    payload: LocalApiBatchStartEnvironmentsRequest,
) -> Result<Value, (StatusCode, String)> {
    let app =
        get_app_handle().map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    let launch_paths = EnvironmentLaunchRuntimeService::resolve_launch_paths(&app)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    let results = EnvironmentLaunchRuntimeService::batch_start_environments_by_uuid(
        payload.env_uuids,
        launch_paths,
        None,
        None,
    )
    .await
    .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))?;

    serde_json::to_value(results).map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))
}

pub async fn batch_stop_environments_local_service(
    _ctx: &LocalApiRequestContext,
    payload: LocalApiBatchStartEnvironmentsRequest,
) -> Result<Value, (StatusCode, String)> {
    let results = KernelService::batch_stop_environments(payload.env_uuids)
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))?;

    serde_json::to_value(results).map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))
}

pub async fn get_environment_status_service(
    _ctx: &LocalApiRequestContext,
    payload: LocalApiStartEnvironmentRequest,
) -> Result<Value, (StatusCode, String)> {
    let status = crate::app::context::AppContext::get()
        .env_status_manager
        .get_status(&payload.env_uuid)
        .await;

    serde_json::to_value(status).map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))
}

pub async fn get_environment_cdp_endpoint_service(
    _ctx: &LocalApiRequestContext,
    payload: LocalApiStartEnvironmentRequest,
) -> Result<Value, (StatusCode, String)> {
    let endpoint = KernelService::get_cdp_endpoint(payload.env_uuid)
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))?;

    serde_json::to_value(endpoint).map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))
}
