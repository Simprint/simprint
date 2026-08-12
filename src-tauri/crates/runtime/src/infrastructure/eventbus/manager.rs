use super::connection::{BrowserConnection, MessageHandler};
use super::error::{EventBusError, Result};
use super::global::EnvConnectionPayload;
use super::message::{Message, MessageType};
use super::topics::Topic;
use super::transport::PipeServer;
use crate::infrastructure::diagnostics::{log_error, log_info, log_warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, oneshot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintConfig {
    pub language: Option<String>,
    pub interface_language: Option<String>,
    pub timezone: Option<String>,
    pub geolocation_prompt: Option<String>,
    pub geolocation: Option<String>,
    pub platform: Option<String>,
    pub user_agent: Option<String>,
    pub sound: Option<bool>,
    pub images: Option<bool>,
    pub video: Option<bool>,
    pub window_size: Option<String>,
    pub window_width: Option<i32>,
    pub window_height: Option<i32>,
    pub window_position: Option<String>,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub resolution: Option<serde_json::Value>,
    pub color_depth: Option<i32>,
    pub device_pixel_ratio: Option<f64>,
    pub max_touch_points: Option<i32>,
    pub canvas: Option<String>,
    pub webgl_image: Option<String>,
    pub webgl_info: Option<String>,
    pub webgl_vendor: Option<String>,
    pub webgl_renderer: Option<String>,
    pub webgpu: Option<String>,
    pub font_fingerprint: Option<String>,
    pub font_list: Option<serde_json::Value>,
    pub audio_context: Option<String>,
    pub speech_voices: Option<String>,
    pub client_rects: Option<String>,
    pub media_devices: Option<String>,
    pub webrtc: Option<String>,
    pub do_not_track: Option<bool>,
    pub device_name: Option<String>,
    pub device_name_random: Option<bool>,
    pub mac_address: Option<String>,
    pub mac_address_mode: Option<String>,
    pub hardware_concurrency: Option<i32>,
    pub device_memory: Option<i32>,
    pub ssl_fingerprint: Option<bool>,
    pub port_scan_protection: Option<bool>,
    pub scan_whitelist: Option<String>,
    pub hardware_acceleration: Option<bool>,
    pub disable_sandbox: Option<bool>,
    pub startup_parameters: Option<String>,
    pub random_fingerprint_on_launch: Option<bool>,
    pub env_id: Option<String>,
    pub env_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    pub env_uuid: String,
    pub user_data_dir: String,
    pub proxy: Option<crate::services::environment::kernel::types::BrowserProxyConfigPayload>,
    pub kernel_version: Option<String>,
    pub extensions: Option<Vec<String>>,
    pub custom_flags: Option<HashMap<String, String>>,
    pub cookies: Option<Vec<CookieGroup>>,
    pub urls: Option<Vec<String>>,
    pub fingerprint_config: Option<FingerprintConfig>,
    pub accounts: Option<Vec<AccountConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    pub is_authenticated: bool,
    pub access_token: Option<String>,
    pub user_info: Option<UserInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub username: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieGroup {
    pub site: String,
    pub cookie_text: String,
}

type DisconnectHandler = Arc<dyn Fn(String) + Send + Sync>;
type ConnectionStatusHandler = Arc<dyn Fn(EnvConnectionPayload) + Send + Sync>;
type AuthInfoProvider = Arc<dyn Fn() -> AuthInfo + Send + Sync>;

pub struct EventBusManager {
    connections: Arc<RwLock<HashMap<String, Arc<BrowserConnection>>>>,
    pending_requests: Arc<RwLock<HashMap<String, HashMap<u32, oneshot::Sender<Message>>>>>,
    message_handler: Arc<RwLock<Option<MessageHandler>>>,
    disconnect_handler: Arc<RwLock<Option<DisconnectHandler>>>,
    connection_status_handler: Arc<RwLock<Option<ConnectionStatusHandler>>>,
    auth_info_provider: Arc<RwLock<Option<AuthInfoProvider>>>,
    sync_master: Arc<RwLock<Option<String>>>,
    sync_slaves: Arc<RwLock<Vec<String>>>,
}

impl EventBusManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            message_handler: Arc::new(RwLock::new(None)),
            disconnect_handler: Arc::new(RwLock::new(None)),
            connection_status_handler: Arc::new(RwLock::new(None)),
            auth_info_provider: Arc::new(RwLock::new(None)),
            sync_master: Arc::new(RwLock::new(None)),
            sync_slaves: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_sync_state(&self) -> (Option<String>, Vec<String>) {
        let master = self.sync_master.read().await.clone();
        let slaves = self.sync_slaves.read().await.clone();
        (master, slaves)
    }

    pub async fn set_sync_state(&self, master: Option<String>, slaves: Vec<String>) {
        let mut master_guard = self.sync_master.write().await;
        let mut slaves_guard = self.sync_slaves.write().await;
        *master_guard = master;
        *slaves_guard = slaves;
    }

    pub async fn send_to_envs(
        &self,
        env_ids: &[String],
        topic: Topic,
        data: Vec<u8>,
    ) -> Vec<(String, Result<()>)> {
        let connections = self.connections.read().await;
        let message = Message::event(topic, data);
        let mut results = Vec::new();
        for env_id in env_ids {
            match connections.get(env_id) {
                Some(connection) => {
                    let result = connection.send(&message).await;
                    if let Err(error) = &result {
                        log_warn(
                            "eventbus",
                            format!("Sync send to {} failed: {}", env_id, error),
                        );
                    }
                    results.push((env_id.clone(), result));
                }
                None => results.push((
                    env_id.clone(),
                    Err(EventBusError::NotConnected(format!(
                        "env {} not connected",
                        env_id
                    ))),
                )),
            }
        }
        results
    }

    pub async fn forward_sync_to_slaves(&self, sender_env_id: &str, data: Vec<u8>) {
        let master = self.sync_master.read().await.clone();
        let slaves = self.sync_slaves.read().await.clone();
        let should_forward = matches!(master.as_ref(), Some(master_id) if master_id == sender_env_id)
            && !slaves.is_empty();

        if should_forward {
            let results = self.send_to_envs(&slaves, Topic::SyncInputEvent, data).await;
            let ok_count = results.iter().filter(|(_, result)| result.is_ok()).count();
            if ok_count < results.len() {
                log_warn(
                    "eventbus",
                    format!(
                        "forward_sync: sent to {}/{} slaves",
                        ok_count,
                        results.len()
                    ),
                );
            }
        }
    }

    pub async fn forward_paste_to_slaves(&self, sender_env_id: &str, data: Vec<u8>) {
        let master = self.sync_master.read().await.clone();
        let slaves = self.sync_slaves.read().await.clone();
        let should_forward = matches!(master.as_ref(), Some(master_id) if master_id == sender_env_id)
            && !slaves.is_empty();

        if should_forward {
            let _ = self.send_to_envs(&slaves, Topic::SyncPaste, data).await;
        }
    }

    async fn emit_connection_status(&self, env_id: &str, status: &str) {
        let handler = self.connection_status_handler.read().await.clone();
        if let Some(handler) = handler {
            handler(EnvConnectionPayload {
                env_id: env_id.to_string(),
                status: status.to_string(),
            });
        }
    }

    pub async fn set_message_handler<F>(&self, handler: F)
    where
        F: Fn(String, Message) + Send + Sync + 'static,
    {
        let mut guard = self.message_handler.write().await;
        *guard = Some(Arc::new(handler));
    }

    pub async fn set_disconnect_handler<F>(&self, handler: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        let mut guard = self.disconnect_handler.write().await;
        *guard = Some(Arc::new(handler));
    }

    pub async fn set_connection_status_handler<F>(&self, handler: F)
    where
        F: Fn(EnvConnectionPayload) + Send + Sync + 'static,
    {
        let mut guard = self.connection_status_handler.write().await;
        *guard = Some(Arc::new(handler));
    }

    pub async fn set_auth_info_provider<F>(&self, provider: F)
    where
        F: Fn() -> AuthInfo + Send + Sync + 'static,
    {
        let mut guard = self.auth_info_provider.write().await;
        *guard = Some(Arc::new(provider));
    }

    async fn create_server(&self, env_id: &str, launch_config: Option<LaunchConfig>) -> Result<()> {
        let server = PipeServer::new(env_id);
        log_info("eventbus", format!("Creating server for env: {}", env_id));

        let pipe_connection = server.accept().await?;
        let mut browser_connection = BrowserConnection::new(pipe_connection);
        browser_connection.handshake().await?;

        let env_id_owned = env_id.to_string();
        let browser_connection = Arc::new(browser_connection);

        {
            let mut connections = self.connections.write().await;
            connections.insert(env_id_owned.clone(), browser_connection.clone());
        }

        if let Some(config) = launch_config {
            let config_data = serde_json::to_vec(&config)
                .map_err(|error| EventBusError::Serialization(error.to_string()))?;
            if let Err(error) =
                browser_connection.send_event(Topic::LaunchConfig, config_data).await
            {
                self.connections.write().await.remove(&env_id_owned);
                return Err(error);
            }
            log_info("eventbus", format!("[{}] Launch config sent", env_id_owned));

            if let Some(fingerprint_config) = config.fingerprint_config {
                let fingerprint_data = serde_json::to_vec(&fingerprint_config)
                    .map_err(|error| EventBusError::Serialization(error.to_string()))?;
                if let Err(error) =
                    browser_connection.send_event(Topic::FingerprintApply, fingerprint_data).await
                {
                    self.connections.write().await.remove(&env_id_owned);
                    return Err(error);
                }
                log_info(
                    "eventbus",
                    format!("[{}] Fingerprint config sent", env_id_owned),
                );
            }
        }

        self.emit_connection_status(&env_id_owned, "connected").await;

        let handler = self.message_handler.read().await.clone();
        let disconnect_handler = self.disconnect_handler.read().await.clone();
        let connections = self.connections.clone();
        let pending_requests = self.pending_requests.clone();
        let status_handler = self.connection_status_handler.clone();

        if let Some(handler) = handler {
            let env_id_for_loop = env_id_owned.clone();
            tokio::spawn(async move {
                loop {
                    match browser_connection.recv().await {
                        Ok(msg) => {
                            if msg.msg_type == MessageType::Response {
                                let mut pending = pending_requests.write().await;
                                if let Some(waiters) = pending.get_mut(&env_id_for_loop) {
                                    if let Some(sender) = waiters.remove(&msg.msg_id) {
                                        let _ = sender.send(msg);
                                        continue;
                                    }
                                }

                                log_warn(
                                    "eventbus",
                                    format!(
                                        "[{}] Received unexpected response message: {}",
                                        env_id_for_loop, msg.msg_id
                                    ),
                                );
                                continue;
                            }

                            if msg.topic == Topic::AuthRequest {
                                if let Err(error) =
                                    Self::handle_auth_request(&browser_connection, msg).await
                                {
                                    log_warn(
                                        "eventbus",
                                        format!(
                                            "[{}] Handle auth request failed: {}",
                                            env_id_for_loop, error
                                        ),
                                    );
                                }
                                continue;
                            }

                            handler(env_id_for_loop.clone(), msg);
                        }
                        Err(error) => {
                            log_info(
                                "eventbus",
                                format!("[{}] Connection closed: {}", env_id_for_loop, error),
                            );

                            {
                                let mut guard = connections.write().await;
                                guard.remove(&env_id_for_loop);
                            }

                            {
                                let mut pending = pending_requests.write().await;
                                pending.remove(&env_id_for_loop);
                            }

                            {
                                let status = status_handler.read().await.clone();
                                if let Some(status) = status {
                                    status(EnvConnectionPayload {
                                        env_id: env_id_for_loop.clone(),
                                        status: "disconnected".into(),
                                    });
                                }
                            }

                            if let Some(disconnect_handler) = &disconnect_handler {
                                disconnect_handler(env_id_for_loop.clone());
                            }

                            break;
                        }
                    }
                }
            });
        }

        Ok(())
    }

    pub fn start_server(
        self: Arc<Self>,
        env_id: String,
        launch_config: Option<LaunchConfig>,
    ) -> mpsc::Receiver<Result<()>> {
        let (tx, rx) = mpsc::channel(1);

        tokio::spawn(async move {
            tokio::select! {
                result = self.create_server(&env_id, launch_config) => {
                    let _ = tx.send(result).await;
                }
                _ = tx.closed() => {
                    log_warn(
                        "eventbus",
                        format!("[{}] Startup listener cancelled", env_id),
                    );
                }
            }
        });

        rx
    }

    pub async fn send(&self, env_id: &str, msg: &Message) -> Result<()> {
        let connections = self.connections.read().await;
        let connection = connections
            .get(env_id)
            .ok_or_else(|| EventBusError::NotConnected(format!("env {} not connected", env_id)))?;
        connection.send(msg).await
    }

    pub async fn send_event(&self, env_id: &str, topic: Topic, data: Vec<u8>) -> Result<()> {
        let message = Message::event(topic, data);
        self.send(env_id, &message).await
    }

    pub async fn send_request(&self, env_id: &str, topic: Topic, data: Vec<u8>) -> Result<Message> {
        let connections = self.connections.read().await;
        let connection = connections
            .get(env_id)
            .ok_or_else(|| EventBusError::NotConnected(format!("env {} not connected", env_id)))?
            .clone();
        drop(connections);

        let message = Message::request(topic, data);
        let msg_id = message.msg_id;
        let (tx, rx) = oneshot::channel();

        {
            let mut pending = self.pending_requests.write().await;
            pending.entry(env_id.to_string()).or_default().insert(msg_id, tx);
        }

        if let Err(error) = connection.send(&message).await {
            let mut pending = self.pending_requests.write().await;
            if let Some(waiters) = pending.get_mut(env_id) {
                waiters.remove(&msg_id);
            }
            return Err(error);
        }

        rx.await.map_err(|error| {
            EventBusError::ReceiveFailed(format!(
                "failed to receive response for env {} msg {}: {}",
                env_id, msg_id, error
            ))
        })
    }

    pub async fn broadcast(&self, topic: Topic, data: Vec<u8>) -> Vec<(String, Result<()>)> {
        let connections = self.connections.read().await;
        let message = Message::event(topic, data);
        let mut results = Vec::new();
        for (env_id, connection) in connections.iter() {
            let result = connection.send(&message).await;
            if let Err(error) = &result {
                log_warn(
                    "eventbus",
                    format!("Broadcast to {} failed: {}", env_id, error),
                );
            }
            results.push((env_id.clone(), result));
        }
        results
    }

    pub async fn connected_envs(&self) -> Vec<String> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }

    pub async fn connected_env_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    pub async fn is_connected(&self, env_id: &str) -> bool {
        let connections = self.connections.read().await;
        connections.contains_key(env_id)
    }

    pub async fn disconnect(&self, env_id: &str) -> Result<()> {
        let connections = self.connections.read().await;
        if let Some(connection) = connections.get(env_id) {
            let msg = Message::event(Topic::Disconnect, vec![]);
            let result = connection.send(&msg).await;
            if let Err(error) = &result {
                log_warn(
                    "eventbus",
                    format!("Disconnect event to {} failed: {}", env_id, error),
                );
            }
            result
        } else {
            Err(EventBusError::NotConnected(format!(
                "env {} not connected",
                env_id
            )))
        }
    }

    pub async fn disconnect_all(&self) {
        let mut connections = self.connections.write().await;
        let mut pending_requests = self.pending_requests.write().await;
        for (env_id, connection) in connections.drain() {
            let message = Message::event(Topic::Disconnect, vec![]);
            let _ = connection.send(&message).await;
            pending_requests.remove(&env_id);
            log_info("eventbus", format!("[{}] Disconnected", env_id));
        }
    }

    async fn get_auth_info(&self) -> AuthInfo {
        let provider = self.auth_info_provider.read().await.clone();
        match provider {
            Some(provider) => provider(),
            None => AuthInfo {
                is_authenticated: false,
                access_token: None,
                user_info: None,
            },
        }
    }

    async fn handle_auth_request(conn: &Arc<BrowserConnection>, msg: Message) -> Result<()> {
        let manager = crate::infrastructure::eventbus::get_eventbus_manager().ok_or_else(|| {
            EventBusError::NotConnected("eventbus manager not initialized".into())
        })?;
        let auth_info = manager.get_auth_info().await;

        let auth_data = serde_json::to_vec(&auth_info)
            .map_err(|error| EventBusError::Serialization(error.to_string()))?;
        let response = Message::success_response(msg.msg_id, Topic::AuthResponse, auth_data);
        conn.send(&response).await?;
        Ok(())
    }

    pub async fn notify_auth_status_changed(&self) {
        let connected_envs = self.connected_envs().await;
        let auth_info = self.get_auth_info().await;

        let auth_data = match serde_json::to_vec(&auth_info) {
            Ok(data) => data,
            Err(error) => {
                log_error(
                    "eventbus",
                    format!("Failed to serialize auth info: {}", error),
                );
                return;
            }
        };

        for env_id in connected_envs {
            if let Err(error) =
                self.send_event(&env_id, Topic::AuthResponse, auth_data.clone()).await
            {
                log_warn(
                    "eventbus",
                    format!("Failed to send auth status change to {}: {}", env_id, error),
                );
            }
        }
    }
}

impl Default for EventBusManager {
    fn default() -> Self {
        Self::new()
    }
}
