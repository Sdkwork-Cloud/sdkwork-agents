//! Gateway bootstrap for sdkwork-agents.
//! Kernel-owned operational routes compose with managed-store business routes via kernel-bridge.

use anyhow::Context;
use axum::Router;
use sdkwork_agent_server::config::ServerConfig;
use std::sync::Arc;

pub struct ApplicationAssembly {
    pub router: Router,
}

pub async fn assemble_application_router() -> anyhow::Result<ApplicationAssembly> {
    let config = Arc::new(ServerConfig::from_env().map_err(|error| {
        anyhow::anyhow!("load kernel server config for sdkwork-agents: {error}")
    })?);
    let router = sdkwork_agents_kernel_bridge::build_agents_served_router(config)
        .await
        .context("compose agents served router")?;
    Ok(ApplicationAssembly { router })
}
