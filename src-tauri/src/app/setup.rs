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

            if let Some(main_window) = app.get_webview_window("main") {
                // 如果程序启动期间得到深度链接参数，则将事件传递并传递打开的链接给前端。
                if let Some(arg_1) = argv.get(1) {
                    if arg_1.contains("://") {
                        // 收到 deep link 时，先在本地持久化 referral_code（若有）
                        crate::infrastructure::deeplink::process_arg(arg_1);
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

    app_handle.plugin(tauri_plugin_upload::init()).unwrap();

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

/// 后台初始化服务器公钥
pub fn init_server_public_key_background(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        match crate::infrastructure::persistence::credential::init_server_public_key().await {
            Ok(_) => {
                log::info!("Server connection successful");
                // 通知前端：服务器连接成功
                let _ = app_handle.emit("server-connected", ());
            }
            Err(e) => {
                log::warn!("Server connection failed: {}", e);
                // 通知前端：服务器连接失败
                let _ = app_handle.emit("server-connection-failed", e);
            }
        }
    });
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
