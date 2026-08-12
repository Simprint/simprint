use super::error::{EventBusError, Result};
use super::message::{HandshakeData, Message, MessageType};
use super::topics::Topic;
use super::transport::PipeConnection;
use crate::infrastructure::diagnostics::{log_debug, log_error, log_info, log_warn};
use std::sync::Arc;
use tokio::sync::mpsc;

pub type MessageHandler = Arc<dyn Fn(String, Message) + Send + Sync>;

pub struct BrowserConnection {
    env_id: String,
    connection: Arc<PipeConnection>,
    is_handshake_complete: bool,
}

impl BrowserConnection {
    pub(crate) fn new(connection: PipeConnection) -> Self {
        let env_id = connection.env_id().to_string();
        Self {
            env_id,
            connection: Arc::new(connection),
            is_handshake_complete: false,
        }
    }

    pub fn env_id(&self) -> &str {
        &self.env_id
    }

    pub fn is_handshake_complete(&self) -> bool {
        self.is_handshake_complete
    }

    pub async fn handshake(&mut self) -> Result<()> {
        log_info(
            "eventbus",
            format!("[{}] Waiting for handshake...", self.env_id),
        );

        let msg = self.connection.recv().await?;
        if msg.topic != Topic::Handshake {
            log_error(
                "eventbus",
                format!("[{}] Expected Handshake, got {:?}", self.env_id, msg.topic),
            );
            return Err(EventBusError::InvalidMessage(format!(
                "expected Handshake, got {:?}",
                msg.topic
            )));
        }

        let handshake_data = HandshakeData::from_bytes(&msg.data)?;
        log_info(
            "eventbus",
            format!(
                "[{}] Received handshake: version={}, client_type={}",
                self.env_id, handshake_data.version, handshake_data.client_type
            ),
        );

        if handshake_data.env_id != self.env_id {
            log_warn(
                "eventbus",
                format!(
                    "[{}] env_id mismatch: expected {}, got {}",
                    self.env_id, self.env_id, handshake_data.env_id
                ),
            );
        }

        let response_data = HandshakeData::tauri_response().to_bytes()?;
        let response = Message::success_response(msg.msg_id, Topic::Handshake, response_data);
        self.connection.send(&response).await?;

        self.is_handshake_complete = true;
        log_info("eventbus", format!("[{}] Handshake complete", self.env_id));
        Ok(())
    }

    pub async fn send(&self, msg: &Message) -> Result<()> {
        if !self.is_handshake_complete {
            return Err(EventBusError::NotConnected("handshake not complete".into()));
        }
        self.connection.send(msg).await
    }

    pub async fn send_event(&self, topic: Topic, data: Vec<u8>) -> Result<()> {
        let msg = Message::event(topic, data);
        self.send(&msg).await
    }

    pub async fn send_request(&self, topic: Topic, data: Vec<u8>) -> Result<Message> {
        let msg = Message::request(topic, data);
        let msg_id = msg.msg_id;
        self.send(&msg).await?;

        loop {
            let response = self.connection.recv().await?;
            if response.msg_type == MessageType::Response && response.msg_id == msg_id {
                return Ok(response);
            }
            log_debug(
                "eventbus",
                format!(
                    "[{}] Received unexpected message while waiting for response: {:?}",
                    self.env_id, response.topic
                ),
            );
        }
    }

    pub async fn recv(&self) -> Result<Message> {
        self.connection.recv().await
    }

    pub fn start_recv_loop(self: Arc<Self>, handler: MessageHandler) -> mpsc::Sender<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        let env_id = self.env_id.clone();

        tokio::spawn(async move {
            log_info("eventbus", format!("[{}] Starting receive loop", env_id));

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        log_info("eventbus", format!("[{}] Receive loop shutdown", env_id));
                        break;
                    }
                    result = self.connection.recv() => {
                        match result {
                            Ok(msg) => handler(env_id.clone(), msg),
                            Err(error) => {
                                log_error("eventbus", format!("[{}] Receive error: {}", env_id, error));
                                break;
                            }
                        }
                    }
                }
            }
        });

        shutdown_tx
    }
}
