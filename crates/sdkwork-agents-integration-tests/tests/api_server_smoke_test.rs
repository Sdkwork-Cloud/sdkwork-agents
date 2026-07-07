use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_agents_contract::env_test_lock;
use tower::util::ServiceExt;

fn restore_optional_env(key: &str, value: Option<String>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

#[tokio::test]
async fn api_server_bootstrap_health_and_metrics_contracts() {
    let _guard = env_test_lock();
    let previous_environment = std::env::var("SDKWORK_AGENTS_ENVIRONMENT").ok();
    let previous_bypass = std::env::var("SDKWORK_AGENTS_DEV_AUTH_BYPASS").ok();

    std::env::set_var("SDKWORK_AGENTS_ENVIRONMENT", "development");
    std::env::set_var("SDKWORK_AGENTS_DEV_AUTH_BYPASS", "true");

    let app = sdkwork_agents_standalone_gateway::build_router()
        .await
        .expect("agents standalone-gateway bootstrap should succeed with dev inline auth");

    for path in ["/healthz", "/readyz", "/livez"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "expected {path} to be ready"
        );
    }

    let metrics = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metrics.status(), StatusCode::OK);

    restore_optional_env("SDKWORK_AGENTS_ENVIRONMENT", previous_environment);
    restore_optional_env("SDKWORK_AGENTS_DEV_AUTH_BYPASS", previous_bypass);
}

#[tokio::test]
async fn gateway_assembly_composes_kernel_router() {
    let _guard = env_test_lock();
    let previous_bypass = std::env::var("SDKWORK_AGENTS_DEV_AUTH_BYPASS").ok();
    std::env::set_var("SDKWORK_AGENTS_ENVIRONMENT", "development");
    std::env::set_var("SDKWORK_AGENTS_DEV_AUTH_BYPASS", "true");

    let assembly = sdkwork_agents_gateway_assembly::assemble_application_router()
        .await
        .expect("gateway assembly should compose kernel routes");

    let healthz = assembly
        .router
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(healthz.status(), StatusCode::OK);

    restore_optional_env("SDKWORK_AGENTS_DEV_AUTH_BYPASS", previous_bypass);
}

#[tokio::test]
async fn app_database_migrate_only_succeeds_with_postgres_baseline_contract() {
    let _guard = env_test_lock();
    let baseline = include_str!("../../../database/ddl/baseline/postgres/0001_agents_baseline.sql");
    assert!(baseline.contains("CREATE TABLE IF NOT EXISTS ai_agent_session"));
    assert!(baseline.contains("CREATE TABLE IF NOT EXISTS ai_agent_task"));
    assert!(!baseline.contains("ai_agent_task_run"));
}
