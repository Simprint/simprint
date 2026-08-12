pub mod types;

use crate::app::{
    ModuleContext, ModuleHealthSnapshot, Result, RuntimeContext, RuntimeError, RuntimeModule,
};
use crate::infrastructure::eventbus::{AuthInfo, get_eventbus_manager};
use async_trait::async_trait;
use serde_json::json;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as AsyncRwLock;
use types::{AuthCommandRequest, AuthCommandResponse};

#[derive(Debug, Clone, Copy)]
enum AuthPhase {
    Dormant,
    RuntimeStarted,
    ContextReady,
    RuntimeStopped,
}

impl AuthPhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Dormant => "dormant",
            Self::RuntimeStarted => "runtime_started",
            Self::ContextReady => "context_ready",
            Self::RuntimeStopped => "runtime_stopped",
        }
    }
}

struct AuthState {
    phase: AuthPhase,
    last_error: Option<String>,
}

pub struct AuthStateStore {
    auth_info: RwLock<AuthInfo>,
}

impl AuthStateStore {
    pub fn new() -> Self {
        Self {
            auth_info: RwLock::new(anonymous_auth_info()),
        }
    }

    pub fn snapshot(&self) -> AuthInfo {
        self.auth_info.read().expect("auth state poisoned").clone()
    }

    pub fn replace(&self, auth_info: AuthInfo) {
        *self.auth_info.write().expect("auth state poisoned") = auth_info;
    }

    pub fn clear(&self) {
        self.replace(anonymous_auth_info());
    }
}

pub struct AuthRuntime {
    state: AsyncRwLock<AuthState>,
    auth_state: Arc<AuthStateStore>,
}

impl AuthRuntime {
    pub fn new(auth_state: Arc<AuthStateStore>) -> Self {
        Self {
            state: AsyncRwLock::new(AuthState {
                phase: AuthPhase::Dormant,
                last_error: None,
            }),
            auth_state,
        }
    }

    pub async fn execute_command(
        &self,
        command: AuthCommandRequest,
    ) -> Result<AuthCommandResponse> {
        let phase = self.state.read().await.phase;
        if !matches!(phase, AuthPhase::ContextReady) {
            return Err(RuntimeError::InvalidState(
                "auth runtime requires initialized context".into(),
            ));
        }

        match command {
            AuthCommandRequest::SetAuthState { auth_info } => {
                self.replace_auth_info(auth_info).await;
                Ok(AuthCommandResponse::Ack)
            }
            AuthCommandRequest::ClearAuthState => {
                self.clear_auth_info().await;
                Ok(AuthCommandResponse::Ack)
            }
            AuthCommandRequest::GetAuthState => Ok(AuthCommandResponse::State {
                auth_info: self.auth_state.snapshot(),
            }),
        }
    }

    pub fn state_store(&self) -> Arc<AuthStateStore> {
        self.auth_state.clone()
    }

    async fn replace_auth_info(&self, auth_info: AuthInfo) {
        self.auth_state.replace(auth_info);
        self.notify_auth_status_changed().await;
    }

    async fn clear_auth_info(&self) {
        self.auth_state.clear();
        self.notify_auth_status_changed().await;
    }

    async fn notify_auth_status_changed(&self) {
        if let Some(manager) = get_eventbus_manager() {
            manager.notify_auth_status_changed().await;
        }
    }
}

#[async_trait]
impl RuntimeModule for AuthRuntime {
    fn name(&self) -> &'static str {
        "auth"
    }

    async fn on_runtime_start(&self, _context: ModuleContext) -> Result<()> {
        self.auth_state.clear();
        let mut state = self.state.write().await;
        state.phase = AuthPhase::RuntimeStarted;
        state.last_error = None;
        Ok(())
    }

    async fn on_context_initialize(&self, context: RuntimeContext) -> Result<()> {
        let mut state = self.state.write().await;
        if !matches!(state.phase, AuthPhase::RuntimeStarted) {
            return Err(RuntimeError::InvalidState(
                "auth runtime requires runtime_started before context init".into(),
            ));
        }
        state.phase = AuthPhase::ContextReady;
        drop(state);

        match context.input.auth_info {
            Some(auth_info) => self.replace_auth_info(auth_info).await,
            None => self.clear_auth_info().await,
        }

        Ok(())
    }

    async fn on_context_destroy(&self) -> Result<()> {
        self.clear_auth_info().await;
        let mut state = self.state.write().await;
        state.phase = AuthPhase::RuntimeStarted;
        Ok(())
    }

    async fn on_runtime_shutdown(&self) -> Result<()> {
        self.clear_auth_info().await;
        let mut state = self.state.write().await;
        state.phase = AuthPhase::RuntimeStopped;
        Ok(())
    }

    async fn health_snapshot(&self) -> ModuleHealthSnapshot {
        let state = self.state.read().await;
        let auth_info = self.auth_state.snapshot();
        ModuleHealthSnapshot {
            name: self.name().into(),
            phase: state.phase.as_str().into(),
            healthy: state.last_error.is_none(),
            detail: json!({
                "is_authenticated": auth_info.is_authenticated,
                "has_access_token": auth_info.access_token.is_some(),
                "has_user_info": auth_info.user_info.is_some(),
                "last_error": state.last_error,
            }),
        }
    }
}

fn anonymous_auth_info() -> AuthInfo {
    AuthInfo {
        is_authenticated: false,
        access_token: None,
        user_info: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{ModuleContext, RuntimeContextInput, event_channel};
    use crate::infrastructure::eventbus::UserInfo;

    #[tokio::test]
    async fn auth_runtime_initializes_from_context_and_updates_state() {
        let auth_state = Arc::new(AuthStateStore::new());
        let runtime = AuthRuntime::new(auth_state.clone());
        let (events, _) = event_channel();

        runtime.on_runtime_start(ModuleContext { events }).await.unwrap();
        runtime
            .on_context_initialize(RuntimeContext::new(
                1,
                RuntimeContextInput {
                    user_id: Some("user-1".into()),
                    workspace_id: None,
                    auth_info: Some(AuthInfo {
                        is_authenticated: true,
                        access_token: Some("token-1".into()),
                        user_info: Some(UserInfo {
                            user_id: "user-1".into(),
                            username: "tester".into(),
                            email: Some("tester@example.com".into()),
                        }),
                    }),
                    attributes: Default::default(),
                },
            ))
            .await
            .unwrap();

        assert_eq!(auth_state.snapshot().is_authenticated, true);

        runtime
            .execute_command(AuthCommandRequest::SetAuthState {
                auth_info: AuthInfo {
                    is_authenticated: true,
                    access_token: Some("token-2".into()),
                    user_info: None,
                },
            })
            .await
            .unwrap();

        match runtime.execute_command(AuthCommandRequest::GetAuthState).await.unwrap() {
            AuthCommandResponse::State { auth_info } => {
                assert_eq!(auth_info.access_token.as_deref(), Some("token-2"));
            }
            other => panic!("unexpected response: {:?}", other),
        }

        runtime.on_context_destroy().await.unwrap();
        assert_eq!(auth_state.snapshot().is_authenticated, false);
    }
}
