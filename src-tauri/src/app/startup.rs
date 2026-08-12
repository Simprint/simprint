use crate::app::init_state::AppInitState;
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager};

pub struct StartupService;

const GENERAL_SETTINGS_STORE_KEY: &str = "general";

/// 启动完成采用双就绪门闩：后端流程与主窗口前端必须都真实就绪。
static BACKEND_STARTUP_READY: AtomicBool = AtomicBool::new(false);
static MAIN_FRONTEND_READY: AtomicBool = AtomicBool::new(false);
static STARTUP_COMPLETED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeneralSettingsSnapshot {
    start_minimized: Option<bool>,
}

fn should_start_minimized(app: &AppHandle) -> bool {
    crate::infrastructure::persistence::tauri_store::get_store_key(app, GENERAL_SETTINGS_STORE_KEY)
        .and_then(|raw| serde_json::from_value::<GeneralSettingsSnapshot>(raw).ok())
        .and_then(|settings| settings.start_minimized)
        .unwrap_or(false)
}

fn try_complete_startup(app: &AppHandle) -> Result<(), ()> {
    let backend_ready = BACKEND_STARTUP_READY.load(Ordering::Acquire);
    let frontend_ready = MAIN_FRONTEND_READY.load(Ordering::Acquire);

    if !backend_ready || !frontend_ready {
        log::info!(
            "Startup gate waiting: backend_ready={backend_ready}, main_frontend_ready={frontend_ready}"
        );
        return Ok(());
    }

    if app.get_webview_window("main").is_none() {
        log::error!("Startup gates are ready, but the main window does not exist");
        return Err(());
    }

    if STARTUP_COMPLETED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Ok(());
    }

    log::info!("Startup gates satisfied; completing startup");

    let mut app_state = AppInitState::default();
    app_state.is_initialized = true;
    app_state.is_updating = false;
    let _ = crate::app::init_state::update_app_init_state(app_state);

    if let Some(main_window) = app.get_webview_window("main") {
        if should_start_minimized(app) {
            log::info!("Start minimized is enabled; keeping the ready main window hidden");
        } else {
            let _ = main_window.show();
            let _ = main_window.set_focus();
            log::info!("Main window shown after both startup gates became ready");
        }
    }

    Ok(())
}

impl StartupService {
    /// 设置应用为更新状态。
    pub async fn set_updating_state() -> Result<(), ()> {
        let mut app_state = AppInitState::default();
        app_state.is_updating = true;
        app_state.is_initialized = false;

        let _ = crate::app::init_state::update_app_init_state(app_state);

        Ok(())
    }

    /// 获取应用状态。
    pub async fn get_app_state() -> Result<AppInitState, ()> {
        Ok(crate::app::init_state::read_app_init_state())
    }

    /// 记录隐藏主窗口的前端已经完成插件、认证、路由、字体与首帧布局。
    pub async fn main_window_ready(app: AppHandle) -> Result<(), ()> {
        MAIN_FRONTEND_READY.store(true, Ordering::Release);
        log::info!("Main window frontend reported ready");
        try_complete_startup(&app)
    }

    /// 记录后端启动流程已经完成，并尝试与前端就绪状态汇合。
    pub fn backend_startup_ready(app: &AppHandle) -> Result<(), ()> {
        BACKEND_STARTUP_READY.store(true, Ordering::Release);
        log::info!("Backend startup flow reported ready");
        try_complete_startup(app)
    }

    /// 显示已经完成初始化的主窗口。
    pub async fn show_main_window(app: AppHandle) -> Result<(), ()> {
        let app_state = crate::app::init_state::read_app_init_state();
        if !app_state.is_initialized || app_state.is_updating {
            log::warn!("Ignored a request to show the main window before startup completed");
            return Ok(());
        }

        if let Some(main_window) = app.get_webview_window("main") {
            if should_start_minimized(&app) {
                log::info!("Start minimized is enabled; skipping main window display");
            } else {
                let _ = main_window.show();
                let _ = main_window.set_focus();
                log::info!("Main window shown");
            }
        }
        Ok(())
    }
}
