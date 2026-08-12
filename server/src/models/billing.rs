use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{Error, Pool, Postgres, Row};
use uuid::Uuid;

use crate::dto::{
    AutoRenewalServiceDto, CouponDto, InvoiceDto, PaymentOrderDto, PlanDto, PlanFeatureDto,
    SubscriptionDto, UserQuotaDto, UserWalletDto, WalletTransactionDto,
};

// ============ Plans ============

/// 查询所有可用套餐
pub async fn fetch_plans(pool: &Pool<Postgres>) -> Result<Vec<PlanDto>, Error> {
    let recs = sqlx::query_as::<_, PlanDto>(
        r#"
        SELECT id, uuid, name, description, price_per_month, price_per_year,
               currency, discount_monthly, discount_yearly, max_environments,
               max_team_members, max_proxies, max_rpa_tasks, is_recommended,
               sort_order, status, created_at, updated_at
        FROM plans
        WHERE status = 'active'
        ORDER BY sort_order ASC, price_per_month ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 根据 UUID 查询套餐
pub async fn fetch_plan_by_uuid(
    pool: &Pool<Postgres>,
    plan_uuid: Uuid,
) -> Result<Option<PlanDto>, Error> {
    let rec = sqlx::query_as::<_, PlanDto>(
        r#"
        SELECT id, uuid, name, description, price_per_month, price_per_year,
               currency, discount_monthly, discount_yearly, max_environments,
               max_team_members, max_proxies, max_rpa_tasks, is_recommended,
               sort_order, status, created_at, updated_at
        FROM plans
        WHERE uuid = $1
        "#,
    )
    .bind(plan_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 查询套餐特性
pub async fn fetch_plan_features(
    pool: &Pool<Postgres>,
    plan_uuid: Uuid,
) -> Result<Vec<PlanFeatureDto>, Error> {
    let recs = sqlx::query_as::<_, PlanFeatureDto>(
        r#"
        SELECT id, plan_uuid, feature_key, feature_name, feature_value,
               is_included, sort_order, created_at
        FROM plan_features
        WHERE plan_uuid = $1
        ORDER BY sort_order ASC
        "#,
    )
    .bind(plan_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

// ============ Subscriptions ============

/// 查询用户当前订阅（按用户）
pub async fn fetch_active_subscription(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<Option<SubscriptionDto>, Error> {
    let rec = sqlx::query_as::<_, SubscriptionDto>(
        r#"
        SELECT id, uuid, workspace_uuid, user_uuid, plan_uuid, billing_period, price, currency,
               started_at, expires_at, next_billing_date, auto_renew, status,
               cancelled_at, created_at, updated_at
        FROM subscriptions
        WHERE user_uuid = $1 AND status IN ('active', 'paused')
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 查询工作空间当前订阅
pub async fn fetch_workspace_active_subscription(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
) -> Result<Option<SubscriptionDto>, Error> {
    let rec = sqlx::query_as::<_, SubscriptionDto>(
        r#"
        SELECT id, uuid, workspace_uuid, user_uuid, plan_uuid, billing_period, price, currency,
               started_at, expires_at, next_billing_date, auto_renew, status,
               cancelled_at, created_at, updated_at
        FROM subscriptions
        WHERE workspace_uuid = $1 AND status IN ('active', 'paused')
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(workspace_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 根据 UUID 查询订阅
pub async fn fetch_subscription_by_uuid(
    pool: &Pool<Postgres>,
    subscription_uuid: Uuid,
) -> Result<Option<SubscriptionDto>, Error> {
    let rec = sqlx::query_as::<_, SubscriptionDto>(
        r#"
        SELECT id, uuid, workspace_uuid, user_uuid, plan_uuid, billing_period, price, currency,
               started_at, expires_at, next_billing_date, auto_renew, status,
               cancelled_at, created_at, updated_at
        FROM subscriptions
        WHERE uuid = $1
        "#,
    )
    .bind(subscription_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 创建订阅
pub async fn insert_subscription(
    pool: &Pool<Postgres>,
    workspace_uuid: Uuid,
    user_uuid: Uuid,
    plan_uuid: Uuid,
    billing_period: &str,
    price: Decimal,
    currency: &str,
    expires_at: chrono::DateTime<chrono::Utc>,
    next_billing_date: NaiveDate,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO subscriptions (workspace_uuid, user_uuid, plan_uuid, billing_period, price, currency,
                                    started_at, expires_at, next_billing_date, auto_renew, status)
        VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP, $7, $8, true, 'active')
        RETURNING uuid;
        "#,
    )
    .bind(workspace_uuid)
    .bind(user_uuid)
    .bind(plan_uuid)
    .bind(billing_period)
    .bind(price)
    .bind(currency)
    .bind(expires_at)
    .bind(next_billing_date)
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

/// 取消订阅
pub async fn cancel_subscription(
    pool: &Pool<Postgres>,
    subscription_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE subscriptions
        SET status = 'cancelled', cancelled_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
        WHERE uuid = $1
        "#,
    )
    .bind(subscription_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 恢复订阅
pub async fn resume_subscription(
    pool: &Pool<Postgres>,
    subscription_uuid: Uuid,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE subscriptions
        SET status = 'active', cancelled_at = NULL, updated_at = CURRENT_TIMESTAMP
        WHERE uuid = $1
        "#,
    )
    .bind(subscription_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 切换自动续费
pub async fn toggle_auto_renew(
    pool: &Pool<Postgres>,
    subscription_uuid: Uuid,
    auto_renew: bool,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE subscriptions
        SET auto_renew = $1, updated_at = CURRENT_TIMESTAMP
        WHERE uuid = $2
        "#,
    )
    .bind(auto_renew)
    .bind(subscription_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

// ============ User Wallet ============

/// 获取用户钱包
pub async fn fetch_user_wallet(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<Option<UserWalletDto>, Error> {
    let rec = sqlx::query_as::<_, UserWalletDto>(
        r#"
        SELECT id, user_uuid, balance, currency, frozen_amount, auto_renewal_combined,
               created_at, updated_at
        FROM user_wallets
        WHERE user_uuid = $1
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 创建用户钱包
pub async fn insert_user_wallet(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    currency: &str,
) -> Result<i32, Error> {
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO user_wallets (user_uuid, balance, currency, frozen_amount, auto_renewal_combined)
        VALUES ($1, 0, $2, 0, 0)
        ON CONFLICT (user_uuid) DO NOTHING
        RETURNING id;
        "#,
    )
    .bind(user_uuid)
    .bind(currency)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// 更新钱包余额
pub async fn update_wallet_balance(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    amount: Decimal,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE user_wallets
        SET balance = balance + $1, updated_at = CURRENT_TIMESTAMP
        WHERE user_uuid = $2
        "#,
    )
    .bind(amount)
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

// ============ Wallet Transactions ============

/// 查询钱包交易记录
pub async fn fetch_wallet_transactions(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    transaction_type: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<WalletTransactionDto>, Error> {
    let recs = sqlx::query_as::<_, WalletTransactionDto>(
        r#"
        SELECT id, uuid, user_uuid, transaction_type, amount, currency,
               balance_before, balance_after, description, order_uuid, status, created_at
        FROM wallet_transactions
        WHERE user_uuid = $1
          AND ($2::varchar IS NULL OR transaction_type = $2)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(user_uuid)
    .bind(transaction_type)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询交易记录总数
pub async fn fetch_wallet_transactions_count(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    transaction_type: Option<&str>,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM wallet_transactions
        WHERE user_uuid = $1
          AND ($2::varchar IS NULL OR transaction_type = $2)
        "#,
    )
    .bind(user_uuid)
    .bind(transaction_type)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 插入钱包交易记录
pub async fn insert_wallet_transaction(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    transaction_type: &str,
    amount: Decimal,
    currency: &str,
    balance_before: Decimal,
    balance_after: Decimal,
    description: Option<&str>,
    order_uuid: Option<Uuid>,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO wallet_transactions (user_uuid, transaction_type, amount, currency,
                                          balance_before, balance_after, description, order_uuid, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'completed')
        RETURNING uuid;
        "#,
    )
    .bind(user_uuid)
    .bind(transaction_type)
    .bind(amount)
    .bind(currency)
    .bind(balance_before)
    .bind(balance_after)
    .bind(description)
    .bind(order_uuid)
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

// ============ Invoices ============

/// 查询发票列表
pub async fn fetch_invoices(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    status: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<InvoiceDto>, Error> {
    let recs = sqlx::query_as::<_, InvoiceDto>(
        r#"
        SELECT id, uuid, user_uuid, invoice_number, amount, currency,
               subscription_uuid, order_uuid, invoice_type, status,
               issued_at, due_at, paid_at, invoice_url, created_at, updated_at
        FROM invoices
        WHERE user_uuid = $1
          AND ($2::varchar IS NULL OR status = $2)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(user_uuid)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询发票总数
pub async fn fetch_invoices_count(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    status: Option<&str>,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM invoices
        WHERE user_uuid = $1
          AND ($2::varchar IS NULL OR status = $2)
        "#,
    )
    .bind(user_uuid)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 创建发票
pub async fn insert_invoice(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    invoice_number: &str,
    amount: Decimal,
    currency: &str,
    subscription_uuid: Option<Uuid>,
    order_uuid: Option<Uuid>,
    invoice_type: &str,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO invoices (user_uuid, invoice_number, amount, currency,
                              subscription_uuid, order_uuid, invoice_type, status, issued_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', CURRENT_TIMESTAMP)
        RETURNING uuid;
        "#,
    )
    .bind(user_uuid)
    .bind(invoice_number)
    .bind(amount)
    .bind(currency)
    .bind(subscription_uuid)
    .bind(order_uuid)
    .bind(invoice_type)
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

// ============ User Quotas ============

/// 获取用户配额
pub async fn fetch_user_quota(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<Option<UserQuotaDto>, Error> {
    let rec = sqlx::query_as::<_, UserQuotaDto>(
        r#"
        SELECT id, user_uuid, max_environments, used_environments, max_team_members,
               max_proxies, used_proxies, max_rpa_tasks, used_rpa_tasks,
               created_at, updated_at
        FROM user_quotas
        WHERE user_uuid = $1
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 创建或更新用户配额
pub async fn upsert_user_quota(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    max_environments: i32,
    max_team_members: i32,
    max_proxies: i32,
    max_rpa_tasks: i32,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        INSERT INTO user_quotas (user_uuid, max_environments, max_team_members, max_proxies, max_rpa_tasks)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (user_uuid) DO UPDATE SET
            max_environments = $2,
            max_team_members = $3,
            max_proxies = $4,
            max_rpa_tasks = $5,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(user_uuid)
    .bind(max_environments)
    .bind(max_team_members)
    .bind(max_proxies)
    .bind(max_rpa_tasks)
    .execute(pool)
    .await?;

    Ok(())
}

// ============ Coupons ============

/// 根据优惠码查询优惠券
pub async fn fetch_coupon_by_code(
    pool: &Pool<Postgres>,
    code: &str,
) -> Result<Option<CouponDto>, Error> {
    let rec = sqlx::query_as::<_, CouponDto>(
        r#"
        SELECT id, uuid, code, name, description, discount_type, discount_value, min_amount, max_discount,
               max_uses, used_count, max_uses_per_user, valid_from, valid_until,
               applicable_to, status, created_at, updated_at
        FROM coupons
        WHERE code = $1 AND status = 'active'
        "#,
    )
    .bind(code)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 根据 UUID 查询优惠券
pub async fn fetch_coupon_by_uuid(
    pool: &Pool<Postgres>,
    coupon_uuid: Uuid,
) -> Result<Option<CouponDto>, Error> {
    let rec = sqlx::query_as::<_, CouponDto>(
        r#"
        SELECT id, uuid, code, name, description, discount_type, discount_value, min_amount, max_discount,
               max_uses, used_count, max_uses_per_user, valid_from, valid_until,
               applicable_to, status, created_at, updated_at
        FROM coupons
        WHERE uuid = $1 AND status = 'active'
        "#,
    )
    .bind(coupon_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 查询用户优惠券使用次数
pub async fn fetch_coupon_user_usage_count(
    pool: &Pool<Postgres>,
    coupon_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<i32, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM coupon_usages
        WHERE coupon_uuid = $1 AND user_uuid = $2
        "#,
    )
    .bind(coupon_uuid)
    .bind(user_uuid)
    .fetch_one(pool)
    .await?;

    Ok(count as i32)
}

/// 记录优惠券使用
pub async fn insert_coupon_usage(
    pool: &Pool<Postgres>,
    coupon_uuid: Uuid,
    user_uuid: Uuid,
    order_uuid: Option<Uuid>,
    discount_amount: Decimal,
) -> Result<i32, Error> {
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO coupon_usages (coupon_uuid, user_uuid, order_uuid, discount_amount)
        VALUES ($1, $2, $3, $4)
        RETURNING id;
        "#,
    )
    .bind(coupon_uuid)
    .bind(user_uuid)
    .bind(order_uuid)
    .bind(discount_amount)
    .fetch_one(pool)
    .await?;

    // 更新优惠券使用次数
    sqlx::query("UPDATE coupons SET used_count = used_count + 1 WHERE uuid = $1")
        .bind(coupon_uuid)
        .execute(pool)
        .await?;

    Ok(id)
}

// ============ User Coupons ============

/// 查询用户优惠券列表
pub async fn fetch_user_coupons(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    status: Option<&str>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<crate::dto::UserCouponWithDetailsDto>, i64), Error> {
    let offset = (page - 1) * page_size;

    let (items, total) = if let Some(s) = status {
        let rows = sqlx::query(
            r#"
            SELECT 
                uc.id, uc.user_uuid, uc.coupon_uuid, uc.status, 
                uc.issued_at, uc.used_at, uc.expires_at,
                c.code, c.name, c.description, c.discount_type, c.discount_value, c.min_amount, c.max_discount
            FROM user_coupons uc
            INNER JOIN coupons c ON uc.coupon_uuid = c.uuid
            WHERE uc.user_uuid = $1 AND uc.status = $2
            ORDER BY uc.issued_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(user_uuid)
        .bind(s)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let items: Vec<crate::dto::UserCouponWithDetailsDto> = rows
            .into_iter()
            .map(|row| crate::dto::UserCouponWithDetailsDto {
                id: row.get(0),
                user_uuid: row.get(1),
                coupon_uuid: row.get(2),
                status: row.get(3),
                issued_at: row.get(4),
                used_at: row.get(5),
                expires_at: row.get(6),
                code: row.get(7),
                name: row.get(8),
                description: row.get(9),
                discount_type: row.get(10),
                discount_value: row.get(11),
                min_amount: row.get(12),
                max_discount: row.get(13),
            })
            .collect();

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM user_coupons
            WHERE user_uuid = $1 AND status = $2
            "#,
        )
        .bind(user_uuid)
        .bind(s)
        .fetch_one(pool)
        .await?;

        (items, total)
    } else {
        let rows = sqlx::query(
            r#"
            SELECT 
                uc.id, uc.user_uuid, uc.coupon_uuid, uc.status, 
                uc.issued_at, uc.used_at, uc.expires_at,
                c.code, c.name, c.description, c.discount_type, c.discount_value, c.min_amount, c.max_discount
            FROM user_coupons uc
            INNER JOIN coupons c ON uc.coupon_uuid = c.uuid
            WHERE uc.user_uuid = $1
            ORDER BY uc.issued_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_uuid)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let items: Vec<crate::dto::UserCouponWithDetailsDto> = rows
            .into_iter()
            .map(|row| crate::dto::UserCouponWithDetailsDto {
                id: row.get(0),
                user_uuid: row.get(1),
                coupon_uuid: row.get(2),
                status: row.get(3),
                issued_at: row.get(4),
                used_at: row.get(5),
                expires_at: row.get(6),
                code: row.get(7),
                name: row.get(8),
                description: row.get(9),
                discount_type: row.get(10),
                discount_value: row.get(11),
                min_amount: row.get(12),
                max_discount: row.get(13),
            })
            .collect();

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM user_coupons
            WHERE user_uuid = $1
            "#,
        )
        .bind(user_uuid)
        .fetch_one(pool)
        .await?;

        (items, total)
    };

    Ok((items, total))
}

/// 根据 UUID 查询用户优惠券
pub async fn fetch_user_coupon_by_uuid(
    pool: &Pool<Postgres>,
    user_coupon_uuid: i32,
) -> Result<Option<crate::dto::UserCouponDto>, Error> {
    let rec = sqlx::query_as::<_, crate::dto::UserCouponDto>(
        r#"
        SELECT id, user_uuid, coupon_uuid, status, issued_at, used_at, expires_at
        FROM user_coupons
        WHERE id = $1
        "#,
    )
    .bind(user_coupon_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 发放优惠券给用户
pub async fn insert_user_coupon(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    coupon_uuid: Uuid,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<i32, Error> {
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO user_coupons (user_uuid, coupon_uuid, expires_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_uuid, coupon_uuid) DO NOTHING
        RETURNING id;
        "#,
    )
    .bind(user_uuid)
    .bind(coupon_uuid)
    .bind(expires_at)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        Error::RowNotFound // 如果已存在，返回错误
    })?;

    Ok(id)
}

/// 更新用户优惠券状态
pub async fn update_user_coupon_status(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    coupon_uuid: Uuid,
    status: &str,
    used_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE user_coupons
        SET status = $1, used_at = $2, updated_at = CURRENT_TIMESTAMP
        WHERE user_uuid = $3 AND coupon_uuid = $4
        "#,
    )
    .bind(status)
    .bind(used_at)
    .bind(user_uuid)
    .bind(coupon_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

/// 批量发放优惠券
pub async fn batch_insert_user_coupons(
    pool: &Pool<Postgres>,
    coupon_uuid: Uuid,
    user_uuids: &[Uuid],
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<usize, Error> {
    let mut tx = pool.begin().await?;
    let mut count = 0;

    for user_uuid in user_uuids {
        let result = sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO user_coupons (user_uuid, coupon_uuid, expires_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_uuid, coupon_uuid) DO NOTHING
            RETURNING id;
            "#,
        )
        .bind(user_uuid)
        .bind(coupon_uuid)
        .bind(expires_at)
        .fetch_optional(&mut *tx)
        .await?;

        if result.is_some() {
            count += 1;
        }
    }

    tx.commit().await?;
    Ok(count)
}

/// 查询用户可用优惠券（未使用且未过期，包含优惠券详细信息）
pub async fn fetch_available_user_coupons(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<Vec<crate::dto::UserCouponWithDetailsDto>, Error> {
    let now = chrono::Utc::now();

    let rows = sqlx::query(
        r#"
        SELECT 
            uc.id, uc.user_uuid, uc.coupon_uuid, uc.status, 
            uc.issued_at, uc.used_at, uc.expires_at,
            c.code, c.name, c.description, c.discount_type, c.discount_value, c.min_amount, c.max_discount
        FROM user_coupons uc
        INNER JOIN coupons c ON uc.coupon_uuid = c.uuid
        WHERE uc.user_uuid = $1
          AND uc.status = 'unused'
          AND c.status = 'active'
          AND (uc.expires_at IS NULL OR uc.expires_at > $2)
          AND (c.valid_until IS NULL OR c.valid_until > $2)
          AND c.valid_from <= $2
        ORDER BY uc.issued_at DESC
        "#,
    )
    .bind(user_uuid)
    .bind(now)
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|row| crate::dto::UserCouponWithDetailsDto {
            id: row.get(0),
            user_uuid: row.get(1),
            coupon_uuid: row.get(2),
            status: row.get(3),
            issued_at: row.get(4),
            used_at: row.get(5),
            expires_at: row.get(6),
            code: row.get(7),
            name: row.get(8),
            description: row.get(9),
            discount_type: row.get(10),
            discount_value: row.get(11),
            min_amount: row.get(12),
            max_discount: row.get(13),
        })
        .collect();

    Ok(items)
}

// ============ Payment Orders ============

/// 创建支付订单
pub async fn insert_payment_order(
    pool: &Pool<Postgres>,
    order_no: &str,
    user_uuid: Uuid,
    order_type: &str,
    amount: Decimal,
    currency: &str,
    payment_channel: Option<&str>,
    description: Option<&str>,
    subscription_uuid: Option<Uuid>,
    coupon_uuid: Option<Uuid>,
    original_amount: Option<Decimal>,
    discount_amount: Option<Decimal>,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO payment_orders (order_no, user_uuid, order_type, amount, currency,
                                     status, payment_channel, description, subscription_uuid,
                                     coupon_uuid, original_amount, discount_amount)
        VALUES ($1, $2, $3, $4, $5, 'pending', $6, $7, $8, $9, $10, $11)
        RETURNING uuid;
        "#,
    )
    .bind(order_no)
    .bind(user_uuid)
    .bind(order_type)
    .bind(amount)
    .bind(currency)
    .bind(payment_channel)
    .bind(description)
    .bind(subscription_uuid)
    .bind(coupon_uuid)
    .bind(original_amount)
    .bind(discount_amount)
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

/// 查询支付订单列表
pub async fn fetch_payment_orders(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    order_type: Option<&str>,
    status: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<PaymentOrderDto>, Error> {
    let recs = sqlx::query_as::<_, PaymentOrderDto>(
        r#"
        SELECT id, uuid, order_no, user_uuid, order_type, amount, currency, status,
               payment_channel, external_order_id, description, subscription_uuid,
               coupon_uuid, original_amount, discount_amount, paid_at, refunded_at,
               created_at, updated_at
        FROM payment_orders
        WHERE user_uuid = $1
          AND ($2::varchar IS NULL OR order_type = $2)
          AND ($3::varchar IS NULL OR status = $3)
        ORDER BY created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(user_uuid)
    .bind(order_type)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询支付订单总数
pub async fn fetch_payment_orders_count(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    order_type: Option<&str>,
    status: Option<&str>,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM payment_orders
        WHERE user_uuid = $1
          AND ($2::varchar IS NULL OR order_type = $2)
          AND ($3::varchar IS NULL OR status = $3)
        "#,
    )
    .bind(user_uuid)
    .bind(order_type)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 根据 UUID 查询支付订单
pub async fn fetch_payment_order_by_uuid(
    pool: &Pool<Postgres>,
    order_uuid: Uuid,
) -> Result<Option<PaymentOrderDto>, Error> {
    let rec = sqlx::query_as::<_, PaymentOrderDto>(
        r#"
        SELECT id, uuid, order_no, user_uuid, order_type, amount, currency, status,
               payment_channel, external_order_id, description, subscription_uuid,
               coupon_uuid, original_amount, discount_amount, paid_at, refunded_at,
               created_at, updated_at
        FROM payment_orders
        WHERE uuid = $1
        "#,
    )
    .bind(order_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 更新订单状态
pub async fn update_payment_order_status(
    pool: &Pool<Postgres>,
    order_uuid: Uuid,
    status: &str,
    external_order_id: Option<&str>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE payment_orders
        SET status = $1,
            external_order_id = COALESCE($2, external_order_id),
            paid_at = CASE WHEN $1 = 'paid' THEN CURRENT_TIMESTAMP ELSE paid_at END,
            updated_at = CURRENT_TIMESTAMP
        WHERE uuid = $3
        "#,
    )
    .bind(status)
    .bind(external_order_id)
    .bind(order_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

// ============ Auto Renewal Services ============

/// 查询用户自动续费服务列表
pub async fn fetch_auto_renewal_services(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<Vec<AutoRenewalServiceDto>, Error> {
    let recs = sqlx::query_as::<_, AutoRenewalServiceDto>(
        r#"
        SELECT id, uuid, user_uuid, service_type, service_uuid, service_name,
               renewal_price, currency, next_bill_date, status, created_at, updated_at
        FROM auto_renewal_services
        WHERE user_uuid = $1 AND status = 'active'
        ORDER BY next_bill_date ASC
        "#,
    )
    .bind(user_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}
