use std::sync::Arc;

use anyhow::Context;
use axum::Router;
use sdkwork_agent_server::config::ServerConfig;
use sdkwork_agents_kernel_bridge::build_agents_served_router;

pub async fn build_router() -> anyhow::Result<Router> {
    let config = Arc::new(
        ServerConfig::from_env().context("load kernel server config for sdkwork-agents")?,
    );
    build_agents_served_router(config)
        .await
        .context("compose agents served router")
}

pub async fn run_agents_app_database_migrate_only() -> Result<(), String> {
    std::env::set_var("SDKWORK_AGENTS_DATABASE_AUTO_MIGRATE", "true");
    sdkwork_agents_database_host::bootstrap_agents_database_from_env().await?;
    tracing::info!("sdkwork-agents application database migration completed");
    Ok(())
}

pub async fn run_kernel_database_migrate_only() -> Result<(), String> {
    let config = ServerConfig::from_env().map_err(|error| error.to_string())?;
    let _persistence = sdkwork_agent_server::persistence::PersistenceState::open_from_config(&config)
        .map_err(|error| format!("kernel persistence migrate/bootstrap failed: {error}"))?;
    tracing::info!("sdkwork-agents kernel runtime persistence opened for migration");
    Ok(())
}
