use super::manager::EventBusManager;
use super::topics::Topic;
use crate::app::EventPublisher;
use crate::infrastructure::diagnostics::{log_debug, log_info};
use serde::Serialize;
use tokio::sync::OnceCell;

use std::sync::Arc;

static EVENTBUS_MANAGER: OnceCell<Arc<EventBusManager>> = OnceCell::const_new();

#[derive(Clone, Serialize)]
pub struct EnvConnectionPayload {
    pub env_id: String,
    pub status: String,
}

pub async fn init_eventbus_manager(events: EventPublisher) -> Arc<EventBusManager> {
    EVENTBUS_MANAGER
        .get_or_init(|| async move {
            let manager = Arc::new(EventBusManager::new());

            let forward_manager = manager.clone();
            manager
                .set_message_handler(move |env_id, msg| {
                    if msg.topic == Topic::SyncInputEvent {
                        let data = msg.data;
                        let manager = forward_manager.clone();
                        let sender = env_id.clone();
                        tokio::spawn(async move {
                            manager.forward_sync_to_slaves(&sender, data).await;
                        });
                    } else if msg.topic == Topic::SyncPaste {
                        let data = msg.data;
                        let manager = forward_manager.clone();
                        let sender = env_id.clone();
                        tokio::spawn(async move {
                            manager.forward_paste_to_slaves(&sender, data).await;
                        });
                    } else if msg.topic == Topic::SyncInputDebug {
                        let s = String::from_utf8_lossy(&msg.data);
                        log_debug("eventbus", format!("[SyncInput] {}", s));
                    }
                })
                .await;

            let event_sink = events.clone();
            manager
                .set_connection_status_handler(move |payload| {
                    let _ = event_sink.emit("eventbus.connection_status", &payload);
                })
                .await;

            manager
                .set_disconnect_handler(|env_id| {
                    log_info("eventbus", format!("Browser disconnected: {}", env_id));
                })
                .await;

            manager
        })
        .await
        .clone()
}

pub fn get_eventbus_manager() -> Option<Arc<EventBusManager>> {
    EVENTBUS_MANAGER.get().cloned()
}

pub fn eventbus_manager() -> Arc<EventBusManager> {
    EVENTBUS_MANAGER
        .get()
        .cloned()
        .expect("EventBusManager not initialized. Call init_eventbus_manager() first.")
}
