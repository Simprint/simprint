use uuid::Uuid;

use crate::dto::WorkspaceQuotaDto;
use crate::entitys::{GetWorkspaceQuotaRequest, UpdateQuotaUsageRequest};
use crate::models;
use crate::svc_ctx::SvcCtx;

/// 获取工作空间配额
pub async fn get_workspace_quota_service(
    svc_ctx: &SvcCtx,
    payload: &GetWorkspaceQuotaRequest,
) -> Result<WorkspaceQuotaDto, String> {
    let workspace_uuid = payload
        .workspace_uuid
        .ok_or_else(|| "工作空间 UUID 不能为空".to_string())?;

    models::fetch_workspace_quota(&svc_ctx.db, workspace_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "配额不存在".to_string())
}

/// 检查配额是否充足
pub async fn check_quota_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    quota_type: &str,
) -> Result<bool, String> {
    models::check_quota(&svc_ctx.db, workspace_uuid, quota_type)
        .await
        .map_err(|e| e.to_string())
}

/// 更新配额使用情况
pub async fn update_quota_usage_service(
    svc_ctx: &SvcCtx,
    payload: &UpdateQuotaUsageRequest,
) -> Result<(), String> {
    match payload.quota_type.as_str() {
        "environments" => {
            if payload.increment {
                models::increment_used_environments(
                    &svc_ctx.db,
                    payload.workspace_uuid,
                    payload.amount,
                )
                .await
            } else {
                models::decrement_used_environments(
                    &svc_ctx.db,
                    payload.workspace_uuid,
                    payload.amount,
                )
                .await
            }
        }
        "proxies" => {
            if payload.increment {
                models::increment_used_proxies(&svc_ctx.db, payload.workspace_uuid, payload.amount)
                    .await
            } else {
                models::decrement_used_proxies(&svc_ctx.db, payload.workspace_uuid, payload.amount)
                    .await
            }
        }
        "team_members" => {
            models::update_used_team_members(&svc_ctx.db, payload.workspace_uuid).await
        }
        _ => return Err("不支持的配额类型".to_string()),
    }
    .map_err(|e| e.to_string())
}
