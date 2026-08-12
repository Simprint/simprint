//! 内核校验模块

use crate::core::error::Result;
use crate::domain::environment::EnvironmentStatus;
use std::fs;
use std::path::{Path, PathBuf};

use super::types::{KernelStatusEmitter, SIGNATURE_HASH_SIZE};
use super::utils::{calculate_file_hash_head, core_dll_name, emit_status, extract_xml_attribute};

/// 校验内核完整性
///
/// # 返回
/// - `Ok(Some(signature))`: 校验通过，并返回实际匹配的签名
/// - `Ok(None)`: 校验失败
/// - `Err`: 发生错误
pub fn verify_kernel(
    _app: &tauri::AppHandle,
    env_uuid: &Option<String>,
    kernel_value: &str,
    kernel_dir: &Path,
    expected_signatures: &[String],
    status_emitter: Option<KernelStatusEmitter>,
) -> Result<Option<String>> {
    emit_status(
        status_emitter.as_ref(),
        env_uuid,
        kernel_value,
        EnvironmentStatus::Verifying,
        Some("校验中…"),
        None,
        None,
        None,
    );

    // 查找版本子目录
    crate::log_info!(
        crate::core::logger::modules::KERNEL,
        "开始查找版本子目录: {}",
        kernel_dir.display()
    );

    let version_dir = match find_version_subdir(kernel_dir) {
        Ok(dir) => {
            crate::log_info!(
                crate::core::logger::modules::KERNEL,
                "找到版本子目录: {}",
                dir.display()
            );
            dir
        }
        Err(e) => {
            crate::log_warn!(
                crate::core::logger::modules::KERNEL,
                "未找到版本子目录: {} - {}",
                kernel_dir.display(),
                e
            );
            return Ok(None);
        }
    };

    let dll_path = version_dir.join(core_dll_name());
    crate::log_info!(
        crate::core::logger::modules::KERNEL,
        "检查核心 DLL: {}",
        dll_path.display()
    );

    if !dll_path.exists() {
        crate::log_warn!(
            crate::core::logger::modules::KERNEL,
            "未找到核心 DLL 文件: {}",
            dll_path.display()
        );
        return Ok(None);
    }

    // 校验 signature
    let expected = expected_signatures
        .iter()
        .map(|signature| signature.trim().to_lowercase())
        .filter(|signature| !signature.is_empty())
        .collect::<Vec<_>>();
    if expected.is_empty() {
        return Err("该内核版本缺少 signature，无法校验核心 DLL".into());
    }
    crate::log_info!(
        crate::core::logger::modules::KERNEL,
        "开始计算 DLL 哈希，可接受值: {}",
        expected.join(", ")
    );

    let local_hash = calculate_file_hash_head(&dll_path, SIGNATURE_HASH_SIZE)?;
    let local = local_hash.to_lowercase();

    crate::log_info!(
        crate::core::logger::modules::KERNEL,
        "DLL 哈希计算完成，实际值: {}",
        local
    );

    let Some(matched_index) = expected.iter().position(|signature| signature == &local) else {
        crate::log_warn!(
            crate::core::logger::modules::KERNEL,
            "核心 DLL 哈希不一致，可接受值: {}, 实际: {}",
            expected.join(", "),
            local
        );
        return Ok(None);
    };

    if matched_index > 0 {
        crate::log_warn!(
            crate::core::logger::modules::KERNEL,
            "使用兼容签名接受已安装内核: {}",
            local
        );
    }

    crate::log_info!(
        crate::core::logger::modules::KERNEL,
        "内核校验通过: {}",
        kernel_dir.display()
    );

    Ok(Some(local))
}

/// 查找版本子目录
///
/// 在内核目录中查找包含 .manifest 文件和核心 DLL 的版本子目录
pub fn find_version_subdir(kernel_dir: &Path) -> Result<PathBuf> {
    let entries = fs::read_dir(kernel_dir)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let manifest_path = path.join(format!("{}.manifest", dir_name));
        if !manifest_path.exists() {
            continue;
        }

        let content = match fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let name_match = extract_xml_attribute(&content, "name");
        let version_match = extract_xml_attribute(&content, "version");

        if let Some(ref name) = name_match {
            if name == &dir_name && path.join(core_dll_name()).exists() {
                return Ok(path);
            }
        }
        if let Some(ref version) = version_match {
            if version == &dir_name && path.join(core_dll_name()).exists() {
                return Ok(path);
            }
        }
    }

    Err("未找到包含核心 DLL 的版本子目录".into())
}
