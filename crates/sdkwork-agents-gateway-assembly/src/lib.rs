//! Application gateway assembly for SDKWork Agents.
//!
//! Agent HTTP route crates live in `sdkwork-kernel`; this assembly composes the
//! served router through `sdkwork-agents-kernel-bridge`.

use std::sync::Arc;

use anyhow::Context;
use sdkwork_agent_server::config::ServerConfig;

pub struct ApplicationAssembly {
    pub router: axum::Router,
}

pub async fn assemble_application_router() -> anyhow::Result<ApplicationAssembly> {
    let config = Arc::new(
        ServerConfig::from_env().context("load kernel server config for agents gateway assembly")?,
    );
    let router = sdkwork_agents_kernel_bridge::build_agents_served_router(config)
        .await
        .context("compose agents served router from sdkwork-kernel")?;
    Ok(ApplicationAssembly { router })
}

pub const ROUTE_CRATE_COUNT: usize = 3;
