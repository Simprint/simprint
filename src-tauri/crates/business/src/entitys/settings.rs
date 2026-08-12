use serde::{Deserialize, Serialize};

/// 更新用户偏好请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdatePreferencesRequest {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub notifications_enabled: Option<bool>,
}

/// 更新用户信息请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateProfileRequest {
    pub nickname: Option<String>,
    pub avatar_hash: Option<String>,
}

/// 修改密码请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// 查询登录历史请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListLoginHistoryRequest {
    pub page: i32,
    pub page_size: i32,
}
