#![cfg(feature = "http-axum")]
//! HTTP contract suite for Agents handlers with typed request context injection.
//!
//! Production mounts use `sdkwork-routes-agents-*-api::build_served_router` with
//! `sdkwork-web-framework`. These tests inject the same domain and web context
//! types at the raw route boundary.

use axum::body::{to_bytes, Body};
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use axum::Extension;
use sdkwork_intelligence_agents_service::{
    build_combined_routes, testing::test_web_context, AgentHttpState, AgentRepository,
    AgentRequestContext, AgentSessionEntrySurface, AgentSessionKind, AgentSessionRecord,
    AgentSessionRuntimeBindingRecord, AgentSessionRuntimeBindingStatus, AgentSessionStatus,
    DenyAllPolicyProvider, IamGatedPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
};
use serde_json::{json, Value};
use tower::ServiceExt;

fn test_agent_context() -> AgentRequestContext {
    AgentRequestContext::new("100001", "100")
        .with_organization_id("0")
        .with_subject_id("100")
        .with_roles(["ai.agents.manage"])
}

fn test_policy_provider() -> IamGatedPolicyProvider {
    IamGatedPolicyProvider::new("policy.agents.test.iam-gated")
}

fn test_deny_policy_provider() -> DenyAllPolicyProvider {
    DenyAllPolicyProvider::new("policy.agents.test.deny", "agent.business.denied")
}

fn build_test_app(state: AgentHttpState) -> axum::Router {
    build_test_app_with_context(state, test_agent_context())
}

fn build_test_app_with_context(
    state: AgentHttpState,
    context: AgentRequestContext,
) -> axum::Router {
    build_combined_routes()
        .with_state(state)
        .layer(Extension(context))
        .layer(Extension(test_web_context()))
}

fn test_manifest(agent_id: &str, display_name: &str) -> Value {
    json!({
        "schema_version": "1.0.0",
        "manifest_type": "agent",
        "agent_id": agent_id,
        "name": agent_id,
        "display_name": display_name,
        "description": "sample",
        "version": "0.1.0",
        "domain": "intelligence",
        "required_capabilities": [{"capability_id": "model.chat"}],
        "optional_capabilities": [{"capability_id": "tool.invoke"}],
        "event_families": ["agent.lifecycle"],
        "owner": { "name": "sdkwork" },
        "status": "active"
    })
}

fn create_body(agent_id: &str, display_name: &str, requested_at: &str) -> Value {
    json!({
        "agentId": agent_id,
        "code": agent_id,
        "displayName": display_name,
        "description": "sample",
        "manifest": test_manifest(agent_id, display_name),
        "defaultCodeTaskIntent": {
            "prompt": "Refactor runtime",
            "contextPaths": ["src/lib.rs"],
            "constraints": ["safe"]
        },
        "visibility": "organization",
        "tags": ["starter"],
        "requestedAt": requested_at
    })
}

async fn create_agent(app: &axum::Router, agent_id: &str, display_name: &str) {
    create_agent_at(app, agent_id, display_name, "2026-06-01T00:00:00Z").await;
}

async fn create_agent_at(
    app: &axum::Router,
    agent_id: &str,
    display_name: &str,
    requested_at: &str,
) {
    let request = Request::builder()
        .method("POST")
        .uri("/app/v3/api/ai/agents")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            create_body(agent_id, display_name, requested_at).to_string(),
        ))
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("create request should succeed");
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn post_json(
    app: &axum::Router,
    uri: &str,
    body: Value,
    expected_status: StatusCode,
) -> Value {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should succeed");
    let status = response.status();
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    assert_eq!(
        status,
        expected_status,
        "{uri}: {}",
        String::from_utf8_lossy(&body_bytes)
    );
    serde_json::from_slice(&body_bytes).expect("response body should be valid json")
}

async fn get_json_response(app: &axum::Router, uri: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should succeed");
    let status = response.status();
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body = serde_json::from_slice(&body_bytes).unwrap_or_else(|error| {
        panic!(
            "{uri}: response body should be valid json: {error}: {}",
            String::from_utf8_lossy(&body_bytes)
        )
    });
    (status, body)
}

async fn create_app_session(
    app: &axum::Router,
    agent_id: &str,
    session_id: &str,
    title: &str,
    requested_at: &str,
) -> Value {
    post_json(
        app,
        &format!("/app/v3/api/ai/agents/{agent_id}/sessions"),
        json!({
            "sessionId": session_id,
            "sessionKind": "assistant",
            "entrySurface": "api",
            "title": title,
            "idempotencyKey": format!("create-{session_id}"),
            "payloadHash": format!("sha256:create-{session_id}"),
            "requestedAt": requested_at
        }),
        StatusCode::CREATED,
    )
    .await
}

async fn create_turn_runtime(
    app: &axum::Router,
    agent_id: &str,
    session_id: &str,
    suffix: &str,
) -> String {
    let binding_id = format!("binding.turn.{suffix}");
    let provider_id = format!("provider.model.{suffix}");
    post_json(
        app,
        &format!("/app/v3/api/ai/agents/{agent_id}/provider_bindings"),
        json!({
            "bindingId": binding_id,
            "providerId": provider_id,
            "implementationKind": "typed-local-provider",
            "configurationProfileId": format!("profile.turn.{suffix}"),
            "capabilities": ["model.chat"],
            "makeDefault": true,
            "requestedAt": "2026-06-28T12:00:00Z"
        }),
        StatusCode::CREATED,
    )
    .await;

    let runtime_binding_id = format!("runtime_binding.turn.{suffix}");
    post_json(
        app,
        &format!("/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/runtime_bindings"),
        json!({
            "runtimeBindingId": runtime_binding_id,
            "hostMode": "managed",
            "transportKind": "in_process",
            "providerBindingId": binding_id,
            "modelId": format!("model.turn.{suffix}"),
            "providerId": provider_id,
            "requestedAt": "2026-06-28T12:00:00Z"
        }),
        StatusCode::CREATED,
    )
    .await;
    runtime_binding_id
}

async fn patch_json(
    app: &axum::Router,
    uri: &str,
    mut body: Value,
    expected_status: StatusCode,
) -> Value {
    if body.get("expectedVersion").is_none() {
        if let Some(object) = body.as_object_mut() {
            object.insert("expectedVersion".to_string(), json!("1"));
        }
    }
    let request = Request::builder()
        .method("PATCH")
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should succeed");
    let status = response.status();
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    assert_eq!(
        status,
        expected_status,
        "{uri}: {}",
        String::from_utf8_lossy(&body_bytes)
    );
    serde_json::from_slice(&body_bytes).expect("response body should be valid json")
}

async fn get_json(app: &axum::Router, uri: &str, expected_status: StatusCode) -> Value {
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should succeed");
    let status = response.status();
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    assert_eq!(
        status,
        expected_status,
        "{uri}: {}",
        String::from_utf8_lossy(&body_bytes)
    );
    serde_json::from_slice(&body_bytes).expect("response body should be valid json")
}

async fn request_without_body(
    app: &axum::Router,
    method: &str,
    uri: &str,
    expected_status: StatusCode,
) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should succeed");
    let status = response.status();
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    assert_eq!(
        status,
        expected_status,
        "{uri}: {}",
        String::from_utf8_lossy(&body_bytes)
    );
    assert!(
        body_bytes.is_empty(),
        "{uri}: a no-content response must not include a response body"
    );
}

fn response_constraints(response: &Value) -> Vec<String> {
    response["data"]["item"]["defaultCodeTaskIntent"]["constraints"]
        .as_array()
        .expect("defaultCodeTaskIntent.constraints should be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("constraint should be a string")
                .to_string()
        })
        .collect()
}

fn response_context_paths(response: &Value) -> Vec<String> {
    response["data"]["item"]["defaultCodeTaskIntent"]["contextPaths"]
        .as_array()
        .expect("defaultCodeTaskIntent.contextPaths should be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("contextPath should be a string")
                .to_string()
        })
        .collect()
}

fn pc_management_profile_constraints(constraints: &[String]) -> Vec<Value> {
    constraints
        .iter()
        .filter_map(|constraint| {
            constraint
                .strip_prefix("sdkwork.agent.pc.config:")
                .map(|encoded| serde_json::from_str(encoded).expect("PC profile should be JSON"))
        })
        .collect()
}

#[tokio::test]
async fn app_create_and_retrieve_agent_should_work() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.alpha", "Alpha").await;

    let request = Request::builder()
        .method("GET")
        .uri("/app/v3/api/ai/agents/agent.alpha")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("get request should succeed");
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["data"]["item"]["agentId"], "agent.alpha");
    assert_eq!(body_json["data"]["item"]["displayName"], "Alpha");
}

#[tokio::test]
async fn app_code_engine_catalog_should_return_engines() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/app/v3/api/ai/code_engines")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .oneshot(request)
        .await
        .expect("catalog request should succeed");
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 0, "envelope code must be 0");
    assert!(body_json["data"]["item"]["engines"].is_array());
}

#[tokio::test]
async fn app_project_session_should_materialize_canonical_code_engine_identity() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let project_id = "project.339967887101923328";

    post_json(
        &app,
        "/app/v3/api/ai/projects",
        json!({
            "projectId": project_id,
            "name": "Canonical code engine session"
        }),
        StatusCode::CREATED,
    )
    .await;

    let created = post_json(
        &app,
        &format!("/app/v3/api/ai/projects/{project_id}/sessions"),
        json!({
            "agentId": "agent.intelligence.codex",
            "sessionKind": "coding",
            "entrySurface": "pc",
            "sourceModule": "sdkwork-birdcoder",
            "sourceContextKind": "agent-project",
            "sourceContextId": project_id,
            "title": "hi",
            "idempotencyKey": "3bf76c8b-8b9c-4d1c-a183-9b0ae342004c",
            "payloadHash": "sha256:89afc39f0d667fa874345a7cae2f6e01cfe74e4b8e0075453bd1d8b2a5ae6de5",
            "requestedAt": "2026-07-27T07:36:34.892Z"
        }),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(
        created["data"]["item"]["agentId"],
        "agent.intelligence.codex"
    );
    assert_eq!(created["data"]["item"]["projectId"], project_id);

    let bindings = get_json(
        &app,
        "/app/v3/api/ai/agents/agent.intelligence.codex/provider_bindings?page=1&page_size=20",
        StatusCode::OK,
    )
    .await;
    assert_eq!(bindings["data"]["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        bindings["data"]["items"][0]["bindingId"],
        "binding.agent-provider.codex"
    );
    assert_eq!(
        bindings["data"]["items"][0]["providerId"],
        "provider.model.codex"
    );
}

#[tokio::test]
async fn app_session_create_should_replay_by_idempotency_key() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let agent_id = "agent.session.idempotent";
    create_agent(&app, agent_id, "Idempotent Session Agent").await;
    let uri = format!("/app/v3/api/ai/agents/{agent_id}/sessions");
    let request = json!({
        "sessionKind": "assistant",
        "entrySurface": "api",
        "title": "Idempotent session",
        "idempotencyKey": "session-create-idempotency-contract",
        "payloadHash": "sha256:session-create-idempotency-contract",
        "requestedAt": "2026-07-27T08:00:00Z"
    });

    let (created, replayed) = tokio::join!(
        post_json(&app, &uri, request.clone(), StatusCode::CREATED),
        post_json(&app, &uri, request.clone(), StatusCode::CREATED),
    );
    assert_eq!(
        replayed["data"]["item"]["sessionId"],
        created["data"]["item"]["sessionId"]
    );

    let listed = get_json(
        &app,
        &format!("{uri}?page=1&page_size=20&include_archived=false"),
        StatusCode::OK,
    )
    .await;
    assert_eq!(listed["data"]["items"].as_array().map(Vec::len), Some(1));

    let conflict = post_json(
        &app,
        &uri,
        json!({
            "sessionKind": "assistant",
            "entrySurface": "api",
            "title": "Different session",
            "idempotencyKey": "session-create-idempotency-contract",
            "payloadHash": "sha256:different-session-create-payload",
            "requestedAt": "2026-07-27T08:00:01Z"
        }),
        StatusCode::CONFLICT,
    )
    .await;
    assert_eq!(conflict["code"], 40901);

    let session_id = created["data"]["item"]["sessionId"]
        .as_str()
        .expect("created session id");
    request_without_body(
        &app,
        "DELETE",
        &format!("{uri}/{session_id}"),
        StatusCode::NO_CONTENT,
    )
    .await;
    let deleted_conflict = post_json(&app, &uri, request, StatusCode::CONFLICT).await;
    assert_eq!(deleted_conflict["code"], 40901);
}

#[tokio::test]
async fn app_mcp_marketplace_should_return_records_array() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/app/v3/api/ai/mcp_servers")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .oneshot(request)
        .await
        .expect("mcp marketplace request should succeed");
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 0, "envelope code must be 0");
    assert!(body_json["data"]["items"].is_array());
    assert!(body_json["data"]["pageInfo"].is_object());
}

#[tokio::test]
async fn app_create_agent_rejects_client_scope_query_selector() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    post_json(
        &app,
        "/app/v3/api/ai/agents?tenant_id=999",
        json!({
            "agentId": "agent.context.scope",
            "code": "agent.context.scope",
            "displayName": "Context Scope",
            "description": "scope should come from request context",
            "manifest": test_manifest("agent.context.scope", "Context Scope"),
            "defaultCodeTaskIntent": {
                "prompt": "Use context scope",
                "contextPaths": ["src/lib.rs"],
                "constraints": ["safe"]
            },
            "visibility": "organization",
            "tags": ["scope"],
            "requestedAt": "2026-06-01T00:00:30Z"
        }),
        StatusCode::BAD_REQUEST,
    )
    .await;
}

