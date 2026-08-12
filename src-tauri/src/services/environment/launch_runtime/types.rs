use serde::Deserialize;
use serde_json::Value;

use crate::services::environment::{AccountInfo, ExtensionInfo};

#[derive(Debug, Clone, Deserialize)]
pub(super) struct EnvironmentLaunchDetail {
    pub environment: Option<EnvironmentInfoLike>,
    pub config: Option<Value>,
    pub cookies: Option<Vec<EnvironmentCookieLike>>,
    pub urls: Option<Vec<EnvironmentUrlLike>>,
    pub proxy: Option<EnvironmentProxyLike>,
    pub accounts: Option<Vec<AccountInfo>>,
    pub extensions: Option<Vec<ExtensionInfo>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct EnvironmentInfoLike {
    pub id: Option<i32>,
    pub uuid: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct EnvironmentProxyLike {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub proxy_type: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct EnvironmentUrlLike {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct EnvironmentCookieLike {
    pub site: String,
    pub cookie_text: String,
}

#[derive(Debug, Clone)]
pub struct LaunchPaths {
    pub profiles_path: String,
    pub cache_path: String,
}
