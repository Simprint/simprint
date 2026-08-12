use thiserror::Error;

/// Simprint Server 错误类型
#[derive(Error, Debug)]
pub enum SimprintError {
    /// 用户不存在
    #[error("用户不存在")]
    UserNotFound,

    /// 邮箱已被注册
    #[error("邮箱已被注册")]
    EmailAlreadyExists,

    /// 邮箱或密码错误
    #[error("邮箱或密码错误")]
    InvalidCredentials,

    /// 用户已被禁用
    #[error("用户已被禁用")]
    UserDisabled,

    /// 验证码错误或已过期
    #[error("验证码错误或已过期")]
    VerificationCodeExpired,

    /// 机器不存在
    #[error("机器不存在")]
    MachineNotFound,

    /// 机器已被绑定
    #[error("机器已被绑定")]
    MachineAlreadyBound,

    /// 用户未绑定到机器
    #[error("用户未绑定到机器")]
    MachineNotBound,

    /// 版本不存在
    #[error("版本不存在")]
    VersionNotFound,

    /// 版本已存在
    #[error("版本已存在")]
    VersionAlreadyExists,

    /// 版本类型不存在
    #[error("版本类型不存在")]
    VersionTypeNotFound,

    /// 版本号为空
    #[error("版本号不能为空")]
    VersionEmpty,

    /// 资源名称为空
    #[error("资源名称不能为空")]
    ResourceNameEmpty,

    /// 灰度发布不存在
    #[error("灰度发布不存在")]
    GrayReleaseNotFound,

    /// 灰度分配失败
    #[error("灰度分配失败")]
    GrayAllocationFailed,

    /// 策略类型不存在
    #[error("策略类型不存在")]
    StrategyTypeNotFound,

    /// 策略配置无效
    #[error("策略配置无效")]
    InvalidStrategyConfig,

    /// 维护不存在
    #[error("维护不存在")]
    MaintenanceNotFound,

    /// 数据库操作失败
    #[error("数据库操作失败: {0}")]
    DatabaseError(#[from] sqlx::Error),

    /// Anyhow错误
    #[error("操作失败: {0}")]
    AnyhowError(#[from] anyhow::Error),

    /// JSON序列化错误
    #[error("JSON序列化错误: {0}")]
    JsonError(#[from] serde_json::Error),

    /// 错误的请求
    #[error("错误的请求: {0}")]
    InvalidRequest(String),

    /// 其他错误
    #[error("{0}")]
    Other(String),
}

impl From<&str> for SimprintError {
    fn from(err: &str) -> Self {
        SimprintError::Other(err.to_string())
    }
}

// Note: Error conversion to Response is handled in handlers layer via map_err
