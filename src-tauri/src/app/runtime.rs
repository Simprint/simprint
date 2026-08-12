use std::collections::BTreeMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, RwLock};

use crate::app::context::AppContext;
use crate::domain::environment::EnvironmentStatus;
use crate::infrastructure::runtime::{
    AuthCommandRequest, AuthInfo, AuthResponse, EmptyPayload, EnvConnectionPayload,
    EnvironmentCommandRequest, EnvironmentCommandResponse, EnvironmentResponse,
    InitializeContextRequest, Message, RuntimeContextInput, StateResponse, SyncCommandRequest,
    SyncCommandResponse, SyncResponse, Topic,
};

/// The environment runtime now lives in this process. The wire protocol is kept as a
/// compatibility adapter for the existing service layer, but no bytes are written to a child
/// process and no runtime executable is started or updated independently.
struct ManagedRuntime {
    host: Arc<runtime::app::RuntimeHost>,
    context_initialized: AtomicBool,
    context_init_lock: Mutex<()>,
    event_task: tokio::task::JoinHandle<()>,
}

pub struct SimprintRuntimeManager {
    app_handle: RwLock<Option<AppHandle>>,
    handle: Mutex<Option<Arc<ManagedRuntime>>>,
}

impl SimprintRuntimeManager {
    pub fn new() -> Self {
        Self {
            app_handle: RwLock::new(None),
            handle: Mutex::new(None),
        }
    }

    pub async fn set_app_handle(&self, app_handle: AppHandle) {
        *self.app_handle.write().await = Some(app_handle);
    }

    pub async fn is_running(&self) -> bool {
        self.handle.lock().await.is_some()
    }

    pub async fn send_environment_command(
        self: &Arc<Self>,
        command: EnvironmentCommandRequest,
    ) -> crate::core::error::Result<EnvironmentCommandResponse> {
        self.ensure_context_ready().await?;
        let message = Message::request_payload(Topic::EnvironmentCommand, &command)
            .map_err(runtime_err_to_string)?;
        let response = self.request(message).await?;
        let payload: EnvironmentResponse = response.payload().map_err(runtime_err_to_string)?;
        Ok(payload.result)
    }

    pub async fn send_sync_command(
        self: &Arc<Self>,
        command: SyncCommandRequest,
    ) -> crate::core::error::Result<SyncCommandResponse> {
        self.ensure_context_ready().await?;
        let message = Message::request_payload(Topic::SyncCommand, &command)
            .map_err(runtime_err_to_string)?;
        let response = self.request(message).await?;
        let payload: SyncResponse = response.payload().map_err(runtime_err_to_string)?;
        Ok(payload.result)
    }

    pub async fn sync_session_state(self: &Arc<Self>) -> crate::core::error::Result<()> {
        if !is_runtime_authenticated() {
            self.stop().await;
            return Ok(());
        }

        self.start_background().await?;
        self.ensure_context_ready().await?;

        let command = AuthCommandRequest::SetAuthState {
            auth_info: current_auth_info(),
        };
        let message = Message::request_payload(Topic::AuthCommand, &command)
            .map_err(runtime_err_to_string)?;
        let response = self.request(message).await?;
        let _: AuthResponse = response.payload().map_err(runtime_err_to_string)?;
        Ok(())
    }

    pub async fn start_background(self: &Arc<Self>) -> crate::core::error::Result<()> {
        self.start_if_needed().await
    }

    pub async fn stop(&self) {
        let runtime = self.handle.lock().await.take();
        let Some(runtime) = runtime else {
            return;
        };

        if let Ok(message) = Message::request_payload(Topic::Shutdown, &EmptyPayload::default()) {
            if let Err(error) = Self::request_with_runtime(runtime.clone(), message).await {
                log::warn!("failed to stop embedded runtime cleanly: {}", error);
            }
        }
        runtime.event_task.abort();
    }

