use crate::{
    dto::versions::Version,
    errors::SimprintError,
    svc_ctx::SvcCtx,
    utils::get_objects::get_version_resource_url,
};
use std::collections::HashMap;

/// 查询浏览器内核最新版本列表
/// 筛选 SIMPRINT_KERNEL_* 类型（或指定 type_code），按 resource_name 分组取最新
/// 返回 HashMap<type_code, Vec<Version>>，已填充可下载 URL
pub async fn get_browser_kernel_list_service(
    svc_ctx: &SvcCtx,
    platform: Option<String>,
    type_code: Option<String>,
) -> Result<HashMap<String, Vec<Version>>, SimprintError> {
    let results = crate::models::versions::query_browser_kernel_latest_versions(
        &svc_ctx.db,
        platform.as_deref(),
        type_code.as_deref(),
    )
    .await
    .map_err(|_| SimprintError::Other("查询浏览器内核列表失败".to_string()))?;

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
