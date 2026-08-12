pub mod cdp;
pub mod job;
pub mod launcher;
pub mod types;

use super::status_manager::EnvironmentStatusManager;
use crate::app::{EventPublisher, Result};
use cdp::CdpEndpointManager;
use job::JobManager;
use launcher::{
    batch_launch_environments, batch_stop_environments, close_rpa_tab, get_cdp_endpoint,
    get_connected_environments, launch_browser, list_rpa_tabs, refresh_proxy, select_rpa_tab,
    set_window_bounds, stop_environment,
};
use std::sync::Arc;
use types::{EnvironmentCommandRequest, EnvironmentCommandResponse};

pub struct KernelRuntime {
    cdp_endpoint_manager: Arc<CdpEndpointManager>,
    job_manager: Arc<JobManager>,
    status_manager: Arc<EnvironmentStatusManager>,
}

impl KernelRuntime {
    pub fn new(status_manager: Arc<EnvironmentStatusManager>) -> Self {
        Self {
            cdp_endpoint_manager: Arc::new(CdpEndpointManager::new()),
            job_manager: Arc::new(JobManager::new()),
            status_manager,
        }
    }

    pub async fn execute(
        &self,
        command: EnvironmentCommandRequest,
        events: EventPublisher,
    ) -> Result<EnvironmentCommandResponse> {
        match command {
            EnvironmentCommandRequest::StartEnvironment { request } => {
                let endpoint = launch_browser(
                    request,
                    self.cdp_endpoint_manager.clone(),
                    self.job_manager.clone(),
                    self.status_manager.clone(),
                    events,
                )
                .await?;
                Ok(EnvironmentCommandResponse::Started { endpoint })
            }
            EnvironmentCommandRequest::BatchStartEnvironments { requests } => {
                let results = batch_launch_environments(
                    requests,
                    self.cdp_endpoint_manager.clone(),
                    self.job_manager.clone(),
                    self.status_manager.clone(),
                    events,
                )
                .await?;
                Ok(EnvironmentCommandResponse::BatchLaunchResults { results })
            }
            EnvironmentCommandRequest::StopEnvironment { env_uuid } => {
                stop_environment(
                    env_uuid,
                    self.cdp_endpoint_manager.clone(),
                    self.job_manager.clone(),
                    self.status_manager.clone(),
                    events,
                )
                .await?;
                Ok(EnvironmentCommandResponse::Ack)
            }
            EnvironmentCommandRequest::BatchStopEnvironments { env_uuids } => {
                let results = batch_stop_environments(
                    env_uuids,
                    self.cdp_endpoint_manager.clone(),
                    self.job_manager.clone(),
                    self.status_manager.clone(),
                    events,
                )
                .await?;
                Ok(EnvironmentCommandResponse::BatchLaunchResults { results })
            }
            EnvironmentCommandRequest::RefreshProxy { env_uuid, proxy } => {
                refresh_proxy(env_uuid, proxy, events).await?;
                Ok(EnvironmentCommandResponse::Ack)
            }
            EnvironmentCommandRequest::SetWindowBounds { request } => {
                set_window_bounds(request, events).await?;
                Ok(EnvironmentCommandResponse::Ack)
            }
            EnvironmentCommandRequest::GetConnectedEnvironments => {
                let env_ids = get_connected_environments().await?;
                Ok(EnvironmentCommandResponse::ConnectedEnvironments { env_ids })
            }
            EnvironmentCommandRequest::GetCdpEndpoint { env_uuid } => {
                let endpoint =
                    get_cdp_endpoint(env_uuid, self.cdp_endpoint_manager.clone()).await?;
                Ok(EnvironmentCommandResponse::CdpEndpoint { endpoint })
            }
            EnvironmentCommandRequest::ListRpaTabs { env_uuid } => {
                let snapshot = list_rpa_tabs(env_uuid).await?;
                Ok(EnvironmentCommandResponse::RpaTabsSnapshot { snapshot })
            }
            EnvironmentCommandRequest::SelectRpaTab { env_uuid, position } => {
                let selection = select_rpa_tab(env_uuid, position).await?;
                Ok(EnvironmentCommandResponse::RpaTabSelected { selection })
            }
            EnvironmentCommandRequest::CloseRpaTab { env_uuid, position } => {
                let result = close_rpa_tab(env_uuid, position).await?;
                Ok(EnvironmentCommandResponse::RpaTabClosed { result })
            }
            EnvironmentCommandRequest::GetEnvironmentStatus { env_uuid } => {
                let status = self.status_manager.get_status(&env_uuid).await;
                Ok(EnvironmentCommandResponse::Status { status })
            }
            EnvironmentCommandRequest::GetAllEnvironmentStatuses => {
                let statuses = self.status_manager.get_all_statuses().await;
                Ok(EnvironmentCommandResponse::AllStatuses { statuses })
            }
        }
    }

