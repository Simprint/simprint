use uuid::Uuid;

use crate::dto::UserPreferenceDto;
use crate::entitys::settings::UpdatePreferencesRequest;
use crate::models;
use crate::svc_ctx::SvcCtx;

/// 获取用户偏好设置
pub async fn get_preferences_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
) -> Result<UserPreferenceDto, String> {
    let preferences = models::preferences::fetch_user_preferences(&svc_ctx.db, user_uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 如果不存在，创建默认设置
    if preferences.is_none() {
        models::preferences::upsert_user_preferences(&svc_ctx.db, user_uuid, None, None, None)
            .await
            .map_err(|e| e.to_string())?;

        return models::preferences::fetch_user_preferences(&svc_ctx.db, user_uuid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "创建偏好设置失败".to_string());
    }

    preferences.ok_or_else(|| "偏好设置不存在".to_string())
}

/// 更新用户偏好设置
pub async fn update_preferences_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    payload: &UpdatePreferencesRequest,
) -> Result<UserPreferenceDto, String> {
    models::preferences::upsert_user_preferences(
        &svc_ctx.db,
        user_uuid,
        payload.theme.as_deref(),
        payload.language.as_deref(),
        payload.notifications_enabled,
    )
    .await
    .map_err(|e| e.to_string())?;

    get_preferences_service(svc_ctx, user_uuid).await
}
