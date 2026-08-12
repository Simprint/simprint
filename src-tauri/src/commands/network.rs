/// 网络模块命令
///
/// 命令层仅负责参数解析和响应，业务逻辑由服务层处理
use crate::app::context::AppContext;
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

/// 文件上传信息
#[derive(serde::Deserialize)]
pub struct FileInfo {
    /// 文件路径
    pub path: String,
    /// 文件名（可选，不指定则使用文件路径中的文件名）
    #[serde(default)]
    pub file_name: Option<String>,
    /// MIME 类型（可选，不指定则自动推断）
    #[serde(default)]
    pub mime_type: Option<String>,
}

/// HTTP GET 请求
#[tauri::command]
pub async fn http_get(url: String) -> std::result::Result<JsonRespnse, String> {
    let ctx = AppContext::get();
    ctx.main_server_client.get(&url).await.map_err(|e| e.to_string())
}

/// HTTP POST 请求
#[tauri::command]
pub async fn http_post(
    url: String,
    data: Option<Value>,
    business_context: tauri::State<'_, business::svc_ctx::SvcCtx>,
) -> std::result::Result<JsonRespnse, String> {
    let payload = data.unwrap_or_else(|| Value::Object(Default::default()));
    if let Some(result) =
        business::dispatcher::dispatch_post(&business_context, &url, &payload).await
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

    let ctx = AppContext::get();
    ctx.main_server_client
        .post(&url, &Some(payload))
        .await
        .map_err(|e| e.to_string())
}

/// HTTP POST 表单请求（用于文件上传）
#[tauri::command]
pub async fn http_post_form(
    url: String,
    files: Option<HashMap<String, Vec<FileInfo>>>,
    fields: Option<HashMap<String, String>>,
) -> std::result::Result<JsonRespnse, String> {
    use reqwest::multipart::{Form, Part};
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;

    let mut form = Form::new();

    // 添加文件字段
    if let Some(files_map) = files {
        for (field_name, file_list) in files_map {
            for file_info in file_list {
                let mut file = File::open(&file_info.path).await.map_err(|e| e.to_string())?;

                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).await.map_err(|e| e.to_string())?;

                // 确定文件名
                let file_name = file_info.file_name.clone().unwrap_or_else(|| {
                    std::path::Path::new(&file_info.path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file")
                        .to_string()
                });

                // 创建 Part
                let mut part = Part::bytes(buffer).file_name(file_name);

                // 设置 MIME 类型
                if let Some(ref mime) = file_info.mime_type {
                    part = part.mime_str(mime).map_err(|e| e.to_string())?;
                }

                form = form.part(field_name.clone(), part);
            }
        }
    }

    // 添加文本字段
    if let Some(fields_map) = fields {
        for (key, value) in fields_map {
            form = form.text(key, value);
        }
    }

    let ctx = AppContext::get();
    ctx.main_server_client.post_form(&url, form).await.map_err(|e| e.to_string())
}

/// HTTP PUT 请求
#[tauri::command]
pub async fn http_put(
    url: String,
    data: Option<Value>,
) -> std::result::Result<JsonRespnse, String> {
    let ctx = AppContext::get();
    ctx.main_server_client.put(&url, &data).await.map_err(|e| e.to_string())
}

/// HTTP DELETE 请求
#[tauri::command]
pub async fn http_delete(url: String, data: Value) -> std::result::Result<JsonRespnse, String> {
    let ctx = AppContext::get();
    ctx.main_server_client.delete(&url, &data).await.map_err(|e| e.to_string())
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
