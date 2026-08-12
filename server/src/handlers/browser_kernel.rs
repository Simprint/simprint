use axum::extract::State;
use std::collections::HashMap;

use crate::entitys::browser_kernel::ListBrowserKernelsRequest;
use crate::services::browser_kernel::get_browser_kernel_list_service;
use crate::svc_ctx::SvcCtx;
use crate::utils::{Json, Response, Result};

/// 查询浏览器内核最新版本列表
pub async fn list_browser_kernels_handler(
    State(svc_ctx): State<SvcCtx>,
    Json(payload): Json<ListBrowserKernelsRequest>,
) -> Result<HashMap<String, Vec<crate::dto::versions::Version>>> {
    let kernels = get_browser_kernel_list_service(
        &svc_ctx,
        payload.platform,
        payload.type_code,
    )
        .await
        .map_err(|e| Response::fail(Some(&e.to_string())))?;

    Ok(Response::success(Some("获取浏览器内核列表成功"), Some(kernels)))
}
