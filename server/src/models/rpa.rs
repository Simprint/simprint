use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::{RpaTaskDto, RpaTaskEnvironmentDto, RpaTaskRunDto, RpaTaskStepDto};

// ============ RPA Tasks ============

/// 创建 RPA 任务
pub async fn insert_rpa_task(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    name: &str,
    description: Option<&str>,
    tags: Option<&serde_json::Value>,
    trigger_type: &str,
    schedule: Option<&str>,
    cron_expression: Option<&str>,
    run_mode: &str,
    retry_count: Option<i32>,
    retry_interval: Option<i32>,
    timeout: Option<i32>,
    concurrency: Option<i32>,
    stop_on_error: Option<bool>,
    notify_on_complete: Option<bool>,
    notify_on_error: Option<bool>,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO rpa_tasks (user_uuid, team_uuid, name, description, tags, trigger_type,
                                schedule, cron_expression, run_mode, retry_count, retry_interval,
                                timeout, concurrency, stop_on_error, notify_on_complete, notify_on_error)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        RETURNING uuid;
        "#,
    )
    .bind(user_uuid)
    .bind(team_uuid)
    .bind(name)
    .bind(description)
    .bind(tags)
    .bind(trigger_type)
    .bind(schedule)
    .bind(cron_expression)
    .bind(run_mode)
    .bind(retry_count)
    .bind(retry_interval)
    .bind(timeout)
    .bind(concurrency)
    .bind(stop_on_error)
    .bind(notify_on_complete)
    .bind(notify_on_error)
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

