pub mod types;

use crate::app::{
    EventPublisher, ModuleContext, ModuleHealthSnapshot, Result, RuntimeContext, RuntimeError,
    RuntimeModule,
};
use crate::infrastructure::eventbus::{Topic, get_eventbus_manager};
use async_trait::async_trait;
use serde_json::json;
use tokio::sync::RwLock;

use self::types::{RunningEnvironment, SyncCommandRequest, SyncCommandResponse};

#[derive(Debug, Clone, Copy)]
enum SyncModulePhase {
    Dormant,
    RuntimeStarted,
    ContextReady,
    RuntimeStopped,
}

impl SyncModulePhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Dormant => "dormant",
            Self::RuntimeStarted => "runtime_started",
            Self::ContextReady => "context_ready",
            Self::RuntimeStopped => "runtime_stopped",
        }
    }
}

struct SyncModuleState {
    phase: SyncModulePhase,
    sync_running: bool,
    master_env_id: Option<String>,
    slave_env_ids: Vec<String>,
    last_error: Option<String>,
}

pub struct SyncRuntimeModule {
    state: RwLock<SyncModuleState>,
    events: RwLock<Option<EventPublisher>>,
}

impl SyncRuntimeModule {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(SyncModuleState {
                phase: SyncModulePhase::Dormant,
                sync_running: false,
                master_env_id: None,
                slave_env_ids: Vec::new(),
                last_error: None,
            }),
            events: RwLock::new(None),
        }
    }

    pub async fn execute_command(
        &self,
        command: SyncCommandRequest,
    ) -> Result<SyncCommandResponse> {
        let phase = self.state.read().await.phase;
        if !matches!(phase, SyncModulePhase::ContextReady) {
            return Err(RuntimeError::InvalidState(
                "sync runtime requires initialized context".into(),
            ));
        }

        match command {
            SyncCommandRequest::GetRunningEnvironments => {
                let environments = self.get_running_environments().await;
                Ok(SyncCommandResponse::RunningEnvironments { environments })
            }
            SyncCommandRequest::StartSync {
                master_env_id,
                slave_env_ids,
            } => {
                self.start_sync(master_env_id, slave_env_ids).await?;
                Ok(SyncCommandResponse::Ack)
            }
            SyncCommandRequest::StopSync => {
                self.stop_sync().await?;
                Ok(SyncCommandResponse::Ack)
            }
        }
    }

    async fn get_running_environments(&self) -> Vec<RunningEnvironment> {
        match get_eventbus_manager() {
            Some(manager) => manager
                .connected_envs()
                .await
                .into_iter()
                .map(|uuid| RunningEnvironment {
                    name: uuid.clone(),
                    uuid,
                    status: "running".to_string(),
                })
                .collect(),
            None => Vec::new(),
        }
    }

    async fn start_sync(&self, master_env_id: String, slave_env_ids: Vec<String>) -> Result<()> {
        let manager = get_eventbus_manager().ok_or_else(|| {
            RuntimeError::InvalidState("eventbus manager is not initialized".into())
        })?;

        manager.set_sync_state(Some(master_env_id.clone()), slave_env_ids.clone()).await;

        let _ = manager.send_event(&master_env_id, Topic::SyncRole, vec![1u8]).await;

        for slave_id in &slave_env_ids {
            let _ = manager.send_event(slave_id, Topic::SyncRole, vec![2u8]).await;
        }

        {
            let mut state = self.state.write().await;
            state.sync_running = true;
            state.master_env_id = Some(master_env_id.clone());
            state.slave_env_ids = slave_env_ids.clone();
            state.last_error = None;
        }

        Ok(())
    }

    async fn stop_sync(&self) -> Result<()> {
        let manager = get_eventbus_manager().ok_or_else(|| {
            RuntimeError::InvalidState("eventbus manager is not initialized".into())
        })?;

        let (master, slaves) = manager.get_sync_state().await;
        let mut to_notify = Vec::new();
        if let Some(master_env_id) = master.clone() {
            to_notify.push(master_env_id);
        }
        to_notify.extend(slaves.clone());

        for env_id in &to_notify {
            let _ = manager.send_event(env_id, Topic::SyncRole, vec![0u8]).await;
        }
        manager.set_sync_state(None, vec![]).await;

        {
            let mut state = self.state.write().await;
            state.sync_running = false;
            state.master_env_id = None;
            state.slave_env_ids.clear();
            state.last_error = None;
        }

        Ok(())
    }
}

