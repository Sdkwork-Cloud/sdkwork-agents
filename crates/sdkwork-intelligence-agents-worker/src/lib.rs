mod config;
mod metrics;
mod readiness;
mod runtime;

pub use config::{SchedulerWorkerConfig, ENV_TASK_WORKER_BIND, ENV_TASK_WORKER_ID};
pub use metrics::SchedulerWorkerMetrics;
pub use readiness::{SchedulerWorkerControl, SchedulerWorkerReadiness};
pub use runtime::{run_scheduler_worker, TaskWorkerClient};

use std::sync::Arc;

use axum::{http::header, response::IntoResponse, routing::get, Router};
use sdkwork_web_bootstrap::{service_router, ServiceRouterConfig};
use sdkwork_web_core::{HttpMetricsDimensions, HttpMetricsRegistry};

pub fn build_operations_router(
    worker: sdkwork_intelligence_agents_service::AgentTaskWorkerHandle,
    control: Arc<SchedulerWorkerControl>,
    metrics: Arc<SchedulerWorkerMetrics>,
) -> Router {
    let http_metrics = HttpMetricsRegistry::with_dimensions(HttpMetricsDimensions {
        service: "sdkwork-intelligence-agents-worker".to_string(),
        environment: std::env::var("SDKWORK_AGENTS_ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string()),
        deployment_profile: std::env::var("SDKWORK_AGENTS_DEPLOYMENT_PROFILE")
            .unwrap_or_else(|_| "standalone".to_string()),
        runtime_target: std::env::var("SDKWORK_AGENTS_RUNTIME_TARGET")
            .unwrap_or_else(|_| "server".to_string()),
        runtime_profile: std::env::var("SDKWORK_AGENTS_PROFILE_ID").unwrap_or_default(),
    });
    let readiness = Arc::new(SchedulerWorkerReadiness::new(worker, control));
    let router = Router::new().route(
        "/metrics",
        get(move || {
            let http_metrics = http_metrics.clone();
            let metrics = metrics.clone();
            async move {
                (
                    [(
                        header::CONTENT_TYPE,
                        "text/plain; version=0.0.4; charset=utf-8",
                    )],
                    format!(
                        "{}{}",
                        http_metrics.render_prometheus(),
                        metrics.render_prometheus()
                    ),
                )
                    .into_response()
            }
        }),
    );

    service_router(
        router,
        ServiceRouterConfig::default()
            .skip_metrics()
            .with_readiness_check(readiness),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use sdkwork_intelligence_agents_service::{
        AgentHttpState, AllowAllPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
    };
    use tower::ServiceExt;

    fn test_worker() -> sdkwork_intelligence_agents_service::AgentTaskWorkerHandle {
        AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            AllowAllPolicyProvider::try_allow("policy.agents.worker.test")
                .expect("test policy provider"),
        )
        .task_worker_handle()
    }

    #[tokio::test]
    async fn operations_router_exposes_canonical_health_and_metrics() {
        let control = Arc::new(SchedulerWorkerControl::default());
        control.mark_started();
        let app = build_operations_router(
            test_worker(),
            control,
            Arc::new(SchedulerWorkerMetrics::default()),
        );

        for path in ["/healthz", "/livez", "/readyz"] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .expect("probe response");
            assert_eq!(response.status(), StatusCode::OK, "probe {path}");
        }

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("metrics response");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn readiness_fails_while_worker_is_draining() {
        let control = Arc::new(SchedulerWorkerControl::default());
        control.mark_started();
        control.begin_draining();
        let app = build_operations_router(
            test_worker(),
            control,
            Arc::new(SchedulerWorkerMetrics::default()),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("readiness response");
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
