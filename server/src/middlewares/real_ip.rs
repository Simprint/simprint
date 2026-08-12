use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::state::{CurrentIpAddr, RequestContext};

pub async fn real_ip(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // 验证来源
    let client_ip_opt = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip());

    // 检查ip地址是否是通过nginx转发(通过nginx就是127.0.0.1)和是否是本地地址
    if let Some(client_ip) = client_ip_opt {
        let client_ip_str = client_ip.to_string();
        if &client_ip_str != "127.0.0.1" && !client_ip.is_loopback() {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // 提取客户端的真实IP
    let real_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .map(str::to_string)
        .or_else(|| {
            req.headers().get("x-real-ip").and_then(|v| v.to_str().ok()).map(str::to_string)
        });

    // 初始化 RequestContext 并设置 IP
    let mut ctx = RequestContext::default();
    if let Some(ip) = real_ip {
        ctx.current_ip_addr = Some(CurrentIpAddr { real_ip: ip });
    }
    req.extensions_mut().insert(ctx);

    Ok(next.run(req).await)
}
