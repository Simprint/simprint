use axum::extract::{Extension, State};

use crate::entitys::{
    BatchUuidRequest, CreateResponse, CreateRpaTaskRequest, DuplicateRpaTaskRequest,
    ExportRpaTaskRequest, ExportRpaTaskResponse, ListRpaTasksRequest, RpaTaskDetailResponse,
    RpaTaskListResponse, UpdateRpaTaskRequest, UuidRequest,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

pub async fn get_rpa_tasks_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListRpaTasksRequest>,
) -> Result<RpaTaskListResponse> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let (items, total) =
        services::rpa::get_rpa_tasks_service(&svc_ctx, ctx.user_uuid_unwrap(), team_uuid, &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("ok"),
        Some(RpaTaskListResponse {
            items,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

pub async fn get_rpa_task_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<RpaTaskDetailResponse> {
    let (task, steps, environment_uuids) =
        services::rpa::get_rpa_task_service(&svc_ctx, payload.uuid)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("ok"),
        Some(RpaTaskDetailResponse {
            task,
            steps,
            environment_uuids,
        }),
    ))
}

pub async fn create_rpa_task_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateRpaTaskRequest>,
) -> Result<CreateResponse> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let uuid = services::rpa::create_rpa_task_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("created"), Some(CreateResponse { uuid })))
}

pub async fn update_rpa_task_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UpdateRpaTaskRequest>,
) -> Result<()> {
    services::rpa::update_rpa_task_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("updated"), None))
}

pub async fn delete_rpa_task_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<()> {
    services::rpa::delete_rpa_task_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("deleted"), None))
}

pub async fn batch_delete_rpa_tasks_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<BatchUuidRequest>,
) -> Result<()> {
    services::rpa::batch_delete_rpa_tasks_service(&svc_ctx, &payload.uuids)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("deleted"), None))
}

pub async fn duplicate_rpa_task_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<DuplicateRpaTaskRequest>,
) -> Result<CreateResponse> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let uuid = services::rpa::duplicate_rpa_task_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("duplicated"), Some(CreateResponse { uuid })))
}

pub async fn export_rpa_task_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<ExportRpaTaskRequest>,
) -> Result<ExportRpaTaskResponse> {
    let (content, filename) = services::rpa::export_rpa_task_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("exported"),
        Some(ExportRpaTaskResponse { content, filename }),
    ))
}
