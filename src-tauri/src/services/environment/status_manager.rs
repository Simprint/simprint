//! 环境状态管理器
//!
//! 维护所有环境的完整生命周期状态

use crate::domain::environment::EnvironmentStatus;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 环境状态管理器
pub struct EnvironmentStatusManager {
    /// 环境状态映射 (env_uuid -> status)
    statuses: Arc<RwLock<HashMap<String, EnvironmentStatus>>>,
}

impl EnvironmentStatusManager {
    /// 创建新的状态管理器
    pub fn new() -> Self {
        Self {
            statuses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 设置环境状态
    pub async fn set_status(&self, env_uuid: &str, status: EnvironmentStatus) {
        let mut statuses = self.statuses.write().await;
        statuses.insert(env_uuid.to_string(), status);
    }

    /// Mark an environment stopped without hiding a launch failure that was already reported.
    pub async fn set_stopped_unless_error(&self, env_uuid: &str) -> bool {
        let mut statuses = self.statuses.write().await;
        if matches!(statuses.get(env_uuid), Some(EnvironmentStatus::Error)) {
            return false;
        }

        statuses.insert(env_uuid.to_string(), EnvironmentStatus::Stopped);
        true
    }

    /// 获取环境状态
    pub async fn get_status(&self, env_uuid: &str) -> Option<EnvironmentStatus> {
        let statuses = self.statuses.read().await;
        statuses.get(env_uuid).cloned()
    }

    /// 获取所有环境状态
    pub async fn get_all_statuses(&self) -> HashMap<String, EnvironmentStatus> {
        let statuses = self.statuses.read().await;
        statuses.clone()
    }

    /// 移除环境状态
    pub async fn remove_status(&self, env_uuid: &str) {
        let mut statuses = self.statuses.write().await;
        statuses.remove(env_uuid);
    }

    /// 清空所有状态
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
