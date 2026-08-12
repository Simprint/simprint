/// 窗口管理命令
///
/// 命令层仅负责参数解析和响应，业务逻辑由服务层处理
use crate::core::error::Result;
use crate::services::window::{WindowLayoutCell, WindowService};
use tauri::AppHandle;

// ============================================================================
// 窗口布局命令
// ============================================================================

#[tauri::command]
pub fn calculate_window_layout(window_count: usize) -> Vec<WindowLayoutCell> {
    WindowService::calculate_window_layout(window_count)
}

#[tauri::command]
pub async fn arrange_environments(env_ids: Vec<String>) -> Result<()> {
    WindowService::arrange_environments(env_ids).await
}

#[tauri::command]
pub fn get_window_size(app_handle: AppHandle) -> (f64, f64) {
    WindowService::get_window_size(&app_handle)
}

// ============================================================================
// 窗口显示控制命令
// ============================================================================

#[tauri::command]
pub fn hide_window(app_handle: AppHandle) -> Result<()> {
    WindowService::hide_window(&app_handle)
}

#[tauri::command]
pub fn show_window(app_handle: AppHandle) -> Result<()> {
    WindowService::show_window(&app_handle)
}

// ============================================================================
// 窗口创建命令
// ============================================================================

#[tauri::command]
pub async fn create_syncer_window(app_handle: AppHandle) -> Result<()> {
    WindowService::create_syncer_window(&app_handle).await
}

/// 创建主窗口（内部使用，不暴露为 Tauri 命令）
pub fn create_main_window(app_handle: AppHandle) -> Result<()> {
    WindowService::create_main_window(&app_handle)
}
