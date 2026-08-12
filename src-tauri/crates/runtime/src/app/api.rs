use super::context::RuntimeContextInput;
use super::state::{HealthSnapshot, RuntimePhase, RuntimeStateSnapshot};
use crate::services::auth::types::AuthCommandResponse;
use crate::services::environment::kernel::types::EnvironmentCommandResponse;
use crate::services::sync::types::SyncCommandResponse;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeContextRequest {
    pub context: RuntimeContextInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DestroyContextRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResponse {
    pub runtime_id: String,
    pub phase: RuntimePhase,
    pub uptime_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateResponse {
    pub state: RuntimeStateSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub health: HealthSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentResponse {
    pub result: EnvironmentCommandResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub result: AuthCommandResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub result: SyncCommandResponse,
}
