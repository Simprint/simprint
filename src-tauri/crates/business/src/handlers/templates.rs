use axum::{extract::Extension, extract::State};

use crate::entitys::{
    CreateFromTemplateRequest, CreateResponse, CreateTemplateRequest, GetTemplateRequest,
    ListTemplatesRequest, TemplateDetailResponse, TemplateListResponse, UpdateTemplateRequest,
    UuidRequest,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 获取模板列表
pub async fn get_templates_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ListTemplatesRequest>,
) -> Result<TemplateListResponse> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let (templates, total) = services::templates::get_templates_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        team_uuid,
        payload.is_public,
        payload.pagination.page,
        payload.pagination.page_size,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(TemplateListResponse {
            items: templates,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

/// 获取模板详情
pub async fn get_template_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<GetTemplateRequest>,
) -> Result<TemplateDetailResponse> {
    let result = services::templates::get_template_service(
        &svc_ctx,
        payload.uuid,
        payload.for_create.unwrap_or(false),
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(result)))
}

/// 创建模板
pub async fn create_template_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<CreateResponse> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let template_uuid = services::templates::create_template_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("创建成功"),
        Some(CreateResponse {
            uuid: template_uuid,
        }),
    ))
}

/// 更新模板
pub async fn update_template_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UpdateTemplateRequest>,
) -> Result<()> {
    services::templates::update_template_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("更新成功"), None))
}

/// 删除模板
pub async fn delete_template_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<UuidRequest>,
) -> Result<()> {
    services::templates::delete_template_service(&svc_ctx, payload.uuid)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("删除成功"), None))
}

/// 应用模板到环境（更新环境配置为模板配置）
pub async fn apply_template_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<crate::entitys::ApplyTemplateRequest>,
) -> Result<()> {
    let workspace_uuid = ctx.current_workspace_uuid.ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let team_uuid = ctx.current_team_uuid.ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    services::templates::apply_template_service(
        &svc_ctx,
        workspace_uuid,
        team_uuid,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("应用成功"), None))
}

/// 从模板创建环境
pub async fn create_from_template_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateFromTemplateRequest>,
) -> Result<CreateResponse> {
    let workspace_uuid = ctx.current_workspace_uuid.ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let team_uuid = ctx.current_team_uuid.ok_or_else(|| Response::fail(Some("请先选择团队")))?;

    let env_uuid = services::templates::create_from_template_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        workspace_uuid,
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("创建成功"),
        Some(CreateResponse { uuid: env_uuid }),
    ))
}
