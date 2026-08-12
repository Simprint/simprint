use crate::infrastructure::diagnostics::unix_now_ms;
use crate::infrastructure::eventbus::AuthInfo;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

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
pub struct RuntimeContext {
    pub context_id: String,
    pub initialized_at_unix_ms: u64,
    pub input: RuntimeContextInput,
}

impl RuntimeContext {
    pub fn new(sequence: u64, input: RuntimeContextInput) -> Self {
        Self {
            context_id: format!("context-{}", sequence),
            initialized_at_unix_ms: unix_now_ms(),
            input,
        }
    }
}
