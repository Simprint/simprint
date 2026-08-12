use crate::app::context::AppContext;
use crate::core::error::Result;

#[tauri::command]
pub fn get_local_api_runtime_running() -> bool {
    AppContext::get().local_api_manager.is_running()
}

#[tauri::command]
pub async fn start_local_api_runtime() -> Result<()> {
    let ctx = AppContext::get();
    let app = crate::app::handle::get_app_handle()?;
    use tauri::Manager;
    let business_context = app.state::<business::svc_ctx::SvcCtx>();
    ctx.local_api_manager.refresh(&business_context).await?;
    Ok(())
}

#[tauri::command]
pub async fn reload_local_api_runtime() -> Result<()> {
    let ctx = AppContext::get();
    let app = crate::app::handle::get_app_handle()?;
    use tauri::Manager;
    let business_context = app.state::<business::svc_ctx::SvcCtx>();
    ctx.local_api_manager.refresh(&business_context).await?;
    Ok(())
}

#[tauri::command]
pub async fn stop_local_api_runtime() -> Result<()> {
    let ctx = AppContext::get();
    ctx.local_api_manager.stop().await;
    Ok(())
}
