//! API assembly for sdkwork-agents.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.
// SDKWORK-ASSEMBLY-LIB-CUSTOM

mod bootstrap;
mod generated;

use std::sync::Arc;

use sdkwork_agents_runtime_facade::AgentsSessionFacade;

pub use bootstrap::{assemble_api_router, ApiAssembly};

/// Build only the authenticated Agents App API business routes for a composing host.
pub async fn assemble_app_business_router() -> anyhow::Result<axum::Router> {
    Ok(assemble_app_business_runtime().await?.router)
}

pub struct AppBusinessRuntimeAssembly {
    pub router: axum::Router,
    pub session_facade: Arc<dyn AgentsSessionFacade>,
    pub reconciliation_worker: Option<tokio::task::JoinHandle<()>>,
}

/// Build the authenticated Agents routes and public in-process facade from one state.
pub async fn assemble_app_business_runtime() -> anyhow::Result<AppBusinessRuntimeAssembly> {
    let state = tokio::task::spawn_blocking(sdkwork_agents_kernel_bridge::build_agent_http_state)
        .await
        .map_err(|error| anyhow::anyhow!("agents state bootstrap worker failed: {error}"))??;
    let session_facade = state.session_facade();
    let reconciliation_worker = state.spawn_turn_reconciliation_worker();
    let router = sdkwork_routes_agents_app_api::build_served_router(state).await;
    Ok(AppBusinessRuntimeAssembly {
        router,
        session_facade,
        reconciliation_worker,
    })
}

/// Apply the Agents managed-store lifecycle from the canonical environment profile.
pub async fn bootstrap_application_database_from_env() -> anyhow::Result<()> {
    sdkwork_agents_database_host::bootstrap_agents_database_from_env()
        .await
        .map(|_| ())
        .map_err(anyhow::Error::msg)
}

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
