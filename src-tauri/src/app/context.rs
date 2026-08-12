/// 应用全局上下文
///
/// 统一管理所有全局状态，替代分散的全局单例
///
/// 注意：由于某些组件需要 Tauri AppHandle，AppContext 采用分阶段初始化：
/// 1. init_early() - 初始化不依赖 AppHandle 的组件
/// 2. init_tauri_dependent() - 初始化依赖 AppHandle 的组件
use std::sync::Arc;
use tokio::sync::OnceCell;

use crate::app::runtime::SimprintRuntimeManager;
use crate::core::config::AppConfig;
use crate::infrastructure::http::encryption::RsaSecret;
use crate::infrastructure::main_server::client::MainServerRequestClient;
use crate::local_api::LocalApiManager;
use crate::mcp::McpManager;
use crate::services::environment::{EnvironmentPositionManager, EnvironmentStatusManager};
use crate::services::mihomo::MihomoManager;

/// 应用上下文
///
/// 包含所有需要全局访问的状态和服务
pub struct AppContext {
    /// 应用配置（不可变）
    pub config: AppConfig,

    /// RSA 密钥对（用于 HTTP 加密）
    pub rsa_keypair: Arc<RsaSecret>,

    /// 环境状态管理器
    pub env_status_manager: Arc<EnvironmentStatusManager>,

    /// 环境位置管理器
    pub env_position_manager: Arc<EnvironmentPositionManager>,

    /// 主服务器 HTTP 客户端
    pub main_server_client: Arc<MainServerRequestClient>,

    /// 本地 API 服务管理器
    pub local_api_manager: Arc<LocalApiManager>,

    /// MCP 服务管理器
    pub mcp_manager: Arc<McpManager>,

    /// Mihomo 集成管理器
    pub mihomo_manager: Arc<MihomoManager>,

    /// 内嵌环境运行时管理器
    pub simprint_runtime_manager: Arc<SimprintRuntimeManager>,
}

/// 全局应用上下文实例
static APP_CONTEXT: OnceCell<Arc<AppContext>> = OnceCell::const_new();

impl AppContext {
    /// 创建新的应用上下文（早期初始化，不依赖 AppHandle）
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        // 初始化 RSA 密钥对
        let rsa_keypair = Arc::new(RsaSecret::new()?);

        // 初始化主服务器 HTTP 客户端并设置拦截器
        let mut main_server_client = MainServerRequestClient::new();

        // 请求拦截器
        main_server_client.before(|rb| {
            Box::pin(crate::infrastructure::main_server::interceptors::request::encrypt(rb))
        });
        main_server_client.before(|rb| {
            Box::pin(crate::infrastructure::main_server::interceptors::request::auth(rb))
        });

        // 响应拦截器
        main_server_client.after(|response| {
            Box::pin(
                crate::infrastructure::main_server::interceptors::response_interceptor(response),
            )
        });

        let main_server_client = Arc::new(main_server_client);

        // 初始化环境状态管理器
        let env_status_manager = Arc::new(EnvironmentStatusManager::new());

        // 初始化环境位置管理器
        let env_position_manager = Arc::new(EnvironmentPositionManager::new());

        // 初始化本地 API 管理器
        let local_api_manager = Arc::new(LocalApiManager::new());

        // 初始化 MCP 管理器
        let mcp_manager = Arc::new(McpManager::new());

        // 初始化 Mihomo 管理器
        let mihomo_manager = Arc::new(MihomoManager::new());

        // 初始化内嵌环境运行时管理器
        let simprint_runtime_manager = Arc::new(SimprintRuntimeManager::new());

        Ok(Self {
            config,
            rsa_keypair,
            env_status_manager,
            env_position_manager,
            main_server_client,
            local_api_manager,
            mcp_manager,
            mihomo_manager,
            simprint_runtime_manager,
        })
    }

    /// 初始化全局上下文（早期阶段）
    pub fn init_early(config: AppConfig) -> anyhow::Result<&'static Arc<AppContext>> {
        let context = Arc::new(Self::new(config)?);
        APP_CONTEXT
            .set(context)
            .map_err(|_| anyhow::anyhow!("AppContext already initialized"))?;
        Ok(APP_CONTEXT.get().unwrap())
    }

    /// 获取应用配置
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// 获取全局上下文（如果未初始化则 panic）
    pub fn get() -> &'static Arc<AppContext> {
        APP_CONTEXT.get().expect("AppContext not initialized")
    }

    /// 尝试获取全局上下文
    pub fn try_get() -> Option<&'static Arc<AppContext>> {
        APP_CONTEXT.get()
    }
}
