pub mod kernel;
pub mod status;
pub mod status_manager;

use crate::app::{
    EventPublisher, ModuleContext, ModuleHealthSnapshot, Result, RuntimeContext, RuntimeError,
    RuntimeModule,
};
use crate::infrastructure::eventbus::{get_eventbus_manager, init_eventbus_manager};
use crate::services::auth::AuthStateStore;
use async_trait::async_trait;
use kernel::KernelRuntime;
use kernel::types::{EnvironmentCommandRequest, EnvironmentCommandResponse};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use status::EnvironmentStatus;
pub use status_manager::EnvironmentStatusManager;

#[derive(Debug, Clone, Copy)]
enum EnvironmentModulePhase {
    Dormant,
    RuntimeStarted,
    ContextReady,
    RuntimeStopped,
}

impl EnvironmentModulePhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Dormant => "dormant",
            Self::RuntimeStarted => "runtime_started",
            Self::ContextReady => "context_ready",
            Self::RuntimeStopped => "runtime_stopped",
        }
    }
}

struct EnvironmentModuleState {
    phase: EnvironmentModulePhase,
    active_environments: u32,
    last_error: Option<String>,
}

pub struct EnvironmentRuntimeModule {
    state: RwLock<EnvironmentModuleState>,
    auth_state: Arc<AuthStateStore>,
    status_manager: Arc<EnvironmentStatusManager>,
    kernel_runtime: Arc<KernelRuntime>,
    events: RwLock<Option<EventPublisher>>,
}

impl EnvironmentRuntimeModule {
    pub fn new(auth_state: Arc<AuthStateStore>) -> Self {
        let status_manager = Arc::new(EnvironmentStatusManager::new());
        Self {
            state: RwLock::new(EnvironmentModuleState {
                phase: EnvironmentModulePhase::Dormant,
                active_environments: 0,
                last_error: None,
            }),
            auth_state,
            status_manager: status_manager.clone(),
            kernel_runtime: Arc::new(KernelRuntime::new(status_manager)),
            events: RwLock::new(None),
        }
    }

    pub async fn execute_command(
        &self,
        command: EnvironmentCommandRequest,
    ) -> Result<EnvironmentCommandResponse> {
        let phase = self.state.read().await.phase;
        if !matches!(phase, EnvironmentModulePhase::ContextReady) {
            return Err(RuntimeError::InvalidState(
                "environment runtime requires initialized context".into(),
            ));
        }

        let events =
            self.events.read().await.clone().ok_or_else(|| {
                RuntimeError::InvalidState("environment runtime not started".into())
            })?;

        self.kernel_runtime.execute(command, events).await
    }
}

#[async_trait]
impl RuntimeModule for EnvironmentRuntimeModule {
    fn name(&self) -> &'static str {
        "environment"
    }

    async fn on_runtime_start(&self, context: ModuleContext) -> Result<()> {
        {
            let mut events = self.events.write().await;
            *events = Some(context.events.clone());
        }
        let manager = init_eventbus_manager(context.events.clone()).await;
        let auth_state = self.auth_state.clone();
        manager.set_auth_info_provider(move || auth_state.snapshot()).await;
        let status_manager = self.status_manager.clone();
        let event_sink = context.events.clone();
        manager
            .set_connection_status_handler(move |payload| {
                let status_manager = status_manager.clone();
                let event_sink = event_sink.clone();
                let emitted_payload = payload.clone();
                let status_value = payload.status.clone();
                let env_id = payload.env_id.clone();
                tokio::spawn(async move {
                    // A pipe connection alone is not launch readiness. The launcher marks the
                    // environment running only after the EventBus handshake and CDP are ready.
                    if status_value == "disconnected" {
                        status_manager.set_stopped_unless_error(&env_id).await;
                    }
                });

                let _ = event_sink.emit("eventbus.connection_status", &emitted_payload);
            })
            .await;
        let kernel_runtime = self.kernel_runtime.clone();
        let events = context.events.clone();
        manager
            .set_disconnect_handler(move |env_id| {
                let kernel_runtime = kernel_runtime.clone();
                let events = events.clone();
                tokio::spawn(async move {
                    kernel_runtime.handle_browser_disconnect(&env_id, events.clone()).await;
                    let _ = events.emit(
                        "environment.disconnected",
                        &serde_json::json!({ "env_uuid": env_id }),
                    );
                });
            })
            .await;

        let mut state = self.state.write().await;
        state.phase = EnvironmentModulePhase::RuntimeStarted;
        state.last_error = None;
        Ok(())
    }

    async fn on_context_initialize(&self, _context: RuntimeContext) -> Result<()> {
        let mut state = self.state.write().await;
        if !matches!(state.phase, EnvironmentModulePhase::RuntimeStarted) {
            return Err(RuntimeError::InvalidState(
                "environment module requires runtime_started before context init".into(),
            ));
        }
        state.phase = EnvironmentModulePhase::ContextReady;
        Ok(())
    }

    async fn on_context_destroy(&self) -> Result<()> {
        if let Some(manager) = get_eventbus_manager() {
            manager.disconnect_all().await;
        }
        self.kernel_runtime.clear_all().await;
        let mut state = self.state.write().await;
        state.phase = EnvironmentModulePhase::RuntimeStarted;
        state.active_environments = 0;
        Ok(())
    }

    async fn on_runtime_shutdown(&self) -> Result<()> {
        if let Some(manager) = get_eventbus_manager() {
            manager.disconnect_all().await;
        }
        self.kernel_runtime.clear_all().await;
        {
            let mut events = self.events.write().await;
            *events = None;
        }
        let mut state = self.state.write().await;
        state.phase = EnvironmentModulePhase::RuntimeStopped;
        state.active_environments = 0;
        Ok(())
    }

    async fn health_snapshot(&self) -> ModuleHealthSnapshot {
        let state = self.state.read().await;
        let connected_envs = self.kernel_runtime.get_connected_env_count().await as u32;
        ModuleHealthSnapshot {
            name: self.name().into(),
            phase: state.phase.as_str().into(),
            healthy: state.last_error.is_none(),
            detail: json!({
                "active_environments": connected_envs,
                "last_error": state.last_error,
            }),
        }
    }
}
