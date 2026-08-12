use uuid::Uuid;

use crate::dto::{RpaTaskDto, RpaTaskStepDto};
use crate::entitys::{
    CreateRpaTaskRequest, DuplicateRpaTaskRequest, ListRpaTasksRequest, UpdateRpaTaskRequest,
};
use crate::models;
use crate::svc_ctx::SvcCtx;

/// List RPA tasks.
pub async fn get_rpa_tasks_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    payload: &ListRpaTasksRequest,
) -> Result<(Vec<RpaTaskDto>, i64), String> {
    let offset = (payload.pagination.page - 1) * payload.pagination.page_size;

    let keyword = payload.filters.as_ref().and_then(|f| f.keyword.as_deref());
    let status = payload.filters.as_ref().and_then(|f| f.status.as_deref());
    let trigger_type = payload.filters.as_ref().and_then(|f| f.trigger_type.as_deref());

    let tasks = models::rpa::fetch_rpa_tasks(
        &svc_ctx.db,
        team_uuid,
        user_uuid,
        keyword,
        status,
        trigger_type,
        offset,
        payload.pagination.page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    let total =
        models::rpa::fetch_rpa_tasks_count(
            &svc_ctx.db,
            team_uuid,
            user_uuid,
            keyword,
            status,
            trigger_type,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok((tasks, total))
}

/// Get RPA task detail.
pub async fn get_rpa_task_service(
    svc_ctx: &SvcCtx,
    task_uuid: Uuid,
) -> Result<(RpaTaskDto, Vec<RpaTaskStepDto>, Vec<Uuid>), String> {
    let task = models::rpa::fetch_rpa_task_by_uuid(&svc_ctx.db, task_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "RPA task not found".to_string())?;

    let steps = models::rpa::fetch_rpa_task_steps(&svc_ctx.db, task_uuid)
        .await
        .map_err(|e| e.to_string())?;

    let environments = models::rpa::fetch_rpa_task_environments(&svc_ctx.db, task_uuid)
        .await
        .map_err(|e| e.to_string())?;

    let environment_uuids: Vec<Uuid> =
        environments.into_iter().map(|e| e.environment_uuid).collect();

    Ok((task, steps, environment_uuids))
}

/// Create an RPA task.
pub async fn create_rpa_task_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    payload: &CreateRpaTaskRequest,
) -> Result<Uuid, String> {
    let tags_json = payload.tags.as_ref().map(|t| serde_json::json!(t));

    let task_uuid = models::rpa::insert_rpa_task(
        &svc_ctx.db,
        user_uuid,
        team_uuid,
        &payload.name,
        payload.description.as_deref(),
        tags_json.as_ref(),
        &payload.trigger_type,
        payload.schedule.as_deref(),
        payload.cron_expression.as_deref(),
        &payload.run_mode,
        payload.retry_count,
        payload.retry_interval,
        payload.timeout,
        payload.concurrency,
        payload.stop_on_error,
        payload.notify_on_complete,
        payload.notify_on_error,
    )
    .await
    .map_err(|e| e.to_string())?;

    // Persist ordered steps.
    if let Some(steps) = &payload.steps {
        for (i, step) in steps.iter().enumerate() {
            models::rpa::insert_rpa_task_step(
                &svc_ctx.db,
                task_uuid,
                &step.step_type,
                &step.name,
                &step.config,
                step.enabled,
                step.position_x,
                step.position_y,
                Some(step.sort_order.unwrap_or(i as i32)),
                step.next_step_uuid,
                step.branch_config.as_ref(),
            )
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    // Persist environment bindings.
    if let Some(env_uuids) = &payload.environment_uuids {
        for (i, env_uuid) in env_uuids.iter().enumerate() {
            models::rpa::insert_rpa_task_environment(
                &svc_ctx.db,
                task_uuid,
                *env_uuid,
                Some(i as i32),
            )
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(task_uuid)
}

/// Update an RPA task.
pub async fn update_rpa_task_service(
    svc_ctx: &SvcCtx,
    payload: &UpdateRpaTaskRequest,
) -> Result<(), String> {
    let tags_json = payload.tags.as_ref().map(|t| serde_json::json!(t));

    models::rpa::update_rpa_task(
        &svc_ctx.db,
        payload.uuid,
        payload.name.as_deref(),
        payload.description.as_deref(),
        tags_json.as_ref(),
        payload.trigger_type.as_deref(),
        payload.schedule.as_deref(),
        payload.cron_expression.as_deref(),
        payload.run_mode.as_deref(),
        payload.retry_count,
        payload.retry_interval,
        payload.timeout,
        payload.concurrency,
        payload.stop_on_error,
        payload.notify_on_complete,
        payload.notify_on_error,
    )
    .await
    .map_err(|e| e.to_string())?;

    // Replace stored steps when the caller sends a full step list.
    if let Some(steps) = &payload.steps {
        // Remove previous step rows first.
        models::rpa::delete_rpa_task_steps(&svc_ctx.db, payload.uuid)
            .await
            .map_err(|e| e.to_string())?;

        // Insert the new step rows.
        for (i, step) in steps.iter().enumerate() {
            models::rpa::insert_rpa_task_step(
                &svc_ctx.db,
                payload.uuid,
                &step.step_type,
                &step.name,
                &step.config,
                step.enabled,
                step.position_x,
                step.position_y,
                Some(step.sort_order.unwrap_or(i as i32)),
                step.next_step_uuid,
                step.branch_config.as_ref(),
            )
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    // Replace environment bindings when provided.
    if let Some(env_uuids) = &payload.environment_uuids {
        // Remove previous environment bindings.
        models::rpa::delete_rpa_task_environments(&svc_ctx.db, payload.uuid)
            .await
            .map_err(|e| e.to_string())?;

        // Insert the new environment bindings.
        for (i, env_uuid) in env_uuids.iter().enumerate() {
            models::rpa::insert_rpa_task_environment(
                &svc_ctx.db,
                payload.uuid,
                *env_uuid,
                Some(i as i32),
            )
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

/// Soft-delete an RPA task.
pub async fn delete_rpa_task_service(svc_ctx: &SvcCtx, task_uuid: Uuid) -> Result<(), String> {
    models::rpa::delete_rpa_task(&svc_ctx.db, task_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// Soft-delete RPA tasks in batch.
pub async fn batch_delete_rpa_tasks_service(
    svc_ctx: &SvcCtx,
    task_uuids: &[Uuid],
) -> Result<u64, String> {
    models::rpa::batch_delete_rpa_tasks(&svc_ctx.db, task_uuids)
        .await
        .map_err(|e| e.to_string())
}

/// Duplicate an RPA task.
pub async fn duplicate_rpa_task_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    payload: &DuplicateRpaTaskRequest,
) -> Result<Uuid, String> {
    // Load source task data.
    let (task, steps, environment_uuids) = get_rpa_task_service(svc_ctx, payload.uuid).await?;

    let new_name = payload
        .new_name
        .clone()
        .unwrap_or_else(|| format!("{} (copy)", task.name));

    // Create duplicated task row.
    let new_task_uuid = models::rpa::insert_rpa_task(
        &svc_ctx.db,
        user_uuid,
        team_uuid,
        &new_name,
        task.description.as_deref(),
        task.tags.as_ref(),
        &task.trigger_type,
        task.schedule.as_deref(),
        task.cron_expression.as_deref(),
        &task.run_mode,
        task.retry_count,
        task.retry_interval,
        task.timeout,
        task.concurrency,
        task.stop_on_error,
        task.notify_on_complete,
        task.notify_on_error,
    )
    .await
    .map_err(|e| e.to_string())?;

    // Copy steps.
    for step in steps {
        models::rpa::insert_rpa_task_step(
            &svc_ctx.db,
            new_task_uuid,
            &step.step_type,
            &step.name,
            &step.config,
            step.enabled,
            step.position_x,
            step.position_y,
            step.sort_order,
            step.next_step_uuid,
            step.branch_config.as_ref(),
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    // Copy environment bindings.
    for (i, env_uuid) in environment_uuids.iter().enumerate() {
        models::rpa::insert_rpa_task_environment(
            &svc_ctx.db,
            new_task_uuid,
            *env_uuid,
            Some(i as i32),
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(new_task_uuid)
}

/// Export an RPA task.
pub async fn export_rpa_task_service(
    svc_ctx: &SvcCtx,
    task_uuid: Uuid,
) -> Result<(String, String), String> {
    let (task, steps, environment_uuids) = get_rpa_task_service(svc_ctx, task_uuid).await?;

    let export_data = serde_json::json!({
        "name": task.name,
        "description": task.description,
        "tags": task.tags,
        "trigger_type": task.trigger_type,
        "schedule": task.schedule,
        "cron_expression": task.cron_expression,
        "run_mode": task.run_mode,
        "retry_count": task.retry_count,
        "retry_interval": task.retry_interval,
        "timeout": task.timeout,
        "concurrency": task.concurrency,
        "stop_on_error": task.stop_on_error,
        "notify_on_complete": task.notify_on_complete,
        "notify_on_error": task.notify_on_error,
        "steps": steps.iter().map(|s| serde_json::json!({
            "step_type": s.step_type,
            "name": s.name,
            "config": s.config,
            "enabled": s.enabled,
            "position_x": s.position_x,
            "position_y": s.position_y,
            "sort_order": s.sort_order,
            "next_step_uuid": s.next_step_uuid,
            "branch_config": s.branch_config,
        })).collect::<Vec<_>>(),
        "environment_uuids": environment_uuids,
    });

    let content = serde_json::to_string_pretty(&export_data).map_err(|e| e.to_string())?;
    let filename = format!("rpa_task_{}.json", task_uuid);

    Ok((content, filename))
}