#[tokio::test]
async fn app_create_agent_rejects_client_scope_body_selector() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    post_json(
        &app,
        "/app/v3/api/ai/agents",
        json!({
            "agentId": "agent.context.body-scope",
            "tenantId": "999",
            "organizationId": "999",
            "ownerUserId": "999",
            "code": "agent.context.body-scope",
            "displayName": "Body Scope",
            "description": "scope selectors are forbidden",
            "manifest": test_manifest("agent.context.body-scope", "Body Scope"),
            "visibility": "organization",
            "requestedAt": "2026-06-01T00:00:31Z"
        }),
        StatusCode::BAD_REQUEST,
    )
    .await;
}

#[tokio::test]
async fn app_agent_response_should_expose_pc_management_profile() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let management_profile = json!({
        "avatar": "robot",
        "categoryId": "assistant",
        "color": "#3b82f6",
        "iconName": "bot",
        "knowledgeBaseIds": ["knowledge.base.product", "knowledge.base.runbook"],
        "systemPrompt": "Answer from approved knowledge only.",
        "type": "independent",
        "welcomeMessage": "How can I help?"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/app/v3/api/ai/agents")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "agentId": "agent.pc.profile",
                "code": "agent.pc.profile",
                "displayName": "PC Profile",
                "description": "sample",
                "manifest": test_manifest("agent.pc.profile", "PC Profile"),
                "defaultCodeTaskIntent": {
                    "prompt": "Answer from approved knowledge only.",
                    "contextPaths": ["knowledge.base.product"],
                    "constraints": [
                        "agent.type=independent",
                        format!("sdkwork.agent.pc.config:{management_profile}")
                    ]
                },
                "visibility": "private",
                "tags": ["assistant"],
                "requestedAt": "2026-06-01T00:00:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("create request should succeed");
    assert_eq!(response.status(), StatusCode::CREATED);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");

    assert_eq!(
        body_json["data"]["item"]["managementProfile"]["avatar"],
        "robot"
    );
    assert_eq!(
        body_json["data"]["item"]["managementProfile"]["categoryId"],
        "assistant"
    );
    assert_eq!(
        body_json["data"]["item"]["managementProfile"]["knowledgeBaseIds"],
        json!(["knowledge.base.product", "knowledge.base.runbook"])
    );
    assert_eq!(
        body_json["data"]["item"]["managementProfile"]["systemPrompt"],
        "Answer from approved knowledge only."
    );
    assert_eq!(
        body_json["data"]["item"]["managementProfile"]["type"],
        "independent"
    );
    assert_eq!(
        body_json["data"]["item"]["managementProfile"]["welcomeMessage"],
        "How can I help?"
    );
}

#[tokio::test]
async fn app_agent_request_should_accept_management_profile_and_store_compatible_intent() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let response = post_json(
        &app,
        "/app/v3/api/ai/agents",
        json!({
            "agentId": "agent.pc.structured",
            "code": "agent.pc.structured",
            "displayName": "Structured PC Agent",
            "description": "sample",
            "manifest": test_manifest("agent.pc.structured", "Structured PC Agent"),
            "defaultCodeTaskIntent": {
                "prompt": "Use approved knowledge",
                "contextPaths": ["src/lib.rs"],
                "constraints": ["safe"]
            },
            "managementProfile": {
                "author": "SDKWork",
                "avatar": "robot",
                "categoryId": "assistant",
                "color": "#3b82f6",
                "debugMode": true,
                "iconName": "bot",
                "jsonMode": true,
                "knowledgeBaseIds": ["knowledge.base.product", "knowledge.base.runbook"],
                "memoryEnabled": true,
                "model": "model.openai.gpt-4",
                "skillIds": ["skill.research.deep"],
                "suggestedPrompts": ["What can you do?", "Summarize this document"],
                "systemPrompt": "Answer from approved knowledge only.",
                "temperature": 0.7,
                "toolIds": ["tool.mcp.filesystem"],
                "type": "independent",
                "users": "12 users",
                "voiceIds": ["voice.default.narrator"],
                "welcomeMessage": "How can I help?"
            },
            "visibility": "private",
            "tags": ["assistant"],
            "requestedAt": "2026-06-01T00:02:00Z"
        }),
        StatusCode::CREATED,
    )
    .await;

    assert_eq!(
        response["data"]["item"]["managementProfile"]["avatar"],
        "robot"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["author"],
        "SDKWork"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["knowledgeBaseIds"],
        json!(["knowledge.base.product", "knowledge.base.runbook"])
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["type"],
        "independent"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["users"],
        "12 users"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["debugMode"],
        true
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["jsonMode"],
        true
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["memoryEnabled"],
        true
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["model"],
        "model.openai.gpt-4"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["temperature"],
        0.7
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["suggestedPrompts"],
        json!(["What can you do?", "Summarize this document"])
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["voiceIds"],
        json!(["voice.default.narrator"])
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["toolIds"],
        json!(["tool.mcp.filesystem"])
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["skillIds"],
        json!(["skill.research.deep"])
    );

    let constraints = response_constraints(&response);
    assert!(
        constraints.iter().any(|constraint| constraint == "safe"),
        "existing constraints should be preserved: {constraints:?}"
    );
    assert!(
        constraints
            .iter()
            .any(|constraint| constraint == "agent.type=independent"),
        "agent.type compatibility constraint should be written: {constraints:?}"
    );
    let pc_profiles = pc_management_profile_constraints(&constraints);
    assert_eq!(pc_profiles.len(), 1);
    assert_eq!(pc_profiles[0]["author"], "SDKWork");
    assert_eq!(pc_profiles[0]["categoryId"], "assistant");
    assert_eq!(pc_profiles[0]["debugMode"], true);
    assert_eq!(pc_profiles[0]["jsonMode"], true);
    assert_eq!(pc_profiles[0]["memoryEnabled"], true);
    assert_eq!(pc_profiles[0]["model"], "model.openai.gpt-4");
    assert_eq!(
        pc_profiles[0]["suggestedPrompts"],
        json!(["What can you do?", "Summarize this document"])
    );
    assert_eq!(pc_profiles[0]["temperature"], 0.7);
    assert_eq!(
        pc_profiles[0]["voiceIds"],
        json!(["voice.default.narrator"])
    );
    assert_eq!(pc_profiles[0]["toolIds"], json!(["tool.mcp.filesystem"]));
    assert_eq!(pc_profiles[0]["skillIds"], json!(["skill.research.deep"]));
    assert_eq!(pc_profiles[0]["type"], "independent");
    assert_eq!(pc_profiles[0]["users"], "12 users");

    let context_paths = response_context_paths(&response);
    for expected_path in [
        "src/lib.rs",
        "knowledge.base.product",
        "knowledge.base.runbook",
    ] {
        assert!(
            context_paths.iter().any(|path| path == expected_path),
            "contextPaths should include {expected_path}: {context_paths:?}"
        );
    }
}

#[tokio::test]
async fn app_agent_management_profile_should_reject_values_outside_openapi_contract() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let base_profile = json!({
        "author": "SDKWork",
        "avatar": "robot",
        "categoryId": "assistant",
        "color": "#3b82f6",
        "debugMode": true,
        "iconName": "bot",
        "jsonMode": true,
        "knowledgeBaseIds": ["knowledge.base.product"],
        "memoryEnabled": true,
        "model": "model.openai.gpt-4",
        "skillIds": ["skill.research.deep"],
        "suggestedPrompts": ["What can you do?"],
        "systemPrompt": "Answer from approved knowledge only.",
        "temperature": 0.7,
        "toolIds": ["tool.mcp.filesystem"],
        "type": "independent",
        "users": "12 users",
        "voiceIds": ["voice.default.narrator"],
        "welcomeMessage": "How can I help?"
    });

    let cases = [
        (
            "model-prefix",
            json!({"model": "provider.openai"}),
            "managementProfile.model must start with model.",
        ),
        (
            "temperature-min",
            json!({"temperature": -0.1}),
            "managementProfile.temperature must be greater than or equal to 0",
        ),
        (
            "temperature-max",
            json!({"temperature": 2.1}),
            "managementProfile.temperature must be less than or equal to 2",
        ),
        (
            "knowledge-base-prefix",
            json!({"knowledgeBaseIds": ["knowledge.document.bad"]}),
            "managementProfile.knowledgeBaseIds items must start with knowledge.base.",
        ),
        (
            "skill-prefix",
            json!({"skillIds": ["tool.web.search"]}),
            "managementProfile.skillIds items must start with skill.",
        ),
        (
            "tool-prefix",
            json!({"toolIds": ["skill.research.deep"]}),
            "managementProfile.toolIds items must start with tool.",
        ),
        (
            "voice-prefix",
            json!({"voiceIds": ["tool.voice.default"]}),
            "managementProfile.voiceIds items must start with voice.",
        ),
        (
            "suggested-prompts-count",
            json!({"suggestedPrompts": [
                "p01", "p02", "p03", "p04", "p05", "p06", "p07",
                "p08", "p09", "p10", "p11", "p12", "p13"
            ]}),
            "managementProfile.suggestedPrompts must contain at most 12 items",
        ),
        (
            "suggested-prompts-length",
            json!({"suggestedPrompts": ["x".repeat(257)]}),
            "managementProfile.suggestedPrompts items must be at most 256 characters",
        ),
    ];

    for (case_id, override_profile, expected_detail) in cases {
        let agent_id = format!("agent.pc.invalid.profile.{case_id}");
        let mut profile = base_profile.clone();
        let profile_object = profile
            .as_object_mut()
            .expect("base profile should be an object");
        for (key, value) in override_profile
            .as_object()
            .expect("override profile should be an object")
        {
            profile_object.insert(key.clone(), value.clone());
        }

        let mut body = create_body(
            agent_id.as_str(),
            format!("InvalidProfile{case_id}").as_str(),
            "2026-06-01T00:02:00Z",
        );
        body["managementProfile"] = profile;

        let response =
            post_json(&app, "/app/v3/api/ai/agents", body, StatusCode::BAD_REQUEST).await;

        assert_eq!(response["code"], 40001);
        assert_eq!(response["detail"], expected_detail);
    }
}

#[tokio::test]
async fn app_update_agent_management_profile_should_preserve_existing_intent_constraints() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let old_profile = json!({
        "avatar": "old",
        "categoryId": "legacy",
        "knowledgeBaseIds": ["knowledge.base.legacy"],
        "type": "legacy",
        "welcomeMessage": "Old welcome"
    });
    post_json(
        &app,
        "/app/v3/api/ai/agents",
        json!({
            "agentId": "agent.pc.update.structured",
            "code": "agent.pc.update.structured",
            "displayName": "Structured Update PC Agent",
            "description": "sample",
            "manifest": test_manifest(
                "agent.pc.update.structured",
                "Structured Update PC Agent"
            ),
            "defaultCodeTaskIntent": {
                "prompt": "Keep current prompt",
                "contextPaths": ["knowledge.base.legacy"],
                "constraints": [
                    "safe",
                    "agent.type=legacy",
                    format!("sdkwork.agent.pc.config:{old_profile}")
                ]
            },
            "visibility": "private",
            "tags": ["assistant"],
            "requestedAt": "2026-06-01T00:03:00Z"
        }),
        StatusCode::CREATED,
    )
    .await;

    let request = Request::builder()
        .method("PATCH")
        .uri("/app/v3/api/ai/agents/agent.pc.update.structured")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "managementProfile": {
                "avatar": "robot",
                "categoryId": "assistant",
                "color": "#16a34a",
                "debugMode": false,
                "iconName": "sparkles",
                "jsonMode": true,
                "knowledgeBaseIds": [
                    "knowledge.base.legacy",
                    "knowledge.base.product"
                ],
                "memoryEnabled": false,
                "model": "model.anthropic.claude-sonnet",
                "skillIds": ["skill.write.release-notes"],
                "suggestedPrompts": ["Draft release notes"],
                "systemPrompt": "Answer with current product knowledge.",
                "temperature": 0.2,
                "toolIds": ["tool.web.search"],
                "type": "independent",
                "voiceIds": ["voice.product.host"],
                "welcomeMessage": "Ask me about the product."
            },
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T00:04:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("update request should succeed");
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let response: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");

    assert_eq!(
        response["data"]["item"]["managementProfile"]["avatar"],
        "robot"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["categoryId"],
        "assistant"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["type"],
        "independent"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["debugMode"],
        false
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["jsonMode"],
        true
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["memoryEnabled"],
        false
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["model"],
        "model.anthropic.claude-sonnet"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["temperature"],
        0.2
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["suggestedPrompts"],
        json!(["Draft release notes"])
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["voiceIds"],
        json!(["voice.product.host"])
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["toolIds"],
        json!(["tool.web.search"])
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["skillIds"],
        json!(["skill.write.release-notes"])
    );

    let constraints = response_constraints(&response);
    assert!(
        constraints.iter().any(|constraint| constraint == "safe"),
        "existing non-profile constraints should be preserved: {constraints:?}"
    );
    assert!(
        constraints
            .iter()
            .all(|constraint| constraint != "agent.type=legacy"),
        "old agent.type compatibility constraint should be replaced: {constraints:?}"
    );
    assert!(
        constraints
            .iter()
            .any(|constraint| constraint == "agent.type=independent"),
        "new agent.type compatibility constraint should be written: {constraints:?}"
    );
    let pc_profiles = pc_management_profile_constraints(&constraints);
    assert_eq!(pc_profiles.len(), 1);
    assert_eq!(pc_profiles[0]["categoryId"], "assistant");
    assert_eq!(pc_profiles[0]["debugMode"], false);
    assert_eq!(pc_profiles[0]["jsonMode"], true);
    assert_eq!(pc_profiles[0]["memoryEnabled"], false);
    assert_eq!(pc_profiles[0]["model"], "model.anthropic.claude-sonnet");
    assert_eq!(
        pc_profiles[0]["suggestedPrompts"],
        json!(["Draft release notes"])
    );
    assert_eq!(pc_profiles[0]["temperature"], 0.2);
    assert_eq!(pc_profiles[0]["voiceIds"], json!(["voice.product.host"]));
    assert_eq!(pc_profiles[0]["toolIds"], json!(["tool.web.search"]));
    assert_eq!(
        pc_profiles[0]["skillIds"],
        json!(["skill.write.release-notes"])
    );
    assert_eq!(pc_profiles[0]["type"], "independent");

    let context_paths = response_context_paths(&response);
    assert_eq!(
        context_paths
            .iter()
            .filter(|path| path.as_str() == "knowledge.base.legacy")
            .count(),
        1,
        "existing contextPath should not be duplicated: {context_paths:?}"
    );
    assert!(
        context_paths
            .iter()
            .any(|path| path == "knowledge.base.product"),
        "new knowledge base id should be appended to contextPaths: {context_paths:?}"
    );
}

