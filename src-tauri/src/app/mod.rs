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
pub mod startup;

use crate::commands;
use components::tray;
use tauri::Manager;

fn initialize_business_context(
    database_config: business::utils::DatabaseConfig,
    user_kernel_catalog: std::path::PathBuf,
) -> anyhow::Result<business::svc_ctx::SvcCtx> {
    // Tauri invokes `setup` from its async runtime. Blocking that same thread with
    // `tauri::async_runtime::block_on` would try to enter Tokio recursively and panic.
    // Keep setup synchronous, but perform the one-time async database bootstrap from
    // a plain OS thread using Tauri's runtime handle.
    std::thread::Builder::new()
        .name("business-database-init".to_string())
        .spawn(move || {
            tauri::async_runtime::block_on(async move {
                let context = business::svc_ctx::SvcCtx::new(&database_config).await?;
                let imported = business::services::browser_kernels::import_catalog_file(
                    &context.db,
                    &user_kernel_catalog,
                )
                .await
                .map_err(anyhow::Error::msg)?;
                let migrated =
                    business::services::browser_kernels::migrate_legacy_environment_bindings(
                        &context.db,
                    )
                    .await
                    .map_err(anyhow::Error::msg)?;
                if imported > 0 || migrated > 0 {
                    log::info!(
                        "Imported {imported} user browser kernel records and migrated {migrated} environment bindings"
                    );
                }
                Ok(context)
            })
        })?
        .join()
        .map_err(|_| anyhow::anyhow!("Local business database initialization thread panicked"))?
}

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
            let user_kernel_catalog =
                crate::core::paths::PathManager::get_config_dir()?.join("browser-kernels.json");
            let business_context =
                initialize_business_context(database_config, user_kernel_catalog.clone())?;
            app.manage(business_context);
            log::info!(
                "Local business database initialized: {}",
                database_file.display()
            );
            log::info!(
                "Optional user browser kernel catalog: {}",
                user_kernel_catalog.display()
            );

            setup::register_deep_link(app.handle().clone())?;

            crate::commands::window::create_main_window(app.handle().clone())?;
            log::info!("Hidden main window created for single-window startup");

            tray::menu(app)?;

            // 初始化依赖 Tauri 的组件
            lifecycle::init_tauri_dependent(app.handle())?;

            setup::init_simprint_runtime_background(app.handle().clone());

            // 初始化会话自动锁定后台任务
            session_lock::init_session_lock_background(
                app.handle().clone(),
                session_lock_manager.clone(),
            );

            if startup::StartupService::backend_startup_ready(app.handle()).is_err() {
                return Err(anyhow::anyhow!("Failed to complete the backend startup gate").into());
            }

            Ok(())
        })
        .invoke_handler(commands::register_handles())
        .on_window_event(events::window_event_handle)
        .build(ctx)
        .expect("error while building application");

    app.run(events::run_event_handle);
}
