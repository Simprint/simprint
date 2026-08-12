use std::sync::Arc;
use std::time::Duration;

use tauri::Manager;
use tokio::{sync::Mutex, time::sleep};

use crate::{
    local_api::types::LocalApiRuntimeConfig,
    mcp::{
        config::{McpConfig, McpManagerStatus, McpServerRuntimeConfig, load_config},
        server::McpServerHandle,
    },
};

struct ManagedMcpRuntime {
    handle: McpServerHandle,
}

pub struct McpManager {
    runtime: Mutex<Option<ManagedMcpRuntime>>,
}

impl McpManager {
    pub fn new() -> Self {
        Self {
            runtime: Mutex::new(None),
        }
    }

    pub async fn start(self: &Arc<Self>, app: &tauri::AppHandle) -> Result<(), String> {
        let config = load_config(app)?;
        self.start_with_config(config).await
    }

    pub async fn reload(self: &Arc<Self>, app: &tauri::AppHandle) -> Result<(), String> {
        let config = load_config(app)?;
        if !config.enabled {
            self.stop().await;
            return Ok(());
        }

        self.start_with_config(config).await
    }

    pub async fn stop(&self) {
        let mut guard = self.runtime.lock().await;
        if let Some(runtime) = guard.take() {
            runtime.handle.shutdown().await;
        }
    }

    pub async fn status(&self) -> McpManagerStatus {
        let mut guard = self.runtime.lock().await;

        if let Some(runtime) = guard.as_ref() {
            if runtime.handle.is_finished() {
                *guard = None;
            } else {
                return McpManagerStatus { running: true };
            }
        }

        McpManagerStatus { running: false }
    }

    async fn start_with_config(&self, config: McpConfig) -> Result<(), String> {
        if !config.enabled {
            return Err("请先启用 MCP 服务开关".to_string());
        }

        let local_api_config = fetch_local_api_runtime_config().await?;
        if local_api_config.api_key.trim().is_empty() {
            return Err("Local API 凭证无效，请稍后重试".to_string());
        }

        let runtime = config.runtime()?;
        let server_config = McpServerRuntimeConfig {
            port: runtime.port,
            local_api_api_key: local_api_config.api_key,
        };
        let health_url = format!("http://127.0.0.1:{}/health", server_config.port);

        self.stop().await;

        let handle = crate::mcp::server::spawn_mcp_server(server_config.clone())
            .map_err(|error| error.to_string())?;
        if let Err(error) = wait_for_mcp_server_ready(&health_url).await {
            handle.shutdown().await;
            return Err(error);
        }

        let mut guard = self.runtime.lock().await;
        *guard = Some(ManagedMcpRuntime { handle });

        Ok(())
    }
}

async fn fetch_local_api_runtime_config() -> Result<LocalApiRuntimeConfig, String> {
    let app = crate::app::handle::get_app_handle().map_err(|error| error.to_string())?;
    let context = app.state::<business::svc_ctx::SvcCtx>();
    let config = business::services::local_api::get_local_api_config_service(
        &context,
        context.local_user_uuid,
    )
    .await
    .map_err(|error| error.to_string())?;
    serde_json::to_value(config)
        .and_then(serde_json::from_value::<LocalApiRuntimeConfig>)
        .map_err(|error| error.to_string())
}

async fn wait_for_mcp_server_ready(health_url: &str) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .map_err(|error| error.to_string())?;

    for _ in 0..20 {
        match client.get(health_url).send().await {
            Ok(response) if response.status().is_success() => return Ok(()),
            Ok(_) | Err(_) => {
                sleep(Duration::from_millis(200)).await;
            }
        }
    }

    Err("MCP 服务启动超时，请稍后重试".to_string())
}
