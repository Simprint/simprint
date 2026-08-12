use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListRpaTasksRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub filters: Option<RpaTaskFilters>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RpaTaskFilters {
    pub keyword: Option<String>,
    pub status: Option<String>,
    pub trigger_type: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateRpaTaskRequest {
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub trigger_type: String,
    pub schedule: Option<String>,
    pub cron_expression: Option<String>,
    pub run_mode: String,
    pub retry_count: Option<i32>,
    pub retry_interval: Option<i32>,
    pub timeout: Option<i32>,
    pub concurrency: Option<i32>,
    pub stop_on_error: Option<bool>,
    pub notify_on_complete: Option<bool>,
    pub notify_on_error: Option<bool>,
    pub environment_uuids: Option<Vec<Uuid>>,
    pub steps: Option<Vec<RpaTaskStepRequest>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RpaTaskStepRequest {
    pub step_type: String,
    pub name: String,
    pub config: serde_json::Value,
    pub enabled: Option<bool>,
    pub position_x: Option<i32>,
    pub position_y: Option<i32>,
    pub sort_order: Option<i32>,
    pub next_step_uuid: Option<Uuid>,
    pub branch_config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateRpaTaskRequest {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub trigger_type: Option<String>,
    pub schedule: Option<String>,
    pub cron_expression: Option<String>,
    pub run_mode: Option<String>,
    pub retry_count: Option<i32>,
    pub retry_interval: Option<i32>,
    pub timeout: Option<i32>,
    pub concurrency: Option<i32>,
    pub stop_on_error: Option<bool>,
    pub notify_on_complete: Option<bool>,
    pub notify_on_error: Option<bool>,
    pub environment_uuids: Option<Vec<Uuid>>,
    pub steps: Option<Vec<RpaTaskStepRequest>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RunRpaTaskRequest {
    pub uuid: Uuid,
    pub environment_uuids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DuplicateRpaTaskRequest {
    pub uuid: Uuid,
    pub new_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportRpaTaskRequest {
    pub uuid: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImportRpaTaskRequest {
    pub import_data: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListRpaRunsRequest {
    pub task_uuid: Uuid,
    #[serde(flatten)]
    pub pagination: Pagination,
    pub status: Option<String>,
}

use crate::dto::{RpaTaskDto, RpaTaskRunDto, RpaTaskStepDto};

#[derive(Debug, Clone, Serialize)]
pub struct RpaTaskListResponse {
    pub items: Vec<RpaTaskDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RpaTaskDetailResponse {
    pub task: RpaTaskDto,
    pub steps: Vec<RpaTaskStepDto>,
    pub environment_uuids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RpaRunsListResponse {
    pub items: Vec<RpaTaskRunDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportRpaTaskResponse {
    pub content: String,
    pub filename: String,
}