    async fn start_if_needed(self: &Arc<Self>) -> crate::core::error::Result<()> {
        let mut guard = self.handle.lock().await;
        if guard.is_some() {
            return Ok(());
        }

        let (events, mut event_rx) = runtime::app::event_channel();
        let host = runtime::app::RuntimeHost::default(events);
        host.start()
            .await
            .map_err(|error| format!("failed to start embedded runtime: {error}"))?;

        let manager = Arc::clone(self);
        let event_task = tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                manager.handle_runtime_event(event.name, event.payload).await;
            }
        });

        *guard = Some(Arc::new(ManagedRuntime {
            host,
            context_initialized: AtomicBool::new(false),
            context_init_lock: Mutex::new(()),
            event_task,
        }));
        log::info!("embedded simprint runtime started");
        Ok(())
    }

    async fn ensure_context_ready(self: &Arc<Self>) -> crate::core::error::Result<()> {
        if !is_runtime_authenticated() {
            return Err("当前未登录，无法初始化环境运行时".into());
        }

        self.start_if_needed().await?;

        let runtime = self.runtime().await.ok_or("内嵌环境运行时未启动")?;
        let _init_guard = runtime.context_init_lock.lock().await;
        if runtime.context_initialized.load(Ordering::SeqCst) {
            return Ok(());
        }

        let message = Message::request_payload(
            Topic::InitializeContext,
            &InitializeContextRequest {
                context: RuntimeContextInput {
                    user_id: None,
                    workspace_id: None,
                    auth_info: Some(current_auth_info()),
                    attributes: BTreeMap::new(),
                },
            },
        )
        .map_err(runtime_err_to_string)?;
        let response = Self::request_with_runtime(runtime.clone(), message).await?;
        let _: StateResponse = response.payload().map_err(runtime_err_to_string)?;
        runtime.context_initialized.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn request(self: &Arc<Self>, message: Message) -> crate::core::error::Result<Message> {
        let runtime = self.runtime().await.ok_or("内嵌环境运行时未启动")?;
        Self::request_with_runtime(runtime, message).await
    }

    async fn request_with_runtime(
        runtime: Arc<ManagedRuntime>,
        message: Message,
    ) -> crate::core::error::Result<Message> {
        let request = to_embedded_message(message);
        let dispatch = runtime
            .host
            .handle_request(request)
            .await
            .map_err(|error| format!("环境运行时请求失败: {error}"))?;
        Ok(from_embedded_message(dispatch.response))
    }

    async fn runtime(&self) -> Option<Arc<ManagedRuntime>> {
        self.handle.lock().await.clone()
    }

    async fn handle_runtime_event(&self, name: String, payload: serde_json::Value) {
        if let Some(app_handle) = self.app_handle.read().await.clone() {
            let _ = app_handle.emit(&name, payload.clone());

            if name == "eventbus.connection_status" {
                if let Ok(connection) = serde_json::from_value::<EnvConnectionPayload>(payload) {
                    let _ = app_handle.emit("env-connection-status", connection.clone());
                    if let Some(ctx) = AppContext::try_get() {
                        match connection.status.as_str() {
                            "disconnected" => {
                                ctx.env_status_manager
                                    .set_stopped_unless_error(&connection.env_id)
                                    .await;
                                ctx.env_position_manager.release_position(&connection.env_id).await;
                            }
                            _ => {}
                        }
                    }
                }
                return;
            }
        }

        if let Some(ctx) = AppContext::try_get() {
            match name.as_str() {
                "environment.stopped"
                | "environment.disconnected"
                | "environment.browser_disconnected" => {
                    if let Some(env_uuid) = payload.get("env_uuid").and_then(|value| value.as_str())
                    {
                        ctx.env_status_manager.set_stopped_unless_error(env_uuid).await;
                        ctx.env_position_manager.release_position(env_uuid).await;
                    }
                }
                "environment.launch_failed" => {
                    if let Some(env_uuid) = payload.get("env_uuid").and_then(|value| value.as_str())
                    {
                        ctx.env_status_manager.set_status(env_uuid, EnvironmentStatus::Error).await;
                    }
                }
                "environment.launch_ready" => {
                    if let Some(env_uuid) = payload.get("env_uuid").and_then(|value| value.as_str())
                    {
                        ctx.env_status_manager
                            .set_status(env_uuid, EnvironmentStatus::Running)
                            .await;
                    }
                }
                _ => {}
            }
        }
    }
}

fn to_embedded_message(message: Message) -> runtime::infrastructure::ipc::Message {
    use runtime::infrastructure::ipc::{Message as EmbeddedMessage, MessageType, Topic};

    let msg_type = match message.msg_type {
        crate::infrastructure::runtime::MessageType::Request => MessageType::Request,
        crate::infrastructure::runtime::MessageType::Response => MessageType::Response,
        crate::infrastructure::runtime::MessageType::Event => MessageType::Event,
    };

    EmbeddedMessage {
        msg_id: message.msg_id,
        msg_type,
        topic: Topic::from(u16::from(message.topic)),
        error_code: message.error_code,
        data: message.data,
    }
}

fn from_embedded_message(message: runtime::infrastructure::ipc::Message) -> Message {
    let msg_type = match message.msg_type {
        runtime::infrastructure::ipc::MessageType::Request => {
            crate::infrastructure::runtime::MessageType::Request
        }
        runtime::infrastructure::ipc::MessageType::Response => {
            crate::infrastructure::runtime::MessageType::Response
        }
        runtime::infrastructure::ipc::MessageType::Event => {
            crate::infrastructure::runtime::MessageType::Event
        }
    };

    Message {
        msg_id: message.msg_id,
        msg_type,
        topic: Topic::from(u16::from(message.topic)),
        error_code: message.error_code,
        data: message.data,
    }
}

fn runtime_err_to_string(
    error: crate::infrastructure::runtime::RuntimeIpcError,
) -> crate::core::error::Error {
    error.to_string().into()
}

fn current_auth_info() -> AuthInfo {
    use crate::infrastructure::persistence::credential::{get_credential, is_login};

    if !is_login() {
        return AuthInfo {
            is_authenticated: false,
            access_token: None,
            user_info: None,
        };
    }

    let credential = get_credential();
    AuthInfo {
        is_authenticated: true,
        access_token: credential.get_access_token(),
        user_info: None,
    }
}

fn is_runtime_authenticated() -> bool {
    crate::infrastructure::persistence::credential::is_login()
}

impl Default for SimprintRuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}
