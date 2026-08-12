/// 更新服务
///
/// 提供检查更新、下载更新和启动安装的业务逻辑
use crate::core::error::{Error, Result};
use crate::infrastructure::updater::service::LockResult;
use crate::infrastructure::updater::types::{
    CheckingPayload, DownloadCompletePayload, DownloadPartialPayload, ErrorPayload,
    FoundUpdatesPayload, InstallStrategy, LatestRelease, NoUpdatesPayload, UpdateEvent,
};
use crate::infrastructure::updater::{checker, downloader, planner, service, verifier};
use crate::services::updater::types::PreparedUpdateInfo;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;
use tauri::AppHandle;
use tokio::time::Instant;

/// 检查更新结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// 是否有可用更新
    pub has_updates: bool,
    /// 更新数量（如果有更新）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_count: Option<usize>,
    /// 是否已有可用的更新计划缓存
    pub plan_available: bool,
}

/// 下载更新结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResult {
    /// 是否有可用更新
    pub has_updates: bool,
    /// 更新任务文件路径（如果下载成功）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks_file: Option<String>,
    /// 安装策略
    pub install_strategy: InstallStrategy,
    /// 成功下载的文件数量
    pub success_count: usize,
}

pub struct UpdateService;

fn map_update_check_error(stage: &str, err: impl std::fmt::Display) -> Error {
    Error::UpdateCheckFailed(format!("{stage}: {err}"))
}

impl UpdateService {
    pub async fn get_prepared_update() -> Result<Option<PreparedUpdateInfo>> {
        Ok(None)
    }

    pub async fn start_prepared_update_install(
        app_handle: AppHandle,
        kind: Option<String>,
    ) -> Result<()> {
        let resolved_kind = match kind {
            Some(kind) => kind,
            None => Self::get_prepared_update()
                .await?
                .map(|update| update.kind)
                .ok_or("当前没有可安装的更新")?,
        };

        let _ = app_handle;
        Err(format!(
            "不支持独立更新组件: {}。环境运行时现已内嵌到 Simprint，请更新主程序。",
            resolved_kind
        )
        .into())
    }

    /// 简单检查是否有可用更新（仅检查，不缓存计划，不发送事件）
    ///
    /// 用于设置页面的「检查更新」按钮，返回布尔值。具体更新逻辑由重启时自动完成。
    pub async fn check_update_available() -> Result<bool> {
        service::init_updater_config()
            .map_err(|e| map_update_check_error("初始化更新配置失败", e))?;
        let check_response = checker::check_updates()
            .await
            .map_err(|e| map_update_check_error("请求更新检查接口失败", e))?;
        let tasks = planner::plan_updates(&check_response)
            .map_err(|e| map_update_check_error("生成更新计划失败", e))?;
        Ok(!tasks.is_empty())
    }

