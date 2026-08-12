use crate::{
    dto::version_types::VersionType, entitys::version_types::*, errors::SimprintError,
    svc_ctx::SvcCtx,
};

/// 创建版本类型
pub async fn create_version_type(
    svc_ctx: &SvcCtx,
    request: CreateVersionTypeRequest,
) -> Result<i32, SimprintError> {
    // 验证类型代码
    if request.type_code.is_empty() {
        return Err(SimprintError::Other("版本类型代码不能为空".to_string()));
    }

    // 检查版本类型是否已存在
    let exists =
        crate::models::version_types::query_version_type_by_code(&svc_ctx.db, &request.type_code)
            .await
            .ok();

    if exists.is_some() {
        return Err(SimprintError::Other("版本类型已存在".to_string()));
    }

    // 创建版本类型
    let id = crate::models::version_types::insert_version_type(&svc_ctx.db, &request).await?;

    Ok(id)
}

/// 根据ID查询版本类型
pub async fn get_version_type_by_id(
    svc_ctx: &SvcCtx,
    id: i32,
) -> Result<VersionType, SimprintError> {
    let version_type = crate::models::version_types::query_version_type_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::Other("版本类型不存在".to_string()))?;

    Ok(version_type)
}

/// 根据代码查询版本类型
pub async fn get_version_type_by_code(
    svc_ctx: &SvcCtx,
    type_code: String,
) -> Result<VersionType, SimprintError> {
    let version_type =
        crate::models::version_types::query_version_type_by_code(&svc_ctx.db, &type_code)
            .await
            .map_err(|_| SimprintError::Other("版本类型不存在".to_string()))?;

    Ok(version_type)
}

/// 查询所有版本类型
pub async fn query_all_version_types_service(
    svc_ctx: &SvcCtx,
) -> Result<VersionTypeListResponse, SimprintError> {
    let list = crate::models::version_types::query_all_version_types(&svc_ctx.db).await?;

    Ok(VersionTypeListResponse { list })
}

/// 查询激活的版本类型
pub async fn query_active_version_types_service(
    svc_ctx: &SvcCtx,
) -> Result<VersionTypeListResponse, SimprintError> {
    let list = crate::models::version_types::query_active_version_types(&svc_ctx.db).await?;

    Ok(VersionTypeListResponse { list })
}

/// 更新版本类型
pub async fn update_version_type_service(
    svc_ctx: &SvcCtx,
    id: i32,
    request: UpdateVersionTypeRequest,
) -> Result<bool, SimprintError> {
    // 检查版本类型是否存在
    crate::models::version_types::query_version_type_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::Other("版本类型不存在".to_string()))?;

    // 更新版本类型
    let success =
        crate::models::version_types::update_version_type(&svc_ctx.db, id, &request).await?;

    if !success {
        return Err(SimprintError::Other("更新版本类型失败".to_string()));
    }

    Ok(true)
}

/// 删除版本类型
pub async fn delete_version_type_service(svc_ctx: &SvcCtx, id: i32) -> Result<bool, SimprintError> {
    let success = crate::models::version_types::delete_version_type(&svc_ctx.db, id).await?;

    if !success {
        return Err(SimprintError::Other("删除版本类型失败".to_string()));
    }

    Ok(true)
}

/// 激活/停用版本类型
pub async fn toggle_version_type_status_service(
    svc_ctx: &SvcCtx,
    id: i32,
    is_active: bool,
) -> Result<bool, SimprintError> {
    let success =
        crate::models::version_types::toggle_version_type_status(&svc_ctx.db, id, is_active)
            .await?;

    if !success {
        return Err(SimprintError::Other("激活/停用版本类型失败".to_string()));
    }

    Ok(true)
}
