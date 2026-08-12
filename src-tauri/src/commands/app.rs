/// 应用状态和生命周期命令
///
/// 命令层仅负责参数解析和响应，业务逻辑由服务层处理
use crate::core::error::Result;
use crate::services::app::{AppService, AutoStartState};
use tauri::AppHandle;

// ============================================================================
// 应用状态命令
// ============================================================================

#[tauri::command]
pub async fn set_updating_state() -> Result<()> {
    crate::app::startup::StartupService::set_updating_state()
        .await
        .map_err(|_| "设置更新状态失败".into())
}

#[tauri::command]
pub async fn get_app_state() -> Result<crate::app::init_state::AppInitState> {
    crate::app::startup::StartupService::get_app_state()
        .await
        .map_err(|_| "获取应用状态失败".into())
}

#[tauri::command]
pub async fn main_window_ready(app: AppHandle) -> Result<()> {
    crate::app::startup::StartupService::main_window_ready(app)
        .await
        .map_err(|_| "记录主窗口就绪状态失败".into())
}

#[tauri::command]
pub async fn show_main_window(app: AppHandle) -> Result<()> {
    crate::app::startup::StartupService::show_main_window(app)
        .await
        .map_err(|_| "显示主窗口失败".into())
}

#[tauri::command]
pub fn get_auto_start_state(app: AppHandle) -> Result<AutoStartState> {
    AppService::get_auto_start_state(&app)
}

#[tauri::command]
pub fn set_auto_start_enabled(app: AppHandle, enabled: bool) -> Result<AutoStartState> {
    AppService::set_auto_start_enabled(&app, enabled)
}

// ============================================================================
// 运行时信息命令
// ============================================================================

#[tauri::command]
pub fn is_dev() -> bool {
    crate::app::runtime_info::RuntimeInfo::is_dev()
}

#[tauri::command]
pub fn get_executable_path() -> Result<String> {
    crate::app::runtime_info::RuntimeInfo::get_executable_path()
}

// ============================================================================
// 进程控制命令
// ============================================================================

#[tauri::command]
pub async fn close_program() {
    std::process::exit(0);
}