    /// 检查更新（仅检查，不下载）
    ///
    /// 检查是否有可用更新，并将更新计划缓存到内存
    ///
    /// # 参数
    /// - `app_handle`: Tauri 应用句柄，用于 emit 事件
    ///
    /// # 返回
    /// 返回检查结果，包含是否有更新、更新数量以及计划是否可用
    pub async fn check_updates(app_handle: AppHandle) -> Result<CheckResult> {
        // 尝试获取锁（非阻塞）
        let _guard = match service::acquire_update_lock().await {
            LockResult::Acquired(guard) => guard,
            LockResult::Busy => {
                log::debug!("更新检查正在进行中，跳过本次检查");
                return Ok(CheckResult {
                    has_updates: false,
                    update_count: None,
                    plan_available: false,
                });
            }
        };

        // 初始化配置
        service::init_updater_config()
            .map_err(|e| map_update_check_error("初始化更新配置失败", e))?;

        // 发送开始检查事件
        service::emit_event(
            &app_handle,
            UpdateEvent::Checking {
                payload: CheckingPayload {},
            },
        );

        // 1. 检查更新
        let check_response = checker::check_updates().await.map_err(|e| {
            service::emit_error_event(&app_handle, 1, format!("检查更新失败: {}", e));
            map_update_check_error("请求更新检查接口失败", e)
        })?;

        // 2. 生成更新计划
        let tasks = planner::plan_updates(&check_response).map_err(|e| {
            let error_event = UpdateEvent::PlanFailed {
                code: 2,
                payload: ErrorPayload {
                    error_message: format!("生成更新计划失败: {}", e),
                },
            };
            service::emit_event(&app_handle, error_event);
            map_update_check_error("生成更新计划失败", e)
        })?;

        if tasks.is_empty() {
            log::info!("无需更新");
            service::emit_event(
                &app_handle,
                UpdateEvent::NoUpdates {
                    payload: NoUpdatesPayload {},
                },
            );
            return Ok(CheckResult {
                has_updates: false,
                update_count: None,
                plan_available: false,
            });
        }

        log::info!("发现 {} 个更新任务", tasks.len());

        // 发送发现更新事件
        service::emit_event(
            &app_handle,
            UpdateEvent::FoundUpdates {
                payload: FoundUpdatesPayload {
                    update_count: tasks.len(),
                },
            },
        );

        // 3. 缓存更新计划（供下载使用）
        service::store_update_plan(&tasks).await?;

        Ok(CheckResult {
            has_updates: true,
            update_count: Some(tasks.len()),
            plan_available: true,
        })
    }

