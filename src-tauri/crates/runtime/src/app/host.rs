use super::api::{
    AuthResponse, DestroyContextRequest, EmptyPayload, EnvironmentResponse, ErrorResponse,
    HandshakeRequest, HandshakeResponse, HealthResponse, InitializeContextRequest, PingResponse,
    StateResponse, SyncResponse,
};
use super::context::RuntimeContext;
use super::error::{Result, RuntimeError};
use super::events::EventPublisher;
use super::module::{ModuleContext, ModuleOrchestrator};
use super::state::{HealthSnapshot, RuntimePhase, RuntimeStateStore};
use crate::infrastructure::diagnostics::{log_error, log_info};
use crate::infrastructure::ipc::{Message, PROTOCOL_VERSION, Topic};
use crate::services::auth::types::AuthCommandRequest;
use crate::services::auth::{AuthRuntime, AuthStateStore};
use crate::services::environment::EnvironmentRuntimeModule;
use crate::services::environment::kernel::types::EnvironmentCommandRequest;
use crate::services::sync::SyncRuntimeModule;
use crate::services::sync::types::SyncCommandRequest;
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::sync::RwLock;

pub enum DispatchControl {
    Continue,
    Shutdown,
}

pub struct DispatchResult {
    pub response: Message,
    pub control: DispatchControl,
}

pub struct RuntimeHost {
    state: RuntimeStateStore,
    context: RwLock<Option<RuntimeContext>>,
    context_sequence: AtomicU64,
    modules: ModuleOrchestrator,
    events: EventPublisher,
    auth: Arc<AuthRuntime>,
    environment: Arc<EnvironmentRuntimeModule>,
    sync: Arc<SyncRuntimeModule>,
}

impl RuntimeHost {
    pub fn new(
        runtime_id: impl Into<String>,
        runtime_version: impl Into<String>,
        modules: ModuleOrchestrator,
        events: EventPublisher,
        auth: Arc<AuthRuntime>,
        environment: Arc<EnvironmentRuntimeModule>,
        sync: Arc<SyncRuntimeModule>,
    ) -> Self {
        Self {
            state: RuntimeStateStore::new(runtime_id, runtime_version),
            context: RwLock::new(None),
            context_sequence: AtomicU64::new(1),
            modules,
            events,
            auth,
            environment,
            sync,
        }
    }

    pub fn default(events: EventPublisher) -> Arc<Self> {
        let auth_state = Arc::new(AuthStateStore::new());
        let auth = Arc::new(AuthRuntime::new(auth_state.clone()));
        let environment = Arc::new(EnvironmentRuntimeModule::new(auth_state));
        let sync = Arc::new(SyncRuntimeModule::new());
        let modules = ModuleOrchestrator::new()
            .register(auth.clone())
            .register(environment.clone())
            .register(sync.clone());

        Arc::new(Self::new(
            "runtime",
            env!("CARGO_PKG_VERSION"),
            modules,
            events,
            auth,
            environment,
            sync,
        ))
    }

    pub async fn start(&self) -> Result<()> {
        if self.state.phase().await != RuntimePhase::Booting {
            return Err(RuntimeError::InvalidState(
                "runtime host can only start from booting phase".into(),
            ));
        }

        self.modules
            .start(ModuleContext {
                events: self.events.clone(),
            })
            .await?;
        self.state.clear_error().await;
        self.state.transition(RuntimePhase::Uninitialized).await;
        self.events.emit("runtime.started", &json!({}))?;
        log_info("runtime", "runtime host started");
        Ok(())
    }

    pub async fn handle_request(&self, request: Message) -> Result<DispatchResult> {
        match request.topic {
            Topic::Handshake => self.handle_handshake(request).await,
            Topic::Ping => self.handle_ping(request).await,
            Topic::QueryState => self.handle_query_state(request).await,
            Topic::QueryHealth => self.handle_query_health(request).await,
            Topic::InitializeContext => self.handle_initialize_context(request).await,
            Topic::DestroyContext => self.handle_destroy_context(request).await,
            Topic::Shutdown => self.handle_shutdown(request).await,
            Topic::EnvironmentCommand => self.handle_environment_command(request).await,
            Topic::SyncCommand => self.handle_sync_command(request).await,
            Topic::AuthCommand => self.handle_auth_command(request).await,
            Topic::RuntimeEvent => Err(RuntimeError::InvalidState(
                "runtime does not accept inbound runtime_event frames".into(),
            )),
            Topic::Unknown(value) => Err(RuntimeError::InvalidState(format!(
                "unknown topic: {}",
                value
            ))),
        }
    }

    pub async fn shutdown_due_to_disconnect(&self) -> Result<()> {
        log_info("runtime", "peer disconnected; shutting down runtime");
        self.shutdown_runtime().await
    }

