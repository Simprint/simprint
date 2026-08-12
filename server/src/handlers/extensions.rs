use axum::extract::{Extension, State};

use crate::dto::ExtensionDto;
use crate::entitys::{
    BatchUpdateExtensionsRequest, BatchUpdateResponse, ExtensionIdRequest, ExtensionsListResponse,
    GetInstalledExtensionsRequest, InstallExtensionRequest, InstalledExtensionsResponse,
    ListExtensionsRequest, ToggleExtensionRequest, UninstallExtensionRequest, UpdateExtensionRequest,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 获取扩展列表
pub async fn get_extensions_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<ListExtensionsRequest>,
) -> Result<ExtensionsListResponse> {
    let (items, total) = services::extensions::get_extensions_service(&svc_ctx, &payload)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("获取成功"),
        Some(ExtensionsListResponse {
            items,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}

/// 获取扩展详情
pub async fn get_extension_detail_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<ExtensionIdRequest>,
) -> Result<ExtensionDto> {
    let extension = services::extensions::get_extension_service(&svc_ctx, &payload.extension_id)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(extension)))
}

/// 获取扩展分类
pub async fn get_extension_categories_handler(
    State(svc_ctx): State<SvcCtx>,
) -> Result<Vec<String>> {
    let categories = services::extensions::get_extension_categories_service(&svc_ctx)
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("获取成功"), Some(categories)))
}

/// 获取已安装的扩展
pub async fn get_installed_extensions_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<GetInstalledExtensionsRequest>,
) -> Result<InstalledExtensionsResponse> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    let (user_extensions, team_extensions) = match payload.scope.as_str() {
        "user" | "personal" => {
            let user_extensions = services::extensions::get_user_installed_extensions_service(
                &svc_ctx,
                ctx.user_uuid_unwrap(),
                team_uuid,
            )
            .await
            .map_err(|e| Response::fail(Some(&e)))?;
            (user_extensions, Vec::new())
        }
        "team" => {
            let team_uuid = team_uuid.ok_or_else(|| Response::fail(Some("未指定团队")))?;
            let team_extensions =
                services::extensions::get_team_installed_extensions_service(&svc_ctx, team_uuid, ctx.user_uuid_unwrap())
                    .await
                    .map_err(|e| Response::fail(Some(&e)))?;
            (Vec::new(), team_extensions)
        }
        _ => {
            // "all" 或其他值，返回全部
            let user_extensions = services::extensions::get_user_installed_extensions_service(
                &svc_ctx,
                ctx.user_uuid_unwrap(),
                team_uuid,
            )
            .await
            .map_err(|e| Response::fail(Some(&e)))?;
            let team_extensions = if let Some(team_uuid) = team_uuid {
                services::extensions::get_team_installed_extensions_service(&svc_ctx, team_uuid, ctx.user_uuid_unwrap())
                    .await
                    .map_err(|e| Response::fail(Some(&e)))?
            } else {
                Vec::new()
            };
            (user_extensions, team_extensions)
        }
    };

    Ok(Response::success(
        Some("获取成功"),
        Some(InstalledExtensionsResponse {
            user_extensions,
            team_extensions,
        }),
    ))
}

/// 安装扩展
pub async fn install_extension_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<InstallExtensionRequest>,
) -> Result<()> {
    let workspace_uuid = ctx.current_workspace_uuid.ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    services::extensions::install_extension_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("安装成功"), None))
}

/// 卸载扩展
pub async fn uninstall_extension_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UninstallExtensionRequest>,
) -> Result<()> {
    let workspace_uuid = ctx.current_workspace_uuid.ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?;

    services::extensions::uninstall_extension_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        team_uuid,
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("卸载成功"), None))
}

/// 更新扩展
pub async fn update_extension_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdateExtensionRequest>,
) -> Result<()> {
    services::extensions::update_extension_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload.extension_id,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("更新成功"), None))
}

/// 批量更新扩展
pub async fn batch_update_extensions_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<BatchUpdateExtensionsRequest>,
) -> Result<BatchUpdateResponse> {
    let updated = services::extensions::batch_update_extensions_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload.extension_ids,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        Some("批量更新完成"),
        Some(BatchUpdateResponse { updated }),
    ))
}

/// 禁用扩展（用户级别）
pub async fn disable_extension_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ToggleExtensionRequest>,
) -> Result<()> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?
        .ok_or_else(|| Response::fail(Some("未指定团队")))?;

    services::extensions::disable_extension_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        team_uuid,
        &payload.extension_id,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("禁用成功"), None))
}

/// 启用扩展（用户级别）
pub async fn enable_extension_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ToggleExtensionRequest>,
) -> Result<()> {
    let team_uuid = services::teams::get_current_team_service(&svc_ctx, ctx.user_uuid_unwrap())
        .await
        .map_err(|e| Response::fail(Some(&e)))?
        .ok_or_else(|| Response::fail(Some("未指定团队")))?;

    services::extensions::enable_extension_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        team_uuid,
        &payload.extension_id,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(Some("启用成功"), None))
}
