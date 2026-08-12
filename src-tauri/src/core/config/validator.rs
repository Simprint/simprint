//! 配置验证
//!
//! 验证配置的有效性

use super::types::AppConfig;
use crate::core::error::{Error, Result};

/// 验证配置
pub fn validate(config: &AppConfig) -> Result<()> {
    validate_updater_config(config)?;
    Ok(())
}

/// 验证更新器配置
fn validate_updater_config(config: &AppConfig) -> Result<()> {
    if config.updater.check_url.is_empty() {
        return Err(Error::ConfigValidationFailed(
            "updater.check_url cannot be empty".to_string(),
        ));
    }

    if !config.updater.check_url.starts_with("http://")
        && !config.updater.check_url.starts_with("https://")
    {
        return Err(Error::ConfigValidationFailed(
            "updater.check_url must start with http:// or https://".to_string(),
        ));
    }

    if config.updater.latest_json_url.is_empty() {
        return Err(Error::ConfigValidationFailed(
            "updater.latest_json_url cannot be empty".to_string(),
        ));
    }

    if !config.updater.latest_json_url.starts_with("http://")
        && !config.updater.latest_json_url.starts_with("https://")
    {
        return Err(Error::ConfigValidationFailed(
            "updater.latest_json_url must start with http:// or https://".to_string(),
        ));
    }

    Ok(())
}
