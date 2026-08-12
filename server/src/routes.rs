pub mod health;
pub mod secret;
pub mod time;
pub mod users;

// 新增模块
pub mod accounts;
pub mod local_api;
pub mod audit;
pub mod billing;
pub mod browser_kernel;
pub mod environments;
pub mod extensions;
pub mod group_permissions;
pub mod messages;
pub mod preferences;
pub mod proxies;
pub mod proxy_visibility;
pub mod referral;
pub mod rpa;
pub mod teams;
pub mod templates;
pub mod workspace_quotas;
pub mod workspaces;

// route core
pub mod route {
    use std::borrow::Cow;

    use axum::{Router, routing::MethodRouter};

    use crate::svc_ctx::SvcCtx;

    // can support request methods enum
    #[derive(Debug, Clone)]
    pub enum RequestMethod {
        GET,
        POST,
        PUT,
        DELETE,
        PATCH,
    }

    /// route item
    pub struct RouteItem {
        pub path: &'static str,
        pub method: RequestMethod,
        pub handler: MethodRouter<SvcCtx>,
    }

    /// route group
    pub struct RouteGroup {
        pub prefix: &'static str,
        pub routes: Vec<RouteItem>,
    }

    /// meta route
    pub struct MetaRoute {
        pub prefix: Cow<'static, str>,
        pub routes: Vec<RouteGroup>,
    }

    impl MetaRoute {
        pub fn new(prefix: String) -> Self {
            Self {
                prefix: Cow::Owned(prefix),
                routes: vec![],
            }
        }

        pub fn add_route_group(&mut self, group: RouteGroup) -> () {
            self.routes.push(group);
        }

        pub fn count(&self) -> usize {
            self.routes.iter().map(|group| group.routes.len()).sum()
        }

        pub fn build(&self) -> Router<SvcCtx> {
            let mut root_router_child = Router::new();

            for contain in &self.routes {
                let RouteGroup { prefix, routes } = contain;

                let parent_prefix = prefix.to_string();
                let mut child_router = Router::new();

                for item in routes {
                    let RouteItem {
                        path,
                        method,
                        handler,
                    } = item;

                    tracing::info!("{:?}+{}{}{}", method, self.prefix, parent_prefix, path,);
                    match method {
                        RequestMethod::GET => {
                            child_router = child_router.route(path, handler.clone());
                        }
                        RequestMethod::POST => {
                            child_router = child_router.route(path, handler.clone());
                        }
                        RequestMethod::PUT => {
                            child_router = child_router.route(path, handler.clone());
                        }
                        RequestMethod::DELETE => {
                            child_router = child_router.route(path, handler.clone());
                        }
                        RequestMethod::PATCH => {
                            child_router = child_router.route(path, handler.clone());
                        }
                    }
                }

                root_router_child =
                    root_router_child.merge(Router::new().nest(prefix, child_router));
            }

            Router::new().nest(self.prefix.as_ref(), root_router_child)
        }
    }

    impl RouteGroup {
        pub fn new(prefix: &'static str) -> Self {
            Self {
                prefix,
                routes: vec![],
            }
        }

        pub fn add_route_item(&mut self, item: RouteItem) {
            self.routes.push(item);
        }
    }

    impl RouteItem {
        pub fn new(
            path: &'static str,
            method: RequestMethod,
            handler: MethodRouter<SvcCtx>,
        ) -> Self {
            Self {
                path,
                method,
                handler,
            }
        }

        pub fn get(path: &'static str, handler: MethodRouter<SvcCtx>) -> Self {
            Self::new(path, RequestMethod::GET, handler)
        }

        pub fn post(path: &'static str, handler: MethodRouter<SvcCtx>) -> Self {
            Self::new(path, RequestMethod::POST, handler)
        }

        pub fn put(path: &'static str, handler: MethodRouter<SvcCtx>) -> Self {
            Self::new(path, RequestMethod::PUT, handler)
        }

        pub fn delete(path: &'static str, handler: MethodRouter<SvcCtx>) -> Self {
            Self::new(path, RequestMethod::DELETE, handler)
        }

        pub fn patch(path: &'static str, handler: MethodRouter<SvcCtx>) -> Self {
            Self::new(path, RequestMethod::PATCH, handler)
        }
    }
}
