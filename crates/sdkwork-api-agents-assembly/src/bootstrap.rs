//! Gateway bootstrap for sdkwork-agents.
//! Kernel-owned operational routes compose with managed-store business routes via kernel-bridge,
//! and dependency-owned app-api surfaces (IAM, Drive) mount same-origin.

mod drive;
mod iam;

use anyhow::Context;
use axum::Router;
use sdkwork_agent_server::config::ServerConfig;
use std::sync::Arc;

pub struct ApiAssembly {
    pub router: Router,
}

pub async fn assemble_api_router() -> anyhow::Result<ApiAssembly> {
    let config = Arc::new(ServerConfig::from_env().map_err(|error| {
        anyhow::anyhow!("load kernel server config for sdkwork-agents: {error}")
    })?);
    let iam_router = iam::wire_iam_app_router()
        .await
        .map_err(anyhow::Error::msg)
        .context("compose embedded IAM app router")?;
    let drive_router = drive::wire_drive_app_router()
        .await
        .map_err(anyhow::Error::msg)
        .context("compose embedded Drive app router")?;
    let agents_router = sdkwork_agents_kernel_bridge::build_agents_served_router(config.clone())
        .await
        .context("compose agents served router")?;
    let router = agents_router
        .merge(iam_router)
        .merge(drive_router)
        .layer(sdkwork_agent_server::middleware::cors_layer(
            config.as_ref(),
        ));
    Ok(ApiAssembly { router })
}
