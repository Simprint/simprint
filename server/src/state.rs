use uuid::Uuid;

/// 当前用户
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_uuid: Uuid,
}

/// 当前工作空间
#[derive(Debug, Clone)]
pub struct CurrentWorkspace {
    pub workspace_uuid: Uuid,
}

/// 当前 IP 地址
#[derive(Debug, Clone)]
pub struct CurrentIpAddr {
    pub real_ip: String,
}

/// 请求上下文 - 包含所有请求相关的上下文信息
///
/// 在中间件中逐步填充，handler 中通过 Extension 获取
#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    /// 当前用户信息
    pub current_user: Option<CurrentUser>,
    /// 当前 IP 地址
    pub current_ip_addr: Option<CurrentIpAddr>,
    /// 当前团队 UUID
    pub current_team_uuid: Option<Uuid>,
    /// 当前工作空间 UUID
    pub current_workspace_uuid: Option<Uuid>,
    /// 资源标识符：method+path（去除 /api/v*/ 前缀，保留前导斜杠）
    /// 例如：POST+/environments, GET+/proxies
    pub resource_identifier: Option<String>,
}

impl RequestContext {
    /// 获取用户 UUID（如果已认证）
    pub fn user_uuid(&self) -> Option<Uuid> {
        self.current_user.as_ref().map(|u| u.user_uuid)
    }

    /// 获取用户 UUID，如果未认证则 panic
    pub fn user_uuid_unwrap(&self) -> Uuid {
        self.current_user.as_ref().expect("用户未认证").user_uuid
    }

    /// 获取 IP 地址
    pub fn ip(&self) -> Option<&str> {
        self.current_ip_addr.as_ref().map(|i| i.real_ip.as_str())
    }

    /// 获取 IP 地址，如果不存在则返回 "unknown"
    pub fn ip_or_unknown(&self) -> &str {
        self.ip().unwrap_or("unknown")
    }

    /// 获取工作空间 UUID
    pub fn workspace_uuid(&self) -> Option<Uuid> {
        self.current_workspace_uuid
    }

    /// 获取工作空间 UUID，如果不存在则 panic
    pub fn workspace_uuid_unwrap(&self) -> Uuid {
        self.current_workspace_uuid.expect("工作空间未设置")
    }

    /// 获取资源标识符
    pub fn resource_identifier(&self) -> Option<&str> {
        self.resource_identifier.as_deref()
    }

    /// 获取资源路径（从资源标识符中提取路径部分）
    /// 例如：POST+/environments -> /environments
    pub fn resource_path(&self) -> Option<&str> {
        self.resource_identifier
            .as_ref()
            .and_then(|id| id.split_once('+').map(|(_, path)| path))
    }
}
