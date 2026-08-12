use serde_json::json;
use tauri::Manager;

use crate::core::error::Result;

use super::types::EnvironmentLaunchDetail;

pub(super) async fn get_environment_launch_detail(
    env_uuid: &str,
) -> Result<EnvironmentLaunchDetail> {
    let app = crate::app::handle::get_app_handle()?;
    let context = app.state::<business::svc_ctx::SvcCtx>();
    let request_context = context.for_current_user()?;
    let data = business::dispatcher::dispatch_post(
        &request_context,
        "environments/detail",
        &json!({ "uuid": env_uuid }),
    )
    .await
    .ok_or("本地环境详情路由不存在")?
    .map_err(crate::core::error::Error::from)?;

    serde_json::from_value(data).map_err(Into::into)
}
