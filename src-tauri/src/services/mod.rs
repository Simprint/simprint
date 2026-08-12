//! 服务层
//!
//! 业务逻辑编排层，负责：
//! - 可复用的业务流程
//! - 编排多个领域对象
//! - 处理事务边界
//! - 权限检查和验证

pub mod app;
pub mod connectivity;
pub mod environment;
pub mod file_system;
pub mod local_extensions;
pub mod mihomo;
pub mod window;
