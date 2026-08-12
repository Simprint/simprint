use super::context::RuntimeContext;
use super::error::{Result, RuntimeError};
use super::events::EventPublisher;
use super::state::ModuleHealthSnapshot;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Clone)]
pub struct ModuleContext {
    pub events: EventPublisher,
}

#[async_trait]
pub trait RuntimeModule: Send + Sync {
    fn name(&self) -> &'static str;

    async fn on_runtime_start(&self, context: ModuleContext) -> Result<()>;

    async fn on_context_initialize(&self, context: RuntimeContext) -> Result<()>;

    async fn on_context_destroy(&self) -> Result<()>;

    async fn on_runtime_shutdown(&self) -> Result<()>;

    async fn health_snapshot(&self) -> ModuleHealthSnapshot;
}

pub struct ModuleOrchestrator {
    modules: Vec<Arc<dyn RuntimeModule>>,
}

impl ModuleOrchestrator {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    pub fn register(mut self, module: Arc<dyn RuntimeModule>) -> Self {
        self.modules.push(module);
        self
    }

    pub fn len(&self) -> usize {
        self.modules.len()
    }

    pub async fn start(&self, context: ModuleContext) -> Result<()> {
        let mut started: Vec<Arc<dyn RuntimeModule>> = Vec::new();

        for module in &self.modules {
            if let Err(error) = module.on_runtime_start(context.clone()).await {
                for rollback in started.into_iter().rev() {
                    let _ = rollback.on_runtime_shutdown().await;
                }
                return Err(wrap_module_error(module.name(), "runtime_start", error));
            }
            started.push(module.clone());
        }

        Ok(())
    }

    pub async fn initialize_context(&self, context: RuntimeContext) -> Result<()> {
        let mut initialized: Vec<Arc<dyn RuntimeModule>> = Vec::new();

        for module in &self.modules {
            if let Err(error) = module.on_context_initialize(context.clone()).await {
                for rollback in initialized.into_iter().rev() {
                    let _ = rollback.on_context_destroy().await;
                }
                return Err(wrap_module_error(
                    module.name(),
                    "context_initialize",
                    error,
                ));
            }
            initialized.push(module.clone());
        }

        Ok(())
    }

    pub async fn destroy_context(&self) -> Result<()> {
        let mut first_error = None;

        for module in self.modules.iter().rev() {
            if let Err(error) = module.on_context_destroy().await {
                if first_error.is_none() {
                    first_error = Some(wrap_module_error(module.name(), "context_destroy", error));
                }
            }
        }

        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        let mut first_error = None;

        for module in self.modules.iter().rev() {
            if let Err(error) = module.on_runtime_shutdown().await {
                if first_error.is_none() {
                    first_error = Some(wrap_module_error(module.name(), "runtime_shutdown", error));
                }
            }
        }

        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub async fn health_snapshot(&self) -> Vec<ModuleHealthSnapshot> {
        let mut snapshots = Vec::with_capacity(self.modules.len());
        for module in &self.modules {
            snapshots.push(module.health_snapshot().await);
        }
        snapshots
    }
}

fn wrap_module_error(
    module: &'static str,
    action: &'static str,
    error: RuntimeError,
) -> RuntimeError {
    RuntimeError::ModuleLifecycle {
        module,
        action,
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::event_channel;
    use crate::app::{RuntimeContextInput, RuntimeError};
    use serde_json::json;
    use tokio::sync::Mutex;

    struct RecorderModule {
        name: &'static str,
        log: Arc<Mutex<Vec<String>>>,
        fail_on_init: bool,
    }

    #[async_trait]
    impl RuntimeModule for RecorderModule {
        fn name(&self) -> &'static str {
            self.name
        }

        async fn on_runtime_start(&self, _context: ModuleContext) -> Result<()> {
            self.log.lock().await.push(format!("{}.start", self.name));
            Ok(())
        }

        async fn on_context_initialize(&self, _context: RuntimeContext) -> Result<()> {
            self.log.lock().await.push(format!("{}.init", self.name));
            if self.fail_on_init {
                return Err(RuntimeError::Internal("boom".into()));
            }
            Ok(())
        }

        async fn on_context_destroy(&self) -> Result<()> {
            self.log.lock().await.push(format!("{}.destroy", self.name));
            Ok(())
        }

        async fn on_runtime_shutdown(&self) -> Result<()> {
            self.log.lock().await.push(format!("{}.shutdown", self.name));
            Ok(())
        }

        async fn health_snapshot(&self) -> ModuleHealthSnapshot {
            ModuleHealthSnapshot {
                name: self.name.into(),
                phase: "ready".into(),
                healthy: true,
                detail: json!({}),
            }
        }
    }

    #[tokio::test]
    async fn initialize_rollback_runs_in_reverse_order() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let (events, _) = event_channel();

        let orchestrator = ModuleOrchestrator::new()
            .register(Arc::new(RecorderModule {
                name: "env",
                log: log.clone(),
                fail_on_init: false,
            }))
            .register(Arc::new(RecorderModule {
                name: "sync",
                log: log.clone(),
                fail_on_init: true,
            }));

        orchestrator.start(ModuleContext { events }).await.unwrap();

        let result = orchestrator
            .initialize_context(RuntimeContext::new(1, RuntimeContextInput::default()))
            .await;
        assert!(result.is_err());

        let log = log.lock().await.clone();
        assert_eq!(
            log,
            vec![
                "env.start",
                "sync.start",
                "env.init",
                "sync.init",
                "env.destroy",
            ]
        );
    }
}
