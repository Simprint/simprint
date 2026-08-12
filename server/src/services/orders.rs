use uuid::Uuid;

use crate::dto::PaymentOrderDto;
use crate::entitys::{CreateRechargeOrderRequest, ListPaymentOrdersRequest};
use crate::models;
use crate::svc_ctx::SvcCtx;

/// 创建充值订单
pub async fn create_recharge_order_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &CreateRechargeOrderRequest,
) -> Result<(Uuid, String), String> {
    let order_no = format!("RCH-{}", chrono::Utc::now().format("%Y%m%d%H%M%S%f"));

    let order_uuid = models::billing::insert_payment_order(
        &svc_ctx.db,
        &order_no,
        user_uuid,
        "recharge",
        payload.amount,
        "CNY",
        Some(&payload.payment_channel),
        Some("钱包充值"),
        None,
        None,
        None,
        None,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok((order_uuid, order_no))
}

/// 获取支付订单列表
pub async fn get_payment_orders_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &ListPaymentOrdersRequest,
) -> Result<(Vec<PaymentOrderDto>, i64), String> {
    let offset = (payload.pagination.page - 1) * payload.pagination.page_size;

    let orders = models::billing::fetch_payment_orders(
        &svc_ctx.db,
        user_uuid,
        payload.order_type.as_deref(),
        payload.status.as_deref(),
        offset,
        payload.pagination.page_size,
    )
    .await
    .map_err(|e| e.to_string())?;

    let total = models::billing::fetch_payment_orders_count(
        &svc_ctx.db,
        user_uuid,
        payload.order_type.as_deref(),
        payload.status.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok((orders, total))
}

/// 查询订单状态
pub async fn get_order_status_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    order_uuid: Uuid,
) -> Result<PaymentOrderDto, String> {
    let order = models::billing::fetch_payment_order_by_uuid(&svc_ctx.db, order_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "订单不存在".to_string())?;

    if order.user_uuid != user_uuid {
        return Err("无权查看此订单".to_string());
    }

    Ok(order)
}
