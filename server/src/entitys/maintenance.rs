use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::dto::maintenance::MaintenanceType;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateMaintenanceRequest {
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub maintenance_type: MaintenanceType,
}

/// 获取维护详情请求
#[derive(Debug, Deserialize, Serialize)]
pub struct GetMaintenanceRequest {
    pub id: i64,
}

/// 查询维护列表请求
#[derive(Debug, Deserialize, Serialize)]
pub struct ListMaintenancesRequest {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// 更新维护状态请求
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateMaintenanceStatusRequest {
    pub id: i64,
    pub status: String,
}

/// 结束维护请求（可为空 body）
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct EndMaintenanceRequest {}

/// 获取当前活跃维护请求（可为空 body）
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct GetActiveMaintenanceRequest {}
