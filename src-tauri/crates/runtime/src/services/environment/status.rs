use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentStatus {
    Verifying,
    Downloading,
    Extracting,
    Ready,
    Initializing,
    Starting,
    Running,
    Stopping,
    Stopped,
    Error,
}
