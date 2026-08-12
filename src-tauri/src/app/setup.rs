use tauri::{AppHandle, Emitter};
#[cfg(desktop)]
use tauri_plugin_autostart::MacosLauncher;

/// 注册插件
pub fn register_plugins(app_handle: &AppHandle) {
    // Register the single instance plugin
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    #[cfg(feature = "production")]
    app_handle
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            use tauri::Manager;

            let app_state = crate::app::init_state::read_app_init_state();
            if app_state.is_initialized && !app_state.is_updating {
                let Some(main_window) = app.get_webview_window("main") else {
                    return;
                };

                // 如果程序启动期间得到深度链接参数，则将事件传递并传递打开的链接给前端。
                if let Some(arg_1) = argv.get(1) {
                    if arg_1.contains("://") {
                        main_window.emit("deep-link-open", arg_1).unwrap();
                    }

                    return;
                }

                let _ = main_window.show();
                let _ = main_window.set_focus();
            }
        }))
        .unwrap();

    // Register process plugin
    app_handle.plugin(tauri_plugin_process::init()).unwrap();

    // Register Tauri's signed updater. Release builds inject the public key
    // into tauri.conf.json before compilation.
    #[cfg(desktop)]
    app_handle.plugin(tauri_plugin_updater::Builder::new().build()).unwrap();

    // deep-link 插件
    app_handle.plugin(tauri_plugin_deep_link::init()).unwrap();

    // opener 插件
    app_handle.plugin(tauri_plugin_opener::init()).unwrap();

    // dialog 插件
    app_handle.plugin(tauri_plugin_dialog::init()).unwrap();

    // store 插件
    app_handle.plugin(tauri_plugin_store::Builder::new().build()).unwrap();

    // clipboard_manager
    app_handle.plugin(tauri_plugin_clipboard_manager::init()).unwrap();

    // 自动启动插件
    #[cfg(desktop)]
    app_handle
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]), /* 传递给应用程序的任意数量的参数 */
        ))
        .unwrap();
}

/// 注册深度链接
#[allow(dead_code)]
pub fn register_deep_link(app: AppHandle) -> Result<(), anyhow::Error> {
    #[cfg(any(windows, target_os = "linux"))]
    {
        use tauri_plugin_deep_link::DeepLinkExt;
        app.deep_link().register_all()?;
    };
    Ok(())
}

pub fn init_simprint_runtime_background(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        use crate::app::context::AppContext;

        let ctx = AppContext::get();
        ctx.simprint_runtime_manager.set_app_handle(app_handle.clone()).await;

        if let Err(error) = ctx.simprint_runtime_manager.start_background().await {
            log::warn!("failed to start embedded environment runtime: {}", error);
        }
    });
}
