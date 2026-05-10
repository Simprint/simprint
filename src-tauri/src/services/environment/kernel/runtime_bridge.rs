use std::collections::HashMap;
use std::path::PathBuf;

use futures::future::try_join_all;
use tauri::AppHandle;

use crate::app::context::AppContext;
use crate::core::error::Result;
use crate::domain::environment::EnvironmentStatus;
use crate::infrastructure::runtime::{
    AccountConfig, EnvironmentCommandRequest, EnvironmentCommandResponse, EnvironmentStartRequest,
    FingerprintConfig, WindowBoundsRequest,
};

use super::extension;
use super::timezone;
use super::types::{
    AccountInfo, BatchLaunchRequest, BatchLaunchResult, CdpEndpointResponse, KernelStatusEmitter,
    ProxyConfig,
};
use super::utils::emit_status;

pub async fn launch_environment(
    app: AppHandle,
    exe_path: String,
    env_uuid: String,
    cache_path: String,
    urls: Option<Vec<String>>,
    proxy: Option<ProxyConfig>,
    fingerprint_config: Option<FingerprintConfig>,
    accounts: Option<Vec<AccountInfo>>,
    extensions: Option<Vec<super::types::ExtensionInfo>>,
    status_emitter: Option<KernelStatusEmitter>,
) -> Result<()> {
    let request = prepare_start_request(
        app,
        exe_path,
        env_uuid,
        cache_path,
        urls,
        proxy,
        fingerprint_config,
        accounts,
        extensions,
        status_emitter,
    )
    .await?;

    let response = AppContext::get()
        .simprint_runtime_manager
        .send_environment_command(EnvironmentCommandRequest::StartEnvironment { request })
        .await?;

    match response {
        EnvironmentCommandResponse::Ack | EnvironmentCommandResponse::Started { .. } => Ok(()),
        other => Err(format!("simprint-runtime 返回了非预期响应: {:?}", other).into()),
    }
}

pub async fn batch_launch_environments(
    app: AppHandle,
    launch_requests: Vec<BatchLaunchRequest>,
    status_emitter: Option<KernelStatusEmitter>,
) -> Result<Vec<BatchLaunchResult>> {
    let requests = try_join_all(launch_requests.into_iter().map(|request| {
        prepare_start_request(
            app.clone(),
            request.exe_path,
            request.env_uuid,
            request.cache_path,
            request.urls,
            request.proxy,
            request.fingerprint_config,
            request.accounts,
            request.extensions,
            status_emitter.clone(),
        )
    }))
    .await?;

    let response = AppContext::get()
        .simprint_runtime_manager
        .send_environment_command(EnvironmentCommandRequest::BatchStartEnvironments { requests })
        .await?;

    match response {
        EnvironmentCommandResponse::BatchLaunchResults { results } => {
            Ok(results.into_iter().map(map_batch_launch_result).collect())
        }
        other => Err(format!("simprint-runtime 返回了非预期响应: {:?}", other).into()),
    }
}

pub async fn stop_environment(env_uuid: String) -> Result<()> {
    let response = AppContext::get()
        .simprint_runtime_manager
        .send_environment_command(EnvironmentCommandRequest::StopEnvironment { env_uuid })
        .await?;

    match response {
        EnvironmentCommandResponse::Ack => Ok(()),
        other => Err(format!("simprint-runtime 返回了非预期响应: {:?}", other).into()),
    }
}

pub async fn batch_stop_environments(env_uuids: Vec<String>) -> Result<Vec<BatchLaunchResult>> {
    let response = AppContext::get()
        .simprint_runtime_manager
        .send_environment_command(EnvironmentCommandRequest::BatchStopEnvironments { env_uuids })
        .await?;

    match response {
        EnvironmentCommandResponse::BatchLaunchResults { results } => {
            Ok(results.into_iter().map(map_batch_launch_result).collect())
        }
        other => Err(format!("simprint-runtime 返回了非预期响应: {:?}", other).into()),
    }
}

pub async fn refresh_proxy(env_uuid: String, proxy: Option<ProxyConfig>) -> Result<()> {
    let proxy = match proxy {
        Some(proxy) => Some(proxy.decrypt_password()?.to_browser_proxy_config()),
        None => None,
    };

    let response = AppContext::get()
        .simprint_runtime_manager
        .send_environment_command(EnvironmentCommandRequest::RefreshProxy { env_uuid, proxy })
        .await?;

    match response {
        EnvironmentCommandResponse::Ack => Ok(()),
        other => Err(format!("simprint-runtime 返回了非预期响应: {:?}", other).into()),
    }
}

pub async fn get_connected_environments() -> Result<Vec<String>> {
    if !crate::infrastructure::persistence::credential::is_login() {
        return Ok(vec![]);
    }

    let response = AppContext::get()
        .simprint_runtime_manager
        .send_environment_command(EnvironmentCommandRequest::GetConnectedEnvironments)
        .await?;

    match response {
        EnvironmentCommandResponse::ConnectedEnvironments { env_ids } => Ok(env_ids),
        other => Err(format!("simprint-runtime 返回了非预期响应: {:?}", other).into()),
    }
}

pub async fn get_cdp_endpoint(env_uuid: String) -> Result<Option<CdpEndpointResponse>> {
    if !crate::infrastructure::persistence::credential::is_login() {
        return Ok(None);
    }

    let response = AppContext::get()
        .simprint_runtime_manager
        .send_environment_command(EnvironmentCommandRequest::GetCdpEndpoint { env_uuid })
        .await?;

    match response {
        EnvironmentCommandResponse::CdpEndpoint { endpoint } => {
            Ok(endpoint.map(|endpoint| CdpEndpointResponse {
                env_uuid: endpoint.env_uuid,
                host: endpoint.host,
                port: endpoint.port,
                version_url: endpoint.version_url,
                list_url: endpoint.list_url,
                browser_ws_url: endpoint.browser_ws_url,
            }))
        }
        other => Err(format!("simprint-runtime 返回了非预期响应: {:?}", other).into()),
    }
}