    async fn handle_handshake(&self, request: Message) -> Result<DispatchResult> {
        let _payload: HandshakeRequest = request.payload()?;
        let response = HandshakeResponse {
            protocol_version: PROTOCOL_VERSION,
            runtime_version: self.state.runtime_version().to_string(),
            runtime_id: self.state.runtime_id().to_string(),
        };
        Ok(DispatchResult {
            response: Message::success_response_payload(
                request.msg_id,
                Topic::Handshake,
                &response,
            )?,
            control: DispatchControl::Continue,
        })
    }

    async fn handle_ping(&self, request: Message) -> Result<DispatchResult> {
        let _: EmptyPayload = request.payload()?;
        let snapshot = self.state.snapshot(self.modules.len()).await;
        let response = PingResponse {
            runtime_id: snapshot.runtime_id.clone(),
            phase: snapshot.phase,
            uptime_ms: snapshot.uptime_ms,
        };
        Ok(DispatchResult {
            response: Message::success_response_payload(request.msg_id, Topic::Ping, &response)?,
            control: DispatchControl::Continue,
        })
    }

    async fn handle_query_state(&self, request: Message) -> Result<DispatchResult> {
        let _: EmptyPayload = request.payload()?;
        let response = StateResponse {
            state: self.state.snapshot(self.modules.len()).await,
        };
        Ok(DispatchResult {
            response: Message::success_response_payload(
                request.msg_id,
                Topic::QueryState,
                &response,
            )?,
            control: DispatchControl::Continue,
        })
    }

    async fn handle_query_health(&self, request: Message) -> Result<DispatchResult> {
        let _: EmptyPayload = request.payload()?;
        let modules = self.modules.health_snapshot().await;
        let response = HealthResponse {
            health: HealthSnapshot {
                runtime: self.state.snapshot(self.modules.len()).await,
                modules,
            },
        };
        Ok(DispatchResult {
            response: Message::success_response_payload(
                request.msg_id,
                Topic::QueryHealth,
                &response,
            )?,
            control: DispatchControl::Continue,
        })
    }

    async fn handle_initialize_context(&self, request: Message) -> Result<DispatchResult> {
        let payload: InitializeContextRequest = request.payload()?;
        let state_phase = self.state.phase().await;
        if state_phase == RuntimePhase::Ready {
            return Err(RuntimeError::AlreadyInitialized);
        }
        if state_phase != RuntimePhase::Uninitialized {
            return Err(RuntimeError::InvalidState(format!(
                "cannot initialize context from phase {:?}",
                state_phase
            )));
        }

        self.state.transition(RuntimePhase::Initializing).await;
        self.state.clear_error().await;

        let sequence = self.context_sequence.fetch_add(1, Ordering::SeqCst);
        let context = RuntimeContext::new(sequence, payload.context);

        if let Err(error) = self.modules.initialize_context(context.clone()).await {
            self.state.record_error(error.to_string()).await;
            self.state.transition(RuntimePhase::Uninitialized).await;
            return Err(error);
        }

        {
            let mut guard = self.context.write().await;
            *guard = Some(context.clone());
        }
        self.state.attach_context(context.context_id.clone()).await;
        self.state.transition(RuntimePhase::Ready).await;
        self.events.emit(
            "runtime.context_initialized",
            &json!({ "context_id": context.context_id }),
        )?;

        let response = StateResponse {
            state: self.state.snapshot(self.modules.len()).await,
        };
        Ok(DispatchResult {
            response: Message::success_response_payload(
                request.msg_id,
                Topic::InitializeContext,
                &response,
            )?,
            control: DispatchControl::Continue,
        })
    }

    async fn handle_destroy_context(&self, request: Message) -> Result<DispatchResult> {
        let _payload: DestroyContextRequest = request.payload()?;
        self.destroy_context().await?;

        let response = StateResponse {
            state: self.state.snapshot(self.modules.len()).await,
        };
        Ok(DispatchResult {
            response: Message::success_response_payload(
                request.msg_id,
                Topic::DestroyContext,
                &response,
            )?,
            control: DispatchControl::Continue,
        })
    }

    async fn handle_shutdown(&self, request: Message) -> Result<DispatchResult> {
        let _: EmptyPayload = request.payload()?;
        self.shutdown_runtime().await?;

        let response = StateResponse {
            state: self.state.snapshot(self.modules.len()).await,
        };
        Ok(DispatchResult {
            response: Message::success_response_payload(
                request.msg_id,
                Topic::Shutdown,
                &response,
            )?,
            control: DispatchControl::Shutdown,
        })
    }

