use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;
use crate::dto::AuditLogDto;

/// 查询审计日志请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListAuditLogsRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub filters: Option<AuditLogFilters>,
}

// ========== 响应结构体 ==========

/// 审计日志列表响应
#[derive(Debug, Clone, Serialize)]
pub struct AuditLogsListResponse {
    pub items: Vec<AuditLogDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 导出响应
#[derive(Debug, Clone, Serialize)]
pub struct ExportResponse {
    pub content: String,
    pub filename: String,
    pub mime_type: String,
}

/// 审计统计响应
#[derive(Debug, Clone, Serialize)]
pub struct AuditStatsResponse {
    pub total_logs: i64,
    pub logs_today: i64,
    pub logs_this_week: i64,
    pub logs_this_month: i64,
    pub top_actions: Vec<ActionCount>,
    pub top_target_types: Vec<TargetTypeCount>,
}

/// 操作计数
#[derive(Debug, Clone, Serialize)]
pub struct ActionCount {
    pub action: String,
    pub count: i64,
}

/// 目标类型计数
#[derive(Debug, Clone, Serialize)]
pub struct TargetTypeCount {
    pub target_type: String,
    pub count: i64,
}

/// 审计日志筛选条件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditLogFilters {
    pub keyword: Option<String>,
    pub action: Option<String>,
    pub target_type: Option<String>,
    pub user_uuid: Option<Uuid>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

/// 导出审计日志请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportAuditLogsRequest {
    pub format: String,
    pub filters: Option<AuditLogFilters>,
    pub max_records: Option<i32>,
}

/// 审计统计请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditStatsRequest {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}
