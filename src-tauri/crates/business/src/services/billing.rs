use rust_decimal::Decimal;
use uuid::Uuid;

use crate::dto::{AutoRenewalServiceDto, InvoiceDto, UserQuotaDto};
use crate::entitys::{AccountInfoResponse, ListInvoicesRequest};
use crate::models;
use crate::svc_ctx::SvcCtx;

// ============ Invoices ============

/// 获取发票列表
pub async fn get_invoices_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &ListInvoicesRequest,
) -> Result<(Vec<InvoiceDto>, i64), String> {
    let offset = (payload.pagination.page - 1) * payload.pagination.page_size;

    let invoices = models::billing::fetch_invoices(
        &svc_ctx.db,
        user_uuid,
        payload.status.as_deref(),
        offset,
        payload.pagination.page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    let total =
        models::billing::fetch_invoices_count(&svc_ctx.db, user_uuid, payload.status.as_deref())
            .await
            .map_err(|e| e.to_string())?;

    Ok((invoices, total))
}

// ============ Quotas ============

/// 获取用户配额
pub async fn get_quota_service(svc_ctx: &SvcCtx, user_uuid: Uuid) -> Result<UserQuotaDto, String> {
    models::billing::fetch_user_quota(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "用户配额不存在".to_string())
}

// ============ Auto Renewal Services ============

/// 获取自动续费服务列表
pub async fn get_auto_renewal_services_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<Vec<AutoRenewalServiceDto>, String> {
    models::billing::fetch_auto_renewal_services(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 获取账户信息（聚合钱包、配额、订阅、用户信息）
pub async fn get_account_info_service(
    svc_ctx: &SvcCtx,
    workspace_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<AccountInfoResponse, String> {
    // 1. 获取用户信息（邮箱等）
    let user_info = models::user::fetch_user_info_by_uuid(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "用户不存在".to_string())?;

    // 2. 获取钱包信息
    let wallet = models::billing::fetch_user_wallet(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 如果钱包不存在，创建一个
    let wallet = if wallet.is_none() {
        models::billing::insert_user_wallet(&svc_ctx.db, user_uuid, "CNY")
            .await
            .map_err(|e| e.to_string())?;

        models::billing::fetch_user_wallet(&svc_ctx.db, user_uuid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "创建钱包失败".to_string())?
    } else {
        wallet.ok_or_else(|| "钱包不存在".to_string())?
    };

    // 3. 获取工作空间配额
    let quota = models::fetch_workspace_quota(&svc_ctx.db, workspace_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "工作空间配额不存在".to_string())?;

    // 4. 获取工作空间订阅
    let subscription = models::billing::fetch_workspace_active_subscription(&svc_ctx.db, workspace_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 5. 计算月结费用（订阅价格）
    let monthly_billing = subscription
        .as_ref()
        .map(|s| {
            if s.billing_period == "yearly" {
                s.price / Decimal::from(12)
            } else {
                s.price
            }
        })
        .unwrap_or(Decimal::ZERO);

    Ok(AccountInfoResponse {
        email: user_info.email,
        wallet_balance: wallet.balance,
        gift_balance: Decimal::ZERO, // TODO: 如果有赠送金字段，从钱包获取
        currency: wallet.currency,
        subscription,
        quota,
        monthly_billing,
    })
}
