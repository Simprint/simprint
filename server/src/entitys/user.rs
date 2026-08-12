use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 注册请求
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>, // 昵称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_secret_key: Option<String>, // 用户公钥（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referral_code: Option<String>, // 推荐码（可选）
}

/// 基本登录请求（邮箱 + 密码）
#[derive(Debug, Deserialize, Serialize)]
pub struct BasicLoginData {
    pub email: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_secret_key: Option<String>,
}

/// 记住密码登录请求（邮箱 + refresh_token）
#[derive(Debug, Deserialize, Serialize)]
pub struct RememberLoginData {
    pub email: String,
    pub refresh_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_secret_key: Option<String>,
}

/// 登录请求（统一结构，通过枚举区分两种登录方式）
///
/// 使用 serde 的 tag 特性，根据 "login_type" 字段自动反序列化为对应的变体
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "login_type", rename_all = "snake_case")]
pub enum LoginRequest {
    /// 基本登录（邮箱 + 密码）
    Basic(BasicLoginData),
    /// 记住密码登录（邮箱 + refresh_token）
    Remember(RememberLoginData),
}

/// 刷新 Token 请求
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// 更新用户信息请求
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct UpdateUserRequest {
    pub nickname: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// 修改密码请求
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct UpdatePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// 校验当前用户密码请求
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct VerifyPasswordRequest {
    pub password: String,
}

/// 重置密码请求
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub code: String,
    pub new_password: String,
}

/// 发送验证码请求
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SendCodeRequest {
    pub email: String,
    pub r#type: String, // register 或 reset_password
}

/// 用户信息响应
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub uuid: Uuid,
    pub id: String,
    pub nickname: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub avatar_hash: Option<String>,
    /// 头像完整 URL（当 avatar_hash 存在时由服务端拼接）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 当前团队详细信息（如果存在）
    pub current_team: Option<crate::dto::TeamDto>,
    /// 当前工作空间详细信息（如果存在）
    pub current_workspace: Option<crate::dto::WorkspaceDto>,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_info: Option<UserResponse>,
}

/// 注册响应
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_info: Option<UserResponse>,
}

/// 密码校验响应
#[derive(Debug, Serialize)]
pub struct VerifyPasswordResponse {
    pub valid: bool,
}
