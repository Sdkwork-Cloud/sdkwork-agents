use anyhow::Context;
use axum::Router;

pub async fn build_router() -> anyhow::Result<Router> {
    let assembly = sdkwork_api_agents_assembly::assemble_api_router()
        .await
        .context("compose agents gateway assembly router")?;
    Ok(assembly.router)
}

pub async fn run_agents_app_database_migrate_only() -> Result<(), String> {
    std::env::set_var("SDKWORK_DATABASE_AUTO_MIGRATE", "true");
    sdkwork_api_agents_assembly::bootstrap_application_database_from_env()
        .await
        .map_err(|error| format!("{error:#}"))?;
    tracing::info!("sdkwork-agents application database migration completed");
    Ok(())
}

pub async fn run_kernel_database_migrate_only() -> Result<(), String> {
    sdkwork_api_agents_assembly::bootstrap_kernel_database_from_env()
        .await
        .map_err(|error| format!("{error:#}"))?;
    tracing::info!("sdkwork-agents kernel runtime persistence opened for migration");
    Ok(())
}
