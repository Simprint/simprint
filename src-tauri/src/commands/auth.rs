use business::services::local_users::{CreateLocalUserRequest, LocalUser, LoginLocalUserRequest};

use crate::app::context::AppContext;

#[tauri::command]
pub async fn list_local_users(
    context: tauri::State<'_, business::svc_ctx::SvcCtx>,
) -> Result<Vec<LocalUser>, String> {
    business::services::local_users::list_local_users(&context).await
}

#[tauri::command]
pub async fn create_local_user(
    payload: CreateLocalUserRequest,
    context: tauri::State<'_, business::svc_ctx::SvcCtx>,
) -> Result<LocalUser, String> {
    let user = business::services::local_users::create_local_user(&context, &payload).await?;
    context.authenticate_user(user.uuid);
    sync_local_session().await?;
    Ok(user)
}

#[tauri::command]
pub async fn login_local_user(
    payload: LoginLocalUserRequest,
    context: tauri::State<'_, business::svc_ctx::SvcCtx>,
) -> Result<LocalUser, String> {
    let user = business::services::local_users::authenticate_local_user(&context, &payload).await?;
    sync_local_session().await?;
    Ok(user)
}

#[tauri::command]
pub async fn get_current_local_user(
    context: tauri::State<'_, business::svc_ctx::SvcCtx>,
) -> Result<Option<LocalUser>, String> {
    business::services::local_users::current_local_user(&context).await
}

#[tauri::command]
pub async fn verify_local_user_password(
    password: Option<String>,
    context: tauri::State<'_, business::svc_ctx::SvcCtx>,
) -> Result<(), String> {
    let user_uuid = context.current_user_uuid().ok_or_else(|| "尚未选择本地用户".to_string())?;
    business::services::local_users::verify_local_user_password(
        &context,
        user_uuid,
        password.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn logout(context: tauri::State<'_, business::svc_ctx::SvcCtx>) -> Result<(), String> {
    context.clear_authenticated_user();
    if let Some(app_context) = AppContext::try_get() {
        app_context.mcp_manager.stop().await;
        app_context.local_api_manager.stop().await;
        app_context.simprint_runtime_manager.stop().await;
    }
    Ok(())
}

#[tauri::command]
pub fn is_logged_in(context: tauri::State<'_, business::svc_ctx::SvcCtx>) -> bool {
    context.current_user_uuid().is_some()
}

async fn sync_local_session() -> Result<(), String> {
    if let Some(context) = AppContext::try_get() {
        context
            .simprint_runtime_manager
            .sync_session_state()
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(crate) fn authenticated_user_uuid() -> Option<uuid::Uuid> {
    use tauri::Manager;
    let app = crate::app::handle::get_app_handle().ok()?;
    app.state::<business::svc_ctx::SvcCtx>().current_user_uuid()
}

pub(crate) fn has_local_session() -> bool {
    authenticated_user_uuid().is_some()
}
