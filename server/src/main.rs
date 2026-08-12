use clap::Parser;
use simprint_server::{
    cli::{Cli, Commands},
    serve,
    utils::IConfig,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let (config_path, command) = cli.command_or_default();
    let config = IConfig::build_by_filepath(&config_path).expect("failed to build config");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(true)
        .with_thread_ids(true)
        .try_init()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    match command {
        Commands::Serve => serve(config).await?,
    }

    Ok(())
}
