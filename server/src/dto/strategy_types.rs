use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub mod processor_config {
    use super::*;

    /// 白名单过滤配置
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub struct FilterWhitelistConfig {
        pub machines: Vec<String>, // 机器码列表
    }

    /// 百分比过滤配置
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub struct FilterPercentageConfig {
        pub percent: f64, // 百分比 (0-100)
    }

    /// 随机过滤配置
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub struct FilterRandomConfig {
        pub count: usize, // 随机选择的数量
    }

    /// 用户标签过滤配置
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub struct FilterUserTagConfig {
        pub tags: Vec<String>, // 标签列表
        pub match_all: bool,   // true: 必须包含所有标签, false: 包含任意一个标签
    }

    /// 时间段转换配置
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub struct TransformTimeRangeConfig {
        pub start_hour: u8, // 开始小时 (0-23)
        pub end_hour: u8,   // 结束小时 (0-23)
        pub days: Vec<u8>,  // 生效星期 (0-6, 0=周日)
    }

    /// 延迟分配配置
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub struct TransformDelayConfig {
        pub delay_seconds: u64, // 延迟秒数
    }

    /// 分步分配配置
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub struct TransformStepConfig {
        pub step_percentage: f64,     // 每一步的百分比
        pub step_interval_hours: u32, // 每步间隔（小时）
    }
}

/// 处理器类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessorType {
    FilterWhitelist(processor_config::FilterWhitelistConfig),
    FilterPercentage(processor_config::FilterPercentageConfig),
    FilterRandom(processor_config::FilterRandomConfig),
    FilterUserTag(processor_config::FilterUserTagConfig),
    TransformTimeRange(processor_config::TransformTimeRangeConfig),
    TransformDelay(processor_config::TransformDelayConfig),
    TransformStep(processor_config::TransformStepConfig),
}

impl ProcessorType {
    /// 获取处理器类型名称
    pub fn type_name(&self) -> &'static str {
        match self {
            ProcessorType::FilterWhitelist(_) => "filter_whitelist",
            ProcessorType::FilterPercentage(_) => "filter_percentage",
            ProcessorType::FilterRandom(_) => "filter_random",
            ProcessorType::FilterUserTag(_) => "filter_user_tag",
            ProcessorType::TransformTimeRange(_) => "transform_time_range",
            ProcessorType::TransformDelay(_) => "transform_delay",
            ProcessorType::TransformStep(_) => "transform_step",
        }
    }
}

/// 策略类型
#[derive(Debug, Deserialize, Serialize, FromRow, Clone)]
pub struct StrategyType {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub processor_type: String,
    pub config_schema: Option<String>,
    pub is_active: bool,
}
