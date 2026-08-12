use uuid::Uuid;

use crate::dto::TemplateDto;
use crate::entitys::{
    ApplyTemplateRequest, AssociationsStatus, CreateFromTemplateRequest, CreateTemplateRequest,
    TemplateDetailResponse, UpdateTemplateRequest,
};
use crate::models;
use crate::services;
use crate::svc_ctx::SvcCtx;

/// 创建模板
pub async fn create_template_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    payload: &CreateTemplateRequest,
) -> Result<Uuid, String> {
    // 确定要存储的完整数据
    let environment_data_json: serde_json::Value = if let Some(env_uuid) = payload.environment_uuid
    {
        // 如果提供了环境 UUID，先查询环境获取 workspace_uuid 和 team_uuid
        let env = models::fetch_environment_by_uuid_unfiltered(&svc_ctx.db, env_uuid)
            .await
            .map_err(|e| format!("查询环境失败: {}", e))?
            .ok_or_else(|| "环境不存在".to_string())?;

        // 获取完整的环境详情（带权限检查）
        let env_detail = services::environments::get_environment_detail_service(
            svc_ctx,
            env.workspace_uuid,
            env.team_uuid,
            user_uuid,
            env_uuid,
        )
        .await
        .map_err(|e| format!("获取环境详情失败: {}", e))?;
        serde_json::to_value(&env_detail).map_err(|e| format!("序列化环境详情失败: {}", e))?
    } else if let Some(ref env_data) = payload.environment_data {
        // 如果直接提供了环境详情数据，使用它
        env_data.clone()
    } else {
        return Err("必须提供 environment_uuid 或 environment_data 之一".to_string());
    };

    // 从环境数据中提取摘要信息（从标准结构 config.window_info 中提取）
    let system_info = environment_data_json
        .get("config")
        .and_then(|v| v.get("window_info"))
        .and_then(|v| v.get("system"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let kernel_info = environment_data_json
        .get("config")
        .and_then(|v| v.get("window_info"))
        .and_then(|v| v.get("kernel"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    models::insert_template(
        &svc_ctx.db,
        user_uuid,
        team_uuid,
        &payload.name,
        payload.description.as_deref(),
        payload.is_public.unwrap_or(false),
        system_info.as_deref(),
        kernel_info.as_deref(),
        &environment_data_json,
    )
    .await
    .map_err(|e| e.to_string())
}

/// 获取模板列表
pub async fn get_templates_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    is_public: Option<bool>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<TemplateDto>, i64), String> {
    let offset = (page - 1) * page_size;

    let templates = models::fetch_templates(
        &svc_ctx.db,
        team_uuid,
        user_uuid,
        is_public,
        offset,
        page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    let total = models::fetch_templates_count(&svc_ctx.db, team_uuid, user_uuid, is_public)
        .await
        .map_err(|e| e.to_string())?;

    Ok((templates, total))
}

/// 获取模板详情
pub async fn get_template_service(
    svc_ctx: &SvcCtx,
    template_uuid: Uuid,
    for_create: bool,
) -> Result<TemplateDetailResponse, String> {
    let template = models::fetch_template_by_uuid(&svc_ctx.db, template_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "模板不存在".to_string())?;

    // 如果 for_create 为 true，检查关联数据是否存在
    let associations_status = if for_create {
        Some(check_template_associations(svc_ctx, &template).await?)
    } else {
        None
    };

    Ok(TemplateDetailResponse {
        template,
        associations_status,
    })
}

/// 检查模板关联数据是否存在
async fn check_template_associations(
    svc_ctx: &SvcCtx,
    template: &TemplateDto,
) -> Result<AssociationsStatus, String> {
    // 解析模板中的环境详情数据
    let env_detail: crate::entitys::EnvironmentDetailResponse =
        serde_json::from_value(template.config_json.clone())
            .map_err(|e| format!("解析模板数据失败: {}", e))?;

    // 检查分组是否存在
    let group_exists = if let Some(group_uuid) = env_detail.environment.group_uuid {
        models::fetch_group_by_uuid(&svc_ctx.db, group_uuid)
            .await
            .map_err(|e| e.to_string())?
            .is_some()
    } else {
        false
    };

    // 检查标签是否存在
    let mut tags_exist = std::collections::HashMap::new();
    for tag in &env_detail.tags {
        let exists = models::fetch_tag_by_uuid(&svc_ctx.db, tag.uuid)
            .await
            .map_err(|e| e.to_string())?
            .is_some();
        tags_exist.insert(tag.uuid, exists);
    }

    // 检查账号是否存在
    let mut accounts_exist = std::collections::HashMap::new();
    for account in &env_detail.accounts {
        let exists = models::fetch_platform_account_by_uuid(&svc_ctx.db, account.uuid)
            .await
            .map_err(|e| e.to_string())?
            .is_some();
        accounts_exist.insert(account.uuid, exists);
    }

    // 检查代理是否存在
    let proxy_exists = if let Some(proxy_uuid) = env_detail.environment.proxy_uuid {
        models::fetch_proxy_by_uuid(&svc_ctx.db, proxy_uuid)
            .await
            .map_err(|e| e.to_string())?
            .is_some()
    } else {
        false
    };

    Ok(AssociationsStatus {
        group_exists,
        tags_exist,
        accounts_exist,
        proxy_exists,
    })
}

/// 更新模板
pub async fn update_template_service(
    svc_ctx: &SvcCtx,
    payload: &UpdateTemplateRequest,
) -> Result<(), String> {
    models::update_template(
        &svc_ctx.db,
        payload.uuid,
        payload.name.as_deref(),
        payload.description.as_deref(),
        payload.is_public,
        payload.config_json.as_ref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// 删除模板
pub async fn delete_template_service(svc_ctx: &SvcCtx, template_uuid: Uuid) -> Result<(), String> {
    models::delete_template(&svc_ctx.db, template_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 应用模板到现有环境（更新环境配置）
pub async fn apply_template_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    user_uuid: Uuid,
    payload: &ApplyTemplateRequest,
) -> Result<(), String> {
    // 获取模板配置（不需要检查关联数据）
    let template_response = get_template_service(svc_ctx, payload.template_uuid, false).await?;
    let template = &template_response.template;

    // 解析模板中的环境详情数据
    // 先转换为字符串再反序列化，确保类型正确
    let env_detail: crate::entitys::EnvironmentDetailResponse = serde_json::from_str(
        &serde_json::to_string(&template.config_json)
            .map_err(|e| format!("序列化模板数据失败: {}", e))?,
    )
    .map_err(|e| format!("解析模板数据失败: {}", e))?;

    // 更新环境配置
    let update_req = crate::entitys::UpdateEnvironmentRequest {
        uuid: payload.environment_uuid,
        name: None,
        description: None,
        group_uuid: None,
        cookies: Some(
            env_detail
                .cookies
                .iter()
                .map(|item| crate::entitys::CookieGroupInput {
                    site: item.site.clone(),
                    cookie_text: item.cookie_text.clone(),
                })
                .collect(),
        ),
        urls: Some(
            env_detail
                .urls
                .iter()
                .map(|item| crate::entitys::UrlInput {
                    url: item.url.clone(),
                    title: item.title.clone(),
                    sort_order: item.sort_order,
                })
                .collect(),
        ),
        config: env_detail.config.map(|config| crate::entitys::EnvironmentConfigRequest {
            window_info: config.window_info,
            basic_settings: config.basic_settings,
            fingerprint_settings: config.fingerprint_settings,
            device_settings: config.device_settings,
            preference_settings: config.preference_settings,
            project_metadata: config.project_metadata,
        }),
    };

    services::environments::update_environment_service(
        svc_ctx,
        workspace_uuid,
        team_uuid,
        user_uuid,
        &update_req,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 增加模板使用次数
    let _ = models::increment_template_usage(&svc_ctx.db, payload.template_uuid).await;

    Ok(())
}

/// 从模板创建环境
pub async fn create_from_template_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    workspace_uuid: Uuid,
    team_uuid: Uuid,
    payload: &CreateFromTemplateRequest,
) -> Result<Uuid, String> {
    // 获取模板（不需要检查关联数据，因为这是直接创建，不经过前端表单）
    let template_response = get_template_service(svc_ctx, payload.template_uuid, false).await?;
    let template = &template_response.template;

    // 解析模板中的环境详情数据
    // 先转换为字符串再反序列化，确保类型正确
    let env_detail: crate::entitys::EnvironmentDetailResponse = serde_json::from_str(
        &serde_json::to_string(&template.config_json)
            .map_err(|e| format!("序列化模板数据失败: {}", e))?,
    )
    .map_err(|e| format!("解析模板数据失败: {}", e))?;

    // 使用提供的参数覆盖模板中的值
    let env_name = payload.name.as_deref().unwrap_or(&env_detail.environment.name);
    let env_description =
        payload.description.as_ref().or(env_detail.environment.description.as_ref());
    let group_uuid = payload.group_uuid.or(env_detail.environment.group_uuid);
    let proxy_uuid = env_detail.environment.proxy_uuid;

    // 提取标签 UUIDs
    let tag_uuids: Vec<Uuid> = env_detail.tags.iter().map(|tag| tag.uuid).collect();

    // 提取账号 UUIDs
    let account_uuids: Vec<Uuid> = env_detail.accounts.iter().map(|acc| acc.uuid).collect();

    // 构建创建环境请求
    let config = env_detail.config.ok_or_else(|| "模板中缺少配置信息".to_string())?;

    let create_req = crate::entitys::CreateEnvironmentRequest {
        name: env_name.to_string(),
        description: env_description.cloned(),
        group_uuid,
        tag_uuids: if tag_uuids.is_empty() {
            None
        } else {
            Some(tag_uuids)
        },
        account_uuids: if account_uuids.is_empty() {
            None
        } else {
            Some(account_uuids)
        },
        proxy_uuid,
        cookies: Some(
            env_detail
                .cookies
                .iter()
                .map(|item| crate::entitys::CookieGroupInput {
                    site: item.site.clone(),
                    cookie_text: item.cookie_text.clone(),
                })
                .collect(),
        ),
        urls: Some(
            env_detail
                .urls
                .iter()
                .map(|item| crate::entitys::UrlInput {
                    url: item.url.clone(),
                    title: item.title.clone(),
                    sort_order: item.sort_order,
                })
                .collect(),
        ),
        config: crate::entitys::EnvironmentConfigRequest {
            window_info: config.window_info,
            basic_settings: config.basic_settings,
            fingerprint_settings: config.fingerprint_settings,
            device_settings: config.device_settings,
            preference_settings: config.preference_settings,
            project_metadata: config.project_metadata,
        },
    };

    // 创建环境
    let env_uuid = services::environments::create_environment_service(
        svc_ctx,
        user_uuid,
        workspace_uuid,
        team_uuid,
        &create_req,
    )
    .await
    .map_err(|e| e.to_string())?;

    // 增加模板使用次数
    let _ = models::increment_template_usage(&svc_ctx.db, payload.template_uuid).await;

    Ok(env_uuid)
}
