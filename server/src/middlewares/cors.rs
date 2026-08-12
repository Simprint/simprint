use axum::http::{
    HeaderName, Method,
    header::{self},
};
use tower_http::cors::CorsLayer;

pub fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            HeaderName::from_static("x-custom-header"),
        ])
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        // allow requests from any origin
        .allow_origin([
            "http://localhost:1420".parse().unwrap(),
            "http://localhost:5173".parse().unwrap(),
            "http://tauri.localhost".parse().unwrap(),
        ])
}
