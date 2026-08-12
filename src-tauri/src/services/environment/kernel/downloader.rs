//! 内核下载模块

use crate::core::error::Result;
use crate::core::utils::hash::calculate_file_hash;
use crate::domain::environment::{EnvironmentStatus, KernelDetail};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::types::{KernelStatusEmitter, SIGNATURE_HASH_SIZE};
use super::utils::{
    calculate_file_hash_head, core_dll_name, emit_status, exe_name, extract_zip_to_dir,
};
use super::verifier::find_version_subdir;

/// 下载并安装内核
///
/// # 流程
/// 1. 下载 zip 文件
/// 2. 校验文件哈希
/// 3. 解压到目标目录
/// 4. 校验核心 DLL 的 signature
/// 5. 清理临时文件
pub async fn download_and_install_kernel(
    app: &tauri::AppHandle,
    env_uuid: &Option<String>,
    kernel_value: &str,
    kernel_dir: &Path,
    kernel_detail: &KernelDetail,
    status_emitter: Option<KernelStatusEmitter>,
) -> Result<PathBuf> {
    if kernel_detail.url.is_empty() {
        return Err("未提供内核下载地址".into());
    }

    emit_status(
        status_emitter.as_ref(),
        env_uuid,
        kernel_value,
        EnvironmentStatus::Downloading,
        Some("下载中…"),
        Some(0.0),
        Some(0),
        None,
    );

    let temp_dir = crate::core::paths::PathManager::get_kernel_cache_dir(app)?;
    fs::create_dir_all(&temp_dir)?;

    let zip_name = format!("{}-{}.zip", kernel_value, Uuid::new_v4());
    let zip_path = temp_dir.join(&zip_name);

    // 下载文件
    download_file(
        app,
        env_uuid,
        kernel_value,
        &kernel_detail.url,
        &zip_path,
        status_emitter.clone(),
    )
    .await?;

    // 校验下载文件哈希
    verify_download_hash(
        env_uuid,
        kernel_value,
        &zip_path,
        &kernel_detail.hash,
        status_emitter.clone(),
    )?;

    // 解压或移动文件
    extract_or_move_kernel(
        env_uuid,
        kernel_value,
        &zip_path,
        kernel_dir,
        kernel_detail.requires_extract,
        status_emitter.clone(),
    )?;

    let exe_path = kernel_dir.join(exe_name());
    if !exe_path.exists() {
        emit_status(
            status_emitter.as_ref(),
            env_uuid,
            kernel_value,
            EnvironmentStatus::Error,
            Some("未找到可执行文件"),
            None,
            None,
            None,
        );
        return Err("解压后未找到可执行文件".into());
    }

    // 校验核心 DLL 的 signature
    if let Some(ref sig) = kernel_detail.signature {
        verify_core_dll_signature(
            env_uuid,
            kernel_value,
            kernel_dir,
            sig,
            status_emitter.clone(),
        )?;
    }

    emit_status(
        status_emitter.as_ref(),
        env_uuid,
        kernel_value,
        EnvironmentStatus::Ready,
        Some("就绪"),
        None,
        None,
        None,
    );

    Ok(exe_path)
}

/// 下载文件
async fn download_file(
    _app: &tauri::AppHandle,
    env_uuid: &Option<String>,
    kernel_value: &str,
    url: &str,
    zip_path: &Path,
    status_emitter: Option<KernelStatusEmitter>,
) -> Result<()> {
    let display_url = redact_download_url(url);
    crate::log_info!(
        crate::core::logger::modules::KERNEL,
        "开始下载内核: {}",
        display_url
    );
    let client = reqwest::Client::new();
    let res = client.get(url).send().await.map_err(|e| {
        crate::log_error!(
            crate::core::logger::modules::KERNEL,
            "内核下载请求失败: {} - {}",
            display_url,
            e
        );
        emit_status(
            status_emitter.as_ref(),
            env_uuid,
            kernel_value,
            EnvironmentStatus::Error,
            Some("下载请求失败"),
            None,
            None,
            None,
        );
        e
    })?;

    if !res.status().is_success() {
        crate::log_error!(
            crate::core::logger::modules::KERNEL,
            "内核下载失败: HTTP {} - {}",
            res.status(),
            display_url
        );
        emit_status(
            status_emitter.as_ref(),
            env_uuid,
            kernel_value,
            EnvironmentStatus::Error,
            Some("下载失败"),
            None,
            None,
            None,
        );
        return Err(format!("下载失败: HTTP {}", res.status()).into());
    }

    let total_len = res.content_length();
    let mut stream = res.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut file = fs::File::create(zip_path).map_err(|e| {
        emit_status(
            status_emitter.as_ref(),
            env_uuid,
            kernel_value,
            EnvironmentStatus::Error,
            Some("创建文件失败"),
            None,
            None,
            None,
        );
        e
    })?;

    use futures::StreamExt;
    let mut last_emitted_pct: f64 = -1.0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            emit_status(
                status_emitter.as_ref(),
                env_uuid,
                kernel_value,
                EnvironmentStatus::Error,
                Some("读取下载失败"),
                None,
                None,
                None,
            );
            e
        })?;
        downloaded += chunk.len() as u64;
        std::io::Write::write_all(&mut file, &chunk).map_err(|e| {
            emit_status(
                status_emitter.as_ref(),
                env_uuid,
                kernel_value,
                EnvironmentStatus::Error,
                Some("保存文件失败"),
                None,
                None,
                None,
            );
            e
        })?;

        if let Some(total) = total_len {
            if total > 0 {
                let pct = (downloaded as f64 / total as f64) * 100.0;
                if (pct - last_emitted_pct) >= 1.0 || downloaded >= total {
                    last_emitted_pct = pct;
                    emit_status(
                        status_emitter.as_ref(),
                        env_uuid,
                        kernel_value,
                        EnvironmentStatus::Downloading,
                        Some("下载中…"),
                        Some(pct.min(100.0)),
                        Some(downloaded),
                        Some(total),
                    );
                }
            }
        }
    }

    if let Some(total) = total_len {
        if total > 0 && downloaded >= total {
            emit_status(
                status_emitter.as_ref(),
                env_uuid,
                kernel_value,
                EnvironmentStatus::Downloading,
                Some("下载中…"),
                Some(100.0),
                Some(downloaded),
                Some(total),
            );
        }
    }

    Ok(())
}