    pub async fn handle_browser_disconnect(&self, env_uuid: &str, events: EventPublisher) {
        self.job_manager.remove(env_uuid).await;
        self.cdp_endpoint_manager.remove(env_uuid).await;
        self.status_manager.set_stopped_unless_error(env_uuid).await;
        let _ = events.emit(
            "environment.browser_disconnected",
            &serde_json::json!({ "env_uuid": env_uuid }),
        );
    }

    pub async fn clear_all(&self) {
        self.job_manager.clear_all().await;
        self.cdp_endpoint_manager.clear_all().await;
        self.status_manager.clear().await;
    }

    pub async fn get_connected_env_count(&self) -> usize {
        match crate::infrastructure::eventbus::get_eventbus_manager() {
            Some(manager) => manager.connected_env_count().await,
            None => 0,
        }
    }

    pub fn status_manager(&self) -> Arc<EnvironmentStatusManager> {
        self.status_manager.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::event_channel;
    use crate::services::environment::EnvironmentStatus;

    #[tokio::test]
    async fn status_queries_return_runtime_environment_statuses() {
        let status_manager = Arc::new(EnvironmentStatusManager::new());
        status_manager.set_status("env-1", EnvironmentStatus::Running).await;
        status_manager.set_status("env-2", EnvironmentStatus::Stopped).await;

        let runtime = KernelRuntime::new(status_manager);
        let (events, _rx) = event_channel();

        let response = runtime
            .execute(
                EnvironmentCommandRequest::GetEnvironmentStatus {
                    env_uuid: "env-1".into(),
                },
                events.clone(),
            )
            .await
            .unwrap();
        match response {
            EnvironmentCommandResponse::Status { status } => {
                assert_eq!(status, Some(EnvironmentStatus::Running));
            }
            other => panic!("unexpected response: {:?}", other),
        }

        let response = runtime
            .execute(EnvironmentCommandRequest::GetAllEnvironmentStatuses, events)
            .await
            .unwrap();
        match response {
            EnvironmentCommandResponse::AllStatuses { statuses } => {
                assert_eq!(statuses.get("env-1"), Some(&EnvironmentStatus::Running));
                assert_eq!(statuses.get("env-2"), Some(&EnvironmentStatus::Stopped));
            }
            other => panic!("unexpected response: {:?}", other),
        }
    }

    #[tokio::test]
    async fn browser_disconnect_marks_environment_stopped() {
        let status_manager = Arc::new(EnvironmentStatusManager::new());
        status_manager.set_status("env-1", EnvironmentStatus::Running).await;

        let runtime = KernelRuntime::new(status_manager.clone());
        let (events, _rx) = event_channel();

        runtime.handle_browser_disconnect("env-1", events).await;

        assert_eq!(
            status_manager.get_status("env-1").await,
            Some(EnvironmentStatus::Stopped)
        );
    }

    #[tokio::test]
    async fn browser_disconnect_preserves_launch_error() {
        let status_manager = Arc::new(EnvironmentStatusManager::new());
        status_manager.set_status("env-1", EnvironmentStatus::Error).await;

        let runtime = KernelRuntime::new(status_manager.clone());
        let (events, _rx) = event_channel();

        runtime.handle_browser_disconnect("env-1", events).await;

        assert_eq!(
            status_manager.get_status("env-1").await,
            Some(EnvironmentStatus::Error)
        );
    }
}
