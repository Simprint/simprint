use std::{future::Future, net::SocketAddr};

use axum::{Router, middleware};

use crate::{
    init_encrypt_secret, middlewares,
    routes::route::MetaRoute,
    routes::{
        accounts, audit, billing, browser_kernel, environments, extensions, group_permissions,
        local_api, messages, preferences, proxies, proxy_visibility, referral, rpa, secret, teams,
        templates, time, users, workspace_quotas, workspaces,
    },
    svc_ctx::SvcCtx,
    utils::IConfig,
};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

/// Start the service using its configured port on every network interface.
///
/// This preserves the original standalone server behavior. Embedded callers
/// should prefer [`serve_on`] or [`serve_on_with_shutdown`] and bind to a
/// loopback address explicitly.
pub async fn serve(config: IConfig) -> anyhow::Result<()> {
    let address = SocketAddr::from(([0, 0, 0, 0], config.app.port));
    serve_on(config, address).await
}

/// Start the service on an explicit address.
pub async fn serve_on(config: IConfig, address: SocketAddr) -> anyhow::Result<()> {
    serve_on_with_shutdown(config, address, std::future::pending()).await
}

/// Start the service on an explicit address and stop it gracefully when the
/// supplied shutdown future completes.
pub async fn serve_on_with_shutdown<F>(
    config: IConfig,
    address: SocketAddr,
    shutdown: F,
) -> anyhow::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    let svc_ctx = SvcCtx::new(&config).await?;
    tracing::info!("Running embedded database migrations");
    MIGRATOR.run(&svc_ctx.db).await?;
    tracing::info!("Embedded database migrations completed");

    init_encrypt_secret(&config).await;

    let listener = tokio::net::TcpListener::bind(address).await?;
    let bound_address = listener.local_addr()?;

    let app = register_all_routes(&svc_ctx);
    let app = register_middlewares(&svc_ctx, app);
    let app = app.with_state(svc_ctx);

    tracing::info!("Starting server on {}", bound_address);

    axum::serve(listener, app).with_graceful_shutdown(shutdown).await?;
    Ok(())
}

fn register_all_routes(svc_ctx: &SvcCtx) -> Router<SvcCtx> {
    let mut meta_route = MetaRoute::new(svc_ctx.config.app.prefix.clone());

    secret::register_routes(&mut meta_route);
    time::register_routes(&mut meta_route);
    users::register_routes(&mut meta_route);
    local_api::register_routes(&mut meta_route);
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
