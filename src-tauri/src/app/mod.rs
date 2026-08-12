pub mod components;
pub mod context;
pub mod events;
pub mod handle;
pub mod init_state;
pub mod lifecycle;
pub mod runtime;
pub mod runtime_info;
pub mod session_lock;
pub mod setup;
pub mod splashscreen;
pub mod startup;

use crate::commands;
use components::tray;
use tauri::Manager;

pub fn run() {
    let ctx = tauri::generate_context!();
    let session_lock_manager = session_lock::SessionLockManager::new();

    let app = tauri::Builder::default()
        .manage(session_lock_manager.clone())
        .setup(move |app| {
            setup::register_plugins(app.handle());

            crate::infrastructure::persistence::tauri_store::ensure_store_loaded(app.handle())
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let log_dir =
                crate::infrastructure::persistence::tauri_store::get_logs_path(app.handle())
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
            crate::core::logger::init_logging(&log_dir);

            let database_file = crate::core::paths::PathManager::get_business_database_file()?;
            let database_config = business::utils::DatabaseConfig::from_path(&database_file);
            let business_context =
                tauri::async_runtime::block_on(business::svc_ctx::SvcCtx::new(&database_config))?;
            app.manage(business_context);
            log::info!(
                "Local business database initialized: {}",
                database_file.display()
            );

            setup::register_deep_link(app.handle().clone())?;

            crate::commands::window::create_splashscreen_window(app.handle().clone())?;

            tray::menu(app)?;

            // 初始化依赖 Tauri 的组件
            lifecycle::init_tauri_dependent(app.handle())?;

            setup::init_simprint_runtime_background(app.handle().clone());

            // 初始化会话自动锁定后台任务
            session_lock::init_session_lock_background(
                app.handle().clone(),
                session_lock_manager.clone(),
            );

            // 初始化应用启动流程（显示 splashscreen）
            splashscreen::init_startup(app.handle().clone());

            Ok(())
        })
        .invoke_handler(commands::register_handles())
        .on_window_event(events::window_event_handle)
        .build(ctx)
        .expect("error while building application");

    app.run(events::run_event_handle);
}
