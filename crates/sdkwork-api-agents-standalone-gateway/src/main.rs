use anyhow::Context;
use sdkwork_api_agents_standalone_gateway::{
    build_router, init_tracing, log_access_urls, run_agents_app_database_migrate_only,
    run_kernel_database_migrate_only, shutdown_signal,
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
        .context("sdkwork-api-agents-standalone-gateway bootstrap failed")?;

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .with_context(|| format!("bind sdkwork-api-agents-standalone-gateway on {bind_address}"))?;

    let local_address = listener
        .local_addr()
        .context("resolve sdkwork-api-agents-standalone-gateway listener address")?;
    log_access_urls(local_address);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("serve sdkwork-api-agents-standalone-gateway")?;
    Ok(())
}
