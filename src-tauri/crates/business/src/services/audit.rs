use chrono::Datelike;
use uuid::Uuid;

use crate::dto::AuditLogDto;
use crate::entitys::{
    ActionCount, AuditStatsResponse, ExportAuditLogsRequest, ListAuditLogsRequest, TargetTypeCount,
};
use crate::models;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;

/// 审计日志宏 - 简化审计日志记录
///
/// # 用法
/// ```rust
/// // 基础用法: action, target_type, detail
/// audit_log!(svc_ctx, ctx, "login", "user", "用户登录");
///
/// // 带目标 UUID
/// audit_log!(svc_ctx, ctx, "delete", "environment", env_uuid, "删除环境");
///
/// // 带目标 UUID 和名称
/// audit_log!(svc_ctx, ctx, "delete", "environment", env_uuid, "生产环境", "删除环境");
/// ```
#[macro_export]
macro_rules! audit_log {
    // 基础: action, target_type, detail
    ($svc_ctx:expr, $ctx:expr, $action:expr, $target_type:expr, $detail:expr) => {
        $crate::services::audit::log_audit(
            $svc_ctx,
            $ctx,
            $action,
            $target_type,
            None,
            None,
            $detail,
        )
    };
    // 带目标 UUID: action, target_type, target_uuid, detail
    ($svc_ctx:expr, $ctx:expr, $action:expr, $target_type:expr, $target_uuid:expr, $detail:expr) => {
        $crate::services::audit::log_audit(
            $svc_ctx,
            $ctx,
            $action,
            $target_type,
            Some($target_uuid),
            None,
            $detail,
        )
    };
    // 完整: action, target_type, target_uuid, target_name, detail
    ($svc_ctx:expr, $ctx:expr, $action:expr, $target_type:expr, $target_uuid:expr, $target_name:expr, $detail:expr) => {
        $crate::services::audit::log_audit(
            $svc_ctx,
            $ctx,
            $action,
            $target_type,
            Some($target_uuid),
            Some($target_name),
            $detail,
        )
    };
}

/// 记录审计日志（宏的内部实现）
///
/// 自动从 RequestContext 中提取 user_uuid、team_uuid、ip_address
pub async fn log_audit(
    svc_ctx: &SvcCtx,
    ctx: &RequestContext,
    action: &str,
    target_type: &str,
    target_uuid: Option<Uuid>,
    target_name: Option<&str>,
    detail: &str,
) {
    // 从 ctx 中提取数据
    let user_uuid = match ctx.user_uuid() {
        Some(uuid) => uuid,
        None => {
            tracing::warn!("audit_log: user_uuid is None, skipping audit log");
            return;
        }
    };

    let team_uuid = ctx.current_team_uuid;
    let ip_address = ctx.ip();

    // 异步记录，忽略错误（审计失败不应影响业务）
    if let Err(e) = log_action_service(
        svc_ctx,
        user_uuid,
        team_uuid,
        action,
        target_type,
        target_uuid,
        target_name,
        Some(detail),
        None, // changes
        ip_address,
        None, // user_agent
        None, // request_id
    )
    .await
    {
        tracing::error!("audit_log failed: {}", e);
    }
}

/// 记录审计日志（允许无用户，用于登录/注册等场景）
pub async fn log_audit_anonymous(
    svc_ctx: &SvcCtx,
    ctx: &RequestContext,
    user_uuid: Uuid,
    action: &str,
    target_type: &str,
    detail: &str,
) {
    let ip_address = ctx.ip();

    if let Err(e) = log_action_service(
        svc_ctx,
        user_uuid,
        None,
        action,
        target_type,
        Some(user_uuid),
        None,
        Some(detail),
        None,
        ip_address,
        None,
        None,
    )
    .await
    {
        tracing::error!("audit_log_anonymous failed: {}", e);
    }
}

/// 记录审计日志
pub async fn log_action_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    action: &str,
    target_type: &str,
    target_uuid: Option<Uuid>,
    target_name: Option<&str>,
    details: Option<&str>,
    changes: Option<&serde_json::Value>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    request_id: Option<&str>,
) -> Result<i64, String> {
    models::insert_audit_log(
        &svc_ctx.db,
        user_uuid,
        team_uuid,
        action,
        target_type,
        target_uuid,
        target_name,
        details,
        changes,
        ip_address,
        user_agent,
        request_id,
    )
    .await
    .map_err(|e| e.to_string())
}

