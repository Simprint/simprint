use super::error::{Result, RuntimeError};
use crate::infrastructure::diagnostics::unix_now_ms;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeEventEnvelope {
    pub name: String,
    pub emitted_at_unix_ms: u64,
    pub payload: Value,
}

#[derive(Clone)]
pub struct EventPublisher {
    tx: mpsc::UnboundedSender<RuntimeEventEnvelope>,
}

impl EventPublisher {
    pub fn emit_value(&self, name: impl Into<String>, payload: Value) -> Result<()> {
        self.tx
            .send(RuntimeEventEnvelope {
                name: name.into(),
                emitted_at_unix_ms: unix_now_ms(),
                payload,
            })
            .map_err(|error| RuntimeError::Internal(format!("event channel closed: {}", error)))
    }

    pub fn emit<T: Serialize>(&self, name: impl Into<String>, payload: &T) -> Result<()> {
        let value = serde_json::to_value(payload)
            .map_err(|error| RuntimeError::Serialization(error.to_string()))?;
        self.emit_value(name, value)
    }
}

pub fn event_channel() -> (
    EventPublisher,
    mpsc::UnboundedReceiver<RuntimeEventEnvelope>,
) {
    let (tx, rx) = mpsc::unbounded_channel();
    (EventPublisher { tx }, rx)
}
