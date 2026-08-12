use uuid::Uuid;

use crate::dto::PlatformAccountDto;
use crate::entitys::{
    BatchImportAccountsRequest, CreateAccountRequest, ListAccountsRequest, UpdateAccountRequest,
};
use crate::models;
use crate::svc_ctx::SvcCtx;

/// 创建账号
pub async fn create_account_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    payload: &CreateAccountRequest,
) -> Result<Uuid, String> {
    models::insert_platform_account(
        &svc_ctx.db,
        user_uuid,
        team_uuid,
        &payload.platform_url,
        payload.platform_name.as_deref(),
        &payload.account,
        payload.password.as_deref(),
        payload.remark.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// 获取账号列表
pub async fn get_accounts_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    payload: &ListAccountsRequest,
) -> Result<(Vec<PlatformAccountDto>, i64), String> {
    let page = payload.pagination.page.max(1);
    let page_size = payload.pagination.page_size.max(1);
    let offset = (page - 1) * page_size;

    let keyword = payload
        .filters
        .as_ref()
        .and_then(|f| f.keyword.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let platform_name = payload
        .filters
        .as_ref()
        .and_then(|f| f.platform_name.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let status = payload
        .filters
        .as_ref()
        .and_then(|f| f.status.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let accounts = models::fetch_platform_accounts(
        &svc_ctx.db,
        team_uuid,
        user_uuid,
        keyword,
        platform_name,
        status,
        offset,
        page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    let total = models::fetch_platform_accounts_count(
        &svc_ctx.db,
        team_uuid,
        user_uuid,
        keyword,
        platform_name,
        status,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok((accounts, total))
}

/// 获取账号详情
pub async fn get_account_service(
    svc_ctx: &SvcCtx,
    account_uuid: Uuid,
) -> Result<PlatformAccountDto, String> {
    models::fetch_platform_account_by_uuid(&svc_ctx.db, account_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "账号不存在".to_string())
}

/// 更新账号
pub async fn update_account_service(
    svc_ctx: &SvcCtx,
    payload: &UpdateAccountRequest,
) -> Result<(), String> {
    models::update_platform_account(
        &svc_ctx.db,
        payload.uuid,
        payload.platform_url.as_deref(),
        payload.platform_name.as_deref(),
        payload.account.as_deref(),
        payload.password.as_deref(),
        payload.remark.as_deref(),
        payload.status.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// 删除账号
pub async fn delete_account_service(svc_ctx: &SvcCtx, account_uuid: Uuid) -> Result<(), String> {
    models::delete_platform_account(&svc_ctx.db, account_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 批量删除账号
pub async fn batch_delete_accounts_service(
    svc_ctx: &SvcCtx,
    account_uuids: &[Uuid],
) -> Result<u64, String> {
    models::batch_delete_platform_accounts(&svc_ctx.db, account_uuids)
        .await
        .map_err(|e| e.to_string())
}

/// 获取环境关联的账号
pub async fn get_environment_accounts_service(
    svc_ctx: &SvcCtx,
    env_uuid: Uuid,
) -> Result<Vec<PlatformAccountDto>, String> {
    models::fetch_environment_accounts(&svc_ctx.db, env_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 设置环境关联的账号
pub async fn set_environment_accounts_service(
    svc_ctx: &SvcCtx,
    env_uuid: Uuid,
    account_uuids: &[Uuid],
) -> Result<(), String> {
    // 清空现有关联
    models::accounts::clear_environment_accounts(&svc_ctx.db, env_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 添加新关联
    for (idx, account_uuid) in account_uuids.iter().enumerate() {
        models::accounts::insert_environment_account(
            &svc_ctx.db,
            env_uuid,
            *account_uuid,
            idx as i32,
        )
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// 批量导入账号
///
/// 接收客户端已解析好的账号列表，直接保存到数据库
pub async fn batch_import_accounts_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    payload: &BatchImportAccountsRequest,
) -> Result<crate::entitys::BatchImportResponse, String> {
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut errors: Vec<String> = vec![];
    for (index, account) in payload.accounts.iter().enumerate() {
        let result = models::insert_platform_account(
            &svc_ctx.db,
            user_uuid,
            team_uuid,
            &account.platform_url,
            account.platform_name.as_deref(),
            &account.account,
            account.password.as_deref(),
            account.remark.as_deref(),
        )
        .await;

        match result {
            Ok(_) => success_count += 1,
            Err(e) => {
                failed_count += 1;
                errors.push(format!("第 {} 项: {}", index + 1, e));
            }
        }
    }

    Ok(crate::entitys::BatchImportResponse {
        success_count,
        failed_count,
        errors,
    })
}