#[async_trait]
impl RuntimeModule for SyncRuntimeModule {
    fn name(&self) -> &'static str {
        "sync"
    }

    async fn on_runtime_start(&self, context: ModuleContext) -> Result<()> {
        {
            let mut events = self.events.write().await;
            *events = Some(context.events.clone());
        }
        let mut state = self.state.write().await;
        state.phase = SyncModulePhase::RuntimeStarted;
        state.sync_running = false;
        state.master_env_id = None;
        state.slave_env_ids.clear();
        state.last_error = None;
        Ok(())
    }

    async fn on_context_initialize(&self, _context: RuntimeContext) -> Result<()> {
        let mut state = self.state.write().await;
        if !matches!(state.phase, SyncModulePhase::RuntimeStarted) {
            return Err(RuntimeError::InvalidState(
                "sync module requires runtime_started before context init".into(),
            ));
        }
        state.phase = SyncModulePhase::ContextReady;
        state.sync_running = false;
        state.master_env_id = None;
        state.slave_env_ids.clear();
        state.last_error = None;
        Ok(())
    }

    async fn on_context_destroy(&self) -> Result<()> {
        self.stop_sync().await?;
        let mut state = self.state.write().await;
        state.phase = SyncModulePhase::RuntimeStarted;
        state.sync_running = false;
        state.master_env_id = None;
        state.slave_env_ids.clear();
        Ok(())
    }

    async fn on_runtime_shutdown(&self) -> Result<()> {
        self.stop_sync().await?;
        let mut state = self.state.write().await;
        state.phase = SyncModulePhase::RuntimeStopped;
        state.sync_running = false;
        state.master_env_id = None;
        state.slave_env_ids.clear();
        state.last_error = None;
        {
            let mut events = self.events.write().await;
            *events = None;
        }
        Ok(())
    }

    async fn health_snapshot(&self) -> ModuleHealthSnapshot {
        let state = self.state.read().await;
        let connected_env_count = match get_eventbus_manager() {
            Some(manager) => manager.connected_env_count().await,
            None => 0,
        };
        ModuleHealthSnapshot {
            name: self.name().into(),
            phase: state.phase.as_str().into(),
            healthy: state.last_error.is_none(),
            detail: json!({
                "sync_running": state.sync_running,
                "master_env_id": state.master_env_id,
                "slave_env_ids": state.slave_env_ids,
                "connected_env_count": connected_env_count,
                "last_error": state.last_error,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{ModuleContext, RuntimeContextInput, event_channel};
    use crate::infrastructure::eventbus::init_eventbus_manager;

    #[tokio::test]
    async fn sync_commands_update_eventbus_sync_state() {
        let (events, _rx) = event_channel();
        let manager = init_eventbus_manager(events.clone()).await;
        manager.set_sync_state(None, vec![]).await;

        let module = SyncRuntimeModule::new();
        module
            .on_runtime_start(ModuleContext {
                events: events.clone(),
            })
            .await
            .unwrap();
        module
            .on_context_initialize(RuntimeContext::new(1, RuntimeContextInput::default()))
            .await
            .unwrap();

        let response = module
            .execute_command(SyncCommandRequest::StartSync {
                master_env_id: "master-1".into(),
                slave_env_ids: vec!["slave-1".into(), "slave-2".into()],
            })
            .await
            .unwrap();
        assert!(matches!(response, SyncCommandResponse::Ack));

        let (master, slaves) = manager.get_sync_state().await;
        assert_eq!(master.as_deref(), Some("master-1"));
        assert_eq!(slaves, vec!["slave-1".to_string(), "slave-2".to_string()]);

        let response = module.execute_command(SyncCommandRequest::StopSync).await.unwrap();
        assert!(matches!(response, SyncCommandResponse::Ack));

        let (master, slaves) = manager.get_sync_state().await;
        assert_eq!(master, None);
        assert!(slaves.is_empty());
    }
}
