//! 内核服务工具函数

use crate::core::error::Result;
use crate::domain::environment::EnvironmentStatus;
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};

use super::types::{EVENT_KERNEL_PREPARE_STATUS, KernelPrepareStatusPayload, KernelStatusEmitter};

/// 解析有效的 profiles 根目录：若传入为空则使用应用默认
pub fn resolve_profiles_base(
    app: &tauri::AppHandle,
    profiles_path_override: &str,
) -> Result<PathBuf> {
    let base = profiles_path_override.trim();
    if !base.is_empty() {
        let p = PathBuf::from(base);
        if p.is_absolute() {
            return Ok(p);
        }
        let app_data = crate::core::paths::PathManager::get_app_data_dir(app)?;
        return Ok(app_data.join(base));
    }
    crate::core::paths::PathManager::get_profiles_dir(app).map_err(Into::into)
}

/// Resolve a catalog-provided installation directory without allowing it to
/// escape the profiles root.
pub fn resolve_kernel_install_dir(base: &Path, install_dir_name: &str) -> Result<PathBuf> {
    let install_dir_name = install_dir_name.trim();
    let mut components = Path::new(install_dir_name).components();
    match (components.next(), components.next()) {
        (Some(std::path::Component::Normal(_)), None) => Ok(base.join(install_dir_name)),
        _ => Err("内核安装目录名称无效".into()),
    }
}

/// 将 zip 解压到目标目录
pub fn extract_zip_to_dir(zip_path: &Path, target_dir: &Path) -> Result<()> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(io::BufReader::new(file))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        let out_path = target_dir.join(&name);

        if name.ends_with('/') || entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out_file)?;
        }
    }
    Ok(())
}

/// 可执行文件名
pub fn exe_name() -> &'static str {
    #[cfg(target_os = "windows")]
    return "simprint.exe";
    #[cfg(not(target_os = "windows"))]
    return "simprint";
}

/// 核心 DLL 文件名
pub fn core_dll_name() -> &'static str {
    #[cfg(target_os = "windows")]
    return "chrome.dll";
    #[cfg(target_os = "macos")]
    return "Chromium Framework";
    #[cfg(target_os = "linux")]
    return "libchrome.so";
}

/// 计算文件前 N 字节的 SHA256 哈希
pub fn calculate_file_hash_head(file_path: &Path, size: u64) -> Result<String> {
    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192];
    let mut total_read = 0u64;

    while total_read < size {
        let to_read = std::cmp::min(buffer.len() as u64, size - total_read) as usize;
        let n = file.read(&mut buffer[..to_read])?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        total_read += n as u64;
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// 从 XML 内容中提取属性值
pub fn extract_xml_attribute(content: &str, attr_name: &str) -> Option<String> {
    let patterns = [format!("{}='", attr_name), format!("{}=\"", attr_name)];

    for pattern in &patterns {
        if let Some(start_idx) = content.find(pattern) {
            let value_start = start_idx + pattern.len();
            let quote_char = if pattern.ends_with('\'') { '\'' } else { '"' };
            if let Some(end_offset) = content[value_start..].find(quote_char) {
                let value = &content[value_start..value_start + end_offset];
                return Some(value.to_string());
            }
        }
    }
    None
}

/// 发送内核准备状态事件
pub fn build_tauri_status_emitter(app: tauri::AppHandle) -> KernelStatusEmitter {
    use tauri::Emitter;

    std::sync::Arc::new(move |payload: KernelPrepareStatusPayload| {
        let _ = app.emit(EVENT_KERNEL_PREPARE_STATUS, payload);
    })
}

pub fn emit_status(
    emitter: Option<&KernelStatusEmitter>,
    env_uuid: &Option<String>,
    kernel_value: &str,
    status: EnvironmentStatus,
    message: Option<&str>,
    progress_percent: Option<f64>,
    downloaded: Option<u64>,
    total: Option<u64>,
) {
    if let Some(emitter) = emitter {
        emitter(KernelPrepareStatusPayload {
            env_uuid: env_uuid.clone(),
            kernel_value: kernel_value.to_string(),
            status,
            message: message.map(String::from),
            progress_percent: progress_percent.filter(|&p| p >= 0.0 && p <= 100.0),
            downloaded,
            total,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_kernel_install_dir;
    use std::path::Path;

    #[test]
    fn kernel_install_dir_must_be_one_normal_component() {
        let base = Path::new("profiles");
        assert_eq!(
            resolve_kernel_install_dir(base, "Chrome 144").unwrap(),
            base.join("Chrome 144")
        );
        assert!(resolve_kernel_install_dir(base, "").is_err());
        assert!(resolve_kernel_install_dir(base, "..").is_err());
        assert!(resolve_kernel_install_dir(base, "nested/kernel").is_err());
    }
}
