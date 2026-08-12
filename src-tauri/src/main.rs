// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::error;

use simprint_lib::core::logger;

/// 屏蔽环境变量
pub fn disable_env_var() {
    let _ = unsafe { std::env::remove_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS") };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 在程序最开始就尝试初始化日志
    // 即使程序崩溃，也尽可能记录信息
    let _ = std::panic::set_hook(Box::new(|panic_info| {
        logger::log_app_panic(panic_info);
    }));

    disable_env_var();

    // 初始化日志系统（兜底目录，Tauri 启动后会在 setup 中按 store 设置重新初始化）
    logger::init_logging(logger::bootstrap_log_dir());

    // 记录应用启动
    logger::log_app_start();

    // 初始化应用核心组件
    if let Err(e) = simprint_lib::app::lifecycle::init_early() {
        error!("Failed to initialize app context: {}", e);
        std::process::exit(-1);
    }

    // 初始化应用状态
    {
        simprint_lib::app::init_state::init_app_init_state().await;
    }

    // 使用 catch_unwind 来捕获可能的 panic
    let result = std::panic::catch_unwind(|| {
        simprint_lib::run();
    });

    if let Err(panic_payload) = result {
        error!("{:?}", panic_payload);
        return Err("应用异常退出".into());
    }

    Ok(())
}
