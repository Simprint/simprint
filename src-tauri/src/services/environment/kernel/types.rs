//! 内核服务相关类型定义

use crate::domain::environment::EnvironmentStatus;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserProxyAuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserProxyConfigPayload {
    pub mode: String,
    pub server: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bypass_list: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<std::collections::HashMap<String, BrowserProxyAuthPayload>>,
}

/// 内核准备状态事件 payload（前端监听 kernel-prepare-status）
#[derive(Debug, Clone, Serialize)]
pub struct KernelPrepareStatusPayload {
    pub env_uuid: Option<String>,
    pub kernel_value: String,
    pub status: EnvironmentStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// 下载进度 0–100，仅 status=downloading 时有效
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

pub type KernelStatusEmitter = Arc<dyn Fn(KernelPrepareStatusPayload) + Send + Sync>;

/// 代理配置结构体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProxyConfig {
    pub host: String,
    pub port: u16,
    pub proxy_type: String,
    pub username: Option<String>,
    pub password: Option<crate::infrastructure::proxy::types::ProxyPassword>,
}

impl ProxyConfig {
    /// 构建 Chromium 代理参数
    pub fn to_proxy_arg(&self) -> String {
        let scheme = match self.proxy_type.to_lowercase().as_str() {
            "socks5" => "socks5",
            "https" => "https",
            _ => "http",
        };
        format!("--proxy-server={}://{}:{}", scheme, self.host, self.port)
    }

    /// 兼容旧调用链。
    pub fn decrypt_password(mut self) -> Result<Self, String> {
        Ok(self)
    }

    /// 转换为 infrastructure 层的 ProxyConfig
    pub fn to_infrastructure_proxy_config(
        &self,
    ) -> crate::infrastructure::proxy::types::ProxyConfig {
        use crate::infrastructure::proxy::types::ProxyType;

        let proxy_type = match self.proxy_type.to_lowercase().as_str() {
            "socks5" => ProxyType::Socks5,
            "https" => ProxyType::Https,
            _ => ProxyType::Http,
        };

        crate::infrastructure::proxy::types::ProxyConfig {
            proxy_type,
            host: self.host.clone(),
            port: self.port,
            username: self.username.clone(),
            password: self.password.clone(),
        }
    }

    pub fn to_browser_proxy_config(&self) -> BrowserProxyConfigPayload {
        let scheme = match self.proxy_type.to_lowercase().as_str() {
            "socks5" => "socks5",
            "https" => "https",
            _ => "http",
        };
        let endpoint = format!("{}://{}:{}", scheme, self.host, self.port);

        let auth = match (&self.username, &self.password) {
            (Some(username), Some(password)) => Some(std::collections::HashMap::from([(
                endpoint.clone(),
                BrowserProxyAuthPayload {
                    username: username.clone(),
                    password: password.value.clone(),
                },
            )])),
            _ => None,
        };

        BrowserProxyConfigPayload {
            mode: "fixed_servers".to_string(),
            server: endpoint,
            bypass_list: None,
            auth,
        }
    }
}

/// 常量定义
pub const EVENT_KERNEL_PREPARE_STATUS: &str = "kernel-prepare-status";
pub const SIGNATURE_HASH_SIZE: u64 = 10 * 1024 * 1024;
pub const TIMEZONE_DETECTION_TIMEOUT_SECS: u64 = 10;

/// 批量启动请求
#[derive(Debug, Deserialize)]
pub struct BatchLaunchRequest {
    pub exe_path: String,
    pub env_uuid: String,
    pub cache_path: String,
    pub urls: Option<Vec<String>>,
    pub proxy: Option<ProxyConfig>,
    pub fingerprint_config: Option<crate::infrastructure::runtime::FingerprintConfig>,
    pub accounts: Option<Vec<AccountInfo>>,
    pub extensions: Option<Vec<ExtensionInfo>>,
}

/// 账号信息（来自后端，密码为明文）
#[derive(Debug, Clone, Deserialize)]
pub struct AccountInfo {
    pub account: String,
    pub password: String,
    pub platform_url: String,
    pub platform_name: Option<String>,
}

/// 扩展信息（来自后端）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExtensionInfo {
    pub extension_id: String,
    pub name: String,
    pub version: String,
    pub download_url: String,
    pub hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}

/// 批量启动结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchLaunchResult {
    pub env_uuid: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Environment-scoped CDP endpoint metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpEndpointResponse {
    pub env_uuid: String,
    pub host: String,
    pub port: u16,
    pub version_url: String,
    pub list_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser_ws_url: Option<String>,
}
