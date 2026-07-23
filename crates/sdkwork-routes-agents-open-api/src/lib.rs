//! Open API route boundary for SDKWork agents managed store.

pub use sdkwork_intelligence_agents_service::{build_open_routes, AgentHttpState};
pub use sdkwork_routes_agents_http_shared::{
    open_route_manifest, wrap_router_with_web_framework, wrap_router_with_web_framework_from_env,
    AgentRequestContext, OPEN_ROUTES,
};

/// Builds the raw open-api route tree without gateway or web-framework middleware.
pub fn build_router() -> axum::Router<AgentHttpState> {
    build_open_routes()
}

/// Builds the open-api router with mandatory sdkwork-web-framework middleware.
pub async fn build_served_router(state: AgentHttpState) -> axum::Router {
    wrap_router_with_web_framework_from_env(
        open_route_manifest(),
        build_open_routes().with_state(state),
    )
    .await
}
pub async fn gateway_mount(state: AgentHttpState) -> axum::Router {
    build_served_router(state).await
}