pub async fn get_environment_status(env_uuid: String) -> Result<Option<EnvironmentStatus>> {
    if !crate::infrastructure::persistence::credential::is_login() {
        return Ok(None);
    }

    let response = AppContext::get()
        .simprint_runtime_manager
        .send_environment_command(EnvironmentCommandRequest::GetEnvironmentStatus { env_uuid })
        .await?;

    match response {
        EnvironmentCommandResponse::Status { status } => Ok(status),
        other => Err(format!("simprint-runtime 返回了非预期响应: {:?}", other).into()),
    }
}

pub async fn get_all_environment_statuses() -> Result<HashMap<String, EnvironmentStatus>> {
    if !crate::infrastructure::persistence::credential::is_login() {
        return Ok(HashMap::new());
    }

    let response = AppContext::get()
        .simprint_runtime_manager
        .send_environment_command(EnvironmentCommandRequest::GetAllEnvironmentStatuses)
        .await?;

    match response {
        EnvironmentCommandResponse::AllStatuses { statuses } => Ok(statuses),
        other => Err(format!("simprint-runtime 返回了非预期响应: {:?}", other).into()),
    }
}

pub async fn set_window_bounds(
    env_uuid: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<()> {
    let response = AppContext::get()
        .simprint_runtime_manager
        .send_environment_command(EnvironmentCommandRequest::SetWindowBounds {
            request: WindowBoundsRequest {
                env_uuid,
                x,
                y,
                width,
                height,
            },
        })
        .await?;

    match response {
        EnvironmentCommandResponse::Ack => Ok(()),
        other => Err(format!("simprint-runtime 返回了非预期响应: {:?}", other).into()),
    }
}

async fn prepare_start_request(
    app: AppHandle,
    exe_path: String,
    env_uuid: String,
    cache_path: String,
    urls: Option<Vec<String>>,
    proxy: Option<ProxyConfig>,
    mut fingerprint_config: Option<FingerprintConfig>,
    accounts: Option<Vec<AccountInfo>>,
    extensions: Option<Vec<super::types::ExtensionInfo>>,
    status_emitter: Option<KernelStatusEmitter>,
) -> Result<EnvironmentStartRequest> {
    let env_id = env_uuid.trim().to_string();
    if !PathBuf::from(&exe_path).exists() {
        return Err("可执行文件不存在".into());
    }

    let user_data_dir =
        PathBuf::from(cache_path.trim()).join("browser").join("cache").join(&env_id);

    if let Some(ctx) = AppContext::try_get() {
        ctx.env_status_manager
            .set_status(&env_id, EnvironmentStatus::Initializing)
            .await;
    }

    emit_status(
        status_emitter.as_ref(),
        &Some(env_id.clone()),
        "",
        EnvironmentStatus::Initializing,
        Some("初始化中…"),
        None,
        None,
        None,
    );

    if let Some(ref mut config) = fingerprint_config {
        if let Some(detected_timezone) = timezone::detect_timezone(proxy.as_ref()).await {
            config.timezone = Some(detected_timezone);
        }
    }

    if let Some(ctx) = AppContext::try_get() {
        ctx.env_status_manager.set_status(&env_id, EnvironmentStatus::Starting).await;
    }

    emit_status(
        status_emitter.as_ref(),
        &Some(env_id.clone()),
        "",
        EnvironmentStatus::Starting,
        Some("启动中…"),
        None,
        None,
        None,
    );

    let display_id = fingerprint_config.as_ref().and_then(|config| config.env_id.clone());
    let window_size = fingerprint_config.as_ref().and_then(|config| {
        if let (Some(width), Some(height)) = (config.window_width, config.window_height) {
            Some(format!("{},{}", width, height))
        } else {
            config.window_size.clone()
        }
    });

    let window_position = if let Some(ctx) = AppContext::try_get() {
        let (x, y) = ctx.env_position_manager.allocate_position(&env_id).await;
        Some(format!("{},{}", x, y))
    } else {
        None
    };

    let accounts = match accounts {
        Some(accounts) => Some(decrypt_accounts(accounts)?),
        None => None,
    };

    let proxy = match proxy {
        Some(proxy) => Some(proxy.decrypt_password()?.to_browser_proxy_config()),
        None => None,
    };

    let extension_dirs = match extensions {
        Some(extensions) if !extensions.is_empty() => {
            let dirs = extension::install_extensions(
                &app,
                &env_id,
                &cache_path,
                &user_data_dir,
                extensions,
            )
            .await?;

            if dirs.is_empty() { None } else { Some(dirs) }
        }
        _ => None,
    };

    Ok(EnvironmentStartRequest {
        exe_path,
        env_uuid: env_id,
        user_data_dir: user_data_dir.to_string_lossy().to_string(),
        urls,
        proxy,
        fingerprint_config,
        accounts,
        display_id,
        window_position,
        window_size,
        extension_dirs,
    })
}

fn decrypt_accounts(accounts: Vec<AccountInfo>) -> Result<Vec<AccountConfig>> {
    let mut decrypted = Vec::with_capacity(accounts.len());
    for account in accounts {
        decrypted.push(AccountConfig {
            url: account.platform_url,
            username: account.account,
            password: account.password,
        });
    }

    Ok(decrypted)
}

fn map_batch_launch_result(
    result: crate::infrastructure::runtime::BatchLaunchResult,
) -> BatchLaunchResult {
    BatchLaunchResult {
        env_uuid: result.env_uuid,
        success: result.success,
        error: result.error,
    }
}
