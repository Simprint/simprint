// use arcadia_codegen::user_services::GetUserPublicKeyRequest;
// use arcadia_utils::secret::{aes::AesSecret, rsa};

use axum::{
    body,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use serde_json::json;

use crate::{
    services::get_user_public_key_service,
    state::CurrentUser,
    svc_ctx::SvcCtx,
    utils::{AesSecret, get_rsa_secret_instance},
};

/// 加密中间件
pub async fn encrypt(
    State(state): State<SvcCtx>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 从请求中获取 CurrentUser（如果存在）
    let current_user = req.extensions().get::<CurrentUser>().cloned();

    // 获取请求路径和请求方法
    // {
    //     let (resource_method, resource_path) = get_request_path_and_method(&req);
    //     let combine = format!("{}+{}", resource_method, resource_path);
    //     if state.config.resources.whitelists.contains(&combine) {
    //         return Ok(next.run(req).await);
    //     }
    // }

    let response = next.run(req).await;

    if let Some(current_user) = current_user {
        match get_user_public_key_service(&state, &current_user.user_uuid).await {
            Ok(public_key) => {
                // 创建AES密钥实例
                let aes_secret = AesSecret::new();

                // 读取响应体
                let (parts, body) = response.into_parts();

                // 将整个响应体转换为字节
                let body_bytes = match body::to_bytes(body, usize::MAX).await {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        // 无法读取响应体，返回错误
                        tracing::error!("Failed to read response body: {}", err);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                };

                // 解析原始JSON响应
                if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                    // 加密JSON内容
                    if let Ok(encrypted_content) =
                        aes_secret.encrypt(json_value.to_string().as_bytes())
                    {
                        // 获取AES密钥的Base64表示
                        let aes_key_base64 = aes_secret.get_key_as_base64();

                        // 使用用户的公钥加密AES密钥
                        if let Ok(encrypted_key) = get_rsa_secret_instance()
                            .encrypt_with_public_key(aes_key_base64.as_bytes(), &public_key)
                        {
                            // 构建加密响应
                            let secure_response = json!({
                                "data": encrypted_content,
                                "encrypted": true,
                                "key": encrypted_key
                            });

                            // 创建新的响应
                            let mut new_response = axum::Json(secure_response).into_response();

                            // 复制原始响应的状态码
                            *new_response.status_mut() = parts.status;

                            // 添加内容类型头
                            new_response.headers_mut().insert(
                                header::CONTENT_TYPE,
                                header::HeaderValue::from_static("application/json"),
                            );

                            return Ok(new_response);
                        }
                    }
                }

                // 如果加密失败，回退到原始响应
                let fallback = fallback_to_unencrypted_response(body_bytes, parts.status);
                return Ok(fallback);
            }
            Err(_) => {
                return Ok(response);
            }
        }
    }

    Ok(response)
}

// 当加密失败时回退到未加密响应
fn fallback_to_unencrypted_response(body: Bytes, status: StatusCode) -> Response {
    // 尝试解析原始响应为JSON
    let json_value = serde_json::from_slice::<serde_json::Value>(&body)
        .unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&body).to_string()}));

    // 添加未加密标志
    let unencrypted_response = json!({
        "encrypted": false,
        "data": json_value
    });

    let mut response = axum::Json(unencrypted_response).into_response();
    *response.status_mut() = status;

    response
}
