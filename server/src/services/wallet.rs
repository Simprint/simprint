use rust_decimal::Decimal;
use uuid::Uuid;

use crate::dto::{UserWalletDto, WalletTransactionDto};
use crate::entitys::ListTransactionsRequest;
use crate::models;
use crate::svc_ctx::SvcCtx;

/// 获取钱包信息
pub async fn get_wallet_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<UserWalletDto, String> {
    let wallet = models::billing::fetch_user_wallet(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 如果钱包不存在，创建一个
    if wallet.is_none() {
        models::billing::insert_user_wallet(&svc_ctx.db, user_uuid, "CNY")
            .await
            .map_err(|e| e.to_string())?;

        return models::billing::fetch_user_wallet(&svc_ctx.db, user_uuid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "创建钱包失败".to_string());
    }

    wallet.ok_or_else(|| "钱包不存在".to_string())
}

/// 扣减钱包余额
pub async fn deduct_wallet_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    amount: Decimal,
    description: &str,
    order_uuid: Option<Uuid>,
) -> Result<(), String> {
    let wallet = models::billing::fetch_user_wallet(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "钱包不存在".to_string())?;

    let balance_before = wallet.balance;
    let balance_after = balance_before - amount;

    if balance_after < Decimal::ZERO {
        return Err("余额不足".to_string());
    }

    // 扣减余额
    models::billing::update_wallet_balance(&svc_ctx.db, user_uuid, -amount)
        .await
        .map_err(|e| e.to_string())?;

    // 记录交易
    models::billing::insert_wallet_transaction(
        &svc_ctx.db,
        user_uuid,
        "debit",
        -amount,
        &wallet.currency,
        balance_before,
        balance_after,
        Some(description),
        order_uuid,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 获取交易记录
pub async fn get_transactions_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &ListTransactionsRequest,
) -> Result<(Vec<WalletTransactionDto>, i64), String> {
    let offset = (payload.pagination.page - 1) * payload.pagination.page_size;

    let transactions = models::billing::fetch_wallet_transactions(
        &svc_ctx.db,
        user_uuid,
        payload.transaction_type.as_deref(),
        offset,
        payload.pagination.page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    let total = models::billing::fetch_wallet_transactions_count(
        &svc_ctx.db,
        user_uuid,
        payload.transaction_type.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok((transactions, total))
}
