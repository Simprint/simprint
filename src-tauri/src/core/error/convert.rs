//! 自动错误转换
//!
//! 实现从其他错误类型到应用错误类型的自动转换

use super::Error;

/// 从 anyhow::Error 转换（兜底转换）
impl From<anyhow::Error> for Error {
    fn from(_err: anyhow::Error) -> Self {
        Self::AppInitFailed
    }
}

/// 从 String 转换
impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::KernelPrepareFailed(err)
    }
}

/// 从 &str 转换
impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Self::KernelPrepareFailed(err.to_string())
    }
}

/// 从 reqwest::Error 转换
impl From<reqwest::Error> for Error {
    fn from(_err: reqwest::Error) -> Self {
        Self::NetworkRequestFailed
    }
}

/// 从 tauri::Error 转换
impl From<tauri::Error> for Error {
    fn from(_err: tauri::Error) -> Self {
        Self::WindowOperationFailed
    }
}

/// 从 zip::result::ZipError 转换
impl From<zip::result::ZipError> for Error {
    fn from(_err: zip::result::ZipError) -> Self {
        Self::KernelInstallFailed
    }
}
