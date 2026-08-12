use uuid::Uuid;

use crate::dto::TagDto;
use crate::entitys::tags::{CreateTagRequest, UpdateTagRequest};
use crate::models;
use crate::svc_ctx::SvcCtx;

/// 创建标签
pub async fn create_tag_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
    payload: &CreateTagRequest,
) -> Result<Uuid, String> {
    models::insert_tag(
        &svc_ctx.db,
        user_uuid,
        team_uuid,
        &payload.name,
        payload.color.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// 获取标签列表
pub async fn get_tags_service(
    svc_ctx: &SvcCtx,
    user_uuid: Uuid,
    team_uuid: Option<Uuid>,
) -> Result<Vec<TagDto>, String> {
    models::fetch_tags(&svc_ctx.db, team_uuid, user_uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 获取标签详情
pub async fn get_tag_service(svc_ctx: &SvcCtx, tag_uuid: Uuid) -> Result<TagDto, String> {
    models::fetch_tag_by_uuid(&svc_ctx.db, tag_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "标签不存在".to_string())
}

/// 更新标签
pub async fn update_tag_service(
    svc_ctx: &SvcCtx,
    payload: &UpdateTagRequest,
) -> Result<(), String> {
    models::update_tag(
        &svc_ctx.db,
        payload.uuid,
        payload.name.as_deref(),
        payload.color.as_deref(),
        payload.sort_order,
    )
    .await
    .map_err(|e| e.to_string())
}

/// 删除标签
pub async fn delete_tag_service(svc_ctx: &SvcCtx, tag_uuid: Uuid) -> Result<(), String> {
    models::delete_tag(&svc_ctx.db, tag_uuid).await.map_err(|e| e.to_string())
}
