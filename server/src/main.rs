use axum::{Router, middleware};
use clap::Parser;
use simprint_server::{
    cli::{Cli, Commands},
    middlewares,
    routes::route::MetaRoute,
    routes::{
        accounts, audit, billing, browser_kernel, environments, extensions, group_permissions,
        local_api, messages, preferences, proxies, proxy_visibility, referral, rpa, secret, teams,
        templates, time, users, workspace_quotas, workspaces,
    },
    svc_ctx::SvcCtx,
    utils::IConfig,
};
use simprint_server::{init_encrypt_secret, init_storage};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let (config_path, command) = cli.command_or_default();
    let config = IConfig::build_by_filepath(&config_path).expect("failed to build config");

    match command {
        Commands::Serve => run_server(config).await?,
    }

    Ok(())
}

async fn run_server(config: IConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 启用更详细的日志以调试中间件问题
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(true)
        .with_thread_ids(true)
        .init();

    let db = SvcCtx::create_db(&config.database).await?;
    tracing::info!("Running embedded database migrations");
    MIGRATOR.run(&db).await?;
    tracing::info!("Embedded database migrations completed");

    let svc_ctx = SvcCtx::new(&config).await?;

    // init resources
    init_encrypt_secret(&config).await;
    init_storage(&config).await;

    let port = &config.app.port;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to create listener");

    let app = register_all_routes(&svc_ctx);
    let app = register_middlewares(&svc_ctx, app);
    let app = app.with_state(svc_ctx);

    tracing::info!("Starting server on port {}", port);

    axum::serve(listener, app).await?;
    Ok(())
}
/// 注册所有路由
fn register_all_routes(svc_ctx: &SvcCtx) -> Router<SvcCtx> {
    let mut meta_route = MetaRoute::new(svc_ctx.config.app.prefix.clone());

    secret::register_routes(&mut meta_route);
    time::register_routes(&mut meta_route);
    users::register_routes(&mut meta_route);
    local_api::register_routes(&mut meta_route);

    // 新增路由
    workspaces::register_routes(&mut meta_route);
    workspace_quotas::register_routes(&mut meta_route);
    teams::register_routes(&mut meta_route);
    browser_kernel::register_routes(&mut meta_route);
    environments::register_routes(&mut meta_route);
    proxies::register_routes(&mut meta_route);
    proxy_visibility::register_routes(&mut meta_route);
    group_permissions::register_routes(&mut meta_route);
    accounts::register_routes(&mut meta_route);
    templates::register_routes(&mut meta_route);
    billing::register_routes(&mut meta_route);
    audit::register_routes(&mut meta_route);
    rpa::register_routes(&mut meta_route);
    referral::register_routes(&mut meta_route);
    extensions::register_routes(&mut meta_route);
    preferences::register_routes(&mut meta_route);
    messages::register_routes(&mut meta_route);

    tracing::info!("---------- {:?} ----------", meta_route.count());
    meta_route.build()
}

/// 注册中间件
/// 中间件顺序保持与原 routes.rs 一致
fn register_middlewares(svc_ctx: &SvcCtx, app: Router<SvcCtx>) -> Router<SvcCtx> {
    app.route_layer(middleware::from_fn_with_state(
        svc_ctx.clone(),
        middlewares::encrypt,
    ))
    .route_layer(middleware::from_fn_with_state(
        svc_ctx.clone(),
        middlewares::auth,
    ))
    .route_layer(middleware::from_fn_with_state(
        svc_ctx.clone(),
        middlewares::local_api_auth,
    ))
    .route_layer(middleware::from_fn_with_state(
        svc_ctx.clone(),
        middlewares::decrypt,
    ))
    .route_layer(middleware::from_fn(middlewares::real_ip))
    .route_layer(middleware::from_fn(middlewares::logger))
    .layer(middlewares::cors())
}