    /// 下载更新（执行下载，支持进度）
    ///
    /// 从内存中的更新计划读取任务，执行下载和校验，实时发送进度事件
    ///
    /// # 参数
    /// - `app_handle`: Tauri 应用句柄，用于 emit 事件
    /// # 返回
    /// 返回下载结果，包含任务文件路径和成功数量
    pub async fn download_updates(
        app_handle: AppHandle,
        _plan_file: Option<String>,
    ) -> Result<DownloadResult> {
        // 尝试获取锁（非阻塞）
        let _guard = match service::acquire_update_lock().await {
            LockResult::Acquired(guard) => guard,
            LockResult::Busy => {
                log::debug!("更新下载正在进行中，跳过本次下载");
                return Err("更新下载正在进行中".into());
            }
        };

        service::clear_installer_package_path().await;

        // 初始化配置
        service::init_updater_config()?;

        // 读取更新计划文件
        let update_plan = service::take_update_plan().await?;

        if update_plan.tasks.is_empty() {
            return Err("更新计划为空".into());
        }

        // 转换为 UpdateTask
        let tasks = service::plan_tasks_to_update_tasks(&update_plan.tasks);

        if should_use_installer_package(&tasks) {
            return Self::download_installer_package(app_handle).await;
        }

        // 计算总大小
        let total_size: u64 = tasks.iter().map(|t| t.artifact.file_size).sum();
        let downloaded_size = Arc::new(AtomicU64::new(0));

        // 发送开始下载事件
        service::emit_event(
            &app_handle,
            UpdateEvent::Downloading {
                payload: crate::infrastructure::updater::types::DownloadingPayload {
                    update_count: tasks.len(),
                },
            },
        );

        // 创建 HTTP 客户端
        let client = Client::builder().timeout(Duration::from_secs(300)).build()?;

        let mut success_count = 0;
        let mut failed_count = 0;
        let mut install_tasks = Vec::new();

        // 遍历所有任务：下载和校验
        let download_start = Instant::now();

        for task in &tasks {
            let resource_name = task.artifact.resource_name.clone();
            let downloaded_size_clone = downloaded_size.clone();
            let app_handle_clone = app_handle.clone();
            let total_size_clone = total_size;

            // 1. 下载阶段（带进度回调）
            // 记录该文件开始下载时的已下载总量
            let file_start_downloaded =
                downloaded_size_clone.load(std::sync::atomic::Ordering::Relaxed);

            let download_result =
                downloader::download_file_with_progress(task, &client, move |file_downloaded| {
                    // file_downloaded 是该文件的累计下载大小
                    // 计算当前总下载大小 = 文件开始时的总量 + 该文件当前已下载大小
                    let total_downloaded = file_start_downloaded + file_downloaded;

                    // 使用 compare_and_swap 确保只更新更大的值（避免并发问题）
                    let current = downloaded_size_clone.load(std::sync::atomic::Ordering::Relaxed);
                    if total_downloaded > current {
                        downloaded_size_clone
                            .store(total_downloaded, std::sync::atomic::Ordering::Relaxed);
                    }

                    // 重新加载确保使用最新的值
                    let final_downloaded =
                        downloaded_size_clone.load(std::sync::atomic::Ordering::Relaxed);

                    // 计算进度百分比
                    let percentage = if total_size_clone > 0 {
                        (final_downloaded as f64 / total_size_clone as f64) * 100.0
                    } else {
                        0.0
                    };

                    // 发送进度事件
                    let progress_event = UpdateEvent::DownloadProgress {
                        payload: crate::infrastructure::updater::types::DownloadProgressPayload {
                            downloaded: final_downloaded,
                            total: total_size_clone,
                            percentage,
                        },
                    };
                    service::emit_event(&app_handle_clone, progress_event);

                    Ok(())
                })
                .await;

            match download_result {
                Ok(_) => {
                    // 下载成功，静默处理
                }
                Err(e) => {
                    log::error!("下载失败: {} - {}", resource_name, e);
                    failed_count += 1;
                    // 清理临时文件
                    let _ = fs::remove_file(&task.temp_path);
                    continue;
                }
            }

            // 2. 校验阶段
            match verifier::verify_file_hash(task) {
                Ok(_) => {
                    // 转换为安装任务
                    install_tasks.push(service::update_task_to_install_task(task));
                    success_count += 1;
                }
                Err(e) => {
                    log::error!("校验失败: {} - {}", resource_name, e);
                    failed_count += 1;
                    // 清理临时文件
                    let _ = fs::remove_file(&task.temp_path);
                    continue;
                }
            }
        }

        let download_duration = download_start.elapsed();

        // 3. 保存安装任务到文件
        let tasks_file_path = if !install_tasks.is_empty() {
            let file_path = service::save_install_tasks(&install_tasks)?;
            Some(file_path.to_string_lossy().to_string())
        } else {
            None
        };

        // 4. 发送结果事件
        if failed_count == 0 {
            log::info!(
                "下载完成，共 {} 个文件，总耗时: {:.2} 秒",
                success_count,
                download_duration.as_secs_f64()
            );

            if let Some(ref tasks_file) = tasks_file_path {
                service::emit_event(
                    &app_handle,
                    UpdateEvent::DownloadComplete {
                        payload: DownloadCompletePayload {
                            tasks_file: tasks_file.clone(),
                            success_count,
                        },
                    },
                );
            }

            Ok(DownloadResult {
                has_updates: true,
                tasks_file: tasks_file_path,
                install_strategy: InstallStrategy::DirectReplace,
                success_count,
            })
        } else if success_count > 0 {
            log::warn!(
                "部分下载完成：成功 {}, 失败 {}，总耗时: {:.2} 秒",
                success_count,
                failed_count,
                download_duration.as_secs_f64()
            );

            // 部分成功，也通知前端
            if let Some(ref tasks_file) = tasks_file_path {
                service::emit_event(
                    &app_handle,
                    UpdateEvent::DownloadPartial {
                        payload: DownloadPartialPayload {
                            tasks_file: tasks_file.clone(),
                            success_count,
                            failed_count,
                            error_message: format!(
                                "部分下载完成：成功 {}, 失败 {}",
                                success_count, failed_count
                            ),
                        },
                    },
                );
            }

            Ok(DownloadResult {
                has_updates: true,
                tasks_file: tasks_file_path,
                install_strategy: InstallStrategy::DirectReplace,
                success_count,
            })
        } else {
            log::error!(
                "所有下载失败，总耗时: {:.2} 秒",
                download_duration.as_secs_f64()
            );

            service::emit_error_event(&app_handle, 3, format!("{} 个下载全部失败", failed_count));

            Err(format!("所有下载失败: {} 个任务", failed_count).into())
        }
    }

