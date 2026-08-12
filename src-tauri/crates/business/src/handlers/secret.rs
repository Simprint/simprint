use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::utils::get_rsa_secret_instance;

/// 获取服务器公钥
///
/// 返回服务器的 RSA 公钥（PEM 格式），用于客户端加密数据
///
/// # Returns
/// 返回纯文本格式的 PEM 公钥字符串
pub async fn get_public_key_handler() -> impl IntoResponse {
    let public_key = get_rsa_secret_instance().get_public_key();
    (StatusCode::OK, public_key).into_response()
}
