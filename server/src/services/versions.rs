use crate::{
    dto::versions::Version, entitys::versions::*, errors::SimprintError, svc_ctx::SvcCtx,
    utils::get_objects::get_version_resource_url,
};
use std::collections::HashMap;

/// 创建版本
pub async fn create_version(
    svc_ctx: &SvcCtx,
    request: CreateVersionRequest,
) -> Result<i32, SimprintError> {
    // 验证版本号
    if request.version.is_empty() {
        return Err(SimprintError::VersionEmpty);
    }

    // 验证资源名称
    if request.resource_name.is_empty() {
        return Err(SimprintError::ResourceNameEmpty);
    }

    // 检查版本是否已存在
    let exists = crate::models::versions::query_version_by_name_and_version(
        &svc_ctx.db,
        &request.resource_name,
        &request.version,
    )
    .await
    .ok();

    if exists.is_some() {
        return Err(SimprintError::VersionAlreadyExists);
    }

    // 创建版本
    let id = crate::models::versions::insert_version(&svc_ctx.db, &request).await?;

    Ok(id)
}

/// 根据ID查询版本
pub async fn get_version_by_id(svc_ctx: &SvcCtx, id: i32) -> Result<Version, SimprintError> {
    let mut version = crate::models::versions::query_version_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::VersionNotFound)?;

    let storage_config = &svc_ctx.config.storage;
    let object_path = version.url.clone().unwrap_or_default();
    version.url = Some(get_version_resource_url(
        &storage_config.public_base_url,
        &storage_config.version_root,
        &object_path,
    ));

    Ok(version)
}

/// 根据资源名称和版本号查询版本
pub async fn get_version_by_name_and_version(
    svc_ctx: &SvcCtx,
    resource_name: String,
    version: String,
) -> Result<Version, SimprintError> {
    let version_data = crate::models::versions::query_version_by_name_and_version(
        &svc_ctx.db,
        &resource_name,
        &version,
    )
    .await
    .map_err(|_| SimprintError::VersionNotFound)?;

    Ok(version_data)
}

/// 查询最新版本
pub async fn get_latest_version(
    svc_ctx: &SvcCtx,
    resource_name: String,
    platform: String,
) -> Result<Version, SimprintError> {
    let version =
        crate::models::versions::query_latest_version(&svc_ctx.db, &resource_name, &platform)
            .await
            .map_err(|_| SimprintError::VersionNotFound)?;

    version.ok_or(SimprintError::VersionNotFound)
}

/// 查询版本列表
pub async fn query_versions_service(
    svc_ctx: &SvcCtx,
    params: QueryVersionParams,
    page_num: i32,
    page_size: i32,
) -> Result<VersionListResponse, SimprintError> {
    let (total, list) = crate::models::versions::query_versions(
        &svc_ctx.db,
        params.resource_name.as_deref(),
        params.platform.as_deref(),
        params.status.as_deref(),
        page_num,
        page_size,
    )
    .await?;

    Ok(VersionListResponse { total, list })
}

/// 更新版本
pub async fn update_version_service(
    svc_ctx: &SvcCtx,
    id: i32,
    request: UpdateVersionRequest,
) -> Result<bool, SimprintError> {
    // 检查版本是否存在
    crate::models::versions::query_version_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::VersionNotFound)?;

    // 更新版本
    let success = crate::models::versions::update_version(&svc_ctx.db, id, &request).await?;

    if !success {
        return Err(SimprintError::VersionNotFound);
    }

    Ok(true)
}

/// 删除版本
pub async fn delete_version_service(svc_ctx: &SvcCtx, id: i32) -> Result<bool, SimprintError> {
    let success = crate::models::versions::delete_version(&svc_ctx.db, id).await?;

    if !success {
        return Err(SimprintError::VersionNotFound);
    }

    Ok(true)
}

/// 设置某版本为最新版本
pub async fn set_latest_version_service(
    svc_ctx: &SvcCtx,
    type_id: i32,
    resource_name: String,
    version_id: i32,
) -> Result<bool, SimprintError> {
    // 检查版本是否存在
    crate::models::versions::query_version_by_id(&svc_ctx.db, version_id)
        .await
        .map_err(|_| SimprintError::VersionNotFound)?;

    // 设置为最新版本
    let success = crate::models::versions::set_as_latest_version(
        &svc_ctx.db,
        type_id,
        &resource_name,
        version_id,
    )
    .await?;

    if !success {
        return Err(SimprintError::Other("设置最新版本失败".to_string()));
    }

    Ok(true)
}

/// 版本回退到指定版本
pub async fn rollback_version_service(
    svc_ctx: &SvcCtx,
    type_id: i32,
    resource_name: String,
    target_version_id: i32,
) -> Result<bool, SimprintError> {
    // 检查目标版本是否存在
    let target_version =
        crate::models::versions::query_version_by_id(&svc_ctx.db, target_version_id)
            .await
            .map_err(|_| SimprintError::VersionNotFound)?;

    // 检查版本是否属于同一资源
    if target_version.type_id != type_id || target_version.resource_name != resource_name {
        return Err(SimprintError::VersionNotFound);
    }

    // 设置目标版本为最新
    let success =
        set_latest_version_service(svc_ctx, type_id, resource_name, target_version_id).await?;

    if !success {
        return Err(SimprintError::Other("版本回退失败".to_string()));
    }

    Ok(true)
}

/// 版本差异对比
pub async fn compare_versions_service(
    svc_ctx: &SvcCtx,
    version_id_1: i32,
    version_id_2: i32,
) -> Result<(Version, Version), SimprintError> {
    let version_1 = crate::models::versions::query_version_by_id(&svc_ctx.db, version_id_1)
        .await
        .map_err(|_| SimprintError::VersionNotFound)?;

    let version_2 = crate::models::versions::query_version_by_id(&svc_ctx.db, version_id_2)
        .await
        .map_err(|_| SimprintError::VersionNotFound)?;

    Ok((version_1, version_2))
}

/// 查询所有激活版本类型的最新版本
pub async fn get_all_active_latest_versions_service(
    svc_ctx: &SvcCtx,
    platform: String,
) -> Result<HashMap<String, Vec<Version>>, SimprintError> {
    let results =
        crate::models::versions::query_all_active_latest_versions(&svc_ctx.db, &platform).await?;

    // 转换为 HashMap<type_code, Vec<Version>>
    let mut map: HashMap<String, Vec<Version>> = HashMap::new();

    let storage_config = &svc_ctx.config.storage;

    for (type_code, _resource_name, version) in results {
        map.entry(type_code).or_insert_with(Vec::new).push(Version {
            url: Some(get_version_resource_url(
                &storage_config.public_base_url,
                &storage_config.version_root,
                &version.url.unwrap_or_default(),
            )),
            ..version
        });
    }

    Ok(map)
}
