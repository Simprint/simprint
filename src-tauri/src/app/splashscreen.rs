use crate::commands::updater;
use std::ffi::OsStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};

const SKIP_UPDATE_ARG: &str = "--skip-update";

fn has_skip_update_arg<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    args.into_iter().any(|arg| arg.as_ref() == OsStr::new(SKIP_UPDATE_ARG))
}

fn should_skip_update() -> bool {
    has_skip_update_arg(std::env::args_os())
}

// 前端就绪标志
static SPLASHSCREEN_FRONTEND_READY: std::sync::OnceLock<Arc<AtomicBool>> =
    std::sync::OnceLock::new();

/// 获取前端就绪标志
fn get_frontend_ready_flag() -> Arc<AtomicBool> {
    SPLASHSCREEN_FRONTEND_READY
        .get_or_init(|| Arc::new(AtomicBool::new(false)))
        .clone()
}

/// 检查前端是否已就绪
fn is_frontend_ready() -> bool {
    get_frontend_ready_flag().load(Ordering::Acquire)
}

/// 设置前端已就绪
pub fn set_frontend_ready() {
    get_frontend_ready_flag().store(true, Ordering::Release);
}

/// 发射加载进度事件到 splashscreen
fn emit_progress(app_handle: &AppHandle, progress: u8, text: &str, status: Option<&str>) {
    if let Some(splash_window) = app_handle.get_webview_window("splashscreen") {
        #[derive(serde::Serialize, Clone)]
        struct ProgressPayload {
            progress: u8,
            text: String,
            status: Option<String>,
        }

        let _ = splash_window.emit(
            "splashscreen-progress",
            ProgressPayload {
                progress,
                text: text.to_string(),
                status: status.map(|s| s.to_string()),
            },
        );
    }
}

/// 发射状态完成事件
fn emit_status_complete(app_handle: &AppHandle, status: &str) {
    if let Some(splash_window) = app_handle.get_webview_window("splashscreen") {
        #[derive(serde::Serialize, Clone)]
        struct StatusCompletePayload {
            status: String,
        }

        let _ = splash_window.emit(
            "splashscreen-status-complete",
            StatusCompletePayload {
                status: status.to_string(),
            },
        );
    }
}

/// 发射加载完成事件
fn emit_ready(app_handle: &AppHandle) {
    // 向 splashscreen 窗口发送加载完成事件
    if let Some(splash_window) = app_handle.get_webview_window("splashscreen") {
        let _ = splash_window.emit("splashscreen-ready", ());
    }
    // 向主窗口发送加载完成事件，通知主窗口加载逻辑已完成
    if let Some(main_window) = app_handle.get_webview_window("main") {
        let _ = main_window.emit("splashscreen-loading-complete", ());
    }
}

/// 发射连接失败事件（公钥初始化失败时调用，停止后续流程）
fn emit_connection_failed(app_handle: &AppHandle) {
    if let Some(splash_window) = app_handle.get_webview_window("splashscreen") {
        let _ = splash_window.emit("splashscreen-connection-failed", ());
    }
}

