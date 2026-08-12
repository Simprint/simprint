//! 环境/内核领域模型
//!
//! 封装浏览器环境和内核相关的业务规则

use serde::{Deserialize, Serialize};

/// 环境状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentStatus {
    /// 验证中
    Verifying,
    /// 下载中
    Downloading,
    /// 解压中
    Extracting,
    /// 就绪
    Ready,
    /// 初始化中
    Initializing,
    /// 启动中
    Starting,
    /// 运行中
    Running,
    /// 关闭中
    Stopping,
    /// 已停止
    Stopped,
    /// 错误
    Error,
}

impl EnvironmentStatus {
    /// 是否可以启动
    pub fn can_launch(&self) -> bool {
        matches!(self, EnvironmentStatus::Ready | EnvironmentStatus::Stopped)
    }

    /// 是否正在运行
    pub fn is_running(&self) -> bool {
        matches!(self, EnvironmentStatus::Running)
    }

    /// 是否处于准备阶段
    pub fn is_preparing(&self) -> bool {
        matches!(
            self,
            EnvironmentStatus::Verifying
                | EnvironmentStatus::Downloading
                | EnvironmentStatus::Extracting
        )
    }

    /// 是否处于过渡状态（初始化中、启动中或关闭中）
    pub fn is_transitioning(&self) -> bool {
        matches!(
            self,
            EnvironmentStatus::Initializing
                | EnvironmentStatus::Starting
                | EnvironmentStatus::Stopping
        )
    }
}

/// 内核详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelDetail {
    /// 下载 URL
    pub url: String,
    /// 文件哈希（用于下载验证）
    pub hash: String,
    /// 签名哈希（用于核心文件验证）
    #[serde(default)]
    pub signature: Option<String>,
    /// 允许复用的历史安装签名；仅用于校验已经存在的内核
    #[serde(default)]
    pub compatible_signatures: Vec<String>,
    /// 是否需要解压
    pub requires_extract: bool,
}

impl KernelDetail {
    /// 创建内核详情
    pub fn new(url: String, hash: String, requires_extract: bool) -> Self {
        Self {
            url,
            hash,
            signature: None,
            compatible_signatures: Vec::new(),
            requires_extract,
        }
    }

    /// 设置签名
    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }

    /// 验证哈希是否有效
    pub fn is_hash_valid(&self) -> bool {
        !self.hash.is_empty() && self.hash.len() == 64 // SHA256 长度
    }
}

/// 环境领域对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    /// 环境 UUID
    pub env_uuid: String,
    /// 内核值
    pub kernel_value: String,
    /// 状态
    pub status: EnvironmentStatus,
    /// 内核详情
    pub kernel_detail: KernelDetail,
}

impl Environment {
    /// 创建新环境
    pub fn new(env_uuid: String, kernel_value: String, kernel_detail: KernelDetail) -> Self {
        Self {
            env_uuid,
            kernel_value,
            status: EnvironmentStatus::Verifying,
            kernel_detail,
        }
    }

    /// 转换状态
    pub fn transition_to(&mut self, new_status: EnvironmentStatus) {
        self.status = new_status;
    }

    /// 是否可以启动
    pub fn can_launch(&self) -> bool {
        self.status.can_launch()
    }

    /// 是否正在运行
    pub fn is_running(&self) -> bool {
        self.status.is_running()
    }

    /// 验证内核是否有效
    pub fn is_kernel_valid(&self) -> bool {
        self.kernel_detail.is_hash_valid()
    }
}