#[tokio::test]
async fn backend_agent_request_should_accept_management_profile_and_store_compatible_intent() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let response = post_json(
        &app,
        "/backend/v3/api/ai/agents",
        json!({
            "agentId": "agent.pc.backend.structured",
            "code": "agent.pc.backend.structured",
            "displayName": "Backend Structured PC Agent",
            "description": "backend structured profile",
            "manifest": test_manifest(
                "agent.pc.backend.structured",
                "Backend Structured PC Agent"
            ),
            "defaultCodeTaskIntent": {
                "prompt": "Use approved knowledge",
                "contextPaths": ["src/lib.rs"],
                "constraints": ["operator-managed"]
            },
            "managementProfile": {
                "author": "SDKWork Backend",
                "avatar": "robot",
                "categoryId": "assistant",
                "color": "#2563eb",
                "debugMode": true,
                "iconName": "bot",
                "jsonMode": false,
                "knowledgeBaseIds": [
                    "knowledge.base.backend.product",
                    "knowledge.base.backend.runbook"
                ],
                "memoryEnabled": true,
                "model": "model.openai.gpt-4o",
                "skillIds": ["skill.ops.runbook"],
                "suggestedPrompts": ["Open incident runbook"],
                "systemPrompt": "Answer with backend approved knowledge only.",
                "temperature": 0.4,
                "toolIds": ["tool.ops.lookup"],
                "type": "independent",
                "users": "42 users",
                "voiceIds": ["voice.ops.dispatcher"],
                "welcomeMessage": "Ask me about backend-managed knowledge."
            },
            "visibility": "organization",
            "tags": ["assistant", "backend"],
            "requestedAt": "2026-06-01T00:10:00Z"
        }),
        StatusCode::CREATED,
    )
    .await;

    assert_eq!(
        response["data"]["item"]["managementProfile"]["avatar"],
        "robot"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["author"],
        "SDKWork Backend"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["knowledgeBaseIds"],
        json!([
            "knowledge.base.backend.product",
            "knowledge.base.backend.runbook"
        ])
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["type"],
        "independent"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["users"],
        "42 users"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["debugMode"],
        true
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["jsonMode"],
        false
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["memoryEnabled"],
        true
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["model"],
        "model.openai.gpt-4o"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["temperature"],
        0.4
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["suggestedPrompts"],
        json!(["Open incident runbook"])
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["voiceIds"],
        json!(["voice.ops.dispatcher"])
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["toolIds"],
        json!(["tool.ops.lookup"])
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["skillIds"],
        json!(["skill.ops.runbook"])
    );

    let constraints = response_constraints(&response);
    assert!(
        constraints
            .iter()
            .any(|constraint| constraint == "operator-managed"),
        "existing backend constraints should be preserved: {constraints:?}"
    );
    assert!(
        constraints
            .iter()
            .any(|constraint| constraint == "agent.type=independent"),
        "backend compatibility agent.type should be written: {constraints:?}"
    );
    let pc_profiles = pc_management_profile_constraints(&constraints);
    assert_eq!(pc_profiles.len(), 1);
    assert_eq!(pc_profiles[0]["author"], "SDKWork Backend");
    assert_eq!(pc_profiles[0]["categoryId"], "assistant");
    assert_eq!(pc_profiles[0]["debugMode"], true);
    assert_eq!(pc_profiles[0]["jsonMode"], false);
    assert_eq!(pc_profiles[0]["memoryEnabled"], true);
    assert_eq!(pc_profiles[0]["model"], "model.openai.gpt-4o");
    assert_eq!(
        pc_profiles[0]["suggestedPrompts"],
        json!(["Open incident runbook"])
    );
    assert_eq!(pc_profiles[0]["temperature"], 0.4);
    assert_eq!(pc_profiles[0]["voiceIds"], json!(["voice.ops.dispatcher"]));
    assert_eq!(pc_profiles[0]["toolIds"], json!(["tool.ops.lookup"]));
    assert_eq!(pc_profiles[0]["skillIds"], json!(["skill.ops.runbook"]));
    assert_eq!(pc_profiles[0]["type"], "independent");
    assert_eq!(pc_profiles[0]["users"], "42 users");

    let context_paths = response_context_paths(&response);
    for expected_path in [
        "src/lib.rs",
        "knowledge.base.backend.product",
        "knowledge.base.backend.runbook",
    ] {
        assert!(
            context_paths.iter().any(|path| path == expected_path),
            "backend contextPaths should include {expected_path}: {context_paths:?}"
        );
    }
}

#[tokio::test]
async fn backend_update_agent_management_profile_should_preserve_existing_intent_constraints() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let old_profile = json!({
        "avatar": "old",
        "categoryId": "legacy",
        "knowledgeBaseIds": ["knowledge.base.backend.legacy"],
        "type": "legacy",
        "welcomeMessage": "Old backend welcome"
    });
    post_json(
        &app,
        "/backend/v3/api/ai/agents",
        json!({
            "agentId": "agent.pc.backend.update.structured",
            "code": "agent.pc.backend.update.structured",
            "displayName": "Backend Structured Update PC Agent",
            "description": "backend structured update",
            "manifest": test_manifest(
                "agent.pc.backend.update.structured",
                "Backend Structured Update PC Agent"
            ),
            "defaultCodeTaskIntent": {
                "prompt": "Keep backend prompt",
                "contextPaths": ["knowledge.base.backend.legacy"],
                "constraints": [
                    "operator-managed",
                    "agent.type=legacy",
                    format!("sdkwork.agent.pc.config:{old_profile}")
                ]
            },
            "visibility": "organization",
            "tags": ["assistant", "backend"],
            "requestedAt": "2026-06-01T00:11:00Z"
        }),
        StatusCode::CREATED,
    )
    .await;

    let response = patch_json(
        &app,
        "/backend/v3/api/ai/agents/agent.pc.backend.update.structured",
        json!({
            "managementProfile": {
                "avatar": "robot",
                "categoryId": "assistant",
                "color": "#0891b2",
                "debugMode": false,
                "iconName": "sparkles",
                "jsonMode": true,
                "knowledgeBaseIds": [
                    "knowledge.base.backend.legacy",
                    "knowledge.base.backend.product"
                ],
                "memoryEnabled": false,
                "model": "model.azure.gpt-4",
                "skillIds": ["skill.ops.triage"],
                "suggestedPrompts": ["Triage latest incident"],
                "systemPrompt": "Answer with current backend-managed knowledge.",
                "temperature": 0.1,
                "toolIds": ["tool.ops.audit"],
                "type": "independent",
                "voiceIds": ["voice.ops.lead"],
                "welcomeMessage": "Ask me about backend-managed product knowledge."
            },
            "requestedAt": "2026-06-01T00:12:00Z"
        }),
        StatusCode::OK,
    )
    .await;

    assert_eq!(
        response["data"]["item"]["managementProfile"]["avatar"],
        "robot"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["categoryId"],
        "assistant"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["type"],
        "independent"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["debugMode"],
        false
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["jsonMode"],
        true
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["memoryEnabled"],
        false
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["model"],
        "model.azure.gpt-4"
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["temperature"],
        0.1
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["suggestedPrompts"],
        json!(["Triage latest incident"])
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["voiceIds"],
        json!(["voice.ops.lead"])
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["toolIds"],
        json!(["tool.ops.audit"])
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["skillIds"],
        json!(["skill.ops.triage"])
    );

    let constraints = response_constraints(&response);
    assert!(
        constraints
            .iter()
            .any(|constraint| constraint == "operator-managed"),
        "backend non-profile constraints should be preserved: {constraints:?}"
    );
    assert!(
        constraints
            .iter()
            .all(|constraint| constraint != "agent.type=legacy"),
        "old backend compatibility agent.type should be replaced: {constraints:?}"
    );
    assert!(
        constraints
            .iter()
            .any(|constraint| constraint == "agent.type=independent"),
        "new backend compatibility agent.type should be written: {constraints:?}"
    );
    let pc_profiles = pc_management_profile_constraints(&constraints);
    assert_eq!(pc_profiles.len(), 1);
    assert_eq!(pc_profiles[0]["categoryId"], "assistant");
    assert_eq!(pc_profiles[0]["debugMode"], false);
    assert_eq!(pc_profiles[0]["jsonMode"], true);
    assert_eq!(pc_profiles[0]["memoryEnabled"], false);
    assert_eq!(pc_profiles[0]["model"], "model.azure.gpt-4");
    assert_eq!(
        pc_profiles[0]["suggestedPrompts"],
        json!(["Triage latest incident"])
    );
    assert_eq!(pc_profiles[0]["temperature"], 0.1);
    assert_eq!(pc_profiles[0]["voiceIds"], json!(["voice.ops.lead"]));
    assert_eq!(pc_profiles[0]["toolIds"], json!(["tool.ops.audit"]));
    assert_eq!(pc_profiles[0]["skillIds"], json!(["skill.ops.triage"]));
    assert_eq!(pc_profiles[0]["type"], "independent");

    let context_paths = response_context_paths(&response);
    assert_eq!(
        context_paths
            .iter()
            .filter(|path| path.as_str() == "knowledge.base.backend.legacy")
            .count(),
        1,
        "backend existing contextPath should not be duplicated: {context_paths:?}"
    );
    assert!(
        context_paths
            .iter()
            .any(|path| path == "knowledge.base.backend.product"),
        "backend new knowledge base id should be appended to contextPaths: {context_paths:?}"
    );
}

#[tokio::test]
async fn composition_slots_should_work_over_http() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let agent_id = "agent.composition.http";
    create_agent(&app, agent_id, "Composition HTTP Agent").await;
    let create_uri = format!("/app/v3/api/ai/agents/{agent_id}/composition_slots");

    post_json(
        &app,
        create_uri.as_str(),
        json!({
            "slotId": "slot.document.invalid",
            "slotKind": "document",
            "targetModule": "drive",
            "targetRef": "document.invalid",
            "requestedAt": "2026-06-17T00:00:00Z"
        }),
        StatusCode::BAD_REQUEST,
    )
    .await;

    let create_body = json!({
        "slotId": "slot.document.product",
        "slotKind": "document",
        "targetModule": "documents",
        "targetRef": "document.product.specification",
        "priority": 1,
        "enabled": true,
        "policyJson": "{}",
        "requestedAt": "2026-06-17T00:00:01Z"
    });
    let create_response =
        post_json(&app, create_uri.as_str(), create_body, StatusCode::CREATED).await;
    assert_eq!(
        create_response["data"]["item"]["slotId"],
        json!("slot.document.product")
    );

    let list_uri = format!("/app/v3/api/ai/agents/{agent_id}/composition_slots");
    let list_response = get_json(&app, list_uri.as_str(), StatusCode::OK).await;
    assert_eq!(list_response["data"]["items"].as_array().unwrap().len(), 1);

    let slot_id = "slot.document.product";
    let get_uri = format!("/app/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}");
    let get_response = get_json(&app, get_uri.as_str(), StatusCode::OK).await;
    assert_eq!(
        get_response["data"]["item"]["targetRef"],
        json!("document.product.specification")
    );

    let update_body = json!({
        "expectedVersion": create_response["data"]["item"]["version"],
        "targetRef": "document.product.specification.v2",
        "requestedAt": "2026-06-17T00:01:00Z"
    });
    let update_uri = format!("/app/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}");
    let update_response = patch_json(&app, update_uri.as_str(), update_body, StatusCode::OK).await;
    assert_eq!(
        update_response["data"]["item"]["targetRef"],
        json!("document.product.specification.v2")
    );

    let delete_uri = format!("/app/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}");
    let delete_request = Request::builder()
        .method("DELETE")
        .uri(delete_uri)
        .body(Body::empty())
        .expect("delete request should be built");
    let delete_response = app
        .clone()
        .oneshot(delete_request)
        .await
        .expect("delete request should succeed");
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
    let delete_body = to_bytes(delete_response.into_body(), usize::MAX)
        .await
        .expect("delete body should be readable");
    assert!(delete_body.is_empty());
}

