/// 环境和内核管理命令
///
/// 命令层仅负责参数解析和响应，业务逻辑由服务层处理
use crate::core::error::Result;
use crate::domain::environment::{EnvironmentStatus, KernelDetail};
use crate::infrastructure::runtime::FingerprintConfig;
use crate::services::connectivity::ProxyExportItem;
use crate::services::connectivity::ProxyExportService;
use crate::services::environment::{
    AccountInfo, BatchLaunchRequest, BatchLaunchResult, CdpEndpointResponse,
    EnvironmentLaunchRuntimeService, EnvironmentService, KernelService, ProxyConfig,
    RunningEnvironment, StartSyncParams,
};
use tauri::AppHandle;

// ============================================================================
// 内核管理命令
// ============================================================================

#[tauri::command]
pub async fn ensure_kernel_ready(
    app: AppHandle,
    env_uuid: Option<String>,
    kernel_value: String,
    profiles_path: String,
    kernel_detail: KernelDetail,
) -> Result<String> {
    KernelService::ensure_kernel_ready(
        app.clone(),
        env_uuid,
        kernel_value,
        profiles_path,
        kernel_detail,
        Some(crate::services::environment::kernel::utils::build_tauri_status_emitter(app)),
    )
    .await
}

#[tauri::command]
pub async fn launch_environment(
    app: AppHandle,
    exe_path: String,
    env_uuid: String,
    cache_path: String,
    urls: Option<Vec<String>>,
    proxy: Option<ProxyConfig>,
    fingerprint_config: Option<FingerprintConfig>,
    accounts: Option<Vec<AccountInfo>>,
    extensions: Option<Vec<crate::services::environment::ExtensionInfo>>,
) -> Result<()> {
    KernelService::launch_environment(
        app.clone(),
        exe_path,
        env_uuid,
        cache_path,
        urls,
        proxy,
        fingerprint_config,
        accounts,
        extensions,
        Some(crate::services::environment::kernel::utils::build_tauri_status_emitter(app)),
    )
    .await
}

#[tauri::command]
pub async fn get_connected_environments() -> Result<Vec<String>> {
    KernelService::get_connected_environments().await
}

#[tauri::command]
pub async fn get_environment_cdp_endpoint(env_uuid: String) -> Result<Option<CdpEndpointResponse>> {
    KernelService::get_cdp_endpoint(env_uuid).await
}

#[tauri::command]
pub async fn stop_environment(env_uuid: String) -> Result<()> {
    KernelService::stop_environment(env_uuid).await
}

#[tauri::command]
pub async fn refresh_environment_proxy(env_uuid: String, proxy: Option<ProxyConfig>) -> Result<()> {
    KernelService::refresh_proxy(env_uuid, proxy).await
}

#[tauri::command]
pub async fn batch_launch_environments(
    app: AppHandle,
    launch_requests: Vec<BatchLaunchRequest>,
) -> Result<Vec<BatchLaunchResult>> {
    KernelService::batch_launch_environments(
        app.clone(),
        launch_requests,
        Some(crate::services::environment::kernel::utils::build_tauri_status_emitter(app)),
    )
    .await
}

#[tauri::command]
pub async fn start_environment_by_uuid(app: AppHandle, env_uuid: String) -> Result<()> {
    let launch_paths = EnvironmentLaunchRuntimeService::resolve_launch_paths(&app)?;
    let status_emitter =
        Some(crate::services::environment::kernel::utils::build_tauri_status_emitter(app));
    EnvironmentLaunchRuntimeService::start_environment_by_uuid(
        env_uuid,
        launch_paths,
        status_emitter,
    )
    .await
}

#[tauri::command]
pub async fn batch_start_environments_by_uuid(
    app: AppHandle,
    env_uuids: Vec<String>,
) -> Result<Vec<BatchLaunchResult>> {
    let launch_paths = EnvironmentLaunchRuntimeService::resolve_launch_paths(&app)?;
    let status_emitter =
        Some(crate::services::environment::kernel::utils::build_tauri_status_emitter(app));
    EnvironmentLaunchRuntimeService::batch_start_environments_by_uuid(
        env_uuids,
        launch_paths,
        status_emitter,
    )
    .await
}

#[tauri::command]
pub async fn batch_stop_environments(env_uuids: Vec<String>) -> Result<Vec<BatchLaunchResult>> {
    KernelService::batch_stop_environments(env_uuids).await
}

#[tauri::command]
pub async fn get_environment_status(env_uuid: String) -> Result<Option<EnvironmentStatus>> {
    KernelService::get_environment_status(env_uuid).await
}

#[tauri::command]
pub async fn get_all_environment_statuses()
-> Result<std::collections::HashMap<String, EnvironmentStatus>> {
    KernelService::get_all_environment_statuses().await
}

// ============================================================================
// 环境同步命令
// ============================================================================

#[tauri::command]
pub async fn get_running_environments() -> Vec<RunningEnvironment> {
    EnvironmentService::get_running_environments().await
}

#[tauri::command]
pub async fn start_sync(params: StartSyncParams) -> Result<()> {
    EnvironmentService::start_sync(params).await
}

#[tauri::command]
pub async fn stop_sync() -> Result<()> {
    EnvironmentService::stop_sync().await
}

// ============================================================================
// 代理导出命令
// ============================================================================

#[tauri::command]
pub async fn export_proxies_to_csv(
    path: String,
    proxies: Vec<ProxyExportItem>,
    export_plain_password: bool,
) -> Result<()> {
    ProxyExportService::export_proxies_to_csv(path, proxies, export_plain_password).await
}