/// 获取审计日志列表
pub async fn get_audit_logs_service(
    svc_ctx: &SvcCtx,
    current_user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    payload: &ListAuditLogsRequest,
) -> Result<(Vec<AuditLogDto>, i64), String> {
    let offset = (payload.pagination.page - 1) * payload.pagination.page_size;

    let user_uuid_filter = payload.filters.as_ref().and_then(|f| f.user_uuid);
    let keyword = payload.filters.as_ref().and_then(|f| f.keyword.as_deref());
    let action = payload.filters.as_ref().and_then(|f| f.action.as_deref());
    let target_type = payload.filters.as_ref().and_then(|f| f.target_type.as_deref());

    let logs = models::fetch_audit_logs(
        &svc_ctx.db,
        current_user_uuid,
        team_uuid,
        user_uuid_filter,
        keyword,
        action,
        target_type,
        offset,
        payload.pagination.page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    let total = models::fetch_audit_logs_count(
        &svc_ctx.db,
        current_user_uuid,
        team_uuid,
        user_uuid_filter,
        keyword,
        action,
        target_type,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok((logs, total))
}

/// 获取审计日志详情
pub async fn get_audit_log_service(
    svc_ctx: &SvcCtx,
    log_uuid: Uuid,
) -> Result<AuditLogDto, String> {
    models::fetch_audit_log_by_uuid(&svc_ctx.db, log_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "审计日志不存在".to_string())
}

/// 获取审计统计
pub async fn get_audit_stats_service(
    svc_ctx: &SvcCtx,
    current_user_uuid: Uuid,
    team_uuid: Option<Uuid>,
) -> Result<AuditStatsResponse, String> {
    // 总数
    let total_logs =
        models::fetch_audit_logs_count(
            &svc_ctx.db,
            current_user_uuid,
            team_uuid,
            None,
            None,
            None,
            None,
        )
            .await
            .map_err(|e| e.to_string())?;

    // 今日数量
    let today = chrono::Utc::now().date_naive();
    let logs_today = models::fetch_audit_logs_count_by_date(&svc_ctx.db, team_uuid, today)
        .await
        .map_err(|e| e.to_string())?;

    // 本周数量
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    let logs_this_week =
        models::fetch_audit_logs_count_since_date(&svc_ctx.db, team_uuid, week_start)
            .await
            .map_err(|e| e.to_string())?;

    // 本月数量
    let month_start = chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
    let logs_this_month =
        models::fetch_audit_logs_count_since_date(&svc_ctx.db, team_uuid, month_start)
            .await
            .map_err(|e| e.to_string())?;

    // 热门操作（Top 5）
    let top_actions_raw = models::fetch_top_actions(&svc_ctx.db, team_uuid, 5)
        .await
        .map_err(|e| e.to_string())?;

    // 热门目标类型（Top 5）
    let top_target_types_raw = models::fetch_top_target_types(&svc_ctx.db, team_uuid, 5)
        .await
        .map_err(|e| e.to_string())?;

    Ok(AuditStatsResponse {
        total_logs,
        logs_today,
        logs_this_week,
        logs_this_month,
        top_actions: top_actions_raw
            .into_iter()
            .map(|(action, count)| ActionCount { action, count })
            .collect(),
        top_target_types: top_target_types_raw
            .into_iter()
            .map(|(target_type, count)| TargetTypeCount { target_type, count })
            .collect(),
    })
}

/// 导出审计日志
pub async fn export_audit_logs_service(
    svc_ctx: &SvcCtx,
    current_user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    payload: &ExportAuditLogsRequest,
) -> Result<(String, String, String), String> {
    // 构建查询请求
    let list_request = ListAuditLogsRequest {
        pagination: crate::entitys::Pagination {
            page: 1,
            page_size: payload.max_records.unwrap_or(1000) as i64,
            sort_by: None,
            sort_order: None,
        },
        filters: payload.filters.clone(),
    };

    let (logs, _total) =
        get_audit_logs_service(svc_ctx, current_user_uuid, team_uuid, &list_request).await?;

    // 根据格式生成导出内容
    let content = match payload.format.as_str() {
        "csv" => export_to_csv(&logs),
        "json" => serde_json::to_string_pretty(&logs).unwrap_or_default(),
        _ => return Err("不支持的导出格式".to_string()),
    };

    let filename = format!(
        "audit_logs_{}.{}",
        chrono::Utc::now().format("%Y%m%d%H%M%S"),
        payload.format
    );

    let mime_type = match payload.format.as_str() {
        "csv" => "text/csv".to_string(),
        "json" => "application/json".to_string(),
        _ => "text/plain".to_string(),
    };

    Ok((content, filename, mime_type))
}

fn export_to_csv(logs: &[AuditLogDto]) -> String {
    let mut csv = String::from("时间,用户UUID,操作,目标类型,目标名称,详情,IP地址\n");
    for log in logs {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            log.created_at.format("%Y-%m-%d %H:%M:%S"),
            log.user_uuid,
            log.action,
            log.target_type,
            log.target_name.as_deref().unwrap_or(""),
            log.details.as_deref().unwrap_or("").replace(',', ";"),
            log.ip_address.as_deref().unwrap_or(""),
        ));
    }
    csv
}
