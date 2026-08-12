/// 网络模块命令
///
/// 命令层仅负责参数解析和响应，业务逻辑由服务层处理
use crate::core::error::Result;
use crate::infrastructure::http::client::JsonRespnse;
use crate::services::connectivity::{DownloadService, ProxyService};
use serde_json::Value;
use std::collections::HashMap;

// 重导出类型供外部使用
pub use crate::infrastructure::proxy::{IpInfo, ProxyConfig, ProxyTestResult};

// ============================================================================
// HTTP 请求命令
// ============================================================================

/// 将业务请求分发到内嵌数据库。未知路由直接在本地失败，不再回退到远程服务。
#[tauri::command]
pub async fn http_post(
    url: String,
    data: Option<Value>,
    business_context: tauri::State<'_, business::svc_ctx::SvcCtx>,
) -> std::result::Result<JsonRespnse, String> {
    let payload = data.unwrap_or_else(|| Value::Object(Default::default()));
    let request_context = business_context.for_current_user().map_err(|error| error.to_string())?;
    if let Some(result) =
        business::dispatcher::dispatch_post(&request_context, &url, &payload).await
    {
        return Ok(match result {
            Ok(data) => JsonRespnse {
                code: Some(1),
                message: Some("OK".to_string()),
                data: (!data.is_null()).then_some(data),
            },
            Err(message) => JsonRespnse {
                code: Some(-1),
                message: Some(message),
                data: None,
            },
        });
    }

    Err(format!("本地业务路由不存在: {url}"))
}

// ============================================================================
// 代理测试命令
// ============================================================================

/// 测试代理连接
#[tauri::command]
pub async fn test_proxy(config: ProxyConfig) -> Result<ProxyTestResult> {
    ProxyService::test_proxy(config).await
}

/// 测试直连 IP（不使用代理）
#[tauri::command]
pub async fn test_direct_ip() -> Result<ProxyTestResult> {
    ProxyService::test_direct_ip().await
}

/// 检测代理 IP 信息（包含地理位置）
#[tauri::command]
pub async fn detect_proxy_ip(config: ProxyConfig) -> Result<Option<IpInfo>> {
    let result = ProxyService::test_proxy(config).await?;
    Ok(result.ip_info)
}

// ============================================================================
// 文件下载命令
// ============================================================================

/// 下载多个文件
#[tauri::command]
pub async fn download_files(
    urls: Vec<String>,
    save_paths: Vec<String>,
    max_retries: Option<u32>,
    retry_delay_ms: Option<u64>,
) -> Result<HashMap<String, bool>> {
    DownloadService::download_files(urls, save_paths, max_retries, retry_delay_ms).await
}
