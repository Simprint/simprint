use clap::{Parser, Subcommand};

/// Simprint Server CLI
#[derive(Debug, Parser)]
#[command(name = "simprint-server")]
#[command(author = "Simprint Team")]
#[command(version)]
#[command(about = "Simprint Server - 客户端网关服务")]
pub struct Cli {
    /// 配置文件路径
    #[arg(short = 'f', long = "config", help = "配置文件路径 (TOML 格式)")]
    pub config: String,

    /// 子命令
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// 可用的子命令
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 启动 HTTP 服务
    Serve,
}

impl Cli {
    /// 获取命令，如果没有指定则默认为 Serve
    pub fn command_or_default(self) -> (String, Commands) {
        let config = self.config;
        let command = self.command.unwrap_or(Commands::Serve);
        (config, command)
    }
}
