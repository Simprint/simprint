use crate::utils::IConfig;

pub mod app;
pub mod caches;
pub mod cli;
pub mod database;
pub mod dto;
pub mod entitys;
pub mod errors;
pub mod handlers;
pub mod middlewares;
pub mod models;
pub mod routes;
pub mod services;
pub mod state;
pub mod svc_ctx;
pub mod utils;

pub use app::{serve, serve_on, serve_on_with_shutdown};

/// Initialize the response-encryption key used by the transitional HTTP API.
pub async fn init_encrypt_secret(config: &IConfig) {
    let key_path = &config.app.encrypt_secret_location;
    utils::init_rsa_secret(key_path).await;
}
