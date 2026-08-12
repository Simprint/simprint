use crate::{
    dto::maintenance::Maintenance, entitys::maintenance::CreateMaintenanceRequest,
    errors::SimprintError, svc_ctx::SvcCtx,
};

/// 创建维护
pub async fn create_maintenance_service(
    svc_ctx: &SvcCtx,
    request: CreateMaintenanceRequest,
) -> Result<i64, SimprintError> {
    let maintenance = crate::models::maintenance::create_maintenance(&svc_ctx.db, request)
        .await
        .map_err(|e| SimprintError::InvalidRequest(format!("创建维护失败: {}", e)))?;

    Ok(maintenance.id)
}

/// 根据ID查询维护
pub async fn get_maintenance_by_id_service(
    svc_ctx: &SvcCtx,
    id: i64,
) -> Result<Option<Maintenance>, SimprintError> {
    let maintenance = crate::models::maintenance::get_maintenance_by_id(&svc_ctx.db, id)
        .await
        .map_err(|e| SimprintError::InvalidRequest(format!("查询维护失败: {}", e)))?;

    Ok(maintenance)
}

/// 查询维护列表
pub async fn list_maintenances_service(
    svc_ctx: &SvcCtx,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<Maintenance>, SimprintError> {
    let maintenances = crate::models::maintenance::list_maintenances(
        &svc_ctx.db,
        limit.map(|l| l as i64),
        offset.map(|o| o as i64),
    )
    .await
    .map_err(|e| SimprintError::InvalidRequest(format!("查询维护列表失败: {}", e)))?;

    Ok(maintenances)
}

/// 更新维护状态
pub async fn update_maintenance_status_service(
    svc_ctx: &SvcCtx,
    id: i64,
    status: String,
) -> Result<bool, SimprintError> {
    let success = crate::models::maintenance::update_maintenance_status(&svc_ctx.db, id, &status)
        .await
        .map_err(|e| SimprintError::InvalidRequest(format!("更新维护状态失败: {}", e)))?;

    Ok(success)
}

/// 结束维护
pub async fn end_maintenance_service(svc_ctx: &SvcCtx) -> Result<bool, SimprintError> {
    let success = crate::models::maintenance::end_maintenance(&svc_ctx.db)
        .await
        .map_err(|e| SimprintError::InvalidRequest(format!("结束维护失败: {}", e)))?;

    Ok(success)
}

/// 获取当前活跃维护
pub async fn get_active_maintenance_service(
    svc_ctx: &SvcCtx,
) -> Result<Option<Maintenance>, SimprintError> {
    let maintenance = crate::models::maintenance::get_active_maintenances(&svc_ctx.db)
        .await
        .map_err(|e| SimprintError::InvalidRequest(format!("查询活跃维护失败: {}", e)))?;

    Ok(maintenance)
}
