use crate::{
    dto::strategy_types::StrategyType, entitys::strategy_types::*, errors::SimprintError,
    svc_ctx::SvcCtx,
};

/// 根据ID查询策略类型
pub async fn get_strategy_type_by_id(
    svc_ctx: &SvcCtx,
    id: i32,
) -> Result<StrategyType, SimprintError> {
    let strategy_type = crate::models::strategy_types::query_strategy_type_by_id(&svc_ctx.db, id)
        .await
        .map_err(|_| SimprintError::StrategyTypeNotFound)?;

    Ok(strategy_type)
}

/// 根据code查询策略类型
pub async fn get_strategy_type_by_code(
    svc_ctx: &SvcCtx,
    code: &str,
) -> Result<StrategyType, SimprintError> {
    let strategy_type =
        crate::models::strategy_types::query_strategy_type_by_code(&svc_ctx.db, &code)
            .await
            .map_err(|_| SimprintError::StrategyTypeNotFound)?;

    Ok(strategy_type)
}

/// 查询所有可用策略类型
pub async fn query_available_strategy_types(
    svc_ctx: &SvcCtx,
) -> Result<StrategyTypeListResponse, SimprintError> {
    let list = crate::models::strategy_types::query_available_strategy_types(&svc_ctx.db).await?;

    Ok(StrategyTypeListResponse { list })
}

/// 根据分类查询策略类型
pub async fn query_strategy_types_by_category(
    svc_ctx: &SvcCtx,
    category: &str,
) -> Result<StrategyTypeListResponse, SimprintError> {
    let list =
        crate::models::strategy_types::query_strategy_types_by_category(&svc_ctx.db, &category)
            .await?;

    Ok(StrategyTypeListResponse { list })
}
