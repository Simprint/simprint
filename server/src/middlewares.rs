mod auth;
mod cors;
mod decrypt;
mod encrypt;
mod local_api_auth;
mod logger;
mod real_ip;

pub use auth::*;
pub use cors::*;
pub use decrypt::*;
pub use encrypt::*;
pub use local_api_auth::*;
pub use logger::*;
pub use real_ip::*;

use axum::extract::{MatchedPath, Request};

/// 获取请求路径和请求方法
pub fn get_request_path_and_method(req: &Request) -> (&str, &str) {
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| req.uri().path(), |path| path.as_str());

    let method = req.method().as_str();

    (method, path)
}

/// 从完整路径中提取业务路径（去除 /api/v*/ 前缀，保留前导斜杠）
/// 例如：/api/v1/environments -> /environments
pub fn extract_resource_path(path: &str) -> &str {
    // 匹配 /api/v{数字}/ 格式的前缀
    if let Some(stripped) = path.strip_prefix("/api/v") {
        // 找到第一个 / 后的内容（包含斜杠）
        if let Some(pos) = stripped.find('/') {
            return &stripped[pos..];
        }
    }
    path
}
