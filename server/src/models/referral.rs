use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::{
    RedeemOptionDto, RedeemRecordDto, ReferralLinkDto, ReferralLinkTierDto, ReferralRewardDto,
    UserReferralPointsDto,
};
use crate::dto::ReferredUserRow;

// ============ Referral Link Tiers ============

/// 查询所有层级
pub async fn fetch_referral_tiers(
    pool: &Pool<Postgres>,
) -> Result<Vec<ReferralLinkTierDto>, Error> {
    let recs = sqlx::query_as::<_, ReferralLinkTierDto>(
        r#"
        SELECT id, uuid, name, unlock_threshold, reward_rate, discount_rate,
               description, sort_order, created_at, updated_at
        FROM referral_link_tiers
        ORDER BY unlock_threshold ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 根据用户当前已解锁的层级，更新其推广链接的解锁状态。
///
/// 约定：
/// - 同一用户的所有推广链接默认创建完成（最多 4 条），分别对应不同 tier_uuid；
/// - 这里根据传入的 unlocked_tier_uuids，将属于这些层级的链接标记为 unlocked = TRUE，
///   其他链接统一标记为 FALSE。
pub async fn update_referral_links_unlock_status_for_user(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    unlocked_tier_uuids: &[Uuid],
) -> Result<(), Error> {
    // 先将该用户所有链接标记为未解锁
    sqlx::query(
        r#"
        UPDATE referral_links
        SET unlocked = FALSE
        WHERE user_uuid = $1;
        "#,
    )
    .bind(user_uuid)
    .execute(pool)
    .await?;

    // 再根据传入的 tier_uuid 集合解锁对应链接
    if !unlocked_tier_uuids.is_empty() {
        sqlx::query(
            r#"
            UPDATE referral_links
            SET unlocked = TRUE
            WHERE user_uuid = $1
              AND tier_uuid = ANY($2::uuid[]);
            "#,
        )
        .bind(user_uuid)
        .bind(unlocked_tier_uuids)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// 根据解锁阈值查询层级
pub async fn fetch_tier_by_threshold(
    pool: &Pool<Postgres>,
    threshold: i32,
) -> Result<Option<ReferralLinkTierDto>, Error> {
    let rec = sqlx::query_as::<_, ReferralLinkTierDto>(
        r#"
        SELECT id, uuid, name, unlock_threshold, reward_rate, discount_rate,
               description, sort_order, created_at, updated_at
        FROM referral_link_tiers
        WHERE unlock_threshold <= $1
        ORDER BY unlock_threshold DESC
        LIMIT 1
        "#,
    )
    .bind(threshold)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 查询下一个层级
pub async fn fetch_next_tier(
    pool: &Pool<Postgres>,
    current_threshold: i32,
) -> Result<Option<ReferralLinkTierDto>, Error> {
    let rec = sqlx::query_as::<_, ReferralLinkTierDto>(
        r#"
        SELECT id, uuid, name, unlock_threshold, reward_rate, discount_rate,
               description, sort_order, created_at, updated_at
        FROM referral_link_tiers
        WHERE unlock_threshold > $1
        ORDER BY unlock_threshold ASC
        LIMIT 1
        "#,
    )
    .bind(current_threshold)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

// ============ Referral Links ============

/// 查询用户的推广链接列表
pub async fn fetch_user_referral_links(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<Vec<ReferralLinkDto>, Error> {
    // 为了保证前端展示顺序与层级解锁顺序一致，这里按对应层级的 unlock_threshold 排序，
    // 对于没有 tier_uuid 的链接，则退而按创建时间排序并排在最后。
    let recs = sqlx::query_as::<_, ReferralLinkDto>(
        r#"
        SELECT rl.id,
               rl.uuid,
               rl.user_uuid,
               rl.code,
               rl.url,
               rl.tier_uuid,
               rl.unlocked,
               rl.is_current,
               rl.reward_rate,
               rl.discount_rate,
               rl.registered_users,
               rl.paid_users,
               rl.total_consumption,
               rl.last_30_days_consumption,
               rl.created_at,
               rl.updated_at
        FROM referral_links rl
        LEFT JOIN referral_link_tiers t
               ON rl.tier_uuid = t.uuid
        WHERE rl.user_uuid = $1
        ORDER BY
            t.unlock_threshold ASC NULLS LAST,
            rl.created_at ASC
        "#,
    )
    .bind(user_uuid)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询当前推广链接
pub async fn fetch_current_referral_link(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<Option<ReferralLinkDto>, Error> {
    let rec = sqlx::query_as::<_, ReferralLinkDto>(
        r#"
        SELECT id, uuid, user_uuid, code, url, tier_uuid, unlocked, is_current,
               reward_rate, discount_rate, registered_users, paid_users,
               total_consumption, last_30_days_consumption, created_at, updated_at
        FROM referral_links
        WHERE user_uuid = $1 AND is_current = true
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 切换当前推广链接
pub async fn switch_current_referral_link(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    link_uuid: Uuid,
) -> Result<(), Error> {
    // 先取消所有当前链接
    sqlx::query("UPDATE referral_links SET is_current = false WHERE user_uuid = $1")
        .bind(user_uuid)
        .execute(pool)
        .await?;

    // 设置新的当前链接
    sqlx::query("UPDATE referral_links SET is_current = true WHERE uuid = $1 AND user_uuid = $2")
        .bind(link_uuid)
        .bind(user_uuid)
        .execute(pool)
        .await?;

    Ok(())
}

// ============ User Referrals ============

/// 查询被邀请用户列表
pub async fn fetch_referred_users(
    pool: &Pool<Postgres>,
    inviter_uuid: Uuid,
    keyword: Option<&str>,
    status: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<ReferredUserRow>, Error> {
    let recs = sqlx::query_as::<_, ReferredUserRow>(
        r#"
        SELECT
            ur.id,
            ui.email,
            ur.status,
            ur.link_uuid,
            ur.total_consumption,
            ur.last_30_days_consumption,
            ur.registered_at
        FROM user_referrals ur
        JOIN user_infos ui ON ui.user_uuid = ur.invitee_uuid
        WHERE ur.inviter_uuid = $1
          AND ($2::text IS NULL OR ui.email ILIKE '%' || $2 || '%')
          AND ($3::varchar IS NULL OR ur.status = $3)
        ORDER BY registered_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(inviter_uuid)
    .bind(keyword)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询被邀请用户总数
pub async fn fetch_referred_users_count(
    pool: &Pool<Postgres>,
    inviter_uuid: Uuid,
    keyword: Option<&str>,
    status: Option<&str>,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM user_referrals ur
        JOIN user_infos ui ON ui.user_uuid = ur.invitee_uuid
        WHERE ur.inviter_uuid = $1
          AND ($2::text IS NULL OR ui.email ILIKE '%' || $2 || '%')
          AND ($3::varchar IS NULL OR ur.status = $3)
        "#,
    )
    .bind(inviter_uuid)
    .bind(keyword)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 获取推广统计
pub async fn fetch_referral_stats(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<(i32, i32, Decimal, Decimal), Error> {
    let stats: (i64, i64, Option<Decimal>, Option<Decimal>) = sqlx::query_as(
        r#"
        SELECT 
            COUNT(*) as total_referrals,
            COUNT(CASE WHEN status = 'paid' THEN 1 END) as paid_referrals,
            COALESCE(SUM(total_consumption), 0) as total_consumption,
            COALESCE(SUM(last_30_days_consumption), 0) as last_30_days_consumption
        FROM user_referrals
        WHERE inviter_uuid = $1
        "#,
    )
    .bind(user_uuid)
    .fetch_one(pool)
    .await?;

    Ok((
        stats.0 as i32,
        stats.1 as i32,
        stats.2.unwrap_or(Decimal::ZERO),
        stats.3.unwrap_or(Decimal::ZERO),
    ))
}

// ============ Referral Rewards ============

/// 查询奖励记录
pub async fn fetch_referral_rewards(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    keyword: Option<&str>,
    reward_type: Option<&str>,
    status: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<ReferralRewardDto>, Error> {
    let recs = sqlx::query_as::<_, ReferralRewardDto>(
        r#"
        SELECT id, uuid, user_uuid, reward_type, points, description,
               referred_user_uuid, link_uuid, status, created_at
        FROM referral_rewards
        WHERE user_uuid = $1
          AND ($2::text IS NULL OR COALESCE(description, '') ILIKE '%' || $2 || '%')
          AND ($3::varchar IS NULL OR reward_type = $3)
          AND ($4::varchar IS NULL OR status = $4)
        ORDER BY created_at DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(user_uuid)
    .bind(keyword)
    .bind(reward_type)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询奖励记录总数
pub async fn fetch_referral_rewards_count(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    keyword: Option<&str>,
    reward_type: Option<&str>,
    status: Option<&str>,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM referral_rewards
        WHERE user_uuid = $1
          AND ($2::text IS NULL OR COALESCE(description, '') ILIKE '%' || $2 || '%')
          AND ($3::varchar IS NULL OR reward_type = $3)
          AND ($4::varchar IS NULL OR status = $4)
        "#,
    )
    .bind(user_uuid)
    .bind(keyword)
    .bind(reward_type)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// 获取总奖励积分
pub async fn fetch_total_reward_points(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<i32, Error> {
    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(points), 0) FROM referral_rewards
        WHERE user_uuid = $1 AND status = 'completed'
        "#,
    )
    .bind(user_uuid)
    .fetch_one(pool)
    .await?;

    Ok(total as i32)
}

// ============ User Referral Points ============

/// 获取用户积分
pub async fn fetch_user_points(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<Option<UserReferralPointsDto>, Error> {
    let rec = sqlx::query_as::<_, UserReferralPointsDto>(
        r#"
        SELECT id, user_uuid, total_points, available_points, used_points,
               pending_points, created_at, updated_at
        FROM user_referral_points
        WHERE user_uuid = $1
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 创建或更新用户积分
pub async fn upsert_user_points(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    total_points: i32,
    available_points: i32,
    used_points: i32,
    pending_points: i32,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        INSERT INTO user_referral_points (user_uuid, total_points, available_points, used_points, pending_points)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (user_uuid) DO UPDATE SET
            total_points = $2,
            available_points = $3,
            used_points = $4,
            pending_points = $5,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(user_uuid)
    .bind(total_points)
    .bind(available_points)
    .bind(used_points)
    .bind(pending_points)
    .execute(pool)
    .await?;

    Ok(())
}

/// 扣减积分
pub async fn deduct_user_points(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    points: i32,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        UPDATE user_referral_points
        SET available_points = available_points - $1,
            used_points = used_points + $1,
            updated_at = CURRENT_TIMESTAMP
        WHERE user_uuid = $2 AND available_points >= $1
        "#,
    )
    .bind(points)
    .bind(user_uuid)
    .execute(pool)
    .await?;

    Ok(())
}

// ============ Redeem Options ============

/// 查询兑换选项
pub async fn fetch_redeem_options(pool: &Pool<Postgres>) -> Result<Vec<RedeemOptionDto>, Error> {
    let recs = sqlx::query_as::<_, RedeemOptionDto>(
        r#"
        SELECT id, uuid, redeem_type, name, description, points_required,
               value, currency, exchange_rate, status, sort_order, created_at, updated_at
        FROM redeem_options
        WHERE status = 'active'
        ORDER BY sort_order ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 根据 UUID 查询兑换选项
pub async fn fetch_redeem_option_by_uuid(
    pool: &Pool<Postgres>,
    option_uuid: Uuid,
) -> Result<Option<RedeemOptionDto>, Error> {
    let rec = sqlx::query_as::<_, RedeemOptionDto>(
        r#"
        SELECT id, uuid, redeem_type, name, description, points_required,
               value, currency, exchange_rate, status, sort_order, created_at, updated_at
        FROM redeem_options
        WHERE uuid = $1
        "#,
    )
    .bind(option_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

// ============ Redeem Records ============

/// 创建兑换记录
pub async fn insert_redeem_record(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    option_uuid: Uuid,
    points_used: i32,
    value: Decimal,
    currency: Option<&str>,
) -> Result<Uuid, Error> {
    let uuid: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO redeem_records (user_uuid, option_uuid, points_used, value, currency, status)
        VALUES ($1, $2, $3, $4, $5, 'completed')
        RETURNING uuid;
        "#,
    )
    .bind(user_uuid)
    .bind(option_uuid)
    .bind(points_used)
    .bind(value)
    .bind(currency)
    .fetch_one(pool)
    .await?;

    Ok(uuid)
}

/// 查询兑换记录
pub async fn fetch_redeem_records(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    offset: i64,
    limit: i64,
) -> Result<Vec<RedeemRecordDto>, Error> {
    let recs = sqlx::query_as::<_, RedeemRecordDto>(
        r#"
        SELECT id, uuid, user_uuid, option_uuid, points_used, value,
               currency, status, created_at, completed_at
        FROM redeem_records
        WHERE user_uuid = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_uuid)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(recs)
}

/// 查询兑换记录总数
pub async fn fetch_redeem_records_count(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<i64, Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM redeem_records WHERE user_uuid = $1
        "#,
    )
    .bind(user_uuid)
    .fetch_one(pool)
    .await?;

    Ok(count)
}
