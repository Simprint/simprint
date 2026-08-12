use serde_json::Value;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::{
    core::error::Result,
    domain::environment::KernelDetail,
    services::environment::{KernelService, KernelStatusEmitter},
};

use super::types::EnvironmentLaunchDetail;

pub(super) struct ResolvedKernelLaunch {
    pub exe_path: String,
}

pub(super) async fn resolve_kernel_launch(
    app: AppHandle,
    detail: &EnvironmentLaunchDetail,
    profiles_path: &str,
    status_emitter: Option<KernelStatusEmitter>,
) -> Result<ResolvedKernelLaunch> {
    let env = detail
        .environment
        .as_ref()
        .ok_or("Environment detail is missing environment uuid.")?;
    let environment_uuid = Uuid::parse_str(&env.uuid)
        .map_err(|error| format!("Invalid environment uuid {}: {error}", env.uuid))?;
    let context = app.state::<business::svc_ctx::SvcCtx>();
    let kernel_detail =
        business::services::browser_kernels::get_environment_kernel(&context.db, environment_uuid)
            .await?
            .ok_or("This environment has no browser kernel binding.")?;
    let kernel_id = kernel_detail.kernel_id.clone();
    let install_dir_name = kernel_detail.install_dir_name.clone();

    let url = kernel_detail
        .url
        .filter(|value| !value.trim().is_empty())
        .ok_or("The selected browser kernel is missing download metadata.")?;
    let hash = kernel_detail.hash;
    let signature = kernel_detail.signature;
    let compatible_signatures = kernel_detail.compatible_signatures.0;

    let exe_path = KernelService::ensure_kernel_ready_for_artifact(
        app,
        Some(env.uuid.clone()),
        kernel_id,
        install_dir_name,
        profiles_path.to_string(),
        KernelDetail {
            url,
            hash,
            signature: Some(signature),
            compatible_signatures,
            requires_extract: kernel_detail.requires_extract,
        },
        status_emitter,
    )
    .await?;

    Ok(ResolvedKernelLaunch { exe_path })
}

pub(super) fn get_window_info(config: Option<&Value>) -> serde_json::Map<String, Value> {
    config
        .and_then(|config| config.get("window_info"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}
