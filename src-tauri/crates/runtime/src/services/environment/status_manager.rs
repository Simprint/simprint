use super::status::EnvironmentStatus;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct EnvironmentStatusManager {
    statuses: Arc<RwLock<HashMap<String, EnvironmentStatus>>>,
}

impl EnvironmentStatusManager {
    pub fn new() -> Self {
        Self {
            statuses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set_status(&self, env_uuid: &str, status: EnvironmentStatus) {
        let mut statuses = self.statuses.write().await;
        statuses.insert(env_uuid.to_string(), status);
    }

    pub async fn set_stopped_unless_error(&self, env_uuid: &str) -> bool {
        let mut statuses = self.statuses.write().await;
        if matches!(statuses.get(env_uuid), Some(EnvironmentStatus::Error)) {
            return false;
        }

        statuses.insert(env_uuid.to_string(), EnvironmentStatus::Stopped);
        true
    }

    pub async fn get_status(&self, env_uuid: &str) -> Option<EnvironmentStatus> {
        let statuses = self.statuses.read().await;
        statuses.get(env_uuid).cloned()
    }

    pub async fn get_all_statuses(&self) -> HashMap<String, EnvironmentStatus> {
        let statuses = self.statuses.read().await;
        statuses.clone()
    }

    pub async fn remove_status(&self, env_uuid: &str) {
        let mut statuses = self.statuses.write().await;
        statuses.remove(env_uuid);
    }

    pub async fn clear(&self) {
        let mut statuses = self.statuses.write().await;
        statuses.clear();
    }
}

impl Default for EnvironmentStatusManager {
    fn default() -> Self {
        Self::new()
    }
}
