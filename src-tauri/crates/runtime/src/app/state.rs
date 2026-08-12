use crate::infrastructure::diagnostics::unix_now_ms;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Instant;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
pub struct ModuleHealthSnapshot {
    pub name: String,
    pub phase: String,
    pub healthy: bool,
    pub detail: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub runtime: RuntimeStateSnapshot,
    pub modules: Vec<ModuleHealthSnapshot>,
}

struct RuntimeStateInner {
    phase: RuntimePhase,
    context_id: Option<String>,
    last_error: Option<String>,
}

pub struct RuntimeStateStore {
    runtime_id: String,
    runtime_version: String,
    booted_at_unix_ms: u64,
    started_at: Instant,
    inner: RwLock<RuntimeStateInner>,
}

impl RuntimeStateStore {
    pub fn new(runtime_id: impl Into<String>, runtime_version: impl Into<String>) -> Self {
        Self {
            runtime_id: runtime_id.into(),
            runtime_version: runtime_version.into(),
            booted_at_unix_ms: unix_now_ms(),
            started_at: Instant::now(),
            inner: RwLock::new(RuntimeStateInner {
                phase: RuntimePhase::Booting,
                context_id: None,
                last_error: None,
            }),
        }
    }

    pub async fn transition(&self, phase: RuntimePhase) {
        let mut inner = self.inner.write().await;
        inner.phase = phase;
    }

    pub async fn phase(&self) -> RuntimePhase {
        self.inner.read().await.phase
    }

    pub async fn attach_context(&self, context_id: String) {
        let mut inner = self.inner.write().await;
        inner.context_id = Some(context_id);
    }

    pub async fn clear_context(&self) {
        let mut inner = self.inner.write().await;
        inner.context_id = None;
    }

    pub async fn record_error(&self, error: impl Into<String>) {
        let mut inner = self.inner.write().await;
        inner.last_error = Some(error.into());
    }

    pub async fn clear_error(&self) {
        let mut inner = self.inner.write().await;
        inner.last_error = None;
    }

    pub async fn snapshot(&self, module_count: usize) -> RuntimeStateSnapshot {
        let inner = self.inner.read().await;
        RuntimeStateSnapshot {
            runtime_id: self.runtime_id.clone(),
            runtime_version: self.runtime_version.clone(),
            phase: inner.phase,
            booted_at_unix_ms: self.booted_at_unix_ms,
            uptime_ms: self.started_at.elapsed().as_millis() as u64,
            context_id: inner.context_id.clone(),
            last_error: inner.last_error.clone(),
            module_count,
        }
    }

    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    pub fn runtime_version(&self) -> &str {
        &self.runtime_version
    }
}
