use std::future::IntoFuture;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use sdkwork_intelligence_agents_worker::{
    build_operations_router, run_scheduler_worker, SchedulerWorkerConfig, SchedulerWorkerControl,
    SchedulerWorkerMetrics,
};

fn main() -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build sdkwork-intelligence-agents-worker runtime")?;
    let result = runtime.block_on(async_main());
    runtime.shutdown_timeout(Duration::from_secs(5));
    result
}

async fn async_main() -> anyhow::Result<()> {
    sdkwork_web_bootstrap::init_tracing_from_env();
    let config = SchedulerWorkerConfig::from_env()?;
    let state = tokio::task::spawn_blocking(sdkwork_agents_kernel_bridge::build_agent_http_state)
        .await
        .context("agents worker state bootstrap task failed")??;
    let worker = state.task_worker_handle();
    let control = Arc::new(SchedulerWorkerControl::default());
    let metrics = Arc::new(SchedulerWorkerMetrics::default());
    let app = build_operations_router(worker.clone(), control.clone(), metrics.clone());
    let listener = tokio::net::TcpListener::bind(config.bind_address)
        .await
        .with_context(|| format!("bind agents task worker on {}", config.bind_address))?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let signal_task = tokio::spawn(wait_for_shutdown(shutdown_tx.clone()));
    let mut worker_task = tokio::spawn(run_scheduler_worker(
        worker,
        config.clone(),
        control,
        metrics,
        shutdown_rx.clone(),
    ));

    tracing::info!(
        bind_address = %config.bind_address,
        worker_id = %config.worker_id,
        max_concurrency = config.max_concurrency,
        tenant_max_concurrency = config.tenant_max_concurrency,
        "sdkwork-intelligence-agents-worker started"
    );
    let server = axum::serve(listener, app)
        .with_graceful_shutdown(wait_for_watch(shutdown_rx))
        .into_future();
    tokio::pin!(server);
    let result = tokio::select! {
        result = &mut server => result.context("serve agents task worker operations listener"),
        result = &mut worker_task => {
            result.context("agents task worker join failed")?;
            Ok(())
        }
    };
    let _ = shutdown_tx.send(true);
    if !worker_task.is_finished() {
        worker_task
            .await
            .context("agents task worker drain join failed")?;
    }
    signal_task.abort();
    result
}

async fn wait_for_watch(mut shutdown: tokio::sync::watch::Receiver<bool>) {
    if *shutdown.borrow() {
        return;
    }
    while shutdown.changed().await.is_ok() {
        if *shutdown.borrow() {
            return;
        }
    }
}

async fn wait_for_shutdown(shutdown: tokio::sync::watch::Sender<bool>) {
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::error!(error = %error, "failed to install SIGTERM handler");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        result = tokio::signal::ctrl_c() => {
            if let Err(error) = result {
                tracing::error!(error = %error, "failed to install Ctrl+C handler");
            }
        }
        _ = terminate => {}
    }
    let _ = shutdown.send(true);
}
