use axum::{extract::Extension, extract::State};

use crate::audit_log;
use crate::entitys::{
    CheckGroupPermissionRequest, GrantGroupPermissionRequest, ListUserGroupPermissionsRequest,
    RevokeGroupPermissionRequest,
};
use crate::services;
use crate::state::RequestContext;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 授予分组权限
pub async fn grant_group_permission_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<GrantGroupPermissionRequest>,
) -> Result<()> {
    services::group_permissions::grant_group_permission_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "grant",
        "group_permission",
        payload.group_uuid,
        &payload.permission_type,
        "授予分组权限"
    )
    .await;

    Ok(Response::success(Some("授权成功"), None))
}

/// 撤销分组权限
pub async fn revoke_group_permission_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<RevokeGroupPermissionRequest>,
) -> Result<()> {
    services::group_permissions::revoke_group_permission_service(
        &svc_ctx,
        ctx.user_uuid_unwrap(),
        &payload,
    )
    .await
    .map_err(|e| Response::fail(Some(&e)))?;

    audit_log!(
        &svc_ctx,
        &ctx,
        "revoke",
        "group_permission",
        payload.group_uuid,
        "",
        "撤销分组权限"
    )
    .await;

    Ok(Response::success(Some("撤销成功"), None))
}

/// 检查分组权限
pub async fn check_group_permission_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CheckGroupPermissionRequest>,
) -> Result<crate::entitys::CheckPermissionResponse> {
    let workspace_uuid = ctx.current_workspace_uuid.ok_or_else(|| Response::fail(Some("请先选择工作空间")))?;
    let has_permission =
        services::group_permissions::check_group_permission_service(&svc_ctx, workspace_uuid, &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    Ok(Response::success(
        None,
        Some(crate::entitys::CheckPermissionResponse {
            has_permission,
            permission_type: if has_permission {
                Some(payload.permission_type.clone())
            } else {
                None
            },
        }),
    ))
}

/// 查询用户的分组权限列表
pub async fn list_user_group_permissions_handler(
    State(svc_ctx): State<SvcCtx>,
    Extension(_ctx): Extension<RequestContext>,
    Json(payload): Json<ListUserGroupPermissionsRequest>,
) -> Result<crate::entitys::GroupPermissionListResponse> {
    let permissions =
        services::group_permissions::list_user_group_permissions_service(&svc_ctx, &payload)
            .await
            .map_err(|e| Response::fail(Some(&e)))?;

    let total = permissions.len() as i64;

    // 转换为详情 DTO（包含用户信息）
    let mut permission_details = vec![];
    for perm in permissions {
        let user_info = crate::models::user::fetch_user_info_by_uuid(&svc_ctx.db, perm.user_uuid)
            .await
            .ok()
            .flatten();

        permission_details.push(crate::dto::GroupMemberPermissionDetailDto {
            group_uuid: perm.group_uuid,
            workspace_uuid: perm.workspace_uuid,
            team_uuid: perm.team_uuid,
            user_uuid: perm.user_uuid,
            permission_type: perm.permission_type,
            granted_by: perm.granted_by,
            user_name: user_info.as_ref().and_then(|u| u.nickname.clone()),
            user_email: user_info.as_ref().map(|u| u.email.clone()),
            created_at: perm.created_at,
            updated_at: perm.updated_at,
        });
    }

    Ok(Response::success(
        None,
        Some(crate::entitys::GroupPermissionListResponse {
            items: permission_details,
            total,
            page: payload.pagination.page,
            page_size: payload.pagination.page_size,
        }),
    ))
}
