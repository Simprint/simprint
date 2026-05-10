use crate::domain::environment::EnvironmentStatus;
use crate::services::environment::kernel::types::BrowserProxyConfigPayload;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintConfig {
    pub language: Option<String>,
    pub interface_language: Option<String>,
    pub timezone: Option<String>,
    pub geolocation_prompt: Option<String>,
    pub geolocation: Option<String>,
    pub platform: Option<String>,
    pub user_agent: Option<String>,
    pub sound: Option<bool>,
    pub images: Option<bool>,
    pub video: Option<bool>,
    pub window_size: Option<String>,
    pub window_width: Option<i32>,
    pub window_height: Option<i32>,
    pub window_position: Option<String>,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub resolution: Option<Value>,
    pub color_depth: Option<i32>,
    pub device_pixel_ratio: Option<f64>,
    pub max_touch_points: Option<i32>,
    pub canvas: Option<String>,
    pub webgl_image: Option<String>,
    pub webgl_info: Option<String>,
    pub webgl_vendor: Option<String>,
    pub webgl_renderer: Option<String>,
    pub webgpu: Option<String>,
    pub font_fingerprint: Option<String>,
    pub font_list: Option<Value>,
    pub audio_context: Option<String>,
    pub speech_voices: Option<String>,
    pub client_rects: Option<String>,
    pub media_devices: Option<String>,
    pub webrtc: Option<String>,
    pub do_not_track: Option<bool>,
    pub device_name: Option<String>,
    pub device_name_random: Option<bool>,
    pub mac_address: Option<String>,
    pub mac_address_mode: Option<String>,
    pub hardware_concurrency: Option<i32>,
    pub device_memory: Option<i32>,
    pub ssl_fingerprint: Option<bool>,
    pub port_scan_protection: Option<bool>,
    pub scan_whitelist: Option<String>,
    pub hardware_acceleration: Option<bool>,
    pub disable_sandbox: Option<bool>,
    pub startup_parameters: Option<String>,
    pub random_fingerprint_on_launch: Option<bool>,
    pub env_id: Option<String>,
    pub env_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    pub is_authenticated: bool,
    pub access_token: Option<String>,
    pub user_info: Option<UserInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub username: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvConnectionPayload {
    pub env_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmptyPayload {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeRequest {
    pub protocol_version: u8,
    pub client_name: String,
    pub client_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeResponse {
    pub protocol_version: u8,
    pub runtime_version: String,
    pub runtime_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeContextInput {
    pub user_id: Option<String>,
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub auth_info: Option<AuthInfo>,
    #[serde(default)]
    pub attributes: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeContextRequest {
    pub context: RuntimeContextInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DestroyContextRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimePhase {
    Booting,
    Uninitialized,
    Initializing,
    Ready,
    Destroying,
    ShuttingDown,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStateSnapshot {
    pub runtime_id: String,
    pub runtime_version: String,
    pub phase: RuntimePhase,
    pub booted_at_unix_ms: u64,
    pub uptime_ms: u64,
    pub context_id: Option<String>,
    pub last_error: Option<String>,
    pub module_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateResponse {
    pub state: RuntimeStateSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEventEnvelope {
    pub name: String,
    pub emitted_at_unix_ms: u64,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningEnvironment {
    pub uuid: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentStartRequest {
    pub exe_path: String,
    pub env_uuid: String,
    pub user_data_dir: String,
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
pub struct BatchLaunchResult {
    pub env_uuid: String,
    pub success: bool,
    pub error: Option<String>,
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
pub struct WindowBoundsRequest {
    pub env_uuid: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentResponse {
    pub result: EnvironmentCommandResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum SyncCommandRequest {
    GetRunningEnvironments,
    StartSync {
        master_env_id: String,
        slave_env_ids: Vec<String>,
    },
    StopSync,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SyncCommandResponse {
    Ack,
    RunningEnvironments {
        environments: Vec<RunningEnvironment>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub result: SyncCommandResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum AuthCommandRequest {
    SetAuthState { auth_info: AuthInfo },
    ClearAuthState,
    GetAuthState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthCommandResponse {
    Ack,
    State { auth_info: AuthInfo },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub result: AuthCommandResponse,
}
