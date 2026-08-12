use config::{Config, ConfigError};
use serde::Deserialize;

/// 数据库配置
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub max_lifetime: u64,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
}

/// Redis配置
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

/// 对象存储配置
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub endpoint: String,
    pub public_base_url: String,
    pub access_key: String,
    pub secret_access_key: String,
    pub bucket: String,
    pub avatar_root: String,
    pub extension_root: String,
    pub version_root: String,
}

/// SMTP 配置
#[derive(Debug, Clone, Deserialize)]
pub struct SmtpConfig {
    pub smtp_server: String,
    pub smtp_username: String,
    pub smtp_password: String,
}

/// 工作空间默认配额配置
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceQuotaConfig {
    /// 默认配额（新用户注册和手动创建工作空间都使用此配置）
    #[serde(default = "default_workspace_quota")]
    pub default: WorkspaceQuotaValues,
}

/// 工作空间配额值
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceQuotaValues {
    pub max_environments: i32,
    pub max_team_members: i32,
    pub max_proxies: i32,
    pub max_rpa_tasks: i32,
}

fn default_workspace_quota() -> WorkspaceQuotaValues {
    WorkspaceQuotaValues {
        max_environments: 8,
        max_team_members: 1,
        max_proxies: 99999,
        max_rpa_tasks: 99999,
    }
}

/// 应用配置
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub port: u16,
    pub secret: String,
    pub prefix: String,
    pub encrypt_secret_location: String,
    pub route_whitelists: Vec<String>,
    /// 推广链接前缀，例如: https://www.example.com/register
    /// 实际推广链接将拼接为: {referral_link_prefix}?referral_code={code}
    pub referral_link_prefix: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IConfig {
    pub app: AppConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub storage: StorageConfig,
    pub smtp: Option<SmtpConfig>,
    #[serde(default = "default_workspace_quota_config")]
    pub workspace_quota: WorkspaceQuotaConfig,
}

fn default_workspace_quota_config() -> WorkspaceQuotaConfig {
    WorkspaceQuotaConfig {
        default: default_workspace_quota(),
    }
}

impl IConfig {
    pub fn build_by_filepath(config_path: &str) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(config::File::with_name(config_path))
            .add_source(config::File::with_name(".").required(false))
            .add_source(config::Environment::with_prefix("APP"))
            .build()?;

        config.try_deserialize()
    }
}
