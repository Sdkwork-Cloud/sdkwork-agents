use tokio::signal;

pub async fn shutdown_signal() {
    #[cfg(unix)]
    let terminate = wait_for_sigterm();

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = wait_for_ctrl_c() => {},
        () = terminate => {},
    }

    tracing::info!("sdkwork-api-agents-standalone-gateway shutdown signal received");
}

async fn wait_for_ctrl_c() {
    if let Err(error) = signal::ctrl_c().await {
        tracing::warn!(
            error = %error,
            "sdkwork-api-agents-standalone-gateway failed to install Ctrl+C shutdown handler"
        );
        std::future::pending::<()>().await;
    }
}

#[cfg(unix)]
async fn wait_for_sigterm() {
    let mut terminate = match signal::unix::signal(signal::unix::SignalKind::terminate()) {
        Ok(terminate) => terminate,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "sdkwork-api-agents-standalone-gateway failed to install SIGTERM shutdown handler"
            );
            std::future::pending::<()>().await;
            return;
        }
    };

    terminate.recv().await;
}
