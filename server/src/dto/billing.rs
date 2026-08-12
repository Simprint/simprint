use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// 套餐 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PlanDto {
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price_per_month: Decimal,
    pub price_per_year: Decimal,
    pub currency: String,
    pub discount_monthly: Option<Decimal>,
    pub discount_yearly: Option<Decimal>,
    pub max_environments: i32,
    pub max_team_members: i32,
    pub max_proxies: i32,
    pub max_rpa_tasks: i32,
    pub is_recommended: Option<bool>,
    pub sort_order: Option<i32>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 套餐特性 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PlanFeatureDto {
    pub id: i32,
    pub plan_uuid: Uuid,
    pub feature_key: String,
    pub feature_name: String,
    pub feature_value: Option<String>,
    pub is_included: Option<bool>,
    pub sort_order: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// 订阅 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct SubscriptionDto {
    pub id: i32,
    pub uuid: Uuid,
    pub workspace_uuid: Uuid,
    pub user_uuid: Uuid,
    pub plan_uuid: Uuid,
    pub billing_period: String,
    pub price: Decimal,
    pub currency: String,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub next_billing_date: Option<NaiveDate>,
    pub auto_renew: Option<bool>,
    pub status: String,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户钱包 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserWalletDto {
    pub id: i32,
    pub user_uuid: Uuid,
    pub balance: Decimal,
    pub currency: String,
    pub frozen_amount: Decimal,
    pub auto_renewal_combined: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 钱包交易 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct WalletTransactionDto {
    pub id: i64,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub transaction_type: String,
    pub amount: Decimal,
    pub currency: String,
    pub balance_before: Decimal,
    pub balance_after: Decimal,
    pub description: Option<String>,
    pub order_uuid: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// 发票 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct InvoiceDto {
    pub id: i32,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub invoice_number: String,
    pub amount: Decimal,
    pub currency: String,
    pub subscription_uuid: Option<Uuid>,
    pub order_uuid: Option<Uuid>,
    pub invoice_type: String,
    pub status: String,
    pub issued_at: Option<DateTime<Utc>>,
    pub due_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub invoice_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户配额 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserQuotaDto {
    pub id: i32,
    pub user_uuid: Uuid,
    pub max_environments: i32,
    pub used_environments: i32,
    pub max_team_members: i32,
    pub max_proxies: i32,
    pub used_proxies: i32,
    pub max_rpa_tasks: i32,
    pub used_rpa_tasks: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 优惠券 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CouponDto {
    pub id: i32,
    pub uuid: Uuid,
    pub code: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub discount_type: String,
    pub discount_value: Decimal,
    pub min_amount: Option<Decimal>,
    pub max_discount: Option<Decimal>,
    pub max_uses: Option<i32>,
    pub used_count: i32,
    pub max_uses_per_user: Option<i32>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub applicable_to: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 优惠券使用记录 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CouponUsageDto {
    pub id: i32,
    pub coupon_uuid: Uuid,
    pub user_uuid: Uuid,
    pub order_uuid: Option<Uuid>,
    pub discount_amount: Decimal,
    pub used_at: DateTime<Utc>,
}

/// 用户优惠券 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserCouponDto {
    pub id: i32,
    pub user_uuid: Uuid,
    pub coupon_uuid: Uuid,
    pub status: String,
    pub issued_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// 用户优惠券详细信息 DTO（包含优惠券详情）
#[derive(Debug, Clone, Serialize)]
pub struct UserCouponWithDetailsDto {
    pub id: i32,
    pub user_uuid: Uuid,
    pub coupon_uuid: Uuid,
    pub status: String,
    pub issued_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    // 优惠券详细信息
    pub code: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub discount_type: String,
    pub discount_value: Decimal,
    pub min_amount: Option<Decimal>,
    pub max_discount: Option<Decimal>,
}

/// 支付订单 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PaymentOrderDto {
    pub id: i64,
    pub uuid: Uuid,
    pub order_no: String,
    pub user_uuid: Uuid,
    pub order_type: String,
    pub amount: Decimal,
    pub currency: String,
    pub status: String,
    pub payment_channel: Option<String>,
    pub external_order_id: Option<String>,
    pub description: Option<String>,
    pub subscription_uuid: Option<Uuid>,
    pub coupon_uuid: Option<Uuid>,
    pub original_amount: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub paid_at: Option<DateTime<Utc>>,
    pub refunded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 自动续费服务 DTO
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AutoRenewalServiceDto {
    pub id: i32,
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub service_type: String,
    pub service_uuid: Option<Uuid>,
    pub service_name: String,
    pub renewal_price: Decimal,
    pub currency: String,
    pub next_bill_date: NaiveDate,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
