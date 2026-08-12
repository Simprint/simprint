mod detail;
mod fingerprint;
mod kernel;
mod paths;
mod types;

use futures::future::try_join_all;
use serde::Deserialize;
use tauri::AppHandle;

use crate::{
    app::handle::get_app_handle,
    core::error::Result,
    infrastructure::{mihomo::MihomoLocalProxy, persistence::tauri_store},
    services::environment::{
        AccountInfo, BatchLaunchRequest, BatchLaunchResult, CookieGroup, ExtensionInfo,
        KernelService, KernelStatusEmitter, ProxyConfig,
    },
};

use self::{
    detail::get_environment_launch_detail,
    fingerprint::build_fingerprint_config,
    kernel::resolve_kernel_launch,
    paths::get_launch_paths,
    types::{EnvironmentProxyLike, LaunchPaths},
};

pub struct EnvironmentLaunchRuntimeService;

const ENVIRONMENT_LOCAL_PROXY_BINDINGS_STORE_KEY: &str = "environment.local_proxy_bindings";
const MIHOMO_LOCAL_PROXIES_STORE_KEY: &str = "mihomo.local_proxies";

#[derive(Debug, Clone, Deserialize)]
struct EnvironmentLocalProxyBinding {
    env_uuid: String,
    node_name: String,
}

impl EnvironmentLaunchRuntimeService {
    pub fn resolve_launch_paths(app: &AppHandle) -> Result<LaunchPaths> {
        get_launch_paths(app)
    }

    pub async fn start_environment_by_uuid(
        env_uuid: String,
        launch_paths: LaunchPaths,
        status_emitter: Option<KernelStatusEmitter>,
        display_id: Option<String>,
    ) -> Result<()> {
        let app = get_app_handle()?;
        let request = Self::build_launch_request(
            app.clone(),
            env_uuid,
            &launch_paths,
            status_emitter.clone(),
            display_id,
        )
        .await?;

        KernelService::launch_environment(
            app,
            request.exe_path,
            request.env_uuid,
            request.cache_path,
            request.cookies,
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
        display_ids_by_env_uuid: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<BatchLaunchResult>> {
        let app = get_app_handle()?;
        let requests = try_join_all(env_uuids.into_iter().map(|env_uuid| {
            let display_id =
                display_ids_by_env_uuid.as_ref().and_then(|items| items.get(&env_uuid).cloned());
            Self::build_launch_request(
                app.clone(),
                env_uuid,
                &launch_paths,
                status_emitter.clone(),
                display_id,
            )
        }))
        .await?;

        KernelService::batch_launch_environments(app, requests, status_emitter).await
    }

    async fn build_launch_request(
        app: AppHandle,
        env_uuid: String,
        launch_paths: &LaunchPaths,
        status_emitter: Option<KernelStatusEmitter>,
        display_id: Option<String>,
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
        fingerprint_config.env_id = display_id;
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
            cookies: normalize_cookies(detail.cookies),
            urls: normalize_urls(detail.urls),
            proxy: resolve_environment_proxy_config(&app, &env.uuid, detail.proxy),
            fingerprint_config: Some(fingerprint_config),
            accounts: normalize_accounts(detail.accounts),
            extensions: normalize_extensions(detail.extensions),
        })
    }
}

fn resolve_environment_proxy_config(
    app: &AppHandle,
    env_uuid: &str,
    remote_proxy: Option<EnvironmentProxyLike>,
) -> Option<ProxyConfig> {
    match resolve_local_proxy_config(app, env_uuid) {
        LocalProxyResolution::Resolved(proxy) => Some(proxy),
        LocalProxyResolution::MissingBindingTarget => None,
        LocalProxyResolution::NoBinding => build_tauri_proxy_config(remote_proxy),
    }
}

fn resolve_local_proxy_config(app: &AppHandle, env_uuid: &str) -> LocalProxyResolution {
    let bindings = tauri_store::get_store_key(app, ENVIRONMENT_LOCAL_PROXY_BINDINGS_STORE_KEY)
        .and_then(|value| {
            serde_json::from_value::<
                    std::collections::HashMap<String, EnvironmentLocalProxyBinding>,
                >(value)
                .ok()
        })
        .unwrap_or_default();

    let Some(binding) = bindings.get(env_uuid) else {
        return LocalProxyResolution::NoBinding;
    };

    let proxies = tauri_store::get_store_key(app, MIHOMO_LOCAL_PROXIES_STORE_KEY)
        .and_then(|value| serde_json::from_value::<Vec<MihomoLocalProxy>>(value).ok())
        .unwrap_or_default();

    let Some(proxy) = proxies.into_iter().find(|proxy| proxy.node_name == binding.node_name) else {
        log::warn!(
            "local proxy binding target missing for env_uuid={} node_name={}",
            binding.env_uuid,
            binding.node_name
        );
        return LocalProxyResolution::MissingBindingTarget;
    };

    let host = proxy.listen_host.trim().to_string();
    if host.is_empty() || proxy.listen_port == 0 {
        return LocalProxyResolution::MissingBindingTarget;
    }

    LocalProxyResolution::Resolved(ProxyConfig {
        host,
        port: proxy.listen_port,
        proxy_type: proxy.proxy_scheme,
        username: None,
        password: None,
    })
}

enum LocalProxyResolution {
    NoBinding,
    MissingBindingTarget,
    Resolved(ProxyConfig),
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

fn normalize_cookies(
    cookies: Option<Vec<self::types::EnvironmentCookieLike>>,
) -> Option<Vec<CookieGroup>> {
    let cookies = cookies?
        .into_iter()
        .map(|item| CookieGroup {
            site: item.site.trim().to_string(),
            cookie_text: item.cookie_text.trim().to_string(),
        })
        .filter(|item| !item.site.is_empty() && !item.cookie_text.is_empty())
        .collect::<Vec<_>>();

    if cookies.is_empty() {
        None
    } else {
        Some(cookies)
    }
}

fn normalize_urls(urls: Option<Vec<self::types::EnvironmentUrlLike>>) -> Option<Vec<String>> {
    let urls = urls?
        .into_iter()
        .map(|item| item.url)
        .map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty())
        .collect::<Vec<_>>();

    if urls.is_empty() { None } else { Some(urls) }
}