#[tokio::test]
async fn agent_tasks_should_work_over_http() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let agent_id = "agent.tasks.http";
    create_agent(&app, agent_id, "Task HTTP Agent").await;

    let create_body = json!({
        "title": "Nightly sync",
        "prompt": "Summarize tenant activity",
        "metadataJson": "{\"deferExecution\":true}",
        "requestedAt": "2026-06-17T00:00:00Z"
    });
    let create_uri = format!("/app/v3/api/ai/agents/{agent_id}/tasks");
    let create_response =
        post_json(&app, create_uri.as_str(), create_body, StatusCode::CREATED).await;
    let task_id = create_response["data"]["item"]["taskId"]
        .as_str()
        .expect("task id")
        .to_string();
    assert_eq!(create_response["data"]["item"]["status"], json!("pending"));

    let list_uri = format!("/app/v3/api/ai/agents/{agent_id}/tasks");
    let list_response = get_json(&app, list_uri.as_str(), StatusCode::OK).await;
    assert_eq!(list_response["data"]["items"].as_array().unwrap().len(), 1);

    let get_uri = format!("/app/v3/api/ai/agents/{agent_id}/tasks/{task_id}");
    let get_response = get_json(&app, get_uri.as_str(), StatusCode::OK).await;
    assert_eq!(
        get_response["data"]["item"]["prompt"],
        json!("Summarize tenant activity")
    );

    let cancel_body = json!({
        "expectedVersion": get_response["data"]["item"]["version"],
        "requestedAt": "2026-06-17T00:01:00Z"
    });
    let cancel_uri = format!("/app/v3/api/ai/agents/{agent_id}/tasks/{task_id}/cancel");
    let cancel_response = post_json(&app, cancel_uri.as_str(), cancel_body, StatusCode::OK).await;
    assert_eq!(
        cancel_response["data"]["item"]["status"],
        json!("cancelled")
    );
}

#[tokio::test]
async fn agent_tasks_execute_should_complete_deferred_task_over_http() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let agent_id = "agent.tasks.execute.http";
    create_agent(&app, agent_id, "Task Execute HTTP Agent").await;

    let create_body = json!({
        "title": "Deferred job",
        "prompt": "Summarize tenant activity",
        "metadataJson": "{\"deferExecution\":true}",
        "requestedAt": "2026-06-17T00:00:00Z"
    });
    let create_uri = format!("/app/v3/api/ai/agents/{agent_id}/tasks");
    let create_response =
        post_json(&app, create_uri.as_str(), create_body, StatusCode::CREATED).await;
    let task_id = create_response["data"]["item"]["taskId"]
        .as_str()
        .expect("task id")
        .to_string();
    assert_eq!(create_response["data"]["item"]["status"], json!("pending"));

    let get_uri = format!("/app/v3/api/ai/agents/{agent_id}/tasks/{task_id}");
    let get_response = get_json(&app, get_uri.as_str(), StatusCode::OK).await;

    let execute_body = json!({
        "expectedVersion": get_response["data"]["item"]["version"],
        "requestedAt": "2026-06-17T00:01:00Z"
    });
    let execute_uri = format!("/app/v3/api/ai/agents/{agent_id}/tasks/{task_id}/execute");
    let execute_response =
        post_json(&app, execute_uri.as_str(), execute_body, StatusCode::OK).await;
    assert_eq!(
        execute_response["data"]["item"]["status"],
        json!("completed")
    );
}

#[tokio::test]
async fn agent_interactions_should_work_over_http() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let agent_id = "agent.interactions.http";
    create_agent(&app, agent_id, "Interaction HTTP Agent").await;

    let session_id = "session.interactions.http";
    create_app_session(
        &app,
        agent_id,
        session_id,
        "HTTP interaction test",
        "2026-06-17T00:00:00Z",
    )
    .await;

    let create_body = json!({
        "kind": "approval",
        "prompt": "Allow file write to /tmp/demo.txt?",
        "requestedAt": "2026-06-17T00:00:01Z"
    });
    let create_uri = format!("/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/interactions");
    let create_response =
        post_json(&app, create_uri.as_str(), create_body, StatusCode::CREATED).await;
    let interaction_id = create_response["data"]["item"]["interactionId"]
        .as_str()
        .expect("interaction id")
        .to_string();
    assert_eq!(create_response["data"]["item"]["status"], json!("pending"));

    post_json(
        &app,
        create_uri.as_str(),
        json!({
            "kind": "user_question",
            "prompt": "Which implementation should be used?",
            "options": [{ "value": "safe", "label": "Safe" }],
            "requestedAt": "2026-06-17T00:00:01.500Z"
        }),
        StatusCode::CREATED,
    )
    .await;

    let list_uri = format!(
        "/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/interactions?kind=user_question&status=pending&page=1&page_size=1"
    );
    let list_response = get_json(&app, list_uri.as_str(), StatusCode::OK).await;
    assert_eq!(list_response["data"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(list_response["data"]["items"][0]["kind"], "user_question");
    assert_eq!(list_response["data"]["items"][0]["status"], "pending");
    assert_eq!(list_response["data"]["pageInfo"]["page"], 1);
    assert_eq!(list_response["data"]["pageInfo"]["pageSize"], 1);
    assert_eq!(list_response["data"]["pageInfo"]["totalItems"], "1");

    for forbidden_query in [
        "pageSize=1",
        "limit=1",
        "page_no=1",
        "pageNo=1",
        "per_page=1",
        "size=1",
        "cursor=0",
    ] {
        get_json(
            &app,
            &format!(
                "/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/interactions?{forbidden_query}"
            ),
            StatusCode::BAD_REQUEST,
        )
        .await;
    }
    get_json(
        &app,
        &format!(
            "/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/interactions?page_size=201"
        ),
        StatusCode::BAD_REQUEST,
    )
    .await;

    let get_uri = format!(
        "/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/interactions/{interaction_id}"
    );
    let get_response = get_json(&app, get_uri.as_str(), StatusCode::OK).await;
    assert_eq!(get_response["data"]["item"]["kind"], json!("approval"));

    let claim_response = post_json(
        &app,
        &format!(
            "/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/interactions/{interaction_id}/claim"
        ),
        json!({
            "claimOwner": "worker.http-contract",
            "expectedVersion": get_response["data"]["item"]["version"],
            "requestedAt": "2026-06-17T00:00:02Z"
        }),
        StatusCode::OK,
    )
    .await;
    let approve_body = json!({
        "approved": true,
        "claimToken": claim_response["data"]["item"]["claimToken"],
        "fencingToken": claim_response["data"]["item"]["fencingToken"],
        "expectedVersion": claim_response["data"]["item"]["interaction"]["version"],
        "requestedAt": "2026-06-17T00:00:03Z"
    });
    let approve_uri = format!(
        "/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/interactions/{interaction_id}/approve"
    );
    let approve_response =
        post_json(&app, approve_uri.as_str(), approve_body, StatusCode::OK).await;
    assert_eq!(
        approve_response["data"]["item"]["status"],
        json!("resolved")
    );

    let pending_approval_response = get_json(
        &app,
        &format!(
            "/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/interactions?kind=approval&status=pending&page=1&page_size=20"
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        pending_approval_response["data"]["items"]
            .as_array()
            .map(Vec::len),
        Some(0)
    );
}

#[tokio::test]
async fn provider_bindings_should_work_over_http() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.rig.http", "RigHttp").await;

    let add_binding_request = Request::builder()
        .method("POST")
        .uri("/app/v3/api/ai/agents/agent.rig.http/provider_bindings")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "bindingId": "binding.rig.default",
                "providerId": "provider.model.rig-rust",
                "implementationKind": "typed-local-provider",
                "configurationProfileId": "profile.rig.local",
                "capabilities": ["model.chat", "tool.invoke"],
                "makeDefault": true,
                "requestedAt": "2026-06-01T00:10:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let add_binding_response = app
        .clone()
        .oneshot(add_binding_request)
        .await
        .expect("add binding request should succeed");
    assert_eq!(add_binding_response.status(), StatusCode::CREATED);
    let body_bytes = to_bytes(add_binding_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(
        body_json["data"]["item"]["bindingId"],
        "binding.rig.default"
    );
    assert_eq!(
        body_json["data"]["item"]["providerId"],
        "provider.model.rig-rust"
    );
    assert_eq!(
        body_json["data"]["item"]["implementationKind"],
        "typed-local-provider"
    );
    assert_eq!(body_json["data"]["item"]["active"], true);

    let activate_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.rig.http/provider_bindings/binding.rig.default/activate")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T00:11:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let activate_response = app
        .clone()
        .oneshot(activate_request)
        .await
        .expect("activate request should succeed");
    assert_eq!(activate_response.status(), StatusCode::OK);

    let list_bindings_request = Request::builder()
        .method("GET")
        .uri("/app/v3/api/ai/agents/agent.rig.http/provider_bindings")
        .body(Body::empty())
        .expect("request should be built");
    let list_bindings_response = app
        .clone()
        .oneshot(list_bindings_request)
        .await
        .expect("list bindings request should succeed");
    assert_eq!(list_bindings_response.status(), StatusCode::OK);
    let body_bytes = to_bytes(list_bindings_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(
        body_json["data"]["items"]
            .as_array()
            .map(|items| items.len()),
        Some(1)
    );
}

#[tokio::test]
async fn app_create_agent_should_accept_implementation_type() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let mut body = create_body(
        "agent.implementation.http.langgraph",
        "ImplementationHttpLangGraph",
        "2026-06-01T00:20:00Z",
    );
    body["implementationProviderId"] = json!("provider.agent.langgraph");
    body["implementationKind"] = json!("protocol-adapter");
    body["implementationType"] = json!("langgraph");

    let response = post_json(&app, "/app/v3/api/ai/agents", body, StatusCode::CREATED).await;

    assert_eq!(
        response["data"]["item"]["implementationProviderId"],
        "provider.agent.langgraph"
    );
    assert_eq!(
        response["data"]["item"]["implementationKind"],
        "protocol-adapter"
    );
    assert_eq!(response["data"]["item"]["implementationType"], "langgraph");
}

#[tokio::test]
async fn backend_update_agent_should_change_implementation_type() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(
        &app,
        "agent.implementation.http.update",
        "ImplementationHttpUpdate",
    )
    .await;

    let response = patch_json(
        &app,
        "/backend/v3/api/ai/agents/agent.implementation.http.update",
        json!({
            "implementationProviderId": "provider.agent.openai",
            "implementationKind": "process-adapter",
            "implementationType": "openai-agents",
            "requestedAt": "2026-06-01T00:21:00Z"
        }),
        StatusCode::OK,
    )
    .await;

    assert_eq!(
        response["data"]["item"]["implementationProviderId"],
        "provider.agent.openai"
    );
    assert_eq!(
        response["data"]["item"]["implementationKind"],
        "process-adapter"
    );
    assert_eq!(
        response["data"]["item"]["implementationType"],
        "openai-agents"
    );
}

#[tokio::test]
async fn app_create_agent_with_invalid_implementation_type_should_return_bad_request() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let mut body = create_body(
        "agent.invalid.implementation-type",
        "InvalidImplementationType",
        "2026-06-01T00:22:00Z",
    );
    body["implementationType"] = json!("unsupported-framework");

    let response = post_json(&app, "/app/v3/api/ai/agents", body, StatusCode::BAD_REQUEST).await;

    assert_eq!(response["code"], 40001);
    assert!(response["detail"]
        .as_str()
        .expect("detail should exist")
        .contains("implementationType must be one of"));
}

#[tokio::test]
async fn provider_bindings_should_apply_pagination_contract() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.rig.paged", "RigPaged").await;

    for (binding_id, requested_at) in [
        ("binding.rig.beta", "2026-06-01T00:11:00Z"),
        ("binding.rig.alpha", "2026-06-01T00:11:00Z"),
    ] {
        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents/agent.rig.paged/provider_bindings")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "bindingId": binding_id,
                    "providerId": "provider.model.rig-rust",
                    "implementationKind": "typed-local-provider",
                    "configurationProfileId": "profile.rig.local",
                    "capabilities": ["model.chat"],
                    "makeDefault": false,
                    "requestedAt": requested_at
                })
                .to_string(),
            ))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("add binding request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let request = Request::builder()
        .method("GET")
        .uri("/app/v3/api/ai/agents/agent.rig.paged/provider_bindings?page=1&page_size=1")
        .body(Body::empty())
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("list bindings request should succeed");
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(
        body_json["data"]["items"].as_array().map(|v| v.len()),
        Some(1)
    );
    assert_eq!(
        body_json["data"]["items"][0]["bindingId"],
        "binding.rig.alpha"
    );
    assert_eq!(body_json["data"]["pageInfo"]["page"], 1);
    assert_eq!(body_json["data"]["pageInfo"]["pageSize"], 1);
    assert_eq!(body_json["data"]["pageInfo"]["totalItems"], "2");
    assert_eq!(body_json["data"]["pageInfo"]["totalPages"], 2);
}

#[tokio::test]
async fn provider_binding_list_missing_agent_should_return_not_found() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    for uri in ["/app/v3/api/ai/agents/agent.missing/provider_bindings"] {
        let request = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("list request should succeed");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/problem+json")
        );
        let body_bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let body_json: Value =
            serde_json::from_slice(&body_bytes).expect("response body should be valid json");
        assert_eq!(body_json["code"], 40401);
        assert_eq!(body_json["detail"], "agent not found");
    }
}

#[tokio::test]
async fn app_agent_preview_response_should_use_agent_runtime_api_contract() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.preview.runtime", "Preview Runtime").await;

    let request = Request::builder()
        .method("POST")
        .uri("/app/v3/api/ai/agents/agent.preview.runtime/preview_responses")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "executionId": "execution.preview.runtime.1",
                "content": "hello",
                "debugMode": true,
                "memoryEnabled": false,
                "model": "model.local",
                "temperature": 0.2,
                "inputPayload": {
                    "agent": {
                        "id": "agent.preview.runtime",
                        "name": "Preview Runtime"
                    },
                    "content": "hello"
                },
                "requestedAt": "2026-06-01T00:20:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("preview request should succeed");
    assert_eq!(response.status(), StatusCode::CREATED);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(
        body_json["data"]["item"]["executionId"],
        "execution.preview.runtime.1"
    );
    assert_eq!(
        body_json["data"]["item"]["agentId"],
        "agent.preview.runtime"
    );
    assert_eq!(body_json["data"]["item"]["operation"], "preview_response");
    assert_eq!(body_json["data"]["item"]["status"], "completed");
    assert_eq!(
        body_json["data"]["item"]["outputPayload"]["content"],
        "hello"
    );
    assert_eq!(
        body_json["data"]["item"]["outputPayload"]["debugMode"],
        true
    );
}

