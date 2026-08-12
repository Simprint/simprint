use crate::app::{Result, RuntimeError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::TcpListener;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

const CDP_HOST: &str = "127.0.0.1";
const CDP_PORT_START: u16 = 29200;
const CDP_PORT_END: u16 = 29499;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpEndpointInfo {
    pub env_uuid: String,
    pub host: String,
    pub port: u16,
    pub version_url: String,
    pub list_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser_ws_url: Option<String>,
}

pub struct CdpEndpointManager {
    ports: Arc<RwLock<HashMap<String, u16>>>,
}

impl CdpEndpointManager {
    pub fn new() -> Self {
        Self {
            ports: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn allocate_port(&self, env_uuid: &str) -> Result<u16> {
        {
            let ports = self.ports.read().await;
            if let Some(port) = ports.get(env_uuid) {
                return Ok(*port);
            }
        }

        let mut ports = self.ports.write().await;
        if let Some(port) = ports.get(env_uuid) {
            return Ok(*port);
        }

        let used_ports: HashSet<u16> = ports.values().copied().collect();
        let port = find_available_port(&used_ports).ok_or_else(|| {
            RuntimeError::Internal("No available CDP port in configured range".into())
        })?;
        ports.insert(env_uuid.to_string(), port);
        Ok(port)
    }

    pub async fn remove(&self, env_uuid: &str) {
        let mut ports = self.ports.write().await;
        ports.remove(env_uuid);
    }

    pub async fn clear_all(&self) {
        let mut ports = self.ports.write().await;
        ports.clear();
    }

    pub async fn get_port(&self, env_uuid: &str) -> Option<u16> {
        let ports = self.ports.read().await;
        ports.get(env_uuid).copied()
    }

    pub async fn get_endpoint(&self, env_uuid: &str) -> Option<CdpEndpointInfo> {
        let port = self.get_port(env_uuid).await?;
        let version_url = format!("http://{}:{}/json/version", CDP_HOST, port);
        let list_url = format!("http://{}:{}/json/list", CDP_HOST, port);
        let browser_ws_url = query_browser_ws_url(&version_url).await;

        Some(CdpEndpointInfo {
            env_uuid: env_uuid.to_string(),
            host: CDP_HOST.to_string(),
            port,
            version_url,
            list_url,
            browser_ws_url,
        })
    }
}

impl Default for CdpEndpointManager {
    fn default() -> Self {
        Self::new()
    }
}

fn find_available_port(used_ports: &HashSet<u16>) -> Option<u16> {
    for port in CDP_PORT_START..=CDP_PORT_END {
        if used_ports.contains(&port) {
            continue;
        }

        if TcpListener::bind((CDP_HOST, port)).is_ok() {
            return Some(port);
        }
    }

    None
}

async fn query_browser_ws_url(version_url: &str) -> Option<String> {
    let client = reqwest::Client::builder().timeout(Duration::from_secs(1)).build().ok()?;
    let response = client.get(version_url).send().await.ok()?;
    let payload = response.json::<serde_json::Value>().await.ok()?;
    payload
        .get("webSocketDebuggerUrl")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
}