fn redact_download_url(url: &str) -> String {
    match reqwest::Url::parse(url) {
        Ok(mut parsed) => {
            let _ = parsed.set_username("");
            let _ = parsed.set_password(None);
            parsed.set_query(None);
            parsed.set_fragment(None);
            parsed.to_string()
        }
        Err(_) => "<invalid kernel download URL>".to_string(),
    }
}

/// 校验下载文件哈希
fn verify_download_hash(
    env_uuid: &Option<String>,
    kernel_value: &str,
    zip_path: &Path,
    expected_hash: &str,
    status_emitter: Option<KernelStatusEmitter>,
) -> Result<()> {
    let actual_hash = calculate_file_hash(zip_path)?;
    let expected = expected_hash.trim().to_lowercase();
    let actual = actual_hash.to_lowercase();

    if actual != expected {
        let _ = fs::remove_file(zip_path);
        emit_status(
            status_emitter.as_ref(),
            env_uuid,
            kernel_value,
            EnvironmentStatus::Error,
            Some("校验失败"),
            None,
            None,
            None,
        );
        return Err(format!(
            "下载文件校验失败（哈希不一致），请重试。预期: {} 实际: {}",
            expected, actual
        )
        .into());
    }

    Ok(())
}

/// 解压或移动内核文件
fn extract_or_move_kernel(
    env_uuid: &Option<String>,
    kernel_value: &str,
    zip_path: &Path,
    kernel_dir: &Path,
    requires_extract: bool,
    status_emitter: Option<KernelStatusEmitter>,
) -> Result<()> {
    if requires_extract {
        emit_status(
            status_emitter.as_ref(),
            env_uuid,
            kernel_value,
            EnvironmentStatus::Extracting,
            Some("解压中…"),
            None,
            None,
            None,
        );
        fs::create_dir_all(kernel_dir)?;
        extract_zip_to_dir(zip_path, kernel_dir).map_err(|e| {
            emit_status(
                status_emitter.as_ref(),
                env_uuid,
                kernel_value,
                EnvironmentStatus::Error,
                Some("解压失败"),
                None,
                None,
                None,
            );
            e
        })?;
        fs::remove_file(zip_path)?;
    } else {
        fs::create_dir_all(kernel_dir)?;
        let dest = kernel_dir.join(exe_name());
        if fs::rename(zip_path, &dest).is_err() {
            fs::copy(zip_path, &dest)?;
            fs::remove_file(zip_path)?;
        }
    }

    Ok(())
}

/// 校验核心 DLL 的 signature
fn verify_core_dll_signature(
    env_uuid: &Option<String>,
    kernel_value: &str,
    kernel_dir: &Path,
    signature: &str,
    status_emitter: Option<KernelStatusEmitter>,
) -> Result<()> {
    let expected = signature.trim().to_lowercase();
    if expected.is_empty() {
        return Ok(());
    }

    let version_dir = find_version_subdir(kernel_dir).map_err(|e| {
        emit_status(
            status_emitter.as_ref(),
            env_uuid,
            kernel_value,
            EnvironmentStatus::Error,
            Some("未找到版本目录"),
            None,
            None,
            None,
        );
        e
    })?;
    let dll_path = version_dir.join(core_dll_name());

    if !dll_path.exists() {
        emit_status(
            status_emitter.as_ref(),
            env_uuid,
            kernel_value,
            EnvironmentStatus::Error,
            Some("未找到核心 DLL"),
            None,
            None,
            None,
        );
        return Err("解压后未找到核心 DLL 文件".into());
    }

    let local_hash = calculate_file_hash_head(&dll_path, SIGNATURE_HASH_SIZE)?;
    let local = local_hash.to_lowercase();

    if local != expected {
        emit_status(
            status_emitter.as_ref(),
            env_uuid,
            kernel_value,
            EnvironmentStatus::Error,
            Some("核心 DLL 校验失败"),
            None,
            None,
            None,
        );
        return Err("解压后核心 DLL 校验失败（与 signature 不一致），请重试".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::redact_download_url;

    #[test]
    fn download_log_url_does_not_include_credentials() {
        assert_eq!(
            redact_download_url(
                "https://user:password@example.test/kernel.zip?token=secret#fragment"
            ),
            "https://example.test/kernel.zip"
        );
        assert_eq!(
            redact_download_url("not a url"),
            "<invalid kernel download URL>"
        );
    }
}
