use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Pagination;

/// 查询模板列表请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListTemplatesRequest {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub keyword: Option<String>,
    pub is_public: Option<bool>,
}

/// 创建模板请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    /// 完整的环境详情数据（EnvironmentDetailResponse 的 JSON 格式）
    /// 如果提供了 environment_uuid，则此字段会被忽略，后端会自动获取环境详情
    pub environment_data: Option<serde_json::Value>,
    /// 环境 UUID（如果提供，后端会自动获取该环境的完整详情数据）
    pub environment_uuid: Option<Uuid>,
}

/// 更新模板请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateTemplateRequest {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub config_json: Option<serde_json::Value>,
}

/// 应用模板请求（更新现有环境）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApplyTemplateRequest {
    pub template_uuid: Uuid,
    pub environment_uuid: Uuid,
}

/// 从模板创建环境请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateFromTemplateRequest {
    pub template_uuid: Uuid,
    pub name: Option<String>,        // 如果提供，将覆盖模板中的名称
    pub description: Option<String>, // 如果提供，将覆盖模板中的描述
    pub group_uuid: Option<Uuid>,    // 如果提供，将覆盖模板中的分组
}

/// 获取模板详情请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetTemplateRequest {
    pub uuid: Uuid,
    /// 是否用于创建环境（如果为 true，将检查关联数据是否存在）
    pub for_create: Option<bool>,
}

// ========== 响应结构体 ==========

/// 模板列表响应
#[derive(Debug, Clone, Serialize)]
pub struct TemplateListResponse {
    pub items: Vec<crate::dto::TemplateDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 关联数据状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationsStatus {
    /// 分组是否存在
    pub group_exists: bool,
    /// 标签是否存在（按 UUID 映射）
    pub tags_exist: std::collections::HashMap<Uuid, bool>,
    /// 账号是否存在（按 UUID 映射）
    pub accounts_exist: std::collections::HashMap<Uuid, bool>,
    /// 代理是否存在
    pub proxy_exists: bool,
}

/// 模板详情响应（包含关联数据状态）
#[derive(Debug, Clone, Serialize)]
pub struct TemplateDetailResponse {
    /// 模板数据
    #[serde(flatten)]
    pub template: crate::dto::TemplateDto,
    /// 关联数据状态（仅在 for_create=true 时返回）
    pub associations_status: Option<AssociationsStatus>,
}
