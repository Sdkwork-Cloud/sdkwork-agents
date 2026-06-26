use anyhow::Context;
use sdkwork_agents_api_server::{
    build_router, init_tracing, run_agents_app_database_migrate_only, run_kernel_database_migrate_only,
    shutdown_signal,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    match std::env::args().nth(1).as_deref() {
        Some("db-migrate") => {
            run_agents_app_database_migrate_only()
                .await
                .map_err(anyhow::Error::msg)?;
            run_kernel_database_migrate_only()
                .await
                .map_err(anyhow::Error::msg)?;
            return Ok(());
        }
        Some("db-migrate:app") => {
            run_agents_app_database_migrate_only()
                .await
                .map_err(anyhow::Error::msg)?;
            return Ok(());
        }
        Some("db-migrate:kernel") => {
            run_kernel_database_migrate_only()
                .await
                .map_err(anyhow::Error::msg)?;
            return Ok(());
        }
        _ => {}
    }

    let bind_address = std::env::var("SDKWORK_AGENTS_APPLICATION_PUBLIC_INGRESS_BIND")
        .or_else(|_| std::env::var("SDKWORK_AGENT_SERVER_BIND"))
        .unwrap_or_else(|_| "127.0.0.1:8095".to_owned());

    let app = build_router()
        .await
        .context("sdkwork-agents-api-server bootstrap failed")?;

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .with_context(|| format!("bind sdkwork-agents-api-server on {bind_address}"))?;

    tracing::info!("sdkwork-agents-api-server listening on {bind_address}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("serve sdkwork-agents-api-server")?;
    Ok(())
}
