use crate::infrastructure::eventbus::{AccountConfig, CookieGroup, FingerprintConfig};
use crate::services::environment::EnvironmentStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentStartRequest {
    pub exe_path: String,
    pub env_uuid: String,
    pub user_data_dir: String,
    pub cookies: Option<Vec<CookieGroup>>,
    pub urls: Option<Vec<String>>,
    pub proxy: Option<BrowserProxyConfigPayload>,
    pub fingerprint_config: Option<FingerprintConfig>,
    pub accounts: Option<Vec<AccountConfig>>,
    pub display_id: Option<String>,
    pub window_position: Option<String>,
    pub window_size: Option<String>,
    pub extension_dirs: Option<Vec<String>>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchLaunchResult {
    pub env_uuid: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowBoundsRequest {
    pub env_uuid: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpaTabInfo {
    pub position: u32,
    pub title: String,
    pub url: String,
    pub active: bool,
    pub target_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpaTabsSnapshot {
    pub tabs: Vec<RpaTabInfo>,
    pub active_position: Option<u32>,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpaTabSelection {
    pub position: u32,
    pub target_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpaTabCloseResult {
    pub closed_position: u32,
    pub active_position: u32,
    pub target_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum EnvironmentCommandRequest {
    StartEnvironment {
        request: EnvironmentStartRequest,
    },
    BatchStartEnvironments {
        requests: Vec<EnvironmentStartRequest>,
    },
    StopEnvironment {
        env_uuid: String,
    },
    BatchStopEnvironments {
        env_uuids: Vec<String>,
    },
    RefreshProxy {
        env_uuid: String,
        proxy: Option<BrowserProxyConfigPayload>,
    },
    SetWindowBounds {
        request: WindowBoundsRequest,
    },
    GetConnectedEnvironments,
    GetCdpEndpoint {
        env_uuid: String,
    },
    ListRpaTabs {
        env_uuid: String,
    },
    SelectRpaTab {
        env_uuid: String,
        position: u32,
    },
    CloseRpaTab {
        env_uuid: String,
        position: u32,
    },
    GetEnvironmentStatus {
        env_uuid: String,
    },
    GetAllEnvironmentStatuses,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EnvironmentCommandResponse {
    Ack,
    Started {
        endpoint: CdpEndpointResponse,
    },
    ConnectedEnvironments {
        env_ids: Vec<String>,
    },
    CdpEndpoint {
        endpoint: Option<CdpEndpointResponse>,
    },
    RpaTabsSnapshot {
        snapshot: RpaTabsSnapshot,
    },
    RpaTabSelected {
        selection: RpaTabSelection,
    },
    RpaTabClosed {
        result: RpaTabCloseResult,
    },
    BatchLaunchResults {
        results: Vec<BatchLaunchResult>,
    },
    Status {
        status: Option<EnvironmentStatus>,
    },
    AllStatuses {
        statuses: HashMap<String, EnvironmentStatus>,
    },
}
