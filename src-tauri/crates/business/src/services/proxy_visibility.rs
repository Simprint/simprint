use uuid::Uuid;

use crate::dto::ProxyDto;
use crate::entitys::{
    BatchSetProxyVisibleRequest, ListVisibleProxiesRequest,
    RemoveProxyVisibleRequest, SetProxyVisibleRequest,
};
use crate::models;
use crate::svc_ctx::SvcCtx;

/// 设置代理对团队可见
pub async fn set_proxy_visible_to_team_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &SetProxyVisibleRequest,
) -> Result<(), String> {
    // 检查权限：只有代理所有者或工作空间所有者可以设置可见性
    let proxy = models::fetch_proxy_by_uuid(&svc_ctx.db, payload.proxy_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "代理不存在".to_string())?;

    let is_owner = proxy.owner_uuid == user_uuid;
    let is_workspace_owner =
        models::check_workspace_owner(&svc_ctx.db, proxy.workspace_uuid, user_uuid)
            .await
            .map_err(|e| e.to_string())?;

    if !is_owner && !is_workspace_owner {
        return Err("只有代理所有者或工作空间所有者可以设置可见性".to_string());
    }

    // 获取团队的工作空间
    let team = models::fetch_team_by_uuid(&svc_ctx.db, payload.team_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "团队不存在".to_string())?;

    if team.workspace_uuid != proxy.workspace_uuid {
        return Err("团队和工作空间不匹配".to_string());
    }

    models::insert_proxy_visible_team(
        &svc_ctx.db,
        payload.proxy_uuid,
        proxy.workspace_uuid,
        payload.team_uuid,
    )
    .await
    .map_err(|e| e.to_string())
}

/// 移除代理对团队的可见性
pub async fn remove_proxy_visible_from_team_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &RemoveProxyVisibleRequest,
) -> Result<(), String> {
    // 检查权限：只有代理所有者或工作空间所有者可以移除可见性
    let proxy = models::fetch_proxy_by_uuid(&svc_ctx.db, payload.proxy_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "代理不存在".to_string())?;

    let is_owner = proxy.owner_uuid == user_uuid;
    let is_workspace_owner =
        models::check_workspace_owner(&svc_ctx.db, proxy.workspace_uuid, user_uuid)
            .await
            .map_err(|e| e.to_string())?;

    if !is_owner && !is_workspace_owner {
        return Err("只有代理所有者或工作空间所有者可以移除可见性".to_string());
    }

    models::remove_proxy_visible_team(&svc_ctx.db, payload.proxy_uuid, payload.team_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 批量设置代理可见性
pub async fn batch_set_proxy_visible_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &BatchSetProxyVisibleRequest,
) -> Result<(), String> {
    // 检查权限
    let proxy = models::fetch_proxy_by_uuid(&svc_ctx.db, payload.proxy_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "代理不存在".to_string())?;

    let is_owner = proxy.owner_uuid == user_uuid;
    let is_workspace_owner =
        models::check_workspace_owner(&svc_ctx.db, proxy.workspace_uuid, user_uuid)
            .await
            .map_err(|e| e.to_string())?;

    if !is_owner && !is_workspace_owner {
        return Err("只有代理所有者或工作空间所有者可以设置可见性".to_string());
    }

    // 批量设置可见性
    for team_uuid in &payload.team_uuids {
        models::insert_proxy_visible_team(
            &svc_ctx.db,
            payload.proxy_uuid,
            proxy.workspace_uuid,
            *team_uuid,
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// 获取可见的代理列表
pub async fn get_visible_proxies_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &ListVisibleProxiesRequest,
) -> Result<Vec<ProxyDto>, String> {
    models::fetch_visible_proxies_for_user(
        &svc_ctx.db,
        payload.workspace_uuid,
        user_uuid,
        payload.team_uuid,
    )
    .await
    .map_err(|e| e.to_string())
}

/// 检查代理可见性
pub async fn check_proxy_visibility_service(
    svc_ctx: &SvcCtx,
    proxy_uuid: Uuid,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
) -> Result<bool, String> {
    models::check_proxy_visibility(&svc_ctx.db, proxy_uuid, workspace_uuid, team_uuid)
        .await
        .map_err(|e| e.to_string())
}
