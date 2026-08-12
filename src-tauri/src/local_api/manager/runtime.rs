use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::Result;
use tokio::sync::Mutex;

use crate::local_api::types::LocalApiRuntimeConfig;

use super::super::server::bootstrap::{LocalApiServerHandle, spawn_local_api_server};

pub struct LocalApiManager {
    server: Mutex<Option<LocalApiServerHandle>>,
    running: Arc<AtomicBool>,
}

impl LocalApiManager {
    pub fn new() -> Self {
        Self {
            server: Mutex::new(None),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn refresh(self: &Arc<Self>, context: &business::svc_ctx::SvcCtx) -> Result<()> {
        let config = business::services::local_api::get_local_api_config_service(
            context,
            context.local_user_uuid,
        )
        .await
        .map_err(anyhow::Error::msg)
        .and_then(|config| serde_json::to_value(config).map_err(Into::into))
        .and_then(|value| {
            serde_json::from_value::<LocalApiRuntimeConfig>(value).map_err(Into::into)
        })?;

        if !config.enabled {
            self.stop().await;
            return Ok(());
        }

        self.restart(config).await
    }

    pub async fn stop(&self) {
        let mut guard = self.server.lock().await;
        if let Some(handle) = guard.take() {
            handle.shutdown().await;
        }
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    async fn restart(&self, config: LocalApiRuntimeConfig) -> Result<()> {
        self.stop().await;
        self.running.store(true, Ordering::SeqCst);
        let handle = match spawn_local_api_server(config, Arc::clone(&self.running)) {
            Ok(handle) => handle,
            Err(error) => {
                self.running.store(false, Ordering::SeqCst);
                return Err(error);
            }
        };
        let mut guard = self.server.lock().await;
        *guard = Some(handle);
        Ok(())
    }
}
