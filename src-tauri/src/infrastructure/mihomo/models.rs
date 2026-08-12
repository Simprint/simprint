use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

const CLASH_VERGE_REV_APP_ID: &str = "io.github.clash-verge-rev.clash-verge-rev";
// Clash Verge Rev 的 config.yaml 只承载基础端口与 controller 配置。
// 如果把 Simprint 的 listeners 编排写回这个文件，再触发 Mihomo reload，
// 内核可能退回到只包含 DIRECT/REJECT/GLOBAL 的最小配置，导致订阅节点与策略组丢失。
// 因此这里必须固定使用 clash-verge.yaml，不能再把 config.yaml 当成候选路径。
const MIHOMO_CONFIG_FILE_PRIMARY: &str = "clash-verge.yaml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MihomoConnectionConfig {
    pub controller: String,
    #[serde(default)]
    pub secret: String,
    #[serde(default = "default_mihomo_config_path")]
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MihomoConnectionInfo {
    pub attached: bool,
    pub controller: Option<String>,
    pub secret: Option<String>,
    pub config_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MihomoStatus {
    pub attached: bool,
    pub controller: Option<String>,
    pub config_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MihomoProxyDelayResult {
    pub name: String,
    pub delay_ms: Option<u64>,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MihomoOverview {
    pub controller: String,
    pub version: Option<String>,
    pub providers: Vec<MihomoProviderOverview>,
    pub groups: Vec<MihomoGroupOverview>,
    pub nodes: Vec<MihomoNodeOverview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MihomoNodeSelectionSnapshot {
    pub controller: String,
    pub selected_node_names: Vec<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplyMihomoNodeSelectionRequest {
    #[serde(default)]
    pub selected_node_names: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMihomoLocalProxyRequest {
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub latency_ms: Option<u64>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MihomoLocalProxy {
    pub id: String,
    pub controller: String,
    pub name: String,
    pub node_name: String,
    pub listener_name: String,
    pub listen_host: String,
    pub listen_port: u16,
    pub listener_type: String,
    pub proxy_scheme: String,
    pub status: String,
    #[serde(default)]
    pub latency_ms: Option<u64>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MihomoProviderOverview {
    pub name: String,
    pub provider_type: Option<String>,
    pub vehicle_type: Option<String>,
    pub updated_at: Option<String>,
    pub node_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MihomoGroupOverview {
    pub name: String,
    pub group_type: String,
    pub selected: Option<String>,
    pub candidates: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MihomoNodeOverview {
    pub name: String,
    pub node_type: String,
    pub alive: Option<bool>,
    pub udp: Option<bool>,
    pub source_provider: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawVersionResponse {
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawProvidersResponse {
    #[serde(default)]
    pub providers: IndexMap<String, RawProvider>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawGroupsResponse {
    GroupsArray(Vec<RawGroup>),
    GroupsMap(IndexMap<String, Value>),
    WrappedGroupsArray { groups: Vec<RawGroup> },
    WrappedGroupsMap { groups: IndexMap<String, Value> },
    WrappedProxiesArray { proxies: Vec<RawGroup> },
    WrappedProxiesMap { proxies: IndexMap<String, Value> },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawGroup {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub now: Option<String>,
    #[serde(default)]
    pub all: Vec<String>,
    #[serde(default)]
    pub hidden: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RawProvider {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub proxies: Vec<RawProviderProxy>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default, rename = "vehicleType")]
    pub vehicle_type: Option<String>,
    #[serde(default, rename = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawProviderProxy {
    pub name: String,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub alive: Option<bool>,
    #[serde(default)]
    pub udp: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RawProxiesResponse {
    #[serde(default)]
    pub proxies: IndexMap<String, RawProxy>,
}

#[derive(Debug, Deserialize)]
pub struct RawProxy {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub now: Option<String>,
    #[serde(default)]
    pub all: Vec<String>,
    #[serde(default)]
    pub alive: Option<bool>,
    #[serde(default)]
    pub udp: Option<bool>,
}

fn default_mihomo_config_path() -> String {
    detect_default_mihomo_config_path()
}

pub fn detect_default_mihomo_config_path() -> String {
    detect_clash_verge_rev_config_dir()
        .map(resolve_mihomo_config_path)
        .unwrap_or_else(|| MIHOMO_CONFIG_FILE_PRIMARY.to_string())
}

fn detect_clash_verge_rev_config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|base| base.join(CLASH_VERGE_REV_APP_ID))
    }

    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

fn resolve_mihomo_config_path(config_dir: PathBuf) -> String {
    config_dir.join(MIHOMO_CONFIG_FILE_PRIMARY).to_string_lossy().to_string()
}