/// 查询 RPA 任务列表
pub async fn fetch_rpa_tasks(
    pool: &Pool<Postgres>,
    team_uuid: Option<Uuid>,
    user_uuid: Uuid,
    keyword: Option<&str>,
    status: Option<&str>,
    trigger_type: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<RpaTaskDto>, Error> {
    let recs = sqlx::query_as::<_, RpaTaskDto>(
        r#"
        SELECT id, uuid, user_uuid, team_uuid, name, description, tags, trigger_type,
               schedule, cron_expression, run_mode, retry_count, retry_interval, timeout,
               concurrency, stop_on_error, notify_on_complete, notify_on_error, status,
               run_count, success_count, last_run_at, next_run_at, created_at, updated_at, deleted_at
             , (SELECT COUNT(*) FROM rpa_task_environments rte WHERE rte.task_uuid = rpa_tasks.uuid) AS environment_count
        FROM rpa_tasks
        WHERE (team_uuid = $1 OR (team_uuid IS NULL AND user_uuid = $2))
          AND ($3::varchar IS NULL OR name ILIKE $3 OR COALESCE(description, '') ILIKE $3)
          AND ($4::varchar IS NULL OR status = $4)
          AND ($5::varchar IS NULL OR trigger_type = $5)
          AND deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT $6 OFFSET $7
        "#,
    )
    .bind(team_uuid)
    .bind(user_uuid)
    .bind(keyword.map(|k| format!("%{}%", k)))
    .bind(status)
    .bind(trigger_type)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询 RPA 任务总数
pub async fn fetch_rpa_tasks_count(
    pool: &Pool<Postgres>,
    team_uuid: Option<Uuid>,
    user_uuid: Uuid,
    keyword: Option<&str>,
    status: Option<&str>,
    trigger_type: Option<&str>,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM rpa_tasks
        WHERE (team_uuid = $1 OR (team_uuid IS NULL AND user_uuid = $2))
          AND ($3::varchar IS NULL OR status = $3)
          AND ($4::varchar IS NULL OR trigger_type = $4)
          AND ($5::varchar IS NULL OR name ILIKE $5 OR COALESCE(description, '') ILIKE $5)
          AND deleted_at IS NULL
        "#,
    )
    .bind(team_uuid)
    .bind(user_uuid)
    .bind(status)
    .bind(trigger_type)
    .bind(keyword.map(|k| format!("%{}%", k)))
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 根据 UUID 查询 RPA 任务
pub async fn fetch_rpa_task_by_uuid(
    pool: &Pool<Postgres>,
    task_uuid: Uuid,
) -> Result<Option<RpaTaskDto>, Error> {
    let rec = sqlx::query_as::<_, RpaTaskDto>(
        r#"
        SELECT id, uuid, user_uuid, team_uuid, name, description, tags, trigger_type,
               schedule, cron_expression, run_mode, retry_count, retry_interval, timeout,
               concurrency, stop_on_error, notify_on_complete, notify_on_error, status,
               run_count, success_count, last_run_at, next_run_at, created_at, updated_at, deleted_at
             , (SELECT COUNT(*) FROM rpa_task_environments rte WHERE rte.task_uuid = rpa_tasks.uuid) AS environment_count
        FROM rpa_tasks
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(task_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 更新 RPA 任务
pub async fn update_rpa_task(
    pool: &Pool<Postgres>,
    task_uuid: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    tags: Option<&serde_json::Value>,
    trigger_type: Option<&str>,
    schedule: Option<&str>,
    cron_expression: Option<&str>,
    run_mode: Option<&str>,
    retry_count: Option<i32>,
    retry_interval: Option<i32>,
    timeout: Option<i32>,
    concurrency: Option<i32>,
    stop_on_error: Option<bool>,
    notify_on_complete: Option<bool>,
    notify_on_error: Option<bool>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE rpa_tasks
        SET name = COALESCE($1, name),
            description = COALESCE($2, description),
            tags = COALESCE($3, tags),
            trigger_type = COALESCE($4, trigger_type),
            schedule = COALESCE($5, schedule),
            cron_expression = COALESCE($6, cron_expression),
            run_mode = COALESCE($7, run_mode),
            retry_count = COALESCE($8, retry_count),
            retry_interval = COALESCE($9, retry_interval),
            timeout = COALESCE($10, timeout),
            concurrency = COALESCE($11, concurrency),
            stop_on_error = COALESCE($12, stop_on_error),
            notify_on_complete = COALESCE($13, notify_on_complete),
            notify_on_error = COALESCE($14, notify_on_error),
            updated_at = CURRENT_TIMESTAMP
        WHERE uuid = $15 AND deleted_at IS NULL
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(tags)
    .bind(trigger_type)
    .bind(schedule)
    .bind(cron_expression)
    .bind(run_mode)
    .bind(retry_count)
    .bind(retry_interval)
    .bind(timeout)
    .bind(concurrency)
    .bind(stop_on_error)
    .bind(notify_on_complete)
    .bind(notify_on_error)
    .bind(task_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 更新任务状态
pub async fn update_rpa_task_status(
    pool: &Pool<Postgres>,
    task_uuid: Uuid,
    status: &str,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE rpa_tasks
        SET status = $1, updated_at = CURRENT_TIMESTAMP
        WHERE uuid = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(status)
    .bind(task_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 软删除 RPA 任务
pub async fn delete_rpa_task(pool: &Pool<Postgres>, task_uuid: Uuid) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE rpa_tasks SET deleted_at = CURRENT_TIMESTAMP
        WHERE uuid = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(task_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 批量软删除 RPA 任务
pub async fn batch_delete_rpa_tasks(
    pool: &Pool<Postgres>,
    task_uuids: &[Uuid],
) -> Result<u64, Error> {
    let result = sqlx::query(
        r#"
        UPDATE rpa_tasks SET deleted_at = CURRENT_TIMESTAMP
        WHERE uuid = ANY($1) AND deleted_at IS NULL
        "#,
    )
    .bind(task_uuids)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

// ============ RPA Task Steps ============

/// 插入任务步骤
pub async fn insert_rpa_task_step(
    pool: &Pool<Postgres>,
    task_uuid: Uuid,
    step_type: &str,
    name: &str,
    config: &serde_json::Value,
    enabled: Option<bool>,
    position_x: Option<i32>,
    position_y: Option<i32>,
    sort_order: Option<i32>,
    next_step_uuid: Option<Uuid>,
    branch_config: Option<&serde_json::Value>,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO rpa_task_steps (task_uuid, step_type, name, config, enabled,
                                     position_x, position_y, sort_order)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING uuid;
        "#,
    )
    .bind(task_uuid)
    .bind(step_type)
    .bind(name)
    .bind(config)
    .bind(enabled.unwrap_or(true))
    .bind(position_x)
    .bind(position_y)
    .bind(sort_order)
    .bind(next_step_uuid)
    .bind(branch_config)
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

/// 查询任务步骤列表
pub async fn fetch_rpa_task_steps(
    pool: &Pool<Postgres>,
    task_uuid: Uuid,
) -> Result<Vec<RpaTaskStepDto>, Error> {
    let recs = sqlx::query_as::<_, RpaTaskStepDto>(
        r#"
        SELECT id, uuid, task_uuid, step_type, name, config, enabled,
               position_x, position_y, sort_order, next_step_uuid, branch_config,
               created_at, updated_at
        FROM rpa_task_steps
        WHERE task_uuid = $1
        ORDER BY sort_order ASC, id ASC
        "#,
    )
    .bind(task_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 删除任务所有步骤
pub async fn delete_rpa_task_steps(pool: &Pool<Postgres>, task_uuid: Uuid) -> Result<(), Error> {
    sqlx::query("DELETE FROM rpa_task_steps WHERE task_uuid = $1")
        .bind(task_uuid)
        .execute(pool)
        .await?;

    Ok(())
}

// ============ RPA Task Environments ============

/// 添加任务环境关联
pub async fn insert_rpa_task_environment(
    pool: &Pool<Postgres>,
    task_uuid: Uuid,
    environment_uuid: Uuid,
    sort_order: Option<i32>,
) -> Result<i32, Error> {
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO rpa_task_environments (task_uuid, environment_uuid, sort_order)
        VALUES ($1, $2, $3)
        RETURNING id;
        "#,
    )
    .bind(task_uuid)
    .bind(environment_uuid)
    .bind(sort_order)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// 查询任务环境关联
pub async fn fetch_rpa_task_environments(
    pool: &Pool<Postgres>,
    task_uuid: Uuid,
) -> Result<Vec<RpaTaskEnvironmentDto>, Error> {
    let recs = sqlx::query_as::<_, RpaTaskEnvironmentDto>(
        r#"
        SELECT id, task_uuid, environment_uuid, sort_order, created_at
        FROM rpa_task_environments
        WHERE task_uuid = $1
        ORDER BY sort_order ASC, id ASC
        "#,
    )
    .bind(task_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 删除任务所有环境关联
pub async fn delete_rpa_task_environments(
    pool: &Pool<Postgres>,
    task_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query("DELETE FROM rpa_task_environments WHERE task_uuid = $1")
        .bind(task_uuid)
        .execute(pool)
        .await?;

    Ok(())
}

// ============ RPA Task Runs ============

/// 创建任务执行记录
pub async fn insert_rpa_task_run(
    pool: &Pool<Postgres>,
    task_uuid: Uuid,
    total_steps: i32,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO rpa_task_runs (task_uuid, status, total_steps, completed_steps, failed_steps)
        VALUES ($1, 'running', $2, 0, 0)
        RETURNING uuid;
        "#,
    )
    .bind(task_uuid)
    .bind(total_steps)
    .fetch_one(pool)
    .await?;

    // 更新任务最后运行时间和运行次数
    sqlx::query(
        r#"
        UPDATE rpa_tasks
        SET last_run_at = CURRENT_TIMESTAMP,
            run_count = COALESCE(run_count, 0) + 1
        WHERE uuid = $1
        "#,
    )
    .bind(task_uuid)
    .execute(pool)
    .await?;

    Ok(uuid)
}

/// 查询任务执行记录列表
pub async fn fetch_rpa_task_runs(
    pool: &Pool<Postgres>,
    task_uuid: Uuid,
    status: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<RpaTaskRunDto>, Error> {
    let recs = sqlx::query_as::<_, RpaTaskRunDto>(
        r#"
        SELECT id, uuid, task_uuid, status, total_steps, completed_steps, failed_steps,
               started_at, finished_at, duration_ms, result_summary, error_message, logs
        FROM rpa_task_runs
        WHERE task_uuid = $1
          AND ($2::varchar IS NULL OR status = $2)
        ORDER BY started_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(task_uuid)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询任务执行记录总数
pub async fn fetch_rpa_task_runs_count(
    pool: &Pool<Postgres>,
    task_uuid: Uuid,
    status: Option<&str>,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM rpa_task_runs
        WHERE task_uuid = $1
          AND ($2::varchar IS NULL OR status = $2)
        "#,
    )
    .bind(task_uuid)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 根据 UUID 查询执行记录
pub async fn fetch_rpa_task_run_by_uuid(
    pool: &Pool<Postgres>,
    run_uuid: Uuid,
) -> Result<Option<RpaTaskRunDto>, Error> {
    let rec = sqlx::query_as::<_, RpaTaskRunDto>(
        r#"
        SELECT id, uuid, task_uuid, status, total_steps, completed_steps, failed_steps,
               started_at, finished_at, duration_ms, result_summary, error_message, logs
        FROM rpa_task_runs
        WHERE uuid = $1
        "#,
    )
    .bind(run_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 更新执行记录状态
pub async fn update_rpa_task_run_status(
    pool: &Pool<Postgres>,
    run_uuid: Uuid,
    status: &str,
    completed_steps: i32,
    failed_steps: i32,
    result_summary: Option<&str>,
    error_message: Option<&str>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE rpa_task_runs
        SET status = $1,
            completed_steps = $2,
            failed_steps = $3,
            result_summary = $4,
            error_message = $5,
            finished_at = CASE WHEN $1 IN ('completed', 'failed', 'stopped') THEN CURRENT_TIMESTAMP ELSE finished_at END,
            duration_ms = CASE WHEN $1 IN ('completed', 'failed', 'stopped')
                          THEN EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - started_at)) * 1000
                          ELSE duration_ms END
        WHERE uuid = $6
        "#,
    )
    .bind(status)
    .bind(completed_steps)
    .bind(failed_steps)
    .bind(result_summary)
    .bind(error_message)
    .bind(run_uuid)
    .execute(pool)
    .await?;

    Ok(())
}


