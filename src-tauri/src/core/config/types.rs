//! 配置类型定义
//!
//! 定义了应用的所有配置结构体

use serde::Deserialize;

/// 服务器配置
#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    /// 服务器基础URL
    pub base_url: String,
    /// API版本
    pub version: String,
    /// 密钥
    pub secret_key: String,
}

/// 更新器配置
#[derive(Deserialize, Debug, Clone)]
pub struct UpdaterConfig {
    pub check_url: String,
    pub latest_json_url: String,

    /// 下载的临时目录（可选）。
    /// - 若为空：默认使用统一根目录下的 `updates`
    /// - 若填写：使用该目录（相对路径基于统一根目录）
    #[serde(default)]
    pub updater_temp_dir: Option<String>,
}

/// 应用配置
#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    /// 服务器配置
    pub server: ServerConfig,
    /// 更新器配置
    pub updater: UpdaterConfig,
}