#[tokio::test]
async fn app_agent_prompt_optimization_should_use_agent_runtime_api_contract() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.prompt.runtime", "Prompt Runtime").await;

    let request = Request::builder()
        .method("POST")
        .uri("/app/v3/api/ai/agents/agent.prompt.runtime/prompt_optimizations")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "executionId": "execution.prompt.runtime.1",
                "prompt": "  answer the user clearly  ",
                "inputPayload": {
                    "agent": {
                        "id": "agent.prompt.runtime",
                        "name": "Prompt Runtime"
                    },
                    "prompt": "answer the user clearly"
                },
                "requestedAt": "2026-06-01T00:21:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("prompt optimization request should succeed");
    assert_eq!(response.status(), StatusCode::CREATED);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(
        body_json["data"]["item"]["executionId"],
        "execution.prompt.runtime.1"
    );
    assert_eq!(body_json["data"]["item"]["agentId"], "agent.prompt.runtime");
    assert_eq!(
        body_json["data"]["item"]["operation"],
        "prompt_optimization"
    );
    assert_eq!(body_json["data"]["item"]["status"], "completed");
    assert_eq!(
        body_json["data"]["item"]["outputPayload"]["optimizedPrompt"],
        "answer the user clearly"
    );
}

#[tokio::test]
async fn app_agent_runtime_execution_missing_agent_should_return_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("POST")
        .uri("/app/v3/api/ai/agents/agent.runtime.missing/preview_responses")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "executionId": "execution.preview.missing.1",
                "content": "hello",
                "requestedAt": "2026-06-01T00:22:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("missing agent request should return problem detail");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/problem+json")
    );

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40401);
    assert_eq!(body_json["detail"], "agent not found");
}

#[tokio::test]
async fn provider_binding_activation_missing_agent_should_return_not_found() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.missing/provider_bindings/binding.rig.default/activate")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T00:11:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("activate request should return problem detail");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/problem+json")
    );
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40401);
    assert_eq!(body_json["detail"], "agent not found");
}

#[tokio::test]
async fn provider_binding_conflicts_should_return_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.rig.conflict", "RigConflict").await;

    let binding_body = json!({
        "bindingId": "binding.rig.default",
        "providerId": "provider.model.rig-rust",
        "implementationKind": "typed-local-provider",
        "configurationProfileId": "profile.rig.local",
        "capabilities": ["model.chat"],
        "makeDefault": true,
        "requestedAt": "2026-06-01T00:10:00Z"
    });
    for expected_status in [StatusCode::CREATED, StatusCode::CONFLICT] {
        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents/agent.rig.conflict/provider_bindings")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(binding_body.to_string()))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("binding request should return response");
        assert_eq!(response.status(), expected_status);
        if expected_status == StatusCode::CONFLICT {
            assert_eq!(
                response
                    .headers()
                    .get(CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok()),
                Some("application/problem+json")
            );
            let body_bytes = to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("response body should be readable");
            let body_json: Value =
                serde_json::from_slice(&body_bytes).expect("response body should be valid json");
            assert_eq!(body_json["code"], 40901);
            assert_eq!(body_json["detail"], "agent provider binding already exists");
        }
    }
}

#[tokio::test]
async fn provider_binding_invalid_standard_ids_should_return_bad_request() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.rig.invalid.ids", "RigInvalidIds").await;

    let request = Request::builder()
        .method("POST")
        .uri("/app/v3/api/ai/agents/agent.rig.invalid.ids/provider_bindings")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "bindingId": " binding.rig.default ",
                "providerId": "provider.model.rig-rust",
                "implementationKind": "typed-local-provider",
                "configurationProfileId": "profile.rig.local",
                "capabilities": ["model.chat"],
                "makeDefault": true,
                "requestedAt": "2026-06-01T00:10:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("binding request should return problem detail");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/problem+json")
    );
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40001);
    assert_eq!(
        body_json["detail"],
        "bindingId must not contain leading or trailing whitespace"
    );
}

#[tokio::test]
async fn provider_binding_invalid_capabilities_should_return_bad_request() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(
        &app,
        "agent.rig.invalid.capabilities",
        "RigInvalidCapabilities",
    )
    .await;

    let request = Request::builder()
        .method("POST")
        .uri("/app/v3/api/ai/agents/agent.rig.invalid.capabilities/provider_bindings")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "bindingId": "binding.rig.default",
                "providerId": "provider.model.rig-rust",
                "implementationKind": "typed-local-provider",
                "configurationProfileId": "profile.rig.local",
                "capabilities": ["model.chat", "model.chat"],
                "makeDefault": true,
                "requestedAt": "2026-06-01T00:10:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("binding request should return problem detail");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40001);
    assert_eq!(
        body_json["detail"],
        "capabilities must not contain duplicate capability id: model.chat"
    );
}

#[tokio::test]
async fn list_should_apply_pagination_contract() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.alpha", "Alpha").await;
    create_agent(&app, "agent.beta", "Beta").await;

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents?page=1&page_size=1")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("list request should succeed");
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");

    assert_eq!(
        body_json["data"]["items"].as_array().map(|v| v.len()),
        Some(1)
    );
    assert_eq!(body_json["data"]["pageInfo"]["page"], 1);
    assert_eq!(body_json["data"]["pageInfo"]["pageSize"], 1);
    assert_eq!(body_json["data"]["pageInfo"]["totalItems"], "2");
    assert_eq!(body_json["data"]["pageInfo"]["totalPages"], 2);
}

#[tokio::test]
async fn list_should_apply_search_query_filter() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.search.alpha", "Alpha Search").await;
    create_agent(&app, "agent.search.beta", "Beta Search").await;

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents?q=beta")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("search list request should succeed");
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");

    let items = body_json["data"]["items"]
        .as_array()
        .expect("items should be array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["agentId"], "agent.search.beta");
    assert_eq!(body_json["data"]["pageInfo"]["totalItems"], "1");
}

#[tokio::test]
async fn delete_agent_without_body_should_return_no_content() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.gamma", "Gamma").await;

    let request = Request::builder()
        .method("DELETE")
        .uri("/app/v3/api/ai/agents/agent.gamma")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("delete request should succeed");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    assert!(body_bytes.is_empty());
}

#[tokio::test]
async fn create_with_invalid_requested_at_should_return_bad_request() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("POST")
        .uri("/app/v3/api/ai/agents")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            create_body("agent.invalid.time", "InvalidTime", "2026-06-01").to_string(),
        ))
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should return validation error");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40001);
    assert!(body_json["detail"]
        .as_str()
        .expect("detail should exist")
        .contains("requestedAt"));
}

#[tokio::test]
async fn create_with_invalid_implementation_provider_id_should_return_bad_request() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let mut body = create_body(
        "agent.invalid.implementation-provider",
        "InvalidImplementationProvider",
        "2026-06-01T03:00:00Z",
    );
    body["implementationProviderId"] = json!("model.rig-rust");
    body["implementationKind"] = json!("typed-local-provider");

    let request = Request::builder()
        .method("POST")
        .uri("/app/v3/api/ai/agents")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("create should return problem detail");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40001);
    assert_eq!(
        body_json["detail"],
        "implementationProviderId must start with provider."
    );
}

#[tokio::test]
async fn create_duplicate_agent_should_return_conflict() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.dup.conflict", "DupConflict").await;

    let duplicate_request = Request::builder()
        .method("POST")
        .uri("/app/v3/api/ai/agents")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            create_body("agent.dup.conflict", "DupConflict", "2026-06-01T03:00:00Z").to_string(),
        ))
        .expect("request should be built");

    let duplicate_response = app
        .clone()
        .oneshot(duplicate_request)
        .await
        .expect("duplicate create should return conflict");
    assert_eq!(duplicate_response.status(), StatusCode::CONFLICT);
    assert_eq!(
        duplicate_response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );

    let body_bytes = to_bytes(duplicate_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40901);
}

#[tokio::test]
async fn create_agent_with_non_standard_agent_id_should_return_bad_request() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("POST")
        .uri("/app/v3/api/ai/agents")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            create_body("pc.agent.invalid", "InvalidAgent", "2026-06-01T03:30:00Z").to_string(),
        ))
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("invalid agent id create should return problem detail");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40001);
    assert_eq!(body_json["detail"], "agentId must start with agent.");
}

#[tokio::test]
async fn restore_with_invalid_requested_at_should_return_bad_request() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.restore.invalid-time", "RestoreInvalidTime").await;

    let delete_request = Request::builder()
        .method("DELETE")
        .uri("/app/v3/api/ai/agents/agent.restore.invalid-time")
        .body(Body::empty())
        .expect("request should be built");
    let delete_response = app
        .clone()
        .oneshot(delete_request)
        .await
        .expect("delete request should succeed");
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let restore_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.restore.invalid-time/restore")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "expectedVersion": "1",
                "requestedAt": "2026-06-01"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let restore_response = app
        .clone()
        .oneshot(restore_request)
        .await
        .expect("restore request should return validation error");
    assert_eq!(restore_response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        restore_response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );
}

#[tokio::test]
async fn app_restore_should_restore_deleted_agent() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.restore.app", "RestoreApp").await;

    let delete_request = Request::builder()
        .method("DELETE")
        .uri("/app/v3/api/ai/agents/agent.restore.app")
        .body(Body::empty())
        .expect("request should be built");
    let delete_response = app
        .clone()
        .oneshot(delete_request)
        .await
        .expect("delete request should succeed");
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let restore_request = Request::builder()
        .method("POST")
        .uri("/app/v3/api/ai/agents/agent.restore.app/restore")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "expectedVersion": "2",
                "requestedAt": "2026-06-01T03:01:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let restore_response = app
        .clone()
        .oneshot(restore_request)
        .await
        .expect("restore request should succeed");
    assert_eq!(restore_response.status(), StatusCode::OK);

    let body_bytes = to_bytes(restore_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["data"]["item"]["status"], "active");
}

#[tokio::test]
async fn backend_restore_should_restore_deleted_agent() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.restore.backend", "RestoreBackend").await;

    let delete_request = Request::builder()
        .method("DELETE")
        .uri("/app/v3/api/ai/agents/agent.restore.backend")
        .body(Body::empty())
        .expect("request should be built");
    let delete_response = app
        .clone()
        .oneshot(delete_request)
        .await
        .expect("delete request should succeed");
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let restore_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.restore.backend/restore")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "expectedVersion": "2",
                "requestedAt": "2026-06-01T04:01:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let restore_response = app
        .clone()
        .oneshot(restore_request)
        .await
        .expect("restore request should succeed");
    assert_eq!(restore_response.status(), StatusCode::OK);

    let body_bytes = to_bytes(restore_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["data"]["item"]["status"], "active");
}

#[tokio::test]
async fn update_with_matching_expected_version_should_succeed() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.expected.update", "ExpectedUpdate").await;

    let update_request = Request::builder()
        .method("PATCH")
        .uri("/backend/v3/api/ai/agents/agent.expected.update")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "displayName": "ExpectedUpdateV2",
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T05:10:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let update_response = app
        .clone()
        .oneshot(update_request)
        .await
        .expect("update request should succeed");
    assert_eq!(update_response.status(), StatusCode::OK);

    let body_bytes = to_bytes(update_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["data"]["item"]["displayName"], "ExpectedUpdateV2");
    assert_eq!(body_json["data"]["item"]["version"], "2");
}

#[tokio::test]
async fn update_with_stale_expected_version_should_return_conflict() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.expected.stale", "ExpectedStale").await;

    let first_update = Request::builder()
        .method("PATCH")
        .uri("/backend/v3/api/ai/agents/agent.expected.stale")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "displayName": "ExpectedStaleV2",
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T05:20:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let first_update_response = app
        .clone()
        .oneshot(first_update)
        .await
        .expect("first update should succeed");
    assert_eq!(first_update_response.status(), StatusCode::OK);

    let stale_update = Request::builder()
        .method("PATCH")
        .uri("/backend/v3/api/ai/agents/agent.expected.stale")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "displayName": "ExpectedStaleV3",
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T05:21:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let stale_update_response = app
        .clone()
        .oneshot(stale_update)
        .await
        .expect("stale update should return conflict");
    assert_eq!(stale_update_response.status(), StatusCode::CONFLICT);
    assert_eq!(
        stale_update_response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );

    let body_bytes = to_bytes(stale_update_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40901);
    assert!(body_json["detail"]
        .as_str()
        .expect("detail should exist")
        .contains("version mismatch"));
}

#[tokio::test]
async fn status_update_with_stale_expected_version_should_return_version_conflict() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.expected.status", "ExpectedStatus").await;

    let first_status_update = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.expected.status/status")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "targetStatus": "active",
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T06:00:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let first_status_response = app
        .clone()
        .oneshot(first_status_update)
        .await
        .expect("first status update should succeed");
    assert_eq!(first_status_response.status(), StatusCode::OK);

    let stale_status_update = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.expected.status/status")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "targetStatus": "disabled",
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T06:01:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let stale_status_response = app
        .clone()
        .oneshot(stale_status_update)
        .await
        .expect("stale status update should return conflict");
    assert_eq!(stale_status_response.status(), StatusCode::CONFLICT);
    assert_eq!(
        stale_status_response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );

    let body_bytes = to_bytes(stale_status_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40901);
}

#[tokio::test]
async fn backend_audit_events_should_return_recorded_items() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit", "Audit").await;

    let status_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.audit/status")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "targetStatus": "active",
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T02:00:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let status_response = app
        .clone()
        .oneshot(status_request)
        .await
        .expect("status request should succeed");
    assert_eq!(status_response.status(), StatusCode::OK);

    let list_request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit/audit_events?page=1&page_size=10")
        .body(Body::empty())
        .expect("request should be built");
    let list_response = app
        .clone()
        .oneshot(list_request)
        .await
        .expect("audit list should succeed");
    assert_eq!(list_response.status(), StatusCode::OK);

    let body_bytes = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    let items = body_json["data"]["items"]
        .as_array()
        .expect("items should be array");
    assert!(!items.is_empty(), "audit list should not be empty");
    assert_eq!(body_json["data"]["pageInfo"]["totalItems"], "2");
}

