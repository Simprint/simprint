use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::AuditLogDto;

/// 记录审计日志
pub async fn insert_audit_log(
    pool: &Pool<Postgres>,
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
) -> Result<i64, Error> {
    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO audit_logs (user_uuid, team_uuid, action, target_type, target_uuid,
                                 target_name, details, changes, ip_address, user_agent, request_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id;
        "#,
    )
    .bind(user_uuid)
    .bind(team_uuid)
    .bind(action)
    .bind(target_type)
    .bind(target_uuid)
    .bind(target_name)
    .bind(details)
    .bind(changes)
    .bind(ip_address)
    .bind(user_agent)
    .bind(request_id)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// 查询审计日志列表
///
/// 查询逻辑：
/// - 当前团队的所有审计日志
/// - 加上当前用户的个人操作日志（team_uuid 为空的，如登录、注册等）
pub async fn fetch_audit_logs(
    pool: &Pool<Postgres>,
    current_user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    user_uuid_filter: Option<Uuid>,
    keyword: Option<&str>,
    action: Option<&str>,
    target_type: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<AuditLogDto>, Error> {
    let recs = sqlx::query_as::<_, AuditLogDto>(
        r#"
        SELECT
            a.id, a.uuid, a.user_uuid, a.team_uuid, a.action, a.target_type, a.target_uuid,
            a.target_name, a.details, a.changes, a.ip_address, a.user_agent, a.request_id, a.created_at,
            ui.nickname AS user_name, ui.email AS user_email
        FROM audit_logs a
        LEFT JOIN user_infos ui ON a.user_uuid = ui.user_uuid
        WHERE (
            ($2::uuid IS NULL OR a.team_uuid = $2)
            OR (a.team_uuid IS NULL AND a.user_uuid = $1)
        )
          AND ($3::uuid IS NULL OR a.user_uuid = $3)
          AND ($4::text IS NULL OR COALESCE(a.details, '') ILIKE '%' || $4 || '%')
          AND ($5::varchar IS NULL OR a.action = $5)
          AND ($6::varchar IS NULL OR a.target_type = $6)
        ORDER BY a.created_at DESC
        LIMIT $7 OFFSET $8
        "#,
    )
    .bind(current_user_uuid)
    .bind(team_uuid)
    .bind(user_uuid_filter)
    .bind(keyword)
    .bind(action)
    .bind(target_type)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询审计日志总数
pub async fn fetch_audit_logs_count(
    pool: &Pool<Postgres>,
    current_user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    user_uuid_filter: Option<Uuid>,
    keyword: Option<&str>,
    action: Option<&str>,
    target_type: Option<&str>,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM audit_logs
        WHERE (
            ($2::uuid IS NULL OR team_uuid = $2)
            OR (team_uuid IS NULL AND user_uuid = $1)
        )
          AND ($3::uuid IS NULL OR user_uuid = $3)
          AND ($4::text IS NULL OR COALESCE(details, '') ILIKE '%' || $4 || '%')
          AND ($5::varchar IS NULL OR action = $5)
          AND ($6::varchar IS NULL OR target_type = $6)
        "#,
    )
    .bind(current_user_uuid)
    .bind(team_uuid)
    .bind(user_uuid_filter)
    .bind(keyword)
    .bind(action)
    .bind(target_type)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 根据 UUID 查询审计日志
pub async fn fetch_audit_log_by_uuid(
    pool: &Pool<Postgres>,
    log_uuid: Uuid,
) -> Result<Option<AuditLogDto>, Error> {
    let rec = sqlx::query_as::<_, AuditLogDto>(
        r#"
        SELECT
            a.id, a.uuid, a.user_uuid, a.team_uuid, a.action, a.target_type, a.target_uuid,
            a.target_name, a.details, a.changes, a.ip_address, a.user_agent, a.request_id, a.created_at,
            ui.nickname AS user_name, ui.email AS user_email
        FROM audit_logs a
        LEFT JOIN user_infos ui ON a.user_uuid = ui.user_uuid
        WHERE a.uuid = $1
        "#,
    )
    .bind(log_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 查询指定日期的审计日志数量
pub async fn fetch_audit_logs_count_by_date(
    pool: &Pool<Postgres>,
    team_uuid: Option<Uuid>,
    date: chrono::NaiveDate,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM audit_logs
        WHERE ($1::uuid IS NULL OR team_uuid = $1)
          AND created_at::date = $2
        "#,
    )
    .bind(team_uuid)
    .bind(date)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 查询指定日期之后的审计日志数量
pub async fn fetch_audit_logs_count_since_date(
    pool: &Pool<Postgres>,
    team_uuid: Option<Uuid>,
    since_date: chrono::NaiveDate,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM audit_logs
        WHERE ($1::uuid IS NULL OR team_uuid = $1)
          AND created_at::date >= $2
        "#,
    )
    .bind(team_uuid)
    .bind(since_date)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 查询热门操作类型
pub async fn fetch_top_actions(
    pool: &Pool<Postgres>,
    team_uuid: Option<Uuid>,
    limit: i64,
) -> Result<Vec<(String, i64)>, Error> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT action, COUNT(*) as count FROM audit_logs
        WHERE ($1::uuid IS NULL OR team_uuid = $1)
        GROUP BY action
        ORDER BY count DESC
        LIMIT $2
        "#,
    )
    .bind(team_uuid)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// 查询热门目标类型
pub async fn fetch_top_target_types(
    pool: &Pool<Postgres>,
    team_uuid: Option<Uuid>,
    limit: i64,
) -> Result<Vec<(String, i64)>, Error> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT target_type, COUNT(*) as count FROM audit_logs
        WHERE ($1::uuid IS NULL OR team_uuid = $1)
        GROUP BY target_type
        ORDER BY count DESC
        LIMIT $2
        "#,
    )
    .bind(team_uuid)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
