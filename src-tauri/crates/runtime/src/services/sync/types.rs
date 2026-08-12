use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunningEnvironment {
    pub uuid: String,
    pub name: String,
    pub status: String,
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
