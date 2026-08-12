use serde::{Deserialize, Serialize};

/// 根据ID获取策略类型请求
#[derive(Debug, Deserialize, Serialize)]
pub struct GetStrategyTypeByIdRequest {
    pub id: i32,
}

/// 根据代码获取策略类型请求
#[derive(Debug, Deserialize, Serialize)]
pub struct GetStrategyTypeByCodeRequest {
    pub code: String,
}

/// 根据分类查询策略类型请求
#[derive(Debug, Deserialize, Serialize)]
pub struct ListStrategyTypesByCategoryRequest {
    pub category: String,
}

/// 策略类型列表响应
#[derive(Debug, Deserialize, Serialize)]
pub struct StrategyTypeListResponse {
    pub list: Vec<crate::dto::strategy_types::StrategyType>,
}

/// 策略类型详细信息响应（包含配置示例）
#[derive(Debug, Deserialize, Serialize)]
pub struct StrategyTypeDetailResponse {
    pub strategy_type: crate::dto::strategy_types::StrategyType,
    pub config_example: String, // JSON 配置示例
}