    /// 启动更新安装并退出主程序
    ///
    /// # 参数
    /// - `app_handle`: Tauri 应用句柄
    ///
    /// # 说明
    /// 启动 updater.exe install 命令，然后退出主程序
    /// 任务文件路径由统一路径层提供（update_tasks.json）
    pub async fn start_update_install(_app_handle: AppHandle) -> Result<()> {
        if let Some(installer_path) = service::take_installer_package_path().await {
            return launch_installer_update(PathBuf::from(installer_path)).await;
        }

        let tasks_file_path = service::get_tasks_file_path()?;
        Self::start_update_install_with_tasks_file(tasks_file_path).await
    }

    pub async fn start_update_install_with_tasks_file(tasks_file_path: PathBuf) -> Result<()> {
        // 1. 检查任务文件是否存在
        if !tasks_file_path.exists() {
            return Err(format!("任务文件不存在: {}", tasks_file_path.display()).into());
        }

        // 2. 获取 updater.exe 所在目录
        let exe_dir = service::get_exe_directory()?;

        // 3. 构建 updater.exe 路径
        let updater_exe = exe_dir.join("updater.exe");

        if !updater_exe.exists() {
            log::error!("找不到 updater.exe: {}", updater_exe.display());
            return Err("UPDATER_NOT_FOUND".into());
        }

        // 4. 启动 updater.exe install <tasks_file>
        #[cfg(target_os = "windows")]
        {
            use std::process::Stdio;
            use tokio::process::Command;
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            Command::new(&updater_exe)
                .arg("install")
                .arg(tasks_file_path.to_string_lossy().to_string())
                .creation_flags(CREATE_NO_WINDOW)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| {
                    log::error!("启动 updater.exe 失败: {}", e);
                    if e.to_string().contains("740") {
                        "UPDATER_NO_PERMISSION"
                    } else {
                        "UPDATER_START_FAILED"
                    }
                })?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            use std::process::Stdio;
            use tokio::process::Command;

            Command::new(&updater_exe)
                .arg("install")
                .arg(tasks_file_path.to_string_lossy().to_string())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
        }

        // 5. 延迟后退出主程序（给 updater 一点时间启动）
        tokio::spawn(async {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            std::process::exit(0);
        });

        Ok(())
    }

    async fn download_installer_package(app_handle: AppHandle) -> Result<DownloadResult> {
        service::emit_event(
            &app_handle,
            UpdateEvent::Downloading {
                payload: crate::infrastructure::updater::types::DownloadingPayload {
                    update_count: 1,
                },
            },
        );

        let latest_release = fetch_latest_release().await?;
        let platform = latest_release
            .platforms
            .get("x86_64-pc-windows-msvc")
            .ok_or("latest.json 中缺少 windows 平台信息")?;
        let installer_url = platform.r2_url.trim();

        if installer_url.is_empty() {
            return Err("latest.json 中缺少 windows.r2_url".into());
        }

        let installer_path = download_installer_file(&app_handle, installer_url).await?;
        let installer_path_string = installer_path.to_string_lossy().to_string();

        service::store_installer_package_path(installer_path_string.clone()).await;

        service::emit_event(
            &app_handle,
            UpdateEvent::DownloadComplete {
                payload: DownloadCompletePayload {
                    tasks_file: installer_path_string.clone(),
                    success_count: 1,
                },
            },
        );

        Ok(DownloadResult {
            has_updates: true,
            tasks_file: Some(installer_path_string),
            install_strategy: InstallStrategy::InstallerPackage,
            success_count: 1,
        })
    }
}

fn should_use_installer_package(
    tasks: &[crate::infrastructure::updater::types::UpdateTask],
) -> bool {
    tasks.iter().any(|task| !is_target_parent_writable(&task.target_path))
}

