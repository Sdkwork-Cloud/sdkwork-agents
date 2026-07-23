//! Backend API route boundary for SDKWork agents managed store.

pub use sdkwork_intelligence_agents_service::{build_backend_routes, AgentHttpState};
pub use sdkwork_routes_agents_http_shared::{
    backend_route_manifest, wrap_router_with_web_framework,
    wrap_router_with_web_framework_from_env, AgentRequestContext, BACKEND_ROUTES,
};
/// Builds the raw backend-api route tree without gateway or web-framework middleware.
pub fn build_router() -> axum::Router<AgentHttpState> {
    build_backend_routes()
}

/// Builds the backend-api router with mandatory sdkwork-web-framework middleware.
pub async fn build_served_router(state: AgentHttpState) -> axum::Router {
    wrap_router_with_web_framework_from_env(
        backend_route_manifest(),
        build_backend_routes().with_state(state),
    )
    .await
}
pub async fn gateway_mount(state: AgentHttpState) -> axum::Router {
    build_served_router(state).await
}