#[tokio::test]
async fn backend_audit_events_action_filter_should_work() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.filter", "AuditFilter").await;

    let status_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.audit.filter/status")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "targetStatus": "active",
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T02:00:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let status_response = app
        .clone()
        .oneshot(status_request)
        .await
        .expect("status request should succeed");
    assert_eq!(status_response.status(), StatusCode::OK);

    let list_request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.filter/audit_events?action=status_changed")
        .body(Body::empty())
        .expect("request should be built");
    let list_response = app
        .clone()
        .oneshot(list_request)
        .await
        .expect("audit filter list should succeed");
    assert_eq!(list_response.status(), StatusCode::OK);

    let body_bytes = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    let items = body_json["data"]["items"]
        .as_array()
        .expect("items should be array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["eventType"], "agent.business.status_changed");
    assert_eq!(body_json["data"]["pageInfo"]["totalItems"], "1");
}

#[tokio::test]
async fn backend_audit_events_should_filter_provider_binding_actions() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.rig", "AuditRig").await;

    let binding_request = Request::builder()
        .method("POST")
        .uri("/app/v3/api/ai/agents/agent.audit.rig/provider_bindings")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "bindingId": "binding.rig.audit",
                "providerId": "provider.model.rig-rust",
                "implementationKind": "typed-local-provider",
                "configurationProfileId": "profile.rig.local",
                "capabilities": ["model.chat"],
                "makeDefault": true,
                "requestedAt": "2026-06-01T02:10:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let binding_response = app
        .clone()
        .oneshot(binding_request)
        .await
        .expect("binding request should succeed");
    assert_eq!(binding_response.status(), StatusCode::CREATED);

    for (action, event_type, _payload_fragment) in [(
        "provider_binding_changed",
        "agent.business.provider_binding_changed",
        "binding_id=binding.rig.audit",
    )] {
        let list_request = Request::builder()
            .method("GET")
            .uri(format!(
                "/backend/v3/api/ai/agents/agent.audit.rig/audit_events?action={action}"
            ))
            .body(Body::empty())
            .expect("request should be built");
        let list_response = app
            .clone()
            .oneshot(list_request)
            .await
            .expect("audit filter list should succeed");
        assert_eq!(list_response.status(), StatusCode::OK);

        let body_bytes = to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let body_json: Value =
            serde_json::from_slice(&body_bytes).expect("response body should be valid json");
        let items = body_json["data"]["items"]
            .as_array()
            .expect("items should be array");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["eventType"], event_type);
        assert!(
            items[0]["payload"]
                .as_str()
                .expect("payload should be string")
                .contains("binding.rig.audit"),
            "payload should include binding_id: {}",
            items[0]["payload"]
        );
        assert_eq!(body_json["data"]["pageInfo"]["totalItems"], "1");
    }
}

#[tokio::test]
async fn backend_audit_events_invalid_action_should_return_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.invalid", "AuditInvalid").await;

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.invalid/audit_events?action=oops")
        .body(Body::empty())
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should return problem detail");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );
}

#[tokio::test]
async fn backend_audit_events_time_range_filter_should_work() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.time", "AuditTime").await;

    let status_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.audit.time/status")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "targetStatus": "active",
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T02:00:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let status_response = app
        .clone()
        .oneshot(status_request)
        .await
        .expect("status request should succeed");
    assert_eq!(status_response.status(), StatusCode::OK);

    let list_request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.time/audit_events?from=2026-06-01T01:00:00Z&to=2026-06-01T03:00:00Z")
        .body(Body::empty())
        .expect("request should be built");
    let list_response = app
        .clone()
        .oneshot(list_request)
        .await
        .expect("audit range list should succeed");
    assert_eq!(list_response.status(), StatusCode::OK);

    let body_bytes = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    let items = body_json["data"]["items"]
        .as_array()
        .expect("items should be array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["eventType"], "agent.business.status_changed");
}

#[tokio::test]
async fn backend_audit_events_invalid_from_should_return_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.badfrom", "AuditBadFrom").await;

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.badfrom/audit_events?from=2026-06-01")
        .body(Body::empty())
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should return problem detail");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn backend_audit_events_from_after_to_should_return_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.rangeerr", "AuditRangeErr").await;

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.rangeerr/audit_events?from=2026-06-01T03:00:00Z&to=2026-06-01T01:00:00Z")
        .body(Body::empty())
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should return problem detail");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn backend_audit_events_page_zero_should_return_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.page.zero", "AuditPageZero").await;

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.page.zero/audit_events?page=0")
        .body(Body::empty())
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should return problem detail");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40001);
}

#[tokio::test]
async fn backend_audit_events_page_size_above_max_should_return_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.page.size", "AuditPageSize").await;

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.page.size/audit_events?page_size=201")
        .body(Body::empty())
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should return problem detail");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40001);
}

#[tokio::test]
async fn backend_audit_events_should_support_combined_filters_with_pagination() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent_at(
        &app,
        "agent.audit.combo",
        "AuditCombo",
        "2026-06-01T00:10:00Z",
    )
    .await;

    let status_active_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.audit.combo/status")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "targetStatus": "active",
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T00:20:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let status_active_response = app
        .clone()
        .oneshot(status_active_request)
        .await
        .expect("status request should succeed");
    assert_eq!(status_active_response.status(), StatusCode::OK);

    let status_disabled_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.audit.combo/status")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "targetStatus": "disabled",
                "expectedVersion": "2",
                "requestedAt": "2026-06-01T00:30:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let status_disabled_response = app
        .clone()
        .oneshot(status_disabled_request)
        .await
        .expect("status request should succeed");
    assert_eq!(status_disabled_response.status(), StatusCode::OK);

    let list_request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.combo/audit_events?action=status_changed&from=2026-06-01T00:15:00Z&to=2026-06-01T00:35:00Z&page=1&page_size=1")
        .body(Body::empty())
        .expect("request should be built");
    let list_response = app
        .clone()
        .oneshot(list_request)
        .await
        .expect("audit filter list should succeed");
    assert_eq!(list_response.status(), StatusCode::OK);

    let body_bytes = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    let items = body_json["data"]["items"]
        .as_array()
        .expect("items should be array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["eventType"], "agent.business.status_changed");
    assert_eq!(items[0]["occurredAt"], "2026-06-01T00:30:00Z");
    assert_eq!(body_json["data"]["pageInfo"]["page"], 1);
    assert_eq!(body_json["data"]["pageInfo"]["pageSize"], 1);
    assert_eq!(body_json["data"]["pageInfo"]["totalItems"], "2");
    assert_eq!(body_json["data"]["pageInfo"]["totalPages"], 2);
}

#[tokio::test]
async fn backend_audit_events_should_sort_by_instant_desc_across_timezones() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    create_agent_at(
        &app,
        "agent.audit.offset",
        "AuditOffset",
        "2026-06-01T09:00:00+08:00",
    )
    .await;

    let status_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.audit.offset/status")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "targetStatus": "active",
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T01:00:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let status_response = app
        .clone()
        .oneshot(status_request)
        .await
        .expect("status request should succeed");
    assert_eq!(status_response.status(), StatusCode::OK);

    let list_request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.offset/audit_events")
        .body(Body::empty())
        .expect("request should be built");
    let list_response = app
        .clone()
        .oneshot(list_request)
        .await
        .expect("audit list should succeed");
    assert_eq!(list_response.status(), StatusCode::OK);

    let body_bytes = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    let items = body_json["data"]["items"]
        .as_array()
        .expect("items should be array");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["eventType"], "agent.business.status_changed");
    assert_eq!(items[0]["occurredAt"], "2026-06-01T01:00:00Z");
    assert_eq!(items[1]["eventType"], "agent.business.created");
    assert_eq!(items[1]["occurredAt"], "2026-06-01T09:00:00+08:00");
}

#[tokio::test]
async fn invalid_query_should_return_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/app/v3/api/ai/agents?page=oops")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should return problem detail");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40001);
}

#[tokio::test]
async fn retrieve_missing_agent_should_return_not_found_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.missing")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should return problem detail");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40401);
}

#[tokio::test]
async fn permission_denied_should_return_permission_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_deny_policy_provider(),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/app/v3/api/ai/agents")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should return problem detail");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40301);
}

#[tokio::test]
async fn delete_missing_agent_should_return_not_found_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("DELETE")
        .uri("/app/v3/api/ai/agents/agent.missing.delete")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T08:00:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should return problem detail");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40401);
}

#[tokio::test]
async fn status_missing_agent_should_return_not_found_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.missing.status/status")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "targetStatus": "active",
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T08:01:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should return problem detail");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40401);
}

#[tokio::test]
async fn restore_missing_agent_should_return_not_found_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.missing.restore/restore")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T08:02:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should return problem detail");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40401);
}

#[tokio::test]
async fn backend_audit_events_permission_denied_should_return_forbidden_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_deny_policy_provider(),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.denied/audit_events")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should return problem detail");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["code"], 40301);
}

#[tokio::test]
async fn backend_route_should_isolate_reads_by_subject_tenant() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let tenant_one_app = build_test_app(state.clone());
    create_agent(&tenant_one_app, "agent.tenant.one", "Tenant One").await;

    let tenant_two_context = AgentRequestContext::new("2", "200")
        .with_organization_id("0")
        .with_subject_id("200")
        .with_roles(["ai.agents.manage"]);
    let tenant_two_app = build_test_app_with_context(state, tenant_two_context);

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents")
        .body(Body::empty())
        .expect("request should be built");
    let response = tenant_two_app
        .oneshot(request)
        .await
        .expect("request should succeed");
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["data"]["items"], json!([]));
}

#[tokio::test]
async fn backend_route_should_accept_trusted_subject_tenant_context() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    get_json(&app, "/backend/v3/api/ai/agents", StatusCode::OK).await;
}