fn is_target_parent_writable(target_path: &Path) -> bool {
    let Some(parent) = target_path.parent() else {
        return false;
    };

    if !parent.exists() {
        return false;
    }

    let probe_name = format!(".simprint-write-test-{}", uuid::Uuid::new_v4());
    let probe_path = parent.join(probe_name);

    match fs::write(&probe_path, b"probe") {
        Ok(_) => {
            let _ = fs::remove_file(&probe_path);
            true
        }
        Err(_) => false,
    }
}

async fn fetch_latest_release() -> Result<LatestRelease> {
    let ctx = crate::app::context::AppContext::get();
    let response = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?
        .get(&ctx.config.updater.latest_json_url)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("latest.json 请求失败: {}", status).into());
    }

    Ok(response.json::<LatestRelease>().await?)
}

async fn download_installer_file(app_handle: &AppHandle, installer_url: &str) -> Result<PathBuf> {
    let client = Client::builder().timeout(Duration::from_secs(300)).build()?;

    let response = client.get(installer_url).send().await?;

    if !response.status().is_success() {
        return Err(format!("下载安装包失败: {}", response.status()).into());
    }

    let total = response.content_length().unwrap_or(0);
    let file_name = installer_file_name(installer_url);
    let installer_dir = crate::core::paths::PathManager::get_updater_dir()?.join("installer");
    fs::create_dir_all(&installer_dir)?;
    let installer_path = installer_dir.join(file_name);

    let mut file = std::fs::File::create(&installer_path)?;
    let mut stream = response.bytes_stream();
    let mut downloaded = 0u64;

    use futures::StreamExt;
    use std::io::Write;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;

        let percentage = if total > 0 {
            (downloaded as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        service::emit_event(
            app_handle,
            UpdateEvent::DownloadProgress {
                payload: crate::infrastructure::updater::types::DownloadProgressPayload {
                    downloaded,
                    total,
                    percentage,
                },
            },
        );
    }

    file.sync_all()?;
    Ok(installer_path)
}

fn installer_file_name(installer_url: &str) -> String {
    reqwest::Url::parse(installer_url)
        .ok()
        .and_then(|url| {
            url.path_segments().and_then(|segments| segments.last().map(|v| v.to_string()))
        })
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| "simprint_setup.exe".to_string())
}

async fn launch_installer_update(installer_path: PathBuf) -> Result<()> {
    if !installer_path.exists() {
        return Err(format!("安装包不存在: {}", installer_path.display()).into());
    }

    #[cfg(target_os = "windows")]
    {
        let exe_dir = service::get_exe_directory()?;
        let updater_exe = exe_dir.join("updater.exe");

        if !updater_exe.exists() {
            return Err(format!("找不到 updater.exe: {}", updater_exe.display()).into());
        }

        launch_elevated_updater_for_installer(&updater_exe, &installer_path)?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = installer_path;
        return Err("当前平台不支持安装包升级分支".into());
    }

    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        std::process::exit(0);
    });

    Ok(())
}

#[cfg(target_os = "windows")]
fn launch_elevated_updater_for_installer(updater_exe: &Path, installer_path: &Path) -> Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWDEFAULT;
    use windows::core::PCWSTR;

    fn to_wide(value: &OsStr) -> Vec<u16> {
        value.encode_wide().chain(std::iter::once(0)).collect()
    }

    let verb = to_wide(OsStr::new("runas"));
    let file = to_wide(updater_exe.as_os_str());
    let params = to_wide(OsStr::new(&format!(
        "install-package \"{}\"",
        installer_path.display()
    )));

    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(verb.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR(params.as_ptr()),
            PCWSTR::null(),
            SW_SHOWDEFAULT,
        )
    };

    let code = result.0 as isize;
    if code <= 32 {
        return Err(format!("启动安装包失败: ShellExecuteW 返回 {}", code).into());
    }

    Ok(())
}
