use axum::http::StatusCode;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;

use crate::{
    app::handle::get_app_handle,
    domain::environment::EnvironmentStatus,
    local_api::{
        client::business::dispatch_data_request,
        entitys::{
            LocalApiBrowserKernelListResponse, LocalApiEnvironmentActionResponse,
            LocalApiEnvironmentDetailResponse, LocalApiEnvironmentListResponse,
            LocalApiGroupListResponse, LocalApiProxyDetail, LocalApiProxyListResponse,
            LocalApiTagItem, LocalApiWorkspaceDetail, LocalApiWorkspaceListResponse,
        },
    },
    mcp::error::McpToolError,
    services::environment::{
        BatchLaunchResult, CdpEndpointResponse, EnvironmentLaunchRuntimeService, KernelService,
    },
};

#[derive(Debug, Clone)]
pub struct LocalApiBridge {
    api_key: String,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct LocalApiEnvironmentListFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_uuids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LocalApiListEnvironmentsRequest {
    pub page: i64,
    pub page_size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<LocalApiEnvironmentListFilters>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct LocalApiListProxiesFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LocalApiListProxiesRequest {
    pub page: i64,
    pub page_size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<LocalApiListProxiesFilters>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LocalApiListBrowserKernelsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_code: Option<String>,
}

impl LocalApiBridge {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }

    pub async fn list_environments(
        &self,
        request: LocalApiListEnvironmentsRequest,
    ) -> Result<LocalApiEnvironmentListResponse, McpToolError> {
        self.dispatch_business("environments/list", "environments.list", &request).await
    }

    pub async fn get_environment(
        &self,
        env_uuid: &str,
    ) -> Result<LocalApiEnvironmentDetailResponse, McpToolError> {
        self.dispatch_business(
            "environments/detail",
            "environments.detail",
            &json!({ "uuid": env_uuid }),
        )
        .await
    }

    pub async fn start_environment(
        &self,
        env_uuid: &str,
    ) -> Result<LocalApiEnvironmentActionResponse, McpToolError> {
        let app = get_app_handle().map_err(|error| McpToolError::internal(error.to_string()))?;
        let launch_paths = EnvironmentLaunchRuntimeService::resolve_launch_paths(&app)
            .map_err(|error| McpToolError::internal(error.to_string()))?;

        EnvironmentLaunchRuntimeService::start_environment_by_uuid(
            env_uuid.to_string(),
            launch_paths,
            None,
            None,
        )
        .await
        .map_err(|error| McpToolError::upstream(error.to_string()))?;

        Ok(LocalApiEnvironmentActionResponse {
            env_uuid: env_uuid.to_string(),
            success: true,
        })
    }

    pub async fn stop_environment(
        &self,
        env_uuid: &str,
    ) -> Result<LocalApiEnvironmentActionResponse, McpToolError> {
        KernelService::stop_environment(env_uuid.to_string())
            .await
            .map_err(|error| McpToolError::upstream(error.to_string()))?;

        Ok(LocalApiEnvironmentActionResponse {
            env_uuid: env_uuid.to_string(),
            success: true,
        })
    }

    pub async fn batch_start_environments(
        &self,
        env_uuids: Vec<String>,
    ) -> Result<Vec<BatchLaunchResult>, McpToolError> {
        let app = get_app_handle().map_err(|error| McpToolError::internal(error.to_string()))?;
        let launch_paths = EnvironmentLaunchRuntimeService::resolve_launch_paths(&app)
            .map_err(|error| McpToolError::internal(error.to_string()))?;

        EnvironmentLaunchRuntimeService::batch_start_environments_by_uuid(
            env_uuids,
            launch_paths,
            None,
            None,
        )
        .await
        .map_err(|error| McpToolError::upstream(error.to_string()))
    }

    pub async fn batch_stop_environments(
        &self,
        env_uuids: Vec<String>,
    ) -> Result<Vec<BatchLaunchResult>, McpToolError> {
        KernelService::batch_stop_environments(env_uuids)
            .await
            .map_err(|error| McpToolError::upstream(error.to_string()))
    }

    pub async fn get_environment_status(
        &self,
        env_uuid: &str,
    ) -> Result<Option<EnvironmentStatus>, McpToolError> {
        KernelService::get_environment_status(env_uuid.to_string())
            .await
            .map_err(|error| McpToolError::upstream(error.to_string()))
    }

    pub async fn get_environment_cdp_endpoint(
        &self,
        env_uuid: &str,
    ) -> Result<Option<CdpEndpointResponse>, McpToolError> {
        KernelService::get_cdp_endpoint(env_uuid.to_string())
            .await
            .map_err(|error| McpToolError::upstream(error.to_string()))
    }

    pub async fn list_groups(&self) -> Result<LocalApiGroupListResponse, McpToolError> {
        self.dispatch_business("groups/list", "groups.list", &json!({})).await
    }

    pub async fn create_group<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<serde_json::Value, McpToolError> {
        self.dispatch_business("groups/create", "groups.create", payload).await
    }

    pub async fn update_group<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<(), McpToolError> {
        let _: serde_json::Value =
            self.dispatch_business("groups/update", "groups.update", payload).await?;
        Ok(())
    }

    pub async fn delete_group(&self, group_uuid: &str) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "groups/delete",
                "groups.delete",
                &json!({ "uuid": group_uuid }),
            )
            .await?;
        Ok(())
    }

    pub async fn list_tags(&self) -> Result<Vec<LocalApiTagItem>, McpToolError> {
        self.dispatch_business("tags/list", "tags.list", &json!({})).await
    }

    pub async fn create_tag<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<serde_json::Value, McpToolError> {
        self.dispatch_business("tags/create", "tags.create", payload).await
    }

    pub async fn update_tag<T: Serialize + ?Sized>(&self, payload: &T) -> Result<(), McpToolError> {
        let _: serde_json::Value =
            self.dispatch_business("tags/update", "tags.update", payload).await?;
        Ok(())
    }

    pub async fn delete_tag(&self, tag_uuid: &str) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business("tags/delete", "tags.delete", &json!({ "uuid": tag_uuid }))
            .await?;
        Ok(())
    }

    pub async fn list_workspaces(&self) -> Result<LocalApiWorkspaceListResponse, McpToolError> {
        self.dispatch_business("workspaces/list", "workspaces.list", &json!({})).await
    }

    pub async fn get_workspace(
        &self,
        workspace_uuid: &str,
    ) -> Result<LocalApiWorkspaceDetail, McpToolError> {
        self.dispatch_business(
            "workspaces/get",
            "workspaces.get",
            &json!({ "uuid": workspace_uuid }),
        )
        .await
    }

    pub async fn switch_workspace(&self, workspace_uuid: &str) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "workspaces/switch",
                "workspaces.switch",
                &json!({ "workspace_uuid": workspace_uuid }),
            )
            .await?;
        Ok(())
    }

    pub async fn list_proxies(
        &self,
        request: LocalApiListProxiesRequest,
    ) -> Result<LocalApiProxyListResponse, McpToolError> {
        self.dispatch_business("proxies/list", "proxies.list", &request).await
    }

    pub async fn get_proxy(&self, proxy_uuid: &str) -> Result<LocalApiProxyDetail, McpToolError> {
        self.dispatch_business(
            "proxies/detail",
            "proxies.detail",
            &json!({ "uuid": proxy_uuid }),
        )
        .await
    }

    pub async fn create_proxy<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<serde_json::Value, McpToolError> {
        self.dispatch_business("proxies/create", "proxies.create", payload).await
    }

    pub async fn update_proxy<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<(), McpToolError> {
        let _: serde_json::Value =
            self.dispatch_business("proxies/update", "proxies.update", payload).await?;
        Ok(())
    }

    pub async fn delete_proxy(&self, proxy_uuid: &str) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "proxies/delete",
                "proxies.delete",
                &json!({ "uuid": proxy_uuid }),
            )
            .await?;
        Ok(())
    }

    pub async fn batch_delete_proxies(&self, proxy_uuids: Vec<String>) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "proxies/batch-delete",
                "proxies.batch-delete",
                &json!({ "uuids": proxy_uuids }),
            )
            .await?;
        Ok(())
    }

    pub async fn list_browser_kernels(
        &self,
        request: LocalApiListBrowserKernelsRequest,
    ) -> Result<LocalApiBrowserKernelListResponse, McpToolError> {
        self.dispatch_business("browser-kernels/list", "browser-kernels.list", &request)
            .await
    }

    pub async fn batch_get_environments(
        &self,
        env_uuids: Vec<String>,
    ) -> Result<crate::local_api::entitys::LocalApiBatchEnvironmentDetailResponse, McpToolError>
    {
        self.dispatch_business(
            "environments/batch-detail",
            "environments.batch-detail",
            &json!({ "uuids": env_uuids }),
        )
        .await
    }

    pub async fn create_environment<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<serde_json::Value, McpToolError> {
        self.dispatch_business("environments/create", "environments.create", payload)
            .await
    }

    pub async fn update_environment<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business("environments/update", "environments.update", payload)
            .await?;
        Ok(())
    }

    pub async fn delete_environment(&self, env_uuid: &str) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/delete",
                "environments.delete",
                &json!({ "uuid": env_uuid }),
            )
            .await?;
        Ok(())
    }

    pub async fn batch_delete_environments(
        &self,
        env_uuids: Vec<String>,
    ) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/batch-delete",
                "environments.batch-delete",
                &json!({ "uuids": env_uuids }),
            )
            .await?;
        Ok(())
    }

    pub async fn list_recycle_bin_environments(
        &self,
        request: LocalApiListEnvironmentsRequest,
    ) -> Result<crate::local_api::entitys::LocalApiRecycleBinEnvironmentListResponse, McpToolError>
    {
        self.dispatch_business(
            "environments/recycle-bin/list",
            "environments.recycle-bin.list",
            &request,
        )
        .await
    }

    pub async fn restore_environment(&self, env_uuid: &str) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/recycle-bin/restore",
                "environments.recycle-bin.restore",
                &json!({ "uuid": env_uuid }),
            )
            .await?;
        Ok(())
    }

    pub async fn batch_restore_environments(
        &self,
        env_uuids: Vec<String>,
    ) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/recycle-bin/batch-restore",
                "environments.recycle-bin.batch-restore",
                &json!({ "uuids": env_uuids }),
            )
            .await?;
        Ok(())
    }

    pub async fn permanent_delete_environment(&self, env_uuid: &str) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/recycle-bin/permanent-delete",
                "environments.recycle-bin.permanent-delete",
                &json!({ "uuid": env_uuid }),
            )
            .await?;
        Ok(())
    }

    pub async fn batch_permanent_delete_environments(
        &self,
        env_uuids: Vec<String>,
    ) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/recycle-bin/batch-permanent-delete",
                "environments.recycle-bin.batch-permanent-delete",
                &json!({ "uuids": env_uuids }),
            )
            .await?;
        Ok(())
    }

    pub async fn set_environment_proxy<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business("environments/set-proxy", "environments.set-proxy", payload)
            .await?;
        Ok(())
    }

    pub async fn set_environment_accounts<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/set-accounts",
                "environments.set-accounts",
                payload,
            )
            .await?;
        Ok(())
    }

    pub async fn assign_tags<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/assign-tags",
                "environments.assign-tags",
                payload,
            )
            .await?;
        Ok(())
    }

    pub async fn batch_assign_tags<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/batch-assign-tags",
                "environments.batch-assign-tags",
                payload,
            )
            .await?;
        Ok(())
    }

    pub async fn remove_tag<T: Serialize + ?Sized>(&self, payload: &T) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/remove-tag",
                "environments.remove-tag",
                payload,
            )
            .await?;
        Ok(())
    }

    pub async fn batch_remove_tags<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/batch-remove-tags",
                "environments.batch-remove-tags",
                payload,
            )
            .await?;
        Ok(())
    }

    pub async fn move_to_group<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/move-to-group",
                "environments.move-to-group",
                payload,
            )
            .await?;
        Ok(())
    }

    pub async fn batch_move_to_group<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/batch-move-to-group",
                "environments.batch-move-to-group",
                payload,
            )
            .await?;
        Ok(())
    }

    pub async fn list_environment_urls(
        &self,
        env_uuid: &str,
    ) -> Result<Vec<crate::local_api::entitys::LocalApiEnvironmentUrlItem>, McpToolError> {
        self.dispatch_business(
            "environments/urls/list",
            "environments.urls.list",
            &json!({ "uuid": env_uuid }),
        )
        .await
    }

    pub async fn add_environment_url<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<serde_json::Value, McpToolError> {
        self.dispatch_business("environments/urls/add", "environments.urls.add", payload)
            .await
    }

    pub async fn delete_environment_url(&self, id: i32) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/urls/delete",
                "environments.urls.delete",
                &json!({ "id": id }),
            )
            .await?;
        Ok(())
    }

    pub async fn clear_environment_urls(&self, env_uuid: &str) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/urls/clear",
                "environments.urls.clear",
                &json!({ "environment_uuid": env_uuid }),
            )
            .await?;
        Ok(())
    }

    pub async fn list_environment_cookies(
        &self,
        env_uuid: &str,
    ) -> Result<Vec<crate::local_api::entitys::LocalApiEnvironmentCookieItem>, McpToolError> {
        self.dispatch_business(
            "environments/cookies/list",
            "environments.cookies.list",
            &json!({ "uuid": env_uuid }),
        )
        .await
    }

    pub async fn add_environment_cookie<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<serde_json::Value, McpToolError> {
        self.dispatch_business(
            "environments/cookies/add",
            "environments.cookies.add",
            payload,
        )
        .await
    }

    pub async fn delete_environment_cookie(&self, id: i32) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/cookies/delete",
                "environments.cookies.delete",
                &json!({ "id": id }),
            )
            .await?;
        Ok(())
    }

    pub async fn clear_environment_cookies(&self, env_uuid: &str) -> Result<(), McpToolError> {
        let _: serde_json::Value = self
            .dispatch_business(
                "environments/cookies/clear",
                "environments.cookies.clear",
                &json!({ "environment_uuid": env_uuid }),
            )
            .await?;
        Ok(())
    }

    async fn dispatch_business<T, P>(
        &self,
        route: &str,
        permission_code: &str,
        payload: &P,
    ) -> Result<T, McpToolError>
    where
        T: DeserializeOwned,
        P: Serialize + ?Sized,
    {
        let payload = serde_json::to_value(payload)
            .map_err(|error| McpToolError::internal(error.to_string()))?;

        dispatch_data_request::<T>(route, permission_code, &self.api_key, payload)
            .await
            .map_err(map_proxy_error)
    }
}

fn map_proxy_error((status, message): (StatusCode, String)) -> McpToolError {
    if status.is_client_error() {
        McpToolError::invalid_params(message)
    } else {
        McpToolError::upstream(message)
    }
}
