//! API assembly for sdkwork-agents.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.
// SDKWORK-ASSEMBLY-LIB-CUSTOM

mod bootstrap;
mod contribution;
mod generated;
mod readiness;

pub use bootstrap::{assemble_api_router, ApiAssembly};
pub use contribution::{assemble_app_api_contribution, ApiAssemblyContribution};

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
