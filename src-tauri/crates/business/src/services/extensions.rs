use uuid::Uuid;

use crate::dto::ExtensionDto;
use crate::entitys::{
    ExtensionGroup, InstallExtensionRequest, InstalledExtensionItem, ListExtensionsRequest,
    UninstallExtensionRequest,
};
use crate::models;
use crate::models::environments as env_models;
use crate::svc_ctx::SvcCtx;
use crate::utils::storage::get_objects;

/// 获取扩展列表
pub async fn get_extensions_service(
    svc_ctx: &SvcCtx,
    payload: &ListExtensionsRequest,
) -> Result<(Vec<ExtensionDto>, i64), String> {
    let offset = (payload.pagination.page - 1) * payload.pagination.page_size;

    let keyword = payload.filters.as_ref().and_then(|f| f.keyword.as_deref());
    let category = payload.filters.as_ref().and_then(|f| f.category.as_deref());
    let sort_by = payload.filters.as_ref().and_then(|f| f.sort_by.as_deref());
    let sort_order = payload.filters.as_ref().and_then(|f| f.sort_order.as_deref());

    let mut extensions = models::extensions::fetch_extensions(
        &svc_ctx.db,
        keyword,
        category,
        sort_by,
        sort_order,
        offset,
        payload.pagination.page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 将 object path 转换为完整 URL
    let public_base_url = svc_ctx.config.storage.public_base_url.as_str();
    let extension_root = svc_ctx.config.storage.extension_root.as_str();
    ExtensionDto::transform_urls_batch(&mut extensions, public_base_url, extension_root);

    let total = models::extensions::fetch_extensions_count(&svc_ctx.db, keyword, category)
        .await
        .map_err(|e| e.to_string())?;

    Ok((extensions, total))
}

/// 获取扩展详情
pub async fn get_extension_service(
    svc_ctx: &SvcCtx,
    extension_id: &str,
) -> Result<ExtensionDto, String> {
    let mut extension = models::extensions::fetch_extension_by_id(&svc_ctx.db, extension_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "扩展不存在".to_string())?;

    // 将 object path 转换为完整 URL
    let public_base_url = svc_ctx.config.storage.public_base_url.as_str();
    let extension_root = svc_ctx.config.storage.extension_root.as_str();
    extension.transform_urls(&public_base_url, &extension_root);

    Ok(extension)
}

/// 获取扩展分类
pub async fn get_extension_categories_service(svc_ctx: &SvcCtx) -> Result<Vec<String>, String> {
    models::extensions::fetch_extension_categories(&svc_ctx.db)
        .await
        .map_err(|e| e.to_string())
}

/// 获取用户已安装的扩展
pub async fn get_user_installed_extensions_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
) -> Result<Vec<InstalledExtensionItem>, String> {
    let public_base_url = svc_ctx.config.storage.public_base_url.as_str();
    let extension_root = svc_ctx.config.storage.extension_root.as_str();

    // 查询用户直接安装的扩展
    let user_installed = models::extensions::fetch_user_extensions(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 查询用户相关的分组中安装的扩展
    let group_installed =
        models::extensions::fetch_user_group_extensions(&svc_ctx.db, user_uuid, team_uuid)
            .await
            .map_err(|e| e.to_string())?;

    // 使用 HashMap 去重，以 extension_id 为 key
    use std::collections::HashMap;
    let mut extension_map: HashMap<String, InstalledExtensionItem> = HashMap::new();

    // 处理用户直接安装的扩展
    for ue in user_installed {
        if let Ok(Some(ext)) =
            models::extensions::fetch_extension_by_id(&svc_ctx.db, &ue.extension_id).await
        {
            // 转换图标 URL
            let mut icon_url = ext.icon_url.clone();
            if let Some(path) = &icon_url {
                if !path.is_empty() && !path.starts_with("http") {
                    icon_url = Some(get_objects::get_extension_icon_url(
                        &public_base_url,
                        &extension_root,
                        path,
                    ));
                }
            }

            // 查询关联的分组
            let group_uuids = models::extensions::fetch_group_uuids_by_extension_id(
                &svc_ctx.db,
                &ue.extension_id,
            )
            .await
            .unwrap_or_default();

            let mut groups = Vec::new();
            for group_uuid in group_uuids {
                if let Ok(Some(group)) =
                    env_models::fetch_group_by_uuid(&svc_ctx.db, group_uuid).await
                {
                    groups.push(ExtensionGroup {
                        uuid: group.uuid,
                        name: group.name,
                    });
                }
            }

            let has_update = ext.version != ue.installed_version;
            extension_map.insert(
                ue.extension_id.clone(),
                InstalledExtensionItem {
                    extension_id: ue.extension_id,
                    name: ext.name,
                    version: ext.version,
                    installed_version: ue.installed_version,
                    has_update,
                    status: ue.status,
                    installed_at: ue.installed_at,
                    homepage: ext.homepage,
                    icon_url,
                    team_uuid: None,
                    scope: "user".to_string(),
                    description: ext.description,
                    category: Some(ext.category),
                    browser: Some(ext.browser),
                    developer: ext.developer,
                    downloads_count: ext.downloads_count,
                    rating: ext.rating,
                    permissions: ext.permissions,
                    file_size: ext.file_size,
                    updated_at: Some(ext.updated_at),
                    groups: if groups.is_empty() {
                        None
                    } else {
                        Some(groups)
                    },
                },
            );
        }
    }

    // 处理分组安装的扩展
    for ge in group_installed {
        // 如果已存在（用户直接安装），分组信息已经包含在内，直接跳过
        if extension_map.contains_key(&ge.extension_id) {
            continue;
        }

        // 只处理新扩展（仅安装在分组中，用户未直接安装）
        if let Ok(Some(ext)) =
            models::extensions::fetch_extension_by_id(&svc_ctx.db, &ge.extension_id).await
        {
            // 转换图标 URL
            let mut icon_url = ext.icon_url.clone();
            if let Some(path) = &icon_url {
                if !path.is_empty() && !path.starts_with("http") {
                    icon_url = Some(get_objects::get_extension_icon_url(
                        &public_base_url,
                        &extension_root,
                        path,
                    ));
                }
            }

            // 查询关联的分组
            let group_uuids = models::extensions::fetch_group_uuids_by_extension_id(
                &svc_ctx.db,
                &ge.extension_id,
            )
            .await
            .unwrap_or_default();

            let mut groups = Vec::new();
            let mut is_team_group = false;
            for group_uuid in group_uuids {
                if let Ok(Some(group)) =
                    env_models::fetch_group_by_uuid(&svc_ctx.db, group_uuid).await
                {
                    if !group.team_uuid.is_nil() {
                        is_team_group = true;
                    }
                    groups.push(ExtensionGroup {
                        uuid: group.uuid,
                        name: group.name,
                    });
                }
            }

            let has_update = ext.version != ge.installed_version;
            let scope = if is_team_group {
                "group-team".to_string()
            } else {
                "group-personal".to_string()
            };
            extension_map.insert(
                ge.extension_id.clone(),
                InstalledExtensionItem {
                    extension_id: ge.extension_id,
                    name: ext.name,
                    version: ext.version,
                    installed_version: ge.installed_version,
                    has_update,
                    status: ge.status,
                    installed_at: ge.installed_at,
                    homepage: ext.homepage,
                    icon_url,
                    team_uuid: None,
                    scope,
                    description: ext.description,
                    category: Some(ext.category),
                    browser: Some(ext.browser),
                    developer: ext.developer,
                    downloads_count: ext.downloads_count,
                    rating: ext.rating,
                    permissions: ext.permissions,
                    file_size: ext.file_size,
                    updated_at: Some(ext.updated_at),
                    groups: if groups.is_empty() {
                        None
                    } else {
                        Some(groups)
                    },
                },
            );
        }
    }

    Ok(extension_map.into_values().collect())
}

/// 获取团队已安装的扩展
pub async fn get_team_installed_extensions_service(
    svc_ctx: &SvcCtx,
    team_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<Vec<InstalledExtensionItem>, String> {
    let public_base_url = svc_ctx.config.storage.public_base_url.as_str();
    let extension_root = svc_ctx.config.storage.extension_root.as_str();

    // 查询团队直接安装的扩展
    let team_installed = models::extensions::fetch_team_extensions(&svc_ctx.db, team_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 查询团队相关的分组中安装的扩展
    let group_installed = models::extensions::fetch_team_group_extensions(&svc_ctx.db, team_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 查询用户禁用的团队插件列表
    let user_disabled_extensions =
        models::extensions::fetch_user_disabled_team_extensions(&svc_ctx.db, user_uuid, team_uuid)
            .await
            .map_err(|e| e.to_string())?;
    let user_disabled_set: std::collections::HashSet<String> =
        user_disabled_extensions.into_iter().collect();

    // 使用 HashMap 去重，以 extension_id 为 key
    use std::collections::HashMap;
    let mut extension_map: HashMap<String, InstalledExtensionItem> = HashMap::new();

    // 处理团队直接安装的扩展
    for te in team_installed {
        if let Ok(Some(ext)) =
            models::extensions::fetch_extension_by_id(&svc_ctx.db, &te.extension_id).await
        {
            // 转换图标 URL
            let mut icon_url = ext.icon_url.clone();
            if let Some(path) = &icon_url {
                if !path.is_empty() && !path.starts_with("http") {
                    icon_url = Some(get_objects::get_extension_icon_url(
                        &public_base_url,
                        &extension_root,
                        path,
                    ));
                }
            }

            // 查询关联的分组（仅查询团队共享的分组）
            let group_uuids = models::extensions::fetch_team_shared_group_uuids_by_extension_id(
                &svc_ctx.db,
                &te.extension_id,
            )
            .await
            .unwrap_or_default();

            let mut groups = Vec::new();
            for group_uuid in group_uuids {
                if let Ok(Some(group)) =
                    env_models::fetch_group_by_uuid(&svc_ctx.db, group_uuid).await
                {
                    groups.push(ExtensionGroup {
                        uuid: group.uuid,
                        name: group.name,
                    });
                }
            }

            let has_update = ext.version != te.installed_version;

            // 检查用户是否禁用了这个团队插件
            let status = if user_disabled_set.contains(&te.extension_id) {
                "disabled".to_string()
            } else {
                te.status
            };

            extension_map.insert(
                te.extension_id.clone(),
                InstalledExtensionItem {
                    extension_id: te.extension_id,
                    name: ext.name,
                    version: ext.version,
                    installed_version: te.installed_version,
                    has_update,
                    status,
                    installed_at: te.installed_at,
                    homepage: ext.homepage,
                    icon_url,
                    team_uuid: Some(team_uuid),
                    scope: "team".to_string(),
                    description: ext.description,
                    category: Some(ext.category),
                    browser: Some(ext.browser),
                    developer: ext.developer,
                    downloads_count: ext.downloads_count,
                    rating: ext.rating,
                    permissions: ext.permissions,
                    file_size: ext.file_size,
                    updated_at: Some(ext.updated_at),
                    groups: if groups.is_empty() {
                        None
                    } else {
                        Some(groups)
                    },
                },
            );
        }
    }

    // 处理分组安装的扩展
    for ge in group_installed {
        // 如果已存在（团队直接安装），分组信息已经包含在内，直接跳过
        if extension_map.contains_key(&ge.extension_id) {
            continue;
        }

        // 只处理新扩展（仅安装在分组中，团队未直接安装）
        if let Ok(Some(ext)) =
            models::extensions::fetch_extension_by_id(&svc_ctx.db, &ge.extension_id).await
        {
            // 转换图标 URL
            let mut icon_url = ext.icon_url.clone();
            if let Some(path) = &icon_url {
                if !path.is_empty() && !path.starts_with("http") {
                    icon_url = Some(get_objects::get_extension_icon_url(
                        &public_base_url,
                        &extension_root,
                        path,
                    ));
                }
            }

            // 查询关联的分组
            let group_uuids = models::extensions::fetch_group_uuids_by_extension_id(
                &svc_ctx.db,
                &ge.extension_id,
            )
            .await
            .unwrap_or_default();

            let mut groups = Vec::new();
            for group_uuid in group_uuids {
                if let Ok(Some(group)) =
                    env_models::fetch_group_by_uuid(&svc_ctx.db, group_uuid).await
                {
                    groups.push(ExtensionGroup {
                        uuid: group.uuid,
                        name: group.name,
                    });
                }
            }

            let has_update = ext.version != ge.installed_version;
            let scope = if ge.is_team_shared {
                "group-team".to_string()
            } else {
                "group-personal".to_string()
            };

            // 检查用户是否禁用了这个团队插件
            let status = if user_disabled_set.contains(&ge.extension_id) {
                "disabled".to_string()
            } else {
                ge.status
            };

            extension_map.insert(
                ge.extension_id.clone(),
                InstalledExtensionItem {
                    extension_id: ge.extension_id,
                    name: ext.name,
                    version: ext.version,
                    installed_version: ge.installed_version,
                    has_update,
                    status,
                    installed_at: ge.installed_at,
                    homepage: ext.homepage,
                    icon_url,
                    team_uuid: Some(team_uuid),
                    scope,
                    description: ext.description,
                    category: Some(ext.category),
                    browser: Some(ext.browser),
                    developer: ext.developer,
                    downloads_count: ext.downloads_count,
                    rating: ext.rating,
                    permissions: ext.permissions,
                    file_size: ext.file_size,
                    updated_at: Some(ext.updated_at),
                    groups: if groups.is_empty() {
                        None
                    } else {
                        Some(groups)
                    },
                },
            );
        }
    }

    Ok(extension_map.into_values().collect())
}

/// 安装扩展
pub async fn install_extension_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    payload: &InstallExtensionRequest,
) -> Result<(), String> {
    // 获取扩展信息
    let extension = models::extensions::fetch_extension_by_id(&svc_ctx.db, &payload.extension_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "扩展不存在".to_string())?;

    let target_type = payload.target_type.as_deref().unwrap_or("user");

    match target_type {
        "user" => {
            models::extensions::insert_user_extension(
                &svc_ctx.db,
                user_uuid,
                &payload.extension_id,
                &extension.version,
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        "team" => {
            // 权限检查：检查用户是否为 owner/admin
            let team_uuid = team_uuid.ok_or_else(|| "未指定团队".to_string())?;

            // 获取工作空间 UUID
            let team = crate::models::teams::fetch_team_by_uuid(&svc_ctx.db, team_uuid)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "团队不存在".to_string())?;
            let workspace_uuid = team.workspace_uuid;

            // 查询用户在团队中的角色
            let member = crate::models::teams::fetch_team_member(
                &svc_ctx.db,
                workspace_uuid,
                team_uuid,
                user_uuid,
            )
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "您不是该团队成员".to_string())?;

            // 检查角色权限
            if member.role != "owner" && member.role != "admin" {
                return Err("权限不足：只有团队所有者或管理员可以安装团队插件".to_string());
            }

            models::extensions::insert_team_extension(
                &svc_ctx.db,
                team_uuid,
                &payload.extension_id,
                &extension.version,
                user_uuid,
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        "group" => {
            // 权限检查
            let is_team_shared = payload.is_team_shared.unwrap_or(false);

            // 必须提供分组数组，即使只有一个分组也需要传入数组
            let group_ids = payload.group_ids.as_ref().ok_or_else(|| "未指定分组".to_string())?;
            if group_ids.is_empty() {
                return Err("分组列表不能为空".to_string());
            }

            for group_uuid in group_ids {
                // 检查权限
                // - is_team_shared=true: 需要"管理"权限
                // - is_team_shared=false: 需要"编辑"权限
                let required_permission = if is_team_shared { "manage" } else { "write" };

                // 获取工作空间 UUID（从团队获取）
                let workspace_uuid = if let Some(team_uuid) = team_uuid {
                    let team = crate::models::teams::fetch_team_by_uuid(&svc_ctx.db, team_uuid)
                        .await
                        .map_err(|e| e.to_string())?
                        .ok_or_else(|| "团队不存在".to_string())?;
                    team.workspace_uuid
                } else {
                    return Err("未指定团队".to_string());
                };

                let has_permission = crate::models::group_member_permissions::check_group_permission(
                    &svc_ctx.db,
                    workspace_uuid,
                    *group_uuid,
                    user_uuid,
                    required_permission,
                )
                .await
                .map_err(|e| e.to_string())?;

                if !has_permission {
                    let msg = if is_team_shared {
                        "权限不足：安装团队共享插件需要分组的管理权限"
                    } else {
                        "权限不足：安装个人插件需要分组的编辑权限"
                    };
                    return Err(msg.to_string());
                }

                models::extensions::insert_group_extension(
                    &svc_ctx.db,
                    *group_uuid,
                    &payload.extension_id,
                    &extension.version,
                    user_uuid,
                    is_team_shared,
                )
                .await
                .map_err(|e| e.to_string())?;
            }
        }
        _ => return Err("无效的安装目标类型".to_string()),
    }

    // 添加审计日志
    let details = format!("安装扩展: {}", payload.extension_id);
    let changes = serde_json::json!({
        "extension_id": payload.extension_id,
        "version": extension.version,
        "target_type": target_type,
        "is_team_shared": payload.is_team_shared,
    });

    let _ = crate::models::audit::insert_audit_log(
        &svc_ctx.db,
        user_uuid,
        team_uuid,
        "install_extension",
        target_type,
        None,
        Some(&payload.extension_id),
        Some(&details),
        Some(&changes),
        None,
        None,
        None,
    )
    .await;

    Ok(())
}

/// 卸载扩展
pub async fn uninstall_extension_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    payload: &UninstallExtensionRequest,
) -> Result<(), String> {
    let target_type = payload.target_type.as_deref().unwrap_or("user");

    match target_type {
        "user" => {
            models::extensions::delete_user_extension(
                &svc_ctx.db,
                user_uuid,
                &payload.extension_id,
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        "team" => {
            // 权限检查：检查用户是否为 owner/admin
            let team_uuid = team_uuid.ok_or_else(|| "未指定团队".to_string())?;

            // 获取工作空间 UUID
            let team = crate::models::teams::fetch_team_by_uuid(&svc_ctx.db, team_uuid)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "团队不存在".to_string())?;
            let workspace_uuid = team.workspace_uuid;

            // 查询用户在团队中的角色
            let member = crate::models::teams::fetch_team_member(
                &svc_ctx.db,
                workspace_uuid,
                team_uuid,
                user_uuid,
            )
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "您不是该团队成员".to_string())?;

            // 检查角色权限
            if member.role != "owner" && member.role != "admin" {
                return Err("权限不足：只有团队所有者或管理员可以卸载团队插件".to_string());
            }

            models::extensions::delete_team_extension(
                &svc_ctx.db,
                team_uuid,
                &payload.extension_id,
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        "group" => {
            // 权限检查
            let group_uuid = payload.target_uuid.ok_or_else(|| "未指定分组".to_string())?;

            // 查询该分组扩展的 is_team_shared 状态
            let group_extensions = models::extensions::fetch_group_extensions(&svc_ctx.db, group_uuid)
                .await
                .map_err(|e| e.to_string())?;

            let group_ext = group_extensions
                .iter()
                .find(|ge| ge.extension_id == payload.extension_id)
                .ok_or_else(|| "该分组未安装此扩展".to_string())?;

            // 根据 is_team_shared 检查权限
            let required_permission = if group_ext.is_team_shared { "manage" } else { "write" };

            // 获取工作空间 UUID
            let workspace_uuid = if let Some(team_uuid) = team_uuid {
                let team = crate::models::teams::fetch_team_by_uuid(&svc_ctx.db, team_uuid)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "团队不存在".to_string())?;
                team.workspace_uuid
            } else {
                return Err("未指定团队".to_string());
            };

            let has_permission = crate::models::group_member_permissions::check_group_permission(
                &svc_ctx.db,
                workspace_uuid,
                group_uuid,
                user_uuid,
                required_permission,
            )
            .await
            .map_err(|e| e.to_string())?;

            if !has_permission {
                let msg = if group_ext.is_team_shared {
                    "权限不足：卸载团队共享插件需要分组的管理权限"
                } else {
                    "权限不足：卸载个人插件需要分组的编辑权限"
                };
                return Err(msg.to_string());
            }

            models::extensions::delete_group_extension(
                &svc_ctx.db,
                group_uuid,
                &payload.extension_id,
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        _ => return Err("无效的卸载目标类型".to_string()),
    }

    // 添加审计日志
    let details = format!("卸载扩展: {}", payload.extension_id);
    let changes = serde_json::json!({
        "extension_id": payload.extension_id,
        "target_type": target_type,
    });

    let _ = crate::models::audit::insert_audit_log(
        &svc_ctx.db,
        user_uuid,
        team_uuid,
        "uninstall_extension",
        target_type,
        payload.target_uuid,
        Some(&payload.extension_id),
        Some(&details),
        Some(&changes),
        None,
        None,
        None,
    )
    .await;

    Ok(())
}

/// 更新扩展
pub async fn update_extension_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    extension_id: &str,
) -> Result<(), String> {
    // 获取最新扩展信息
    let extension = models::extensions::fetch_extension_by_id(&svc_ctx.db, extension_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "扩展不存在".to_string())?;

    // 更新用户扩展版本
    models::extensions::insert_user_extension(
        &svc_ctx.db,
        user_uuid,
        extension_id,
        &extension.version,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 批量更新扩展
pub async fn batch_update_extensions_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    extension_ids: &[String],
) -> Result<u64, String> {
    let mut updated = 0u64;

    for extension_id in extension_ids {
        if update_extension_service(svc_ctx, user_uuid, extension_id).await.is_ok() {
            updated += 1;
        }
    }

    Ok(updated)
}

/// 获取环境的扩展列表（动态合并 4 个层级）
///
/// 合并逻辑：
/// 1. 用户个人全局插件
/// 2. 团队全局插件
/// 3. 该环境所属分组的个人插件
/// 4. 该环境所属分组的团队插件
pub async fn get_environment_extensions_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Uuid,
    group_uuid: Option<Uuid>,
) -> Result<Vec<crate::dto::environments::ExtensionSummaryDto>, String> {
    use std::collections::HashMap;

    let public_base_url = svc_ctx.config.storage.public_base_url.as_str();
    let extension_root = svc_ctx.config.storage.extension_root.as_str();

    // 使用 HashMap 去重，key 为 extension_id
    let mut extension_map: HashMap<String, crate::dto::environments::ExtensionSummaryDto> = HashMap::new();

    // 0. 查询用户禁用的插件列表
    let disabled_extensions: std::collections::HashSet<String> = sqlx::query_scalar::<_, String>(
        r#"
        SELECT extension_id FROM user_extensions
        WHERE user_uuid = $1 AND status = 'disabled'
        "#,
    )
    .bind(user_uuid)
    .fetch_all(&svc_ctx.db)
    .await
    .map_err(|e| e.to_string())?
    .into_iter()
    .collect();

    // 1. 查询用户全局插件（排除 disabled 状态）
    let user_extensions = models::extensions::fetch_user_extensions(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    for ue in user_extensions {
        if let Ok(Some(ext)) = models::extensions::fetch_extension_by_id(&svc_ctx.db, &ue.extension_id).await {
            let mut icon_url = ext.icon_url.clone();
            if let Some(path) = &icon_url {
                if !path.is_empty() && !path.starts_with("http") {
                    icon_url = Some(crate::utils::storage::get_objects::get_extension_icon_url(
                        public_base_url,
                        extension_root,
                        path,
                    ));
                }
            }

            let mut download_url = ext.download_url.clone();
            if let Some(path) = &download_url {
                if !path.is_empty() && !path.starts_with("http") {
                    download_url = Some(crate::utils::storage::get_objects::get_extension_crx_url(
                        public_base_url,
                        extension_root,
                        path,
                    ));
                }
            }

            extension_map.insert(
                ue.extension_id.clone(),
                crate::dto::environments::ExtensionSummaryDto {
                    extension_id: ue.extension_id,
                    name: ext.name,
                    version: ext.version,
                    icon_url,
                    download_url,
                    hash: ext.hash,
                    scope: "user".to_string(),
                },
            );
        }
    }

    // 2. 查询团队全局插件
    let team_extensions = models::extensions::fetch_team_extensions(&svc_ctx.db, team_uuid)
        .await
        .map_err(|e| e.to_string())?;

    for te in team_extensions {
        // 跳过用户禁用的插件
        if disabled_extensions.contains(&te.extension_id) {
            continue;
        }

        if let Ok(Some(ext)) = models::extensions::fetch_extension_by_id(&svc_ctx.db, &te.extension_id).await {
            let mut icon_url = ext.icon_url.clone();
            if let Some(path) = &icon_url {
                if !path.is_empty() && !path.starts_with("http") {
                    icon_url = Some(crate::utils::storage::get_objects::get_extension_icon_url(
                        public_base_url,
                        extension_root,
                        path,
                    ));
                }
            }

            let mut download_url = ext.download_url.clone();
            if let Some(path) = &download_url {
                if !path.is_empty() && !path.starts_with("http") {
                    download_url = Some(crate::utils::storage::get_objects::get_extension_crx_url(
                        public_base_url,
                        extension_root,
                        path,
                    ));
                }
            }

            extension_map.insert(
                te.extension_id.clone(),
                crate::dto::environments::ExtensionSummaryDto {
                    extension_id: te.extension_id,
                    name: ext.name,
                    version: ext.version,
                    icon_url,
                    download_url,
                    hash: ext.hash,
                    scope: "team".to_string(),
                },
            );
        }
    }

    // 3. 如果环境有分组，查询分组插件
    if let Some(group_uuid) = group_uuid {
        let group_extensions = models::extensions::fetch_group_extensions(&svc_ctx.db, group_uuid)
            .await
            .map_err(|e| e.to_string())?;

        for ge in group_extensions {
            // 跳过用户禁用的插件
            if disabled_extensions.contains(&ge.extension_id) {
                continue;
            }

            if let Ok(Some(ext)) = models::extensions::fetch_extension_by_id(&svc_ctx.db, &ge.extension_id).await {
                let mut icon_url = ext.icon_url.clone();
                if let Some(path) = &icon_url {
                    if !path.is_empty() && !path.starts_with("http") {
                        icon_url = Some(crate::utils::storage::get_objects::get_extension_icon_url(
                        public_base_url,
                        extension_root,
                            path,
                        ));
                    }
                }

                let mut download_url = ext.download_url.clone();
                if let Some(path) = &download_url {
                    if !path.is_empty() && !path.starts_with("http") {
                        download_url = Some(crate::utils::storage::get_objects::get_extension_crx_url(
                        public_base_url,
                        extension_root,
                            path,
                        ));
                    }
                }

                let scope = if ge.is_team_shared {
                    "group-team".to_string()
                } else {
                    "group-personal".to_string()
                };

                extension_map.insert(
                    ge.extension_id.clone(),
                    crate::dto::environments::ExtensionSummaryDto {
                        extension_id: ge.extension_id,
                        name: ext.name,
                        version: ext.version,
                        icon_url,
                        download_url,
                        hash: ext.hash,
                        scope,
                    },
                );
            }
        }
    }

    Ok(extension_map.into_values().collect())
}

/// 禁用扩展（用户级别）
///
/// 用户可以禁用团队插件，通过在 user_team_extension_preferences 中设置 is_disabled = true
pub async fn disable_extension_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Uuid,
    extension_id: &str,
) -> Result<(), String> {
    // 检查扩展是否存在
    let _extension = models::extensions::fetch_extension_by_id(&svc_ctx.db, extension_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "扩展不存在".to_string())?;

    // 设置用户对团队插件的禁用状态
    models::extensions::set_user_team_extension_preference(
        &svc_ctx.db,
        user_uuid,
        team_uuid,
        extension_id,
        true,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 启用扩展（用户级别）
///
/// 删除用户对团队插件的禁用设置
pub async fn enable_extension_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Uuid,
    extension_id: &str,
) -> Result<(), String> {
    // 删除用户对团队插件的偏好设置（或设置为 false）
    models::extensions::delete_user_team_extension_preference(
        &svc_ctx.db,
        user_uuid,
        team_uuid,
        extension_id,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}