/// 初始化应用启动流程（显示 splashscreen 并控制加载）
pub fn init_startup(app_handle: AppHandle) {
    // 注意：splashscreen 窗口由前端控制显示，确保内容准备好后再显示
    // 窗口在 tauri.conf.json 中配置为 visible: false，前端会在内容准备好后调用 show()

    // 异步执行加载流程
    tauri::async_runtime::spawn(async move {
        let app_handle_clone = app_handle.clone();

        // 等待前端通知准备就绪（无限期等待，直到前端调用 splashscreen_ready）
        while !is_frontend_ready() {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // 步骤1: 初始化应用
        emit_progress(&app_handle_clone, 10, "正在初始化...", Some("init"));
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        emit_status_complete(&app_handle_clone, "init");

        // 步骤2: 加载配置
        emit_progress(&app_handle_clone, 30, "加载配置中...", Some("config"));
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
        emit_status_complete(&app_handle_clone, "config");

        // 步骤3: 初始化安全上下文
        emit_progress(
            &app_handle_clone,
            50,
            "初始化安全上下文...",
            Some("security"),
        );
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        emit_status_complete(&app_handle_clone, "security");

        // 步骤4: 连接服务器
        emit_progress(&app_handle_clone, 70, "连接服务器...", Some("server"));

        // 等待服务器连接（使用现有的后台初始化）
        // 这里我们等待一段时间，实际连接由 init_server_public_key_background 处理
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

        // 检查服务器连接状态
        if let Err(e) =
            crate::infrastructure::persistence::credential::init_server_public_key().await
        {
            log::warn!("Server connection failed: {}", e);
            emit_status_complete(&app_handle_clone, "server");
            emit_progress(&app_handle_clone, 90, "服务器连接失败", None);
            // 发送连接失败事件，停止后续流程
            emit_connection_failed(&app_handle_clone);
            // 不再继续后续步骤（不创建主窗口，不发送 ready 事件）
            return;
        }

        log::info!("Server connection successful");
        emit_status_complete(&app_handle_clone, "server");
        emit_progress(&app_handle_clone, 90, "服务器连接成功", None);

        // 步骤4.1: 检查并处理更新（自动检查、下载、安装）
        // 跳过参数只在更新入口消费，不传入更新服务、下载器或安装器。
        if should_skip_update() {
            log::info!("Skipping startup update flow because {SKIP_UPDATE_ARG} was provided");
            emit_progress(
                &app_handle_clone,
                92,
                "已跳过更新检查",
                Some("update-check"),
            );
            emit_status_complete(&app_handle_clone, "update-check");
        } else {
            emit_progress(&app_handle_clone, 92, "检查更新...", Some("update-check"));
            let updates_available = match updater::check_updates(app_handle_clone.clone()).await {
                Ok(result) => result.has_updates,
                Err(e) => {
                    log::error!("Update check failed: {}", e);
                    emit_progress(
                        &app_handle_clone,
                        92,
                        "检查更新失败，继续启动",
                        Some("update-check"),
                    );
                    false
                }
            };

            if updates_available {
                emit_progress(
                    &app_handle_clone,
                    94,
                    "检测到更新，开始下载...",
                    Some("update-download"),
                );
                match updater::download_updates(app_handle_clone.clone(), None).await {
                    Ok(download_result) => {
                        if download_result.success_count > 0 {
                            emit_progress(
                                &app_handle_clone,
                                96,
                                "下载完成，准备安装",
                                Some("update-install"),
                            );
                            // 触发安装并退出（updater.exe 负责后续重启）
                            if let Err(e) =
                                updater::start_update_install(app_handle_clone.clone()).await
                            {
                                log::error!("Update installation start failed: {}", e);
                                emit_progress(
                                    &app_handle_clone,
                                    96,
                                    "安装启动失败，继续当前版本",
                                    Some("update-install"),
                                );
                            }
                            // 无论安装启动是否成功，都不再继续创建主窗口，交由 updater.exe 或用户重启
                            return;
                        } else {
                            log::warn!("Update download failed, continuing with current version");
                            emit_progress(
                                &app_handle_clone,
                                94,
                                "下载失败，继续启动当前版本",
                                Some("update-download"),
                            );
                        }
                    }
                    Err(e) => {
                        log::error!("Update download error: {}", e);
                        emit_progress(
                            &app_handle_clone,
                            94,
                            "下载更新失败，继续启动当前版本",
                            Some("update-download"),
                        );
                    }
                }
            }
        }

        // 创建主窗口（在步骤4完成后）
        if let Err(e) = crate::commands::window::create_main_window(app_handle_clone.clone()).await
        {
            log::error!("Failed to create main window: {}", e);
            // 不阻止加载流程继续
        }

        // 步骤5: 准备就绪
        emit_progress(&app_handle_clone, 100, "准备就绪", Some("ready"));
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        emit_status_complete(&app_handle_clone, "ready");

        // 发射加载完成事件（前端会自动关闭 splashscreen 并显示主窗口）
        emit_ready(&app_handle_clone);
    });
}

#[cfg(test)]
mod tests {
    use super::has_skip_update_arg;

    #[test]
    fn detects_skip_update_argument() {
        assert!(has_skip_update_arg(["simprint.exe", "--skip-update"]));
    }

    #[test]
    fn does_not_treat_other_arguments_as_skip_update() {
        assert!(!has_skip_update_arg([
            "simprint.exe",
            "--skip-update-check",
            "simprint://open"
        ]));
    }
}
