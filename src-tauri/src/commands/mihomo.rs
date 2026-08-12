use crate::{
    app::context::AppContext,
    infrastructure::mihomo::{
        ApplyMihomoNodeSelectionRequest, MihomoConnectionConfig, MihomoConnectionInfo,
        MihomoLocalProxy, MihomoNodeSelectionSnapshot, MihomoOverview, MihomoProxyDelayResult,
        MihomoStatus, UpdateMihomoLocalProxyRequest,
    },
};

#[tauri::command]
pub async fn test_and_attach_mihomo(
    app: tauri::AppHandle,
    config: MihomoConnectionConfig,
) -> Result<MihomoStatus, String> {
    AppContext::get().mihomo_manager.attach(&app, config).await
}

#[tauri::command]
pub async fn get_mihomo_status(app: tauri::AppHandle) -> Result<MihomoStatus, String> {
    Ok(AppContext::get().mihomo_manager.status(&app).await)
}

#[tauri::command]
pub async fn get_mihomo_connection_info(
    app: tauri::AppHandle,
) -> Result<MihomoConnectionInfo, String> {
    Ok(AppContext::get().mihomo_manager.connection_info(&app).await)
}

#[tauri::command]
pub async fn get_mihomo_overview(app: tauri::AppHandle) -> Result<MihomoOverview, String> {
    AppContext::get().mihomo_manager.overview(&app).await
}

#[tauri::command]
pub async fn test_mihomo_proxy_delay(
    app: tauri::AppHandle,
    proxy_name: String,
) -> Result<MihomoProxyDelayResult, String> {
    AppContext::get().mihomo_manager.test_proxy_delay(&app, proxy_name).await
}

#[tauri::command]
pub async fn test_mihomo_group_delays(
    app: tauri::AppHandle,
    group_name: String,
) -> Result<Vec<MihomoProxyDelayResult>, String> {
    AppContext::get().mihomo_manager.test_group_delays(&app, group_name).await
}

#[tauri::command]
pub async fn get_mihomo_node_selection(
    app: tauri::AppHandle,
) -> Result<MihomoNodeSelectionSnapshot, String> {
    AppContext::get().mihomo_manager.get_node_selection(&app).await
}

#[tauri::command]
pub async fn apply_mihomo_node_selection(
    app: tauri::AppHandle,
    request: ApplyMihomoNodeSelectionRequest,
) -> Result<Vec<MihomoLocalProxy>, String> {
    AppContext::get().mihomo_manager.apply_node_selection(&app, request).await
}

#[tauri::command]
pub async fn get_local_mihomo_proxies(
    app: tauri::AppHandle,
) -> Result<Vec<MihomoLocalProxy>, String> {
    AppContext::get().mihomo_manager.get_local_proxies(&app).await
}

#[tauri::command]
pub async fn ensure_mihomo_local_proxy_listeners(app: tauri::AppHandle) -> Result<bool, String> {
    AppContext::get().mihomo_manager.ensure_local_proxy_listeners(&app).await
}

#[tauri::command]
pub async fn update_local_mihomo_proxy(
    app: tauri::AppHandle,
    request: UpdateMihomoLocalProxyRequest,
) -> Result<MihomoLocalProxy, String> {
    AppContext::get().mihomo_manager.update_local_proxy(&app, request).await
}
