//! 浏览器内核准备与启动服务
//!
//! 负责：检查/下载/解压内核、校验 chrome.dll 哈希、启动环境进程

use crate::core::error::Result;
use crate::domain::environment::{EnvironmentStatus, KernelDetail};
use std::fs;

pub mod downloader;
pub mod extension;
mod runtime_bridge;
mod state;
pub mod timezone;
pub mod types;
pub mod utils;
pub mod verifier;

// 重新导出常用类型
pub use types::{
    AccountInfo, BatchLaunchRequest, BatchLaunchResult, CdpEndpointResponse, ExtensionInfo,
    KernelPrepareStatusPayload, KernelStatusEmitter, ProxyConfig,
};

/// 内核服务
pub struct KernelService;

impl KernelService {
    /// 确保内核已就绪：目录不存在则下载并解压，存在则校验 chrome.dll 哈希
    pub async fn ensure_kernel_ready(
        app: tauri::AppHandle,
        env_uuid: Option<String>,
        kernel_value: String,
        profiles_path: String,
        kernel_detail: KernelDetail,
        status_emitter: Option<KernelStatusEmitter>,
    ) -> Result<String> {
        let kernel_value = kernel_value.trim().to_string();
        if kernel_value.is_empty() {
            return Err("内核版本不能为空".into());
        }

        let _prepare_guard = state::acquire_kernel_prepare_lock(&kernel_value).await;
        let base = utils::resolve_profiles_base(&app, &profiles_path)?;
        let kernel_dir = base.join(&kernel_value);
        let exe_path = kernel_dir.join(utils::exe_name());

        // 目录已存在：校验内核
        if kernel_dir.exists() && kernel_dir.is_dir() {
            // 检查可执行文件是否存在
            if !exe_path.exists() {
                crate::log_warn!(
                    crate::core::logger::modules::KERNEL,
                    "内核目录存在但未找到可执行文件，删除目录并重新下载: {}",
                    kernel_dir.display()
                );
                utils::emit_status(
                    status_emitter.as_ref(),
                    &env_uuid,
                    &kernel_value,
                    EnvironmentStatus::Error,
                    Some("未找到可执行文件，重新下载"),
                    None,
                    None,
                    None,
                );
                let _ = fs::remove_dir_all(&kernel_dir);
            } else {
                // 校验 signature
                let signature = kernel_detail
                    .signature
                    .as_deref()
                    .filter(|s| !s.trim().is_empty())
                    .ok_or("该内核版本缺少 signature，无法校验核心 DLL")?;

                match verifier::verify_kernel(
                    &app,
                    &env_uuid,
                    &kernel_value,
                    &kernel_dir,
                    signature,
                    status_emitter.clone(),
                ) {
                    Ok(true) => {
                        // 校验通过
                        utils::emit_status(
                            status_emitter.as_ref(),
                            &env_uuid,
                            &kernel_value,
                            EnvironmentStatus::Ready,
                            Some("就绪"),
                            None,
                            None,
                            None,
                        );
                        return Ok(exe_path.to_string_lossy().to_string());
                    }
                    Ok(false) => {
                        // 校验失败，删除目录重新下载
                        crate::log_warn!(
                            crate::core::logger::modules::KERNEL,
                            "内核校验失败，删除目录并重新下载: {}",
                            kernel_dir.display()
                        );
                        utils::emit_status(
                            status_emitter.as_ref(),
                            &env_uuid,
                            &kernel_value,
                            EnvironmentStatus::Error,
                            Some("校验失败，重新下载"),
                            None,
                            None,
                            None,
                        );
                        let _ = fs::remove_dir_all(&kernel_dir);
                    }
                    Err(e) => {
                        crate::log_warn!(
                            crate::core::logger::modules::KERNEL,
                            "内核校验出错，删除目录并重新下载: {} - {}",
                            kernel_dir.display(),
                            e
                        );
                        utils::emit_status(
                            status_emitter.as_ref(),
                            &env_uuid,
                            &kernel_value,
                            EnvironmentStatus::Error,
                            Some("校验出错，重新下载"),
                            None,
                            None,
                            None,
                        );
                        let _ = fs::remove_dir_all(&kernel_dir);
                    }
                }
            }
        }

        // 目录不存在或校验失败：下载并安装
        let exe_path = downloader::download_and_install_kernel(
            &app,
            &env_uuid,
            &kernel_value,
            &kernel_dir,
            &kernel_detail,
            status_emitter,
        )
        .await?;

        Ok(exe_path.to_string_lossy().to_string())
    }

    /// 启动环境：在可执行文件所在目录下启动进程
    pub async fn launch_environment(
        app: tauri::AppHandle,
        exe_path: String,
        env_uuid: String,
        cache_path: String,
        urls: Option<Vec<String>>,
        proxy: Option<ProxyConfig>,
        fingerprint_config: Option<crate::infrastructure::runtime::FingerprintConfig>,
        accounts: Option<Vec<types::AccountInfo>>,
        extensions: Option<Vec<types::ExtensionInfo>>,
        status_emitter: Option<KernelStatusEmitter>,
    ) -> Result<()> {
        runtime_bridge::launch_environment(
            app,
            exe_path,
            env_uuid,
            cache_path,
            urls,
            proxy,
            fingerprint_config,
            accounts,
            extensions,
            status_emitter,
        )
        .await
    }

    /// 获取当前已连接的环境 ID 列表
    pub async fn get_connected_environments() -> Result<Vec<String>> {
        runtime_bridge::get_connected_environments().await
    }

    pub async fn get_cdp_endpoint(env_uuid: String) -> Result<Option<CdpEndpointResponse>> {
        runtime_bridge::get_cdp_endpoint(env_uuid).await
    }

    /// 停止环境
    pub async fn stop_environment(env_uuid: String) -> Result<()> {
        runtime_bridge::stop_environment(env_uuid).await
    }

    pub async fn refresh_proxy(env_uuid: String, proxy: Option<ProxyConfig>) -> Result<()> {
        runtime_bridge::refresh_proxy(env_uuid, proxy).await
    }

    /// 批量启动环境：并发启动所有环境
    pub async fn batch_launch_environments(
        app: tauri::AppHandle,
        launch_requests: Vec<BatchLaunchRequest>,
        status_emitter: Option<KernelStatusEmitter>,
    ) -> Result<Vec<BatchLaunchResult>> {
        runtime_bridge::batch_launch_environments(app, launch_requests, status_emitter).await
    }

    /// 批量停止环境：并发停止所有环境
    pub async fn batch_stop_environments(env_uuids: Vec<String>) -> Result<Vec<BatchLaunchResult>> {
        runtime_bridge::batch_stop_environments(env_uuids).await
    }

    pub async fn get_environment_status(
        env_uuid: String,
    ) -> Result<Option<crate::domain::environment::EnvironmentStatus>> {
        runtime_bridge::get_environment_status(env_uuid).await
    }

    pub async fn get_all_environment_statuses()
    -> Result<std::collections::HashMap<String, crate::domain::environment::EnvironmentStatus>>
    {
        runtime_bridge::get_all_environment_statuses().await
    }

    pub async fn set_window_bounds(
        env_uuid: String,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<()> {
        runtime_bridge::set_window_bounds(env_uuid, x, y, width, height).await
    }
}
