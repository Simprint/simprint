use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

use crate::dto::UserPreferenceDto;

/// 获取用户偏好设置
pub async fn fetch_user_preferences(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
) -> Result<Option<UserPreferenceDto>, Error> {
    let rec = sqlx::query_as::<_, UserPreferenceDto>(
        r#"
        SELECT id, user_uuid, theme, language, notifications_enabled, created_at, updated_at
        FROM user_preferences
        WHERE user_uuid = $1
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(pool)
    .await?;

    Ok(rec)
}

/// 创建或更新用户偏好设置
pub async fn upsert_user_preferences(
    pool: &Pool<Postgres>,
    user_uuid: Uuid,
    theme: Option<&str>,
    language: Option<&str>,
    notifications_enabled: Option<bool>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
        INSERT INTO user_preferences (user_uuid, theme, language, notifications_enabled)
        VALUES ($1, COALESCE($2, 'system'), COALESCE($3, 'zh-CN'), COALESCE($4, true))
        ON CONFLICT (user_uuid) DO UPDATE SET
            theme = COALESCE($2, user_preferences.theme),
            language = COALESCE($3, user_preferences.language),
            notifications_enabled = COALESCE($4, user_preferences.notifications_enabled),
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(user_uuid)
    .bind(theme)
    .bind(language)
    .bind(notifications_enabled)
    .execute(pool)
    .await?;

    Ok(())
}
