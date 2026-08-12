use axum::{
    body::{self, Body},
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::{
    svc_ctx::SvcCtx,
    utils::{self, AesSecret, get_rsa_secret_instance},
};

/// 被加密的请求的结构体
#[derive(Debug, Deserialize)]
struct EncryptedRequest {
    /// 是否加密
    encrypted: bool,
    /// 加密的内容
    #[serde(default)]
    data: String,
    /// 使用服务器公钥加密后的AES密钥
    #[serde(default)]
    key: String,
}

/// 解密中间件
pub async fn decrypt(
    State(state): State<SvcCtx>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 获取请求路径和请求方法
    {
        // if state
        //     .config
        //     .client_gateway_service_config
        //     .resources
        //     .whitelists
        //     .contains(&combine)
        // {
        //     return Ok(next.run(req).await);
        // }
    }

    // 检查内容是否是json
    if !is_json_content_type(req.headers()) {
        return Ok(next.run(req).await);
    }

    // 读取请求体
    let (parts, body) = req.into_parts();
    let body_bytes = match body::to_bytes(body, 1024 * 1024 * 10).await {
        // 限制请求体大小为10MB
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("读取请求体失败: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match serde_json::from_slice::<EncryptedRequest>(&body_bytes) {
        Ok(encrypted_request) => {
            if !encrypted_request.encrypted {
                // let req = Request::from_parts(parts, Body::from(body_bytes));
                // return Ok(next.run(req).await);
                tracing::error!("请求未加密, 拒绝请求");
                return Err(StatusCode::BAD_REQUEST);
            }

            // 获取服务器RSA实例以解密AES密钥
            let rsa_secret = get_rsa_secret_instance();

            // 解密aes密钥(解密后是base64编码的字符串对应的字节数组)
            // 解密失败后返回正常的json信息并设置状态码为400
            let aes_key = if let Ok(aes_key) = rsa_secret.decrypt(&encrypted_request.key) {
                aes_key
            } else {
                let mut response = utils::Response::<()>::fail(Some("LASDE")).into_response();
                *response.status_mut() = StatusCode::UNPROCESSABLE_ENTITY;
                return Ok(response);
            };

            let aes_key = std::str::from_utf8(&aes_key).map_err(|_| {
                tracing::error!("AES密钥转换为字符串失败");
                StatusCode::BAD_REQUEST
            })?;

            // 使用解密后的AES密钥创建AES实例
            let aes_secret = AesSecret::try_from(aes_key).map_err(|_| {
                tracing::error!("创建AES实例失败");
                StatusCode::BAD_REQUEST
            })?;

            // 使用aes密钥解密请求体
            let decrypted_data = aes_secret.decrypt(&encrypted_request.data).map_err(|e| {
                tracing::error!("解密请求体失败: {:?}", e);
                StatusCode::BAD_REQUEST
            })?;

            // 将解密后的请求体转换为JSON
            let mut decrypted_json_value: serde_json::Value =
                serde_json::from_slice(&decrypted_data).map_err(|_| {
                    tracing::error!("解析解密数据为JSON失败");
                    StatusCode::BAD_REQUEST
                })?;

            // 获取api_secret
            if let Some(api_secret) = decrypted_json_value.get("api_secret") {
                // 检查api_secret是否与配置中的一致
                let app_secret = &state.config.app.secret;
                if api_secret != &serde_json::Value::String(app_secret.clone()) {
                    tracing::error!("API密钥不匹配");
                    return Err(StatusCode::UNAUTHORIZED);
                }

                // 移除api_secret字段
                decrypted_json_value.as_object_mut().and_then(|obj| obj.remove("api_secret"));
            } else {
                tracing::error!("请求中缺少api_secret字段");
                return Err(StatusCode::BAD_REQUEST);
            }

            // 重建请求体
            let json_bytes = serde_json::to_vec(&decrypted_json_value).map_err(|_| {
                tracing::error!("序列化解密数据为JSON失败");
                StatusCode::BAD_REQUEST
            })?;

            // 将解密后的请求体放入请求的扩展中
            let req = Request::from_parts(parts, Body::from(json_bytes));
            Ok(next.run(req).await)
        }
        Err(e) => {
            // 如果解析失败，可能不是加密请求，直接通过
            // let req = Request::from_parts(parts, Body::from(body_bytes));
            // Ok(next.run(req).await)
            tracing::error!("解析加密请求失败: {:?}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    }
}

/// 检查请求头是否为JSON内容类型
fn is_json_content_type(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|content_type| content_type.starts_with("application/json"))
        .unwrap_or(false)
}