    async fn handle_environment_command(&self, request: Message) -> Result<DispatchResult> {
        let command: EnvironmentCommandRequest = request.payload()?;
        let result = self.environment.execute_command(command).await?;
        let response = EnvironmentResponse { result };
        Ok(DispatchResult {
            response: Message::success_response_payload(
                request.msg_id,
                Topic::EnvironmentCommand,
                &response,
            )?,
            control: DispatchControl::Continue,
        })
    }

    async fn handle_sync_command(&self, request: Message) -> Result<DispatchResult> {
        let command: SyncCommandRequest = request.payload()?;
        let result = self.sync.execute_command(command).await?;
        let response = SyncResponse { result };
        Ok(DispatchResult {
            response: Message::success_response_payload(
                request.msg_id,
                Topic::SyncCommand,
                &response,
            )?,
            control: DispatchControl::Continue,
        })
    }

    async fn handle_auth_command(&self, request: Message) -> Result<DispatchResult> {
        let command: AuthCommandRequest = request.payload()?;
        let result = self.auth.execute_command(command).await?;
        let response = AuthResponse { result };
        Ok(DispatchResult {
            response: Message::success_response_payload(
                request.msg_id,
                Topic::AuthCommand,
                &response,
            )?,
            control: DispatchControl::Continue,
        })
    }

    async fn destroy_context(&self) -> Result<()> {
        let context_id = {
            let guard = self.context.read().await;
            guard
                .as_ref()
                .map(|context| context.context_id.clone())
                .ok_or(RuntimeError::NotInitialized)?
        };

        self.state.transition(RuntimePhase::Destroying).await;
        if let Err(error) = self.modules.destroy_context().await {
            self.state.record_error(error.to_string()).await;
            self.state.transition(RuntimePhase::Ready).await;
            return Err(error);
        }

        {
            let mut guard = self.context.write().await;
            *guard = None;
        }
        self.state.clear_context().await;
        self.state.transition(RuntimePhase::Uninitialized).await;
        self.events.emit(
            "runtime.context_destroyed",
            &json!({ "context_id": context_id }),
        )?;
        Ok(())
    }

    async fn shutdown_runtime(&self) -> Result<()> {
        let phase = self.state.phase().await;
        if matches!(phase, RuntimePhase::Stopped | RuntimePhase::ShuttingDown) {
            return Ok(());
        }

        self.state.transition(RuntimePhase::ShuttingDown).await;

        if self.context.read().await.is_some() {
            if let Err(error) = self.modules.destroy_context().await {
                log_error(
                    "runtime",
                    format!("context destroy during shutdown failed: {}", error),
                );
                self.state.record_error(error.to_string()).await;
            }
            let mut guard = self.context.write().await;
            *guard = None;
            self.state.clear_context().await;
        }

        if let Err(error) = self.modules.shutdown().await {
            self.state.record_error(error.to_string()).await;
            self.state.transition(RuntimePhase::Failed).await;
            return Err(error);
        }

        self.state.transition(RuntimePhase::Stopped).await;
        self.events.emit("runtime.stopped", &json!({}))?;
        log_info("runtime", "runtime host stopped");
        Ok(())
    }

    pub fn error_response_for(&self, request: &Message, error: &RuntimeError) -> Result<Message> {
        Message::error_response_payload(
            request.msg_id,
            request.topic,
            error.code(),
            &ErrorResponse {
                message: error.to_string(),
            },
        )
        .map_err(RuntimeError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{EmptyPayload, InitializeContextRequest, event_channel};
    use crate::infrastructure::ipc::{Message, Topic};

    #[tokio::test]
    async fn host_supports_an_in_process_lifecycle() {
        let (events, _event_rx) = event_channel();
        let auth_state = Arc::new(AuthStateStore::new());
        let host = Arc::new(RuntimeHost::new(
            "test-runtime",
            "0.0.0-test",
            ModuleOrchestrator::new(),
            events,
            Arc::new(AuthRuntime::new(auth_state.clone())),
            Arc::new(EnvironmentRuntimeModule::new(auth_state)),
            Arc::new(SyncRuntimeModule::new()),
        ));
        host.start().await.unwrap();

        let initialize = Message::request_payload(
            Topic::InitializeContext,
            &InitializeContextRequest {
                context: Default::default(),
            },
        )
        .unwrap();
        let initialized = host.handle_request(initialize).await.unwrap();
        let state: StateResponse = initialized.response.payload().unwrap();
        assert_eq!(state.state.phase, RuntimePhase::Ready);

        let shutdown = Message::request_payload(Topic::Shutdown, &EmptyPayload::default()).unwrap();
        let stopped = host.handle_request(shutdown).await.unwrap();
        let state: StateResponse = stopped.response.payload().unwrap();
        assert_eq!(state.state.phase, RuntimePhase::Stopped);
    }
}
