mod detail;
mod fingerprint;
mod kernel;
mod paths;
mod types;

use futures::future::try_join_all;
use tauri::AppHandle;

use crate::{
    app::handle::get_app_handle,
    core::error::Result,
    services::environment::{
        AccountInfo, BatchLaunchRequest, BatchLaunchResult, ExtensionInfo, KernelService,
        KernelStatusEmitter, ProxyConfig,
    },
};

use self::{
    detail::get_environment_launch_detail,
    fingerprint::build_fingerprint_config,
    kernel::{get_window_info, resolve_kernel_launch},
    paths::get_launch_paths,
    types::{EnvironmentProxyLike, LaunchPaths},
};

pub struct EnvironmentLaunchRuntimeService;

impl EnvironmentLaunchRuntimeService {
    pub fn resolve_launch_paths(app: &AppHandle) -> Result<LaunchPaths> {
        get_launch_paths(app)
    }

    pub async fn start_environment_by_uuid(
        env_uuid: String,
        launch_paths: LaunchPaths,
        status_emitter: Option<KernelStatusEmitter>,
    ) -> Result<()> {
        let app = get_app_handle()?;
        let request = Self::build_launch_request(
            app.clone(),
            env_uuid,
            &launch_paths,
            status_emitter.clone(),
        )
        .await?;

        KernelService::launch_environment(
            app,
            request.exe_path,
            request.env_uuid,
            request.cache_path,
            request.urls,
            request.proxy,
            request.fingerprint_config,
            request.accounts,
            request.extensions,
            status_emitter,
        )
        .await
    }

    pub async fn batch_start_environments_by_uuid(
        env_uuids: Vec<String>,
        launch_paths: LaunchPaths,
        status_emitter: Option<KernelStatusEmitter>,
    ) -> Result<Vec<BatchLaunchResult>> {
        let app = get_app_handle()?;
        let requests = try_join_all(env_uuids.into_iter().map(|env_uuid| {
            Self::build_launch_request(app.clone(), env_uuid, &launch_paths, status_emitter.clone())
        }))
        .await?;

        KernelService::batch_launch_environments(app, requests, status_emitter).await
    }

    async fn build_launch_request(
        app: AppHandle,
        env_uuid: String,
        launch_paths: &LaunchPaths,
        status_emitter: Option<KernelStatusEmitter>,
    ) -> Result<BatchLaunchRequest> {
        let detail = get_environment_launch_detail(&env_uuid).await?;
        let env = detail
            .environment
            .as_ref()
            .ok_or("Environment detail is missing environment uuid.")?;

        let resolved_kernel = resolve_kernel_launch(
            app.clone(),
            &detail,
            &launch_paths.profiles_path,
            status_emitter.clone(),
        )
        .await?;

        let mut fingerprint_config = build_fingerprint_config(detail.config.as_ref());
        fingerprint_config.env_id =
            Some(env.id.map(|id| id.to_string()).unwrap_or_else(|| env.uuid.clone()));
        fingerprint_config.env_name = Some(
            env.name
                .clone()
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| "Unnamed Environment".to_string()),
        );

        Ok(BatchLaunchRequest {
            exe_path: resolved_kernel.exe_path,
            env_uuid: env.uuid.clone(),
            cache_path: launch_paths.cache_path.clone(),
            urls: normalize_urls(detail.config.as_ref()),
            proxy: build_tauri_proxy_config(detail.proxy),
            fingerprint_config: Some(fingerprint_config),
            accounts: normalize_accounts(detail.accounts),
            extensions: normalize_extensions(detail.extensions),
        })
    }
}

fn build_tauri_proxy_config(proxy: Option<EnvironmentProxyLike>) -> Option<ProxyConfig> {
    let proxy = proxy?;
    let host = proxy.host?;
    let port = proxy.port?;

    Some(ProxyConfig {
        host,
        port,
        proxy_type: proxy.proxy_type.unwrap_or_else(|| "http".to_string()),
        username: proxy.username,
        password: proxy.password.map(crate::infrastructure::proxy::types::ProxyPassword::plain),
    })
}

fn normalize_accounts(accounts: Option<Vec<AccountInfo>>) -> Option<Vec<AccountInfo>> {
    accounts.filter(|items| !items.is_empty())
}

fn normalize_extensions(extensions: Option<Vec<ExtensionInfo>>) -> Option<Vec<ExtensionInfo>> {
    extensions.filter(|items| !items.is_empty())
}

fn normalize_urls(config: Option<&serde_json::Value>) -> Option<Vec<String>> {
    let urls = get_window_info(config)
        .get("urls")
        .and_then(serde_json::Value::as_array)?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .map(|url| url.to_string())
        .collect::<Vec<_>>();

    if urls.is_empty() { None } else { Some(urls) }
}
