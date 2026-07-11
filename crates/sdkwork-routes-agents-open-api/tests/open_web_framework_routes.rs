use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_agents_contract::env_test_lock;
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_intelligence_agents_service::{
    AgentHttpState, IamGatedPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
};
use sdkwork_routes_agents_open_api::{
    build_router, open_route_manifest, wrap_router_with_web_framework,
};
use tower::util::ServiceExt;

const AGENTS_APP_ID: &str = "sdkwork-agents";
const DEFAULT_TENANT_ID: &str = "100001";

fn test_state() -> AgentHttpState {
    AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.agents.test.iam-gated"),
    )
}

fn test_app() -> axum::Router {
    wrap_router_with_web_framework(
        IamWebRequestContextResolver::new(None),
        open_route_manifest(),
        build_router().with_state(test_state()),
    )
}

fn agents_dev_api_key(user_id: &str, api_key_id: &str) -> String {
    format!(
        "api_key_id={api_key_id};tenant_id={DEFAULT_TENANT_ID};user_id={user_id};app_id={AGENTS_APP_ID};permission_scope=ai.agents.manage"
    )
}

#[tokio::test]
async fn open_router_web_framework_rejects_unauthenticated_requests() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/agent/v3/api/ai/agents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn open_router_web_framework_accepts_dev_api_key() {
    let _guard = env_test_lock();
    std::env::set_var("SDKWORK_ENV", "dev");
    std::env::set_var("SDKWORK_IAM_ALLOW_DEV_AUTH_FALLBACK", "true");
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/agent/v3/api/ai/agents")
                .header("x-api-key", agents_dev_api_key("30001", "key-1"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}
