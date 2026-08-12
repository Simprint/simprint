use chrono::Utc;
use serde_json::{Value, json};

use crate::utils::{Response, Result};

/// 健康检查
pub async fn health_check_handler() -> Result<Value> {
    Ok(Response::success(
        Some("服务正常"),
        Some(json!({
            "status": "ok",
            "timestamp": Utc::now().to_rfc3339(),
        })),
    ))
}