#[tokio::test]
async fn app_workspace_initialization_scopes_projects_and_imports_idempotently() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let ensured = post_json(
        &app,
        "/app/v3/api/ai/workspaces/default",
        json!({ "name": "My Workspace" }),
        StatusCode::CREATED,
    )
    .await;
    let workspace_id = ensured["data"]["item"]["workspaceId"]
        .as_str()
        .expect("default Workspace id")
        .to_string();
    assert_eq!(workspace_id, "workspace.default.100");
    assert_eq!(ensured["data"]["item"]["isDefault"], true);

    let ensured_again = post_json(
        &app,
        "/app/v3/api/ai/workspaces/default",
        json!({}),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(ensured_again["data"]["item"]["workspaceId"], workspace_id);

    let workspaces = get_json(
        &app,
        "/app/v3/api/ai/workspaces?page=1&page_size=20",
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        workspaces["data"]["items"].as_array().map(Vec::len),
        Some(1)
    );

    let created = post_json(
        &app,
        "/app/v3/api/ai/projects",
        json!({
            "projectId": "project.workspace.scoped",
            "workspaceId": workspace_id,
            "name": "Workspace scoped",
            "visibility": "private",
            "driveAccessMode": "owner_library"
        }),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(
        created["data"]["item"]["workspaceId"],
        "workspace.default.100"
    );

    let imported = post_json(
        &app,
        "/app/v3/api/ai/projects/import",
        json!({
            "workspaceId": "workspace.default.100",
            "name": "Drive sandbox",
            "sourceKind": "drive_sandbox",
            "sourceRef": "drive://space.alpha/root.alpha",
            "driveSpaceId": "space.alpha",
            "driveRootEntryId": "root.alpha",
            "driveLogicalPath": "/sandbox"
        }),
        StatusCode::OK,
    )
    .await;
    let imported_project_id = imported["data"]["item"]["projectId"]
        .as_str()
        .expect("imported Project id")
        .to_string();
    assert_eq!(imported["data"]["item"]["driveSpaceId"], "space.alpha");

    let imported_again = post_json(
        &app,
        "/app/v3/api/ai/projects/import",
        json!({
            "workspaceId": "workspace.default.100",
            "name": "Drive sandbox duplicate",
            "sourceKind": "drive_sandbox",
            "sourceRef": "drive://space.alpha/root.alpha",
            "driveSpaceId": "space.alpha",
            "driveRootEntryId": "root.alpha",
            "driveLogicalPath": "/sandbox"
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        imported_again["data"]["item"]["projectId"],
        imported_project_id
    );

    let listed = get_json(
        &app,
        "/app/v3/api/ai/projects?workspace_id=workspace.default.100&page=1&page_size=20",
        StatusCode::OK,
    )
    .await;
    assert_eq!(listed["data"]["items"].as_array().map(Vec::len), Some(2));
    assert!(listed["data"]["items"]
        .as_array()
        .expect("Project items")
        .iter()
        .all(|item| item["workspaceId"] == "workspace.default.100"));
}

#[tokio::test]
async fn app_workspace_lifecycle_should_create_retrieve_update_archive_and_delete() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    post_json(
        &app,
        "/app/v3/api/ai/workspaces/default",
        json!({}),
        StatusCode::CREATED,
    )
    .await;
    let created = post_json(
        &app,
        "/app/v3/api/ai/workspaces",
        json!({ "name": "Client Workspace" }),
        StatusCode::CREATED,
    )
    .await;
    let workspace_id = created["data"]["item"]["workspaceId"]
        .as_str()
        .expect("Workspace id")
        .to_string();
    assert_eq!(created["data"]["item"]["isDefault"], false);

    let retrieved = get_json(
        &app,
        &format!("/app/v3/api/ai/workspaces/{workspace_id}"),
        StatusCode::OK,
    )
    .await;
    assert_eq!(retrieved["data"]["item"]["name"], "Client Workspace");

    let updated = patch_json(
        &app,
        &format!("/app/v3/api/ai/workspaces/{workspace_id}"),
        json!({
            "expectedVersion": "0",
            "name": "Renamed Workspace"
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(updated["data"]["item"]["version"], "1");

    let archived = post_json(
        &app,
        &format!("/app/v3/api/ai/workspaces/{workspace_id}/archive"),
        json!({ "expectedVersion": "1" }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(archived["data"]["item"]["status"], "archived");

    let request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/app/v3/api/ai/workspaces/{workspace_id}?expectedVersion=2"
        ))
        .body(Body::empty())
        .expect("delete request should be built");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("delete request should succeed");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let workspaces = get_json(
        &app,
        "/app/v3/api/ai/workspaces?page=1&page_size=20&status=active",
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        workspaces["data"]["items"].as_array().map(Vec::len),
        Some(1)
    );
}

#[tokio::test]
async fn app_project_crud_should_be_versioned_listed_archived_and_deleted() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let project_id = "project.http.commercial";

    let created = post_json(
        &app,
        "/app/v3/api/ai/projects",
        json!({
            "projectId": project_id,
            "name": "Commercial workspace",
            "description": "HTTP project contract",
            "visibility": "private",
            "driveAccessMode": "owner_library"
        }),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(created["data"]["item"]["projectId"], project_id);
    assert_eq!(created["data"]["item"]["version"], "0");

    let retrieved = get_json(
        &app,
        &format!("/app/v3/api/ai/projects/{project_id}"),
        StatusCode::OK,
    )
    .await;
    assert_eq!(retrieved["data"]["item"]["name"], "Commercial workspace");

    let updated = patch_json(
        &app,
        &format!("/app/v3/api/ai/projects/{project_id}"),
        json!({
            "expectedVersion": created["data"]["item"]["version"],
            "name": "Commercial workspace renamed"
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        updated["data"]["item"]["name"],
        "Commercial workspace renamed"
    );
    assert_eq!(updated["data"]["item"]["version"], "1");

    let listed = get_json(
        &app,
        "/app/v3/api/ai/projects?q=renamed&page=1&page_size=20",
        StatusCode::OK,
    )
    .await;
    assert_eq!(listed["data"]["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(listed["data"]["items"][0]["projectId"], project_id);

    let archived = post_json(
        &app,
        &format!("/app/v3/api/ai/projects/{project_id}/archive"),
        json!({
            "expectedVersion": updated["data"]["item"]["version"]
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(archived["data"]["item"]["status"], "archived");
    assert_eq!(archived["data"]["item"]["version"], "2");

    request_without_body(
        &app,
        "DELETE",
        &format!("/app/v3/api/ai/projects/{project_id}"),
        StatusCode::NO_CONTENT,
    )
    .await;
    get_json(
        &app,
        &format!("/app/v3/api/ai/projects/{project_id}"),
        StatusCode::NOT_FOUND,
    )
    .await;
}

#[tokio::test]
async fn app_project_composition_slot_crud_should_match_generated_sdk_contract() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let project_id = "project.http.composition";
    let slot_id = "slot.documents";

    post_json(
        &app,
        "/app/v3/api/ai/projects",
        json!({
            "projectId": project_id,
            "name": "Composition workspace",
            "visibility": "private",
            "driveAccessMode": "explicit_resources"
        }),
        StatusCode::CREATED,
    )
    .await;

    post_json(
        &app,
        &format!("/app/v3/api/ai/projects/{project_id}/composition_slots"),
        json!({
            "slotId": "slot.document.invalid",
            "slotKind": "document",
            "targetModule": "drive",
            "targetRef": "document.invalid"
        }),
        StatusCode::BAD_REQUEST,
    )
    .await;

    let created = post_json(
        &app,
        &format!("/app/v3/api/ai/projects/{project_id}/composition_slots"),
        json!({
            "slotId": slot_id,
            "slotKind": "document",
            "targetModule": "documents",
            "targetRef": "document.project.specification",
            "targetVersionRef": "document.version.1",
            "priority": 10,
            "enabled": true,
            "policyJson": "{\"mode\":\"system\"}"
        }),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(created["data"]["item"]["version"], "0");

    let listed = get_json(
        &app,
        &format!(
            "/app/v3/api/ai/projects/{project_id}/composition_slots?slot_kind=document&enabled=true&page=1&page_size=20"
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(listed["data"]["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(listed["data"]["items"][0]["slotId"], slot_id);

    let retrieved = get_json(
        &app,
        &format!("/app/v3/api/ai/projects/{project_id}/composition_slots/{slot_id}"),
        StatusCode::OK,
    )
    .await;
    assert_eq!(retrieved["data"]["item"]["targetModule"], "documents");

    let updated = patch_json(
        &app,
        &format!("/app/v3/api/ai/projects/{project_id}/composition_slots/{slot_id}"),
        json!({
            "expectedVersion": "0",
            "enabled": false,
            "clearTargetVersionRef": true
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(updated["data"]["item"]["version"], "1");
    assert_eq!(updated["data"]["item"]["enabled"], false);
    assert!(updated["data"]["item"]["targetVersionRef"].is_null());

    let missing_version_request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/app/v3/api/ai/projects/{project_id}/composition_slots/{slot_id}"
        ))
        .body(Body::empty())
        .expect("delete request should be built");
    let missing_version_response = app
        .clone()
        .oneshot(missing_version_request)
        .await
        .expect("delete request should complete");
    assert_eq!(missing_version_response.status(), StatusCode::BAD_REQUEST);
    request_without_body(
        &app,
        "DELETE",
        &format!(
            "/app/v3/api/ai/projects/{project_id}/composition_slots/{slot_id}?expected_version=1"
        ),
        StatusCode::NO_CONTENT,
    )
    .await;
}

#[tokio::test]
async fn app_turn_should_replay_same_idempotency_payload_and_reject_conflicts() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let agent_id = "agent.turn.replay.http";
    create_agent(&app, agent_id, "Replay Turn HTTP Agent").await;

    let session_id = "session.turn.replay.http";
    create_app_session(
        &app,
        agent_id,
        session_id,
        "Idempotent HTTP turn",
        "2026-06-28T12:00:00Z",
    )
    .await;
    let runtime_binding_id = create_turn_runtime(&app, agent_id, session_id, "replay-http").await;

    let uri = format!("/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/turns");
    let payload = json!({
        "content": "One agent turn",
        "turnMode": "interactive",
        "runtimeBindingId": runtime_binding_id,
        "idempotencyKey": "idem.http.complete.1",
        "payloadHash": "sha256:http-complete-1",
        "clientRequestId": "client.http.complete.1",
        "requestedAt": "2026-06-28T12:00:01Z"
    });
    let first = post_json(&app, &uri, payload.clone(), StatusCode::OK).await;
    let replay = post_json(&app, &uri, payload, StatusCode::OK).await;
    assert_eq!(
        replay["data"]["item"]["items"][0]["itemId"],
        first["data"]["item"]["items"][0]["itemId"]
    );
    assert_eq!(
        replay["data"]["item"]["items"][1]["itemId"],
        first["data"]["item"]["items"][1]["itemId"]
    );

    let conflict = post_json(
        &app,
        &uri,
        json!({
            "content": "A different payload",
            "turnMode": "interactive",
            "runtimeBindingId": runtime_binding_id,
            "idempotencyKey": "idem.http.complete.1",
            "payloadHash": "sha256:http-complete-2",
            "clientRequestId": "client.http.complete.2",
            "requestedAt": "2026-06-28T12:00:02Z"
        }),
        StatusCode::CONFLICT,
    )
    .await;
    assert_eq!(conflict["title"], "Conflict");
}

#[tokio::test]
async fn app_session_should_support_flat_create_rename_project_move_filter_and_delete() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let agent_id = "agent.session.commercial.http";
    create_agent(&app, agent_id, "Commercial Session Agent").await;
    let project_id = "project.session.commercial";
    post_json(
        &app,
        "/app/v3/api/ai/projects",
        json!({
            "projectId": project_id,
            "name": "Session project"
        }),
        StatusCode::CREATED,
    )
    .await;

    let session_id = "session.commercial.http";
    let created = post_json(
        &app,
        &format!("/app/v3/api/ai/agents/{agent_id}/sessions"),
        json!({
            "sessionId": session_id,
            "sessionKind": "assistant",
            "entrySurface": "api",
            "title": "Unsorted session",
            "idempotencyKey": "create-session-commercial-http",
            "payloadHash": "sha256:create-session-commercial-http",
            "requestedAt": "2026-06-28T12:00:00Z"
        }),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(created["data"]["item"]["sessionId"], session_id);
    assert_eq!(created["data"]["item"]["lastItemSequence"], "0");

    let wrong_agent_id = "agent.session.commercial.other";
    create_agent(&app, wrong_agent_id, "Other Commercial Session Agent").await;
    let mismatch = patch_json(
        &app,
        &format!("/app/v3/api/ai/agents/{wrong_agent_id}/sessions/{session_id}"),
        json!({
            "expectedVersion": created["data"]["item"]["version"],
            "title": "Must not change"
        }),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(mismatch["code"], 40401);
    let close_mismatch = post_json(
        &app,
        &format!("/app/v3/api/ai/agents/{wrong_agent_id}/sessions/{session_id}/close"),
        json!({
            "expectedVersion": created["data"]["item"]["version"],
            "requestedAt": "2026-06-28T12:00:01Z"
        }),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(close_mismatch["code"], 40401);
    get_json(
        &app,
        &format!("/app/v3/api/ai/agents/{wrong_agent_id}/sessions/{session_id}/items"),
        StatusCode::NOT_FOUND,
    )
    .await;

    let delete_request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/app/v3/api/ai/agents/{wrong_agent_id}/sessions/{session_id}"
        ))
        .body(Body::empty())
        .expect("delete mismatch request should be built");
    let delete_response = app
        .clone()
        .oneshot(delete_request)
        .await
        .expect("delete mismatch request should complete");
    assert_eq!(delete_response.status(), StatusCode::NOT_FOUND);

    let moved = patch_json(
        &app,
        &format!("/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}"),
        json!({
            "expectedVersion": created["data"]["item"]["version"],
            "title": "Project session",
            "projectId": project_id
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(moved["data"]["item"]["projectId"], project_id);
    assert_eq!(moved["data"]["item"]["title"], "Project session");

    let listed = get_json(
        &app,
        &format!(
            "/app/v3/api/ai/agents/{agent_id}/sessions?project_id={project_id}&page=1&page_size=20"
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(listed["data"]["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(listed["data"]["items"][0]["sessionId"], session_id);

    request_without_body(
        &app,
        "DELETE",
        &format!("/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}"),
        StatusCode::NO_CONTENT,
    )
    .await;
    get_json(
        &app,
        &format!("/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}"),
        StatusCode::NOT_FOUND,
    )
    .await;
}

#[tokio::test]
async fn app_turn_should_return_ordered_session_items() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let agent_id = "agent.turn.http";
    create_agent(&app, agent_id, "Turn HTTP Agent").await;

    let session_id = "session.turn.http";
    let session_response = create_app_session(
        &app,
        agent_id,
        session_id,
        "HTTP turn test",
        "2026-06-28T12:00:00Z",
    )
    .await;
    assert_eq!(session_response["data"]["item"]["sessionId"], session_id);
    let runtime_binding_id = create_turn_runtime(&app, agent_id, session_id, "turn-http").await;

    let completion = post_json(
        &app,
        &format!("/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/turns"),
        json!({
            "content": "Hello over HTTP",
            "turnMode": "interactive",
            "runtimeBindingId": runtime_binding_id,
            "idempotencyKey": "turn-http-1",
            "payloadHash": "sha256:turn-http-1",
            "requestedAt": "2026-06-28T12:00:01Z"
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(completion["data"]["item"]["items"][0]["kind"], "user_input");
    assert_eq!(
        completion["data"]["item"]["items"][0]["content"],
        "Hello over HTTP"
    );
    assert_eq!(
        completion["data"]["item"]["items"][1]["kind"],
        "assistant_output"
    );
    assert!(completion["data"]["item"]["items"][1]["content"]
        .as_str()
        .unwrap_or("")
        .contains("Hello over HTTP"));

    let newest_page = get_json(
        &app,
        &format!(
            "/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/items?status=completed&sort=-sequence&page=1&page_size=1"
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(newest_page["data"]["items"][0]["kind"], "assistant_output");
    assert_eq!(newest_page["data"]["pageInfo"]["page"], 1);
    assert_eq!(newest_page["data"]["pageInfo"]["pageSize"], 1);
    assert_eq!(newest_page["data"]["pageInfo"]["totalItems"], "2");
    assert_eq!(newest_page["data"]["pageInfo"]["hasMore"], true);

    let filtered_page = get_json(
        &app,
        &format!(
            "/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/items?kind=user_input&status=completed&sort=sequence&page=1&page_size=1"
        ),
        StatusCode::OK,
    )
    .await;
    assert_eq!(filtered_page["data"]["items"][0]["kind"], "user_input");
    assert_eq!(filtered_page["data"]["pageInfo"]["totalItems"], "1");

    get_json(
        &app,
        &format!("/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/items?sort=createdAt"),
        StatusCode::BAD_REQUEST,
    )
    .await;
}

#[tokio::test]
async fn app_turn_stream_should_return_typed_delta_and_completion_events() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let agent_id = "agent.turn.stream.http";
    create_agent(&app, agent_id, "Stream Turn HTTP Agent").await;

    let session_id = "session.turn.stream.http";
    create_app_session(
        &app,
        agent_id,
        session_id,
        "HTTP stream turn test",
        "2026-06-28T12:00:00Z",
    )
    .await;
    let runtime_binding_id =
        create_turn_runtime(&app, agent_id, session_id, "turn-stream-http").await;
    let payload = json!({
        "content": "Hello over SSE",
        "turnMode": "interactive",
        "runtimeBindingId": runtime_binding_id,
        "idempotencyKey": "turn-stream-http-1",
        "payloadHash": "sha256:turn-stream-http-1",
        "requestedAt": "2026-06-28T12:00:01Z"
    });
    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/turns?stream=true"
        ))
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .expect("stream turn request should be built");
    let response = app
        .oneshot(request)
        .await
        .expect("stream turn request should complete");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("stream turn body should be readable");
    let body = String::from_utf8(body.to_vec()).expect("stream turn body should be UTF-8");
    let events = body
        .lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .map(|data| serde_json::from_str::<Value>(data).expect("SSE data must be JSON"))
        .collect::<Vec<_>>();
    assert!(!events.is_empty());
    for (index, event) in events.iter().take(events.len() - 1).enumerate() {
        assert_eq!(event["eventType"], "delta");
        assert_eq!(event["index"], index);
        assert!(event["delta"].is_string());
    }
    let completion = events.last().expect("completion event should exist");
    assert_eq!(completion["eventType"], "completion");
    assert_eq!(completion["response"]["code"], 0);
    assert_eq!(
        completion["response"]["data"]["item"]["turn"]["sessionId"],
        session_id
    );
}

#[tokio::test]
async fn open_api_turn_should_use_trusted_tenant_scope() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let agent_id = "agent.turn.open";
    create_agent(&app, agent_id, "Open Turn Agent").await;

    let session_id = "session.turn.open";
    create_app_session(
        &app,
        agent_id,
        session_id,
        "Open API turn",
        "2026-06-28T12:00:00Z",
    )
    .await;
    let runtime_binding_id = create_turn_runtime(&app, agent_id, session_id, "turn-open").await;

    let completion = post_json(
        &app,
        &format!("/agent/v3/api/ai/agents/{agent_id}/sessions/{session_id}/turns"),
        json!({
            "content": "Open API hello",
            "turnMode": "interactive",
            "runtimeBindingId": runtime_binding_id,
            "idempotencyKey": "turn-open-1",
            "payloadHash": "sha256:turn-open-1",
            "requestedAt": "2026-06-28T12:00:01Z"
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        completion["data"]["item"]["items"][1]["kind"],
        "assistant_output"
    );
}

#[tokio::test]
async fn backend_archive_session_should_transition_status() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let agent_id = "agent.archive.session";
    create_agent(&app, agent_id, "Archive Session Agent").await;

    let session_id = "session.archive.backend";
    let session_response = create_app_session(
        &app,
        agent_id,
        session_id,
        "Archive test session",
        "2026-06-28T12:00:00Z",
    )
    .await;
    let expected_version = session_response["data"]["item"]["version"]
        .as_str()
        .expect("session version");

    let closed = post_json(
        &app,
        &format!("/backend/v3/api/ai/agents/{agent_id}/sessions/{session_id}/close"),
        json!({
            "expectedVersion": expected_version,
            "requestedAt": "2026-06-28T12:00:00Z"
        }),
        StatusCode::OK,
    )
    .await;
    let closed_version = closed["data"]["item"]["version"]
        .as_str()
        .expect("closed session version");

    let archived = post_json(
        &app,
        &format!("/backend/v3/api/ai/agents/{agent_id}/sessions/{session_id}/archive"),
        json!({
            "expectedVersion": closed_version,
            "requestedAt": "2026-06-28T12:00:00Z"
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(archived["data"]["item"]["sessionId"], session_id);
    assert_eq!(archived["data"]["item"]["status"], "archived");
}

#[tokio::test]
async fn app_session_activity_snapshot_supports_newest_first_cursor_and_scope_binding() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let agent_id = "agent.activity.http";
    create_agent(&app, agent_id, "Activity HTTP Agent").await;
    let low_session = create_app_session(
        &app,
        agent_id,
        "session.activity.http.low",
        "Low",
        "2099-07-27T10:00:00Z",
    )
    .await;
    create_app_session(
        &app,
        agent_id,
        "session.activity.http.high",
        "High",
        "2099-07-27T10:00:00Z",
    )
    .await;

    let (status, first) = get_json_response(
        &app,
        "/app/v3/api/ai/session_activity_summaries?page_size=1",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["data"]["pageInfo"]["mode"], "cursor");
    assert_eq!(first["data"]["pageInfo"]["hasMore"], true);
    assert_eq!(
        first["data"]["items"][0]["session"]["sessionId"],
        "session.activity.http.high"
    );
    let cursor = first["data"]["pageInfo"]["nextCursor"]
        .as_str()
        .expect("next cursor")
        .to_string();

    let (status, second) = get_json_response(
        &app,
        &format!("/app/v3/api/ai/session_activity_summaries?page_size=1&cursor={cursor}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        second["data"]["items"][0]["session"]["sessionId"],
        "session.activity.http.low"
    );

    let (status, problem) = get_json_response(
        &app,
        &format!(
            "/app/v3/api/ai/session_activity_summaries?page_size=1&agent_id={agent_id}&cursor={cursor}"
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(problem["code"], 40003);

    post_json(
        &app,
        &format!("/app/v3/api/ai/agents/{agent_id}/sessions/session.activity.http.low/close"),
        json!({
            "expectedVersion": low_session["data"]["item"]["version"],
            "requestedAt": "2099-07-27T11:00:00Z"
        }),
        StatusCode::OK,
    )
    .await;
    let (status, exhausted) = get_json_response(
        &app,
        &format!("/app/v3/api/ai/session_activity_summaries?page_size=1&cursor={cursor}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(exhausted["data"]["items"]
        .as_array()
        .expect("items")
        .is_empty());
    assert_eq!(exhausted["data"]["pageInfo"]["hasMore"], false);
    assert_eq!(exhausted["data"]["pageInfo"]["nextCursor"], cursor);

    let (_, converged) = get_json_response(
        &app,
        "/app/v3/api/ai/session_activity_summaries?page_size=1",
    )
    .await;
    assert_eq!(
        converged["data"]["items"][0]["session"]["sessionId"],
        "session.activity.http.low"
    );

    create_app_session(
        &app,
        agent_id,
        "session.activity.http.new-head",
        "New head",
        "2099-07-27T12:00:00Z",
    )
    .await;
    let (_, refreshed) = get_json_response(
        &app,
        "/app/v3/api/ai/session_activity_summaries?page_size=1",
    )
    .await;
    assert_eq!(
        refreshed["data"]["items"][0]["session"]["sessionId"],
        "session.activity.http.new-head"
    );
}

#[tokio::test]
async fn app_session_activity_snapshot_rejects_invalid_pagination_parameters() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    for page_size in [0, 201] {
        let (status, problem) = get_json_response(
            &app,
            &format!("/app/v3/api/ai/session_activity_summaries?page_size={page_size}"),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(problem["code"], 40003);
    }

    for alias in [
        "page=1",
        "pageSize=20",
        "limit=20",
        "page_no=1",
        "pageNo=1",
        "per_page=20",
        "size=20",
    ] {
        let (status, _) = get_json_response(
            &app,
            &format!("/app/v3/api/ai/session_activity_summaries?{alias}"),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "alias accepted: {alias}");
    }

    for empty_filter in ["workspace_id=", "project_id=", "agent_id="] {
        let (status, problem) = get_json_response(
            &app,
            &format!("/app/v3/api/ai/session_activity_summaries?{empty_filter}"),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "empty filter accepted: {empty_filter}"
        );
        assert_eq!(problem["code"], 40003);
    }

    let oversized_cursor = "a".repeat(2049);
    let (status, problem) = get_json_response(
        &app,
        &format!("/app/v3/api/ai/session_activity_summaries?cursor={oversized_cursor}"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(problem["code"], 40003);
}

#[tokio::test]
async fn app_session_activity_snapshot_preserves_latest_failed_runtime_binding() {
    let repository = InMemoryAgentRepository::new();
    let agent_id = "agent.activity.failed.binding";
    let session_id = "session.activity.failed.binding";
    repository
        .insert_session(AgentSessionRecord {
            id: 1,
            session_id: session_id.to_string(),
            tenant_id: 100001,
            organization_id: 0,
            agent_id: agent_id.to_string(),
            owner_user_id: 100,
            project_id: None,
            session_kind: AgentSessionKind::Assistant,
            entry_surface: AgentSessionEntrySurface::Api,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: Some("Failed binding".to_string()),
            status: AgentSessionStatus::Active,
            item_count: 0,
            last_item_sequence: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            idempotency_key: None,
            payload_hash: None,
            created_by: 100,
            updated_by: 100,
            version: 0,
            created_at: "2099-07-27T10:00:00Z".to_string(),
            updated_at: "2099-07-27T10:00:00Z".to_string(),
            last_item_at: None,
            closed_at: None,
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        })
        .expect("session should persist");
    repository
        .insert_session_runtime_binding(AgentSessionRuntimeBindingRecord {
            id: 2,
            tenant_id: 100001,
            organization_id: 0,
            session_id: session_id.to_string(),
            runtime_binding_id: "runtime_binding.activity.failed".to_string(),
            runtime_location_id: None,
            host_mode: "managed".to_string(),
            transport_kind: "in_process".to_string(),
            provider_binding_id: "binding.activity.failed".to_string(),
            model_id: "model.activity.failed".to_string(),
            provider_id: "provider.activity.failed".to_string(),
            provider_session_id: None,
            provider_session_tree_id: None,
            provider_parent_session_id: None,
            provider_forked_from_session_id: None,
            status: AgentSessionRuntimeBindingStatus::Failed,
            is_current: false,
            version: 1,
            created_at: "2099-07-27T10:00:00Z".to_string(),
            updated_at: "2099-07-27T10:01:00Z".to_string(),
            activated_at: Some("2099-07-27T10:00:00Z".to_string()),
            deactivated_at: None,
        })
        .expect("failed current binding should persist");
    let state = AgentHttpState::new(
        repository,
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);

    let (status, response) = get_json_response(
        &app,
        &format!("/app/v3/api/ai/session_activity_summaries?agent_id={agent_id}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        response["data"]["items"][0]["latestRuntimeBinding"]["status"],
        "failed"
    );
    assert!(response["data"]["items"][0]["currentRuntimeBinding"].is_null());
    assert_eq!(response["data"]["items"][0]["presentationPhase"], "failed");
}

#[tokio::test]
async fn app_session_activity_snapshot_moves_user_state_updates_to_head() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let agent_id = "agent.activity.user.state";
    let low_session_id = "session.activity.user.state.low";
    create_agent(&app, agent_id, "User State Activity Agent").await;
    create_app_session(
        &app,
        agent_id,
        low_session_id,
        "Low before pin",
        "2026-06-01T10:00:00Z",
    )
    .await;
    create_app_session(
        &app,
        agent_id,
        "session.activity.user.state.high",
        "High before pin",
        "2026-06-02T10:00:00Z",
    )
    .await;

    patch_json(
        &app,
        &format!("/app/v3/api/ai/agents/{agent_id}/sessions/{low_session_id}/user_state"),
        json!({
            "expectedVersion": null,
            "pinned": true,
            "customTitle": "Pinned in another app"
        }),
        StatusCode::OK,
    )
    .await;

    let (status, response) = get_json_response(
        &app,
        &format!("/app/v3/api/ai/session_activity_summaries?agent_id={agent_id}&page_size=1"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let item = &response["data"]["items"][0];
    assert_eq!(item["session"]["sessionId"], low_session_id);
    assert_eq!(item["userState"]["resourceId"], low_session_id);
    assert_eq!(item["userState"]["customTitle"], "Pinned in another app");
    assert_eq!(item["freshness"]["source"], "user_state");
    assert_eq!(item["freshness"]["userStateVersion"], "0");
}

#[tokio::test]
async fn app_provider_session_without_live_evidence_is_unknown_not_ready() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        test_policy_provider(),
    );
    let app = build_test_app(state);
    let agent_id = "agent.activity.provider";
    create_agent(&app, agent_id, "Provider Activity Agent").await;
    post_json(
        &app,
        &format!("/app/v3/api/ai/agents/{agent_id}/provider_bindings"),
        json!({
            "bindingId": "binding.agent-provider.codex",
            "providerId": "provider.activity.codex",
            "implementationKind": "typed-local-provider",
            "configurationProfileId": "profile.activity.codex",
            "capabilities": ["model.chat"],
            "makeDefault": true,
            "requestedAt": "2099-07-27T10:00:00Z"
        }),
        StatusCode::CREATED,
    )
    .await;
    let session_id = "session.activity.provider";
    create_app_session(
        &app,
        agent_id,
        session_id,
        "Provider",
        "2099-07-27T10:01:00Z",
    )
    .await;
    post_json(
        &app,
        &format!("/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/runtime_bindings"),
        json!({
            "runtimeBindingId": "runtime_binding.activity.provider",
            "hostMode": "local",
            "transportKind": "provider-session-history",
            "providerBindingId": "binding.agent-provider.codex",
            "modelId": "model.activity.codex",
            "providerId": "provider.activity.codex",
            "providerSessionId": "provider.activity.missing",
            "requestedAt": "2099-07-27T10:02:00Z"
        }),
        StatusCode::CREATED,
    )
    .await;

    let (status, body) = get_json_response(
        &app,
        &format!("/app/v3/api/ai/session_activity_summaries?agent_id={agent_id}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let item = &body["data"]["items"][0];
    assert_eq!(item["presentationPhase"], "unknown");
    assert_eq!(item["providerActivity"]["freshness"], "unsupported");
    assert!(item["providerActivity"]["freshUntil"].is_null());
}
