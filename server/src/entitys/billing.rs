use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;
use crate::dto::{
    CouponDto, InvoiceDto, PaymentOrderDto, PlanDto, PlanFeatureDto,
    UserCouponWithDetailsDto, WalletTransactionDto,
};

/// 订阅套餐请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscribePlanRequest {
    pub plan_uuid: Uuid,
    pub billing_period: String,
    pub coupon_code: Option<String>,
    /// 支付方式：wallet（钱包）、alipay（支付宝）、wechat（微信）
    pub payment_method: Option<String>,
}

/// 取消订阅请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CancelSubscriptionRequest {
    pub subscription_uuid: Uuid,
}

/// 恢复订阅请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResumeSubscriptionRequest {
    pub subscription_uuid: Uuid,
}

/// 切换自动续费请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToggleAutoRenewRequest {
    pub subscription_uuid: Uuid,
    pub auto_renew: bool,
}

/// 查询交易记录请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListTransactionsRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub transaction_type: Option<String>,
}

/// 查询发票列表请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListInvoicesRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub status: Option<String>,
}

/// 验证优惠券请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VerifyCouponRequest {
    pub code: String,
    pub amount: Decimal,
}

/// 创建充值订单请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateRechargeOrderRequest {
    pub amount: Decimal,
    pub payment_channel: String,
}

/// 查询支付订单请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListPaymentOrdersRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub order_type: Option<String>,
    pub status: Option<String>,
}

/// 获取用户优惠券列表请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetUserCouponsRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub status: Option<String>, // unused, used, expired
}

/// 发放优惠券请求（管理员功能）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IssueCouponRequest {
    pub coupon_uuid: Uuid,
    pub user_uuid: Uuid,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>, // 可选的自定义过期时间
}

/// 批量发放优惠券请求（管理员功能）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchIssueCouponRequest {
    pub coupon_uuid: Uuid,
    pub user_uuids: Vec<Uuid>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>, // 可选的自定义过期时间
}

/// 获取套餐价格请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetPlanPriceRequest {
    pub plan_uuid: Uuid,
    pub billing_period: String, // monthly, yearly
    pub coupon_code: Option<String>,
}

/// 获取套餐列表请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetPlansRequest {
    /// 优惠券代码（可选）
    pub coupon_code: Option<String>,
    /// 计费周期（用于计算价格，默认为 monthly）
    #[serde(default = "default_billing_period")]
    pub billing_period: String,
}

fn default_billing_period() -> String {
    "monthly".to_string()
}

// ========== 响应结构体 ==========

/// 带特性的套餐结构
#[derive(Debug, Clone, Serialize)]
pub struct PlanWithFeatures {
    pub plan: PlanDto,
    pub features: Vec<PlanFeatureDto>,
    /// 计算后的价格信息（如果提供了优惠券代码）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calculated_price: Option<PlanPriceInfo>,
}

/// 套餐价格信息（用于套餐列表响应）
#[derive(Debug, Clone, Serialize)]
pub struct PlanPriceInfo {
    pub original_price: Decimal,
    pub plan_discount: Decimal,
    pub coupon_discount: Decimal,
    pub final_price: Decimal,
    pub total_saved: Decimal,
    pub billing_period: String, // monthly, yearly
}

/// 套餐列表响应
#[derive(Debug, Clone, Serialize)]
pub struct PlansResponse {
    pub plans: Vec<PlanWithFeatures>,
}

/// 套餐详情响应
#[derive(Debug, Clone, Serialize)]
pub struct PlanDetailResponse {
    pub plan: PlanDto,
    pub features: Vec<PlanFeatureDto>,
}

/// 订阅响应
#[derive(Debug, Clone, Serialize)]
pub struct SubscribeResponse {
    pub subscription_uuid: Uuid,
}

/// 交易记录列表响应
#[derive(Debug, Clone, Serialize)]
pub struct TransactionsListResponse {
    pub items: Vec<WalletTransactionDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 发票列表响应
#[derive(Debug, Clone, Serialize)]
pub struct InvoicesListResponse {
    pub items: Vec<InvoiceDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 优惠券验证响应
#[derive(Debug, Clone, Serialize)]
pub struct VerifyCouponResponse {
    pub coupon_uuid: Uuid,
    pub discount_type: String,
    pub discount_value: rust_decimal::Decimal,
    pub discount_amount: rust_decimal::Decimal,
}

/// 创建订单响应
#[derive(Debug, Clone, Serialize)]
pub struct CreateOrderResponse {
    pub order_uuid: Uuid,
    pub order_no: String,
}

/// 支付订单列表响应
#[derive(Debug, Clone, Serialize)]
pub struct PaymentOrdersListResponse {
    pub items: Vec<PaymentOrderDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 优惠券验证结果
#[derive(Debug, Clone, Serialize)]
pub struct CouponValidationResult {
    pub coupon: CouponDto,
    pub discount_amount: rust_decimal::Decimal,
}

/// 账户信息响应
#[derive(Debug, Clone, Serialize)]
pub struct AccountInfoResponse {
    pub email: String,
    pub wallet_balance: rust_decimal::Decimal,
    pub gift_balance: rust_decimal::Decimal,
    pub currency: String,
    pub subscription: Option<crate::dto::SubscriptionDto>,
    pub quota: crate::dto::WorkspaceQuotaDto,
    pub monthly_billing: rust_decimal::Decimal,
}

/// 用户优惠券列表响应
#[derive(Debug, Clone, Serialize)]
pub struct UserCouponsListResponse {
    pub items: Vec<UserCouponWithDetailsDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 套餐价格响应
#[derive(Debug, Clone, Serialize)]
pub struct PlanPriceResponse {
    pub original_price: Decimal,
    pub plan_discount: Decimal,      // 套餐级折扣金额
    pub coupon_discount: Decimal,    // 优惠券折扣金额
    pub final_price: Decimal,        // 最终价格
    pub total_saved: Decimal,        // 总节省金额
    pub coupon_info: Option<CouponDto>, // 优惠券信息（如果使用了优惠券）
}
