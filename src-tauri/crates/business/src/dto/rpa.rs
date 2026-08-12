use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// RPA 任务 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RpaTaskDto {
    pub id: i32,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub team_uuid: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<serde_json::Value>,
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
    pub status: String,
    pub run_count: Option<i32>,
    pub success_count: Option<i32>,
    pub environment_count: Option<i64>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// RPA 任务步骤 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RpaTaskStepDto {
    pub id: i32,
    pub uuid: Uuid,
    pub task_uuid: Uuid,
    pub step_type: String,
    pub name: String,
    pub config: serde_json::Value,
    pub enabled: Option<bool>,
    pub position_x: Option<i32>,
    pub position_y: Option<i32>,
    pub sort_order: Option<i32>,
    pub next_step_uuid: Option<Uuid>,
    pub branch_config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// RPA 任务环境关联 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RpaTaskEnvironmentDto {
    pub id: i32,
    pub task_uuid: Uuid,
    pub environment_uuid: Uuid,
    pub sort_order: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// RPA 任务执行记录 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RpaTaskRunDto {
    pub id: i64,
    pub uuid: Uuid,
    pub task_uuid: Uuid,
    pub status: String,
    pub total_steps: i32,
    pub completed_steps: i32,
    pub failed_steps: i32,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub result_summary: Option<String>,
    pub error_message: Option<String>,
    pub logs: Option<serde_json::Value>,
}
