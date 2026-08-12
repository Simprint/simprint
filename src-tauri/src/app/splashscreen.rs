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

/// 初始化应用启动流程（显示 splashscreen 并控制加载）
pub fn init_startup(app_handle: AppHandle) {
    // 注意：splashscreen 窗口由前端控制显示，确保内容准备好后再显示
    // 窗口在 tauri.conf.json 中配置为 visible: false，前端会在内容准备好后调用 show()

    // 异步执行加载流程
    tauri::async_runtime::spawn(async move {
        let app_handle_clone = app_handle.clone();

        // 等待前端完成首帧渲染并明确通知后端，再开始本地启动流程。
        while !is_frontend_ready() {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // 后端初始化在 Tauri setup 阶段已经完成。这里只同步真实状态，
        // 不再用固定延迟模拟工作进度。
        emit_progress(&app_handle_clone, 10, "正在初始化...", Some("init"));
        emit_status_complete(&app_handle_clone, "init");

        // 步骤2: 加载配置
        emit_progress(&app_handle_clone, 30, "加载配置中...", Some("config"));
        emit_status_complete(&app_handle_clone, "config");

        // 步骤3: 初始化安全上下文
        emit_progress(
            &app_handle_clone,
            50,
            "初始化安全上下文...",
            Some("security"),
        );
        emit_status_complete(&app_handle_clone, "security");

        // 步骤4: 本地业务数据库已经在 Tauri setup 阶段完成初始化。
        // 启动流程不再连接远程业务服务器，也不再依赖远程公钥。
        emit_progress(&app_handle_clone, 70, "加载本地数据...", Some("local-data"));
        emit_status_complete(&app_handle_clone, "local-data");
        emit_progress(&app_handle_clone, 90, "本地数据已就绪", None);

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

        // 后端流程完成后再创建隐藏主窗口，避免开发模式下两个 WebView
        // 同时冷启动并争用 Vite 转换资源。主窗口仍需通过前端真实就绪门闩。
        if let Err(error) = crate::commands::window::create_main_window(app_handle_clone.clone()) {
            log::error!("Failed to create main window: {error}");
            emit_progress(
                &app_handle_clone,
                100,
                "主窗口创建失败",
                Some("main-window"),
            );
            return;
        }
        log::info!("Hidden main window created after backend startup flow");

        emit_progress(
            &app_handle_clone,
            100,
            "正在准备主窗口...",
            Some("main-window"),
        );

        if crate::app::startup::StartupService::backend_startup_ready(&app_handle_clone).is_err() {
            log::error!("Failed to complete the backend startup gate");
        }
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
