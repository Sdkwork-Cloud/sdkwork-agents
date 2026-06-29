#![cfg(feature = "http-axum")]
//! Legacy gateway-trusted HTTP contract suite for managed agents managed store handlers.
//!
//! Production mounts use `sdkwork-routes-agent-*-api::build_served_router` with
//! `sdkwork-web-framework`. This file exercises handler contracts through
//! `build_combined_router()` and gateway subject headers.

use axum::body::{to_bytes, Body};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Request, StatusCode};
use axum::Extension;
use sdkwork_intelligence_agents_service::{
    build_combined_router, testing::test_web_context, AgentHttpState, AgentRequestContext,
    AllowAllPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository, PolicyMode,
};
use serde_json::{json, Value};
use tower::ServiceExt;

fn auth_headers(mut request: Request<Body>) -> Request<Body> {
    let headers = request.headers_mut();
    headers.insert("x-subject-id", HeaderValue::from_static("u-1"));
    headers.insert("x-subject-tenant-id", HeaderValue::from_static("100001"));
    request
}

fn test_agent_context() -> AgentRequestContext {
    AgentRequestContext::new("100001", "100")
        .with_organization_id("0")
        .with_subject_id("u-1")
        .with_roles(["agent.write", "agent.read"])
}

fn build_test_app(state: AgentHttpState) -> axum::Router {
    build_combined_router(state)
        .layer(Extension(test_agent_context()))
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
        .oneshot(auth_headers(request))
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
        .oneshot(auth_headers(request))
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
        .oneshot(auth_headers(request))
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
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/app/v3/api/ai/code_engines")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .oneshot(auth_headers(request))
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
async fn app_mcp_marketplace_should_return_records_array() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/app/v3/api/ai/mcp_servers")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .oneshot(auth_headers(request))
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
async fn app_create_agent_should_derive_scope_from_request_context() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    let response = post_json(
        &app,
        "/app/v3/api/ai/agents?tenant_id=999",
        json!({
            "agentId": "agent.context.scope",
            "organizationId": "999",
            "ownerUserId": "999",
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
        StatusCode::CREATED,
    )
    .await;

    assert_eq!(response["data"]["item"]["tenantId"], "100001");
    assert_eq!(response["data"]["item"]["organizationId"], "0");
    assert_eq!(response["data"]["item"]["ownerUserId"], "100");
}

#[tokio::test]
async fn app_agent_response_should_expose_pc_management_profile() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
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
        .oneshot(auth_headers(request))
        .await
        .expect("create request should succeed");
    assert_eq!(response.status(), StatusCode::CREATED);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");

    assert_eq!(body_json["data"]["item"]["managementProfile"]["avatar"], "robot");
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
        AllowAllPolicyProvider::allow("policy.memory"),
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

    assert_eq!(response["data"]["item"]["managementProfile"]["avatar"], "robot");
    assert_eq!(response["data"]["item"]["managementProfile"]["author"], "SDKWork");
    assert_eq!(
        response["data"]["item"]["managementProfile"]["knowledgeBaseIds"],
        json!(["knowledge.base.product", "knowledge.base.runbook"])
    );
    assert_eq!(response["data"]["item"]["managementProfile"]["type"], "independent");
    assert_eq!(response["data"]["item"]["managementProfile"]["users"], "12 users");
    assert_eq!(response["data"]["item"]["managementProfile"]["debugMode"], true);
    assert_eq!(response["data"]["item"]["managementProfile"]["jsonMode"], true);
    assert_eq!(response["data"]["item"]["managementProfile"]["memoryEnabled"], true);
    assert_eq!(
        response["data"]["item"]["managementProfile"]["model"],
        "model.openai.gpt-4"
    );
    assert_eq!(response["data"]["item"]["managementProfile"]["temperature"], 0.7);
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
        .oneshot(auth_headers(request))
        .await
        .expect("update request should succeed");
    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let response: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");

    assert_eq!(response["data"]["item"]["managementProfile"]["avatar"], "robot");
    assert_eq!(
        response["data"]["item"]["managementProfile"]["categoryId"],
        "assistant"
    );
    assert_eq!(response["data"]["item"]["managementProfile"]["type"], "independent");
    assert_eq!(response["data"]["item"]["managementProfile"]["debugMode"], false);
    assert_eq!(response["data"]["item"]["managementProfile"]["jsonMode"], true);
    assert_eq!(
        response["data"]["item"]["managementProfile"]["memoryEnabled"],
        false
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["model"],
        "model.anthropic.claude-sonnet"
    );
    assert_eq!(response["data"]["item"]["managementProfile"]["temperature"], 0.2);
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    let response = post_json(
        &app,
        "/backend/v3/api/ai/agents?tenant_id=100001",
        json!({
            "agentId": "agent.pc.backend.structured",
            "organizationId": "0",
            "ownerUserId": "100",
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

    assert_eq!(response["data"]["item"]["managementProfile"]["avatar"], "robot");
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
    assert_eq!(response["data"]["item"]["managementProfile"]["type"], "independent");
    assert_eq!(response["data"]["item"]["managementProfile"]["users"], "42 users");
    assert_eq!(response["data"]["item"]["managementProfile"]["debugMode"], true);
    assert_eq!(response["data"]["item"]["managementProfile"]["jsonMode"], false);
    assert_eq!(response["data"]["item"]["managementProfile"]["memoryEnabled"], true);
    assert_eq!(
        response["data"]["item"]["managementProfile"]["model"],
        "model.openai.gpt-4o"
    );
    assert_eq!(response["data"]["item"]["managementProfile"]["temperature"], 0.4);
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
        "/backend/v3/api/ai/agents?tenant_id=100001",
        json!({
            "agentId": "agent.pc.backend.update.structured",
            "organizationId": "0",
            "ownerUserId": "100",
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
        "/backend/v3/api/ai/agents/agent.pc.backend.update.structured?tenant_id=100001",
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

    assert_eq!(response["data"]["item"]["managementProfile"]["avatar"], "robot");
    assert_eq!(
        response["data"]["item"]["managementProfile"]["categoryId"],
        "assistant"
    );
    assert_eq!(response["data"]["item"]["managementProfile"]["type"], "independent");
    assert_eq!(response["data"]["item"]["managementProfile"]["debugMode"], false);
    assert_eq!(response["data"]["item"]["managementProfile"]["jsonMode"], true);
    assert_eq!(
        response["data"]["item"]["managementProfile"]["memoryEnabled"],
        false
    );
    assert_eq!(
        response["data"]["item"]["managementProfile"]["model"],
        "model.azure.gpt-4"
    );
    assert_eq!(response["data"]["item"]["managementProfile"]["temperature"], 0.1);
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);
    let agent_id = "agent.composition.http";
    create_agent(&app, agent_id, "Composition HTTP Agent").await;

    let create_body = json!({
        "data": {
            "tenantId": "100001",
            "organizationId": "0",
            "slotId": "slot.knowledge.product",
            "slotKind": "knowledge",
            "targetModule": "knowledgebase",
            "targetRef": "kb.space.product",
            "priority": "1",
            "enabled": true,
            "policyJson": "{}"
        },
        "requestedAt": "2026-06-17T00:00:00Z"
    });
    let create_uri = format!("/app/v3/api/ai/agents/{agent_id}/composition_slots");
    let create_response =
        post_json(&app, create_uri.as_str(), create_body, StatusCode::CREATED).await;
    assert_eq!(
        create_response["data"]["item"]["slotId"],
        json!("slot.knowledge.product")
    );

    let list_uri = format!("/app/v3/api/ai/agents/{agent_id}/composition_slots");
    let list_response = get_json(&app, list_uri.as_str(), StatusCode::OK).await;
    assert_eq!(list_response["data"]["items"].as_array().unwrap().len(), 1);

    let slot_id = "slot.knowledge.product";
    let get_uri = format!("/app/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}");
    let get_response = get_json(&app, get_uri.as_str(), StatusCode::OK).await;
    assert_eq!(get_response["data"]["item"]["targetRef"], json!("kb.space.product"));

    let update_body = json!({
        "data": {
            "tenantId": "100001",
            "targetRef": "kb.space.product.v2"
        },
        "requestedAt": "2026-06-17T00:01:00Z"
    });
    let update_uri = format!("/app/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}");
    let update_response = patch_json(&app, update_uri.as_str(), update_body, StatusCode::OK).await;
    assert_eq!(
        update_response["data"]["item"]["targetRef"],
        json!("kb.space.product.v2")
    );

    let delete_uri = format!(
        "/app/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}?expectedVersion={}&requestedAt=2026-06-17T00:02:00Z",
        update_response["data"]["item"]["version"].as_str().unwrap()
    );
    let delete_request = Request::builder()
        .method("DELETE")
        .uri(delete_uri)
        .body(Body::empty())
        .expect("delete request should be built");
    let delete_response = app
        .clone()
        .oneshot(auth_headers(delete_request))
        .await
        .expect("delete request should succeed");
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_body = to_bytes(delete_response.into_body(), usize::MAX)
        .await
        .expect("delete body should be readable");
    let delete_json: Value = serde_json::from_slice(&delete_body).expect("valid json");
    assert!(delete_json["data"]["item"]["deletedAt"].is_string());
}

#[tokio::test]
async fn provider_bindings_should_work_over_http() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
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
        .oneshot(auth_headers(add_binding_request))
        .await
        .expect("add binding request should succeed");
    assert_eq!(add_binding_response.status(), StatusCode::CREATED);
    let body_bytes = to_bytes(add_binding_response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["data"]["item"]["bindingId"], "binding.rig.default");
    assert_eq!(body_json["data"]["item"]["providerId"], "provider.model.rig-rust");
    assert_eq!(
        body_json["data"]["item"]["implementationKind"],
        "typed-local-provider"
    );
    assert_eq!(body_json["data"]["item"]["active"], true);

    let activate_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.rig.http/provider_bindings/binding.rig.default/activate?tenant_id=100001")
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
        .oneshot(auth_headers(activate_request))
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
        .oneshot(auth_headers(list_bindings_request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
    assert_eq!(response["data"]["item"]["implementationKind"], "protocol-adapter");
    assert_eq!(response["data"]["item"]["implementationType"], "langgraph");
}

#[tokio::test]
async fn backend_update_agent_should_change_implementation_type() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
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
        "/backend/v3/api/ai/agents/agent.implementation.http.update?tenant_id=100001",
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
    assert_eq!(response["data"]["item"]["implementationKind"], "process-adapter");
    assert_eq!(response["data"]["item"]["implementationType"], "openai-agents");
}

#[tokio::test]
async fn app_create_agent_with_invalid_implementation_type_should_return_bad_request() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
            .oneshot(auth_headers(request))
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
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
            .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(
        body_json["data"]["item"]["executionId"],
        "execution.preview.runtime.1"
    );
    assert_eq!(body_json["data"]["item"]["agentId"], "agent.preview.runtime");
    assert_eq!(body_json["data"]["item"]["operation"], "preview_response");
    assert_eq!(body_json["data"]["item"]["status"], "completed");
    assert_eq!(body_json["data"]["item"]["outputPayload"]["content"], "hello");
    assert_eq!(body_json["data"]["item"]["outputPayload"]["debugMode"], true);
}

#[tokio::test]
async fn app_agent_prompt_optimization_should_use_agent_runtime_api_contract() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
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
    assert_eq!(response.status(), StatusCode::OK);

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
    assert_eq!(body_json["data"]["item"]["operation"], "prompt_optimization");
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.missing/provider_bindings/binding.rig.default/activate?tenant_id=100001")
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
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
            .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.alpha", "Alpha").await;
    create_agent(&app, "agent.beta", "Beta").await;

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents?tenant_id=100001&page=1&page_size=1")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.search.alpha", "Alpha Search").await;
    create_agent(&app, "agent.search.beta", "Beta Search").await;

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents?tenant_id=100001&q=beta")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(auth_headers(request))
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
async fn missing_subject_header_should_return_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents?tenant_id=100001")
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
    assert_eq!(body_json["title"], "Bad Request");
    assert_eq!(body_json["status"], 400);
    assert_eq!(body_json["code"], 40001);
    assert!(body_json["detail"]
        .as_str()
        .expect("detail should exist")
        .contains("x-subject-id"));
}

#[tokio::test]
async fn delete_without_requested_at_should_return_bad_request() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.gamma", "Gamma").await;

    let request = Request::builder()
        .method("DELETE")
        .uri("/app/v3/api/ai/agents/agent.gamma")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from("{}"))
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(auth_headers(request))
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
}

#[tokio::test]
async fn create_with_invalid_requested_at_should_return_bad_request() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
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
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
        .oneshot(auth_headers(duplicate_request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.restore.invalid-time", "RestoreInvalidTime").await;

    let delete_request = Request::builder()
        .method("DELETE")
        .uri("/app/v3/api/ai/agents/agent.restore.invalid-time")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T04:00:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let delete_response = app
        .clone()
        .oneshot(auth_headers(delete_request))
        .await
        .expect("delete request should succeed");
    assert_eq!(delete_response.status(), StatusCode::OK);

    let restore_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.restore.invalid-time/restore?tenant_id=100001")
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
        .oneshot(auth_headers(restore_request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.restore.app", "RestoreApp").await;

    let delete_request = Request::builder()
        .method("DELETE")
        .uri("/app/v3/api/ai/agents/agent.restore.app")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T03:00:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let delete_response = app
        .clone()
        .oneshot(auth_headers(delete_request))
        .await
        .expect("delete request should succeed");
    assert_eq!(delete_response.status(), StatusCode::OK);

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
        .oneshot(auth_headers(restore_request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.restore.backend", "RestoreBackend").await;

    let delete_request = Request::builder()
        .method("DELETE")
        .uri("/app/v3/api/ai/agents/agent.restore.backend")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "expectedVersion": "1",
                "requestedAt": "2026-06-01T04:00:00Z"
            })
            .to_string(),
        ))
        .expect("request should be built");
    let delete_response = app
        .clone()
        .oneshot(auth_headers(delete_request))
        .await
        .expect("delete request should succeed");
    assert_eq!(delete_response.status(), StatusCode::OK);

    let restore_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.restore.backend/restore?tenant_id=100001")
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
        .oneshot(auth_headers(restore_request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.expected.update", "ExpectedUpdate").await;

    let update_request = Request::builder()
        .method("PATCH")
        .uri("/backend/v3/api/ai/agents/agent.expected.update?tenant_id=100001")
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
        .oneshot(auth_headers(update_request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.expected.stale", "ExpectedStale").await;

    let first_update = Request::builder()
        .method("PATCH")
        .uri("/backend/v3/api/ai/agents/agent.expected.stale?tenant_id=100001")
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
        .oneshot(auth_headers(first_update))
        .await
        .expect("first update should succeed");
    assert_eq!(first_update_response.status(), StatusCode::OK);

    let stale_update = Request::builder()
        .method("PATCH")
        .uri("/backend/v3/api/ai/agents/agent.expected.stale?tenant_id=100001")
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
        .oneshot(auth_headers(stale_update))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.expected.status", "ExpectedStatus").await;

    let first_status_update = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.expected.status/status?tenant_id=100001")
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
        .oneshot(auth_headers(first_status_update))
        .await
        .expect("first status update should succeed");
    assert_eq!(first_status_response.status(), StatusCode::OK);

    let stale_status_update = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.expected.status/status?tenant_id=100001")
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
        .oneshot(auth_headers(stale_status_update))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit", "Audit").await;

    let status_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.audit/status?tenant_id=100001")
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
        .oneshot(auth_headers(status_request))
        .await
        .expect("status request should succeed");
    assert_eq!(status_response.status(), StatusCode::OK);

    let list_request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit/audit_events?tenant_id=100001&page=1&page_size=10")
        .body(Body::empty())
        .expect("request should be built");
    let list_response = app
        .clone()
        .oneshot(auth_headers(list_request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.filter", "AuditFilter").await;

    let status_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.audit.filter/status?tenant_id=100001")
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
        .oneshot(auth_headers(status_request))
        .await
        .expect("status request should succeed");
    assert_eq!(status_response.status(), StatusCode::OK);

    let list_request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.filter/audit_events?tenant_id=100001&action=status_changed")
        .body(Body::empty())
        .expect("request should be built");
    let list_response = app
        .clone()
        .oneshot(auth_headers(list_request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
        .oneshot(auth_headers(binding_request))
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
                "/backend/v3/api/ai/agents/agent.audit.rig/audit_events?tenant_id=100001&action={action}"
            ))
            .body(Body::empty())
            .expect("request should be built");
        let list_response = app
            .clone()
            .oneshot(auth_headers(list_request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.invalid", "AuditInvalid").await;

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.invalid/audit_events?tenant_id=100001&action=oops")
        .body(Body::empty())
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.time", "AuditTime").await;

    let status_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.audit.time/status?tenant_id=100001")
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
        .oneshot(auth_headers(status_request))
        .await
        .expect("status request should succeed");
    assert_eq!(status_response.status(), StatusCode::OK);

    let list_request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.time/audit_events?tenant_id=100001&from=2026-06-01T01:00:00Z&to=2026-06-01T03:00:00Z")
        .body(Body::empty())
        .expect("request should be built");
    let list_response = app
        .clone()
        .oneshot(auth_headers(list_request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.badfrom", "AuditBadFrom").await;

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.badfrom/audit_events?tenant_id=100001&from=2026-06-01")
        .body(Body::empty())
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(auth_headers(request))
        .await
        .expect("request should return problem detail");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn backend_audit_events_from_after_to_should_return_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.rangeerr", "AuditRangeErr").await;

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.rangeerr/audit_events?tenant_id=100001&from=2026-06-01T03:00:00Z&to=2026-06-01T01:00:00Z")
        .body(Body::empty())
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(auth_headers(request))
        .await
        .expect("request should return problem detail");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn backend_audit_events_page_zero_should_return_problem_detail() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.page.zero", "AuditPageZero").await;

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.page.zero/audit_events?tenant_id=100001&page=0")
        .body(Body::empty())
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    create_agent(&app, "agent.audit.page.size", "AuditPageSize").await;

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.page.size/audit_events?tenant_id=100001&page_size=201")
        .body(Body::empty())
        .expect("request should be built");
    let response = app
        .clone()
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
        .uri("/backend/v3/api/ai/agents/agent.audit.combo/status?tenant_id=100001")
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
        .oneshot(auth_headers(status_active_request))
        .await
        .expect("status request should succeed");
    assert_eq!(status_active_response.status(), StatusCode::OK);

    let status_disabled_request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.audit.combo/status?tenant_id=100001")
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
        .oneshot(auth_headers(status_disabled_request))
        .await
        .expect("status request should succeed");
    assert_eq!(status_disabled_response.status(), StatusCode::OK);

    let list_request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.combo/audit_events?tenant_id=100001&action=status_changed&from=2026-06-01T00:15:00Z&to=2026-06-01T00:35:00Z&page=1&page_size=1")
        .body(Body::empty())
        .expect("request should be built");
    let list_response = app
        .clone()
        .oneshot(auth_headers(list_request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
        .uri("/backend/v3/api/ai/agents/agent.audit.offset/status?tenant_id=100001")
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
        .oneshot(auth_headers(status_request))
        .await
        .expect("status request should succeed");
    assert_eq!(status_response.status(), StatusCode::OK);

    let list_request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.offset/audit_events?tenant_id=100001")
        .body(Body::empty())
        .expect("request should be built");
    let list_response = app
        .clone()
        .oneshot(auth_headers(list_request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/app/v3/api/ai/agents?page=oops")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.missing?tenant_id=100001")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider {
            provider_id: "policy.memory".to_string(),
            mode: PolicyMode::Deny("agent.business.denied".to_string()),
        },
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/app/v3/api/ai/agents")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
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
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.missing.status/status?tenant_id=100001")
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
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("POST")
        .uri("/backend/v3/api/ai/agents/agent.missing.restore/restore?tenant_id=100001")
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
        .oneshot(auth_headers(request))
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
        AllowAllPolicyProvider {
            provider_id: "policy.memory".to_string(),
            mode: PolicyMode::Deny("agent.business.denied".to_string()),
        },
    );
    let app = build_test_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents/agent.audit.denied/audit_events?tenant_id=100001")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .clone()
        .oneshot(auth_headers(request))
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
async fn backend_route_should_reject_subject_tenant_mismatch() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_combined_router(state);

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents?tenant_id=100001")
        .header("x-subject-id", "u-1")
        .header("x-subject-tenant-id", "2")
        .body(Body::empty())
        .expect("request should be built");
    let response = app.oneshot(request).await.expect("request should succeed");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["title"], "Forbidden");
}

#[tokio::test]
async fn backend_route_should_accept_matching_subject_tenant_header() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_combined_router(state);

    get_json(
        &app,
        "/backend/v3/api/ai/agents?tenant_id=100001",
        StatusCode::OK,
    )
    .await;
}

#[tokio::test]
async fn backend_route_should_reject_missing_subject_headers() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_combined_router(state);

    let request = Request::builder()
        .method("GET")
        .uri("/backend/v3/api/ai/agents?tenant_id=100001")
        .body(Body::empty())
        .expect("request should be built");
    let response = app.oneshot(request).await.expect("request should succeed");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("response body should be valid json");
    assert_eq!(body_json["title"], "Bad Request");
}

#[tokio::test]
async fn app_chat_message_turn_should_return_completion() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);
    let agent_id = "agent.chat.http";
    create_agent(&app, agent_id, "Chat HTTP Agent").await;

    let session_id = "session.chat.http";
    let session_response = post_json(
        &app,
        &format!("/app/v3/api/ai/agents/{agent_id}/sessions"),
        json!({
            "data": {
                "tenantId": "0",
                "organizationId": "0",
                "ownerUserId": "0",
                "sessionId": session_id,
                "title": "HTTP chat test"
            },
            "requestedAt": "2026-06-28T12:00:00Z"
        }),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(session_response["data"]["item"]["sessionId"], session_id);

    let completion = post_json(
        &app,
        &format!("/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/messages"),
        json!({
            "content": "Hello over HTTP",
            "requestedAt": "2026-06-28T12:00:01Z"
        }),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(completion["data"]["item"]["userMessage"]["role"], "user");
    assert_eq!(completion["data"]["item"]["userMessage"]["content"], "Hello over HTTP");
    assert_eq!(completion["data"]["item"]["assistantMessage"]["role"], "assistant");
    assert!(completion["data"]["item"]["assistantMessage"]["content"]
        .as_str()
        .unwrap_or("")
        .contains("Hello over HTTP"));
}

#[tokio::test]
async fn open_api_chat_message_should_accept_body_tenant_id() {
    let state = AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.memory"),
    );
    let app = build_test_app(state);
    let agent_id = "agent.chat.open";
    create_agent(&app, agent_id, "Open Chat Agent").await;

    let session_id = "session.chat.open";
    post_json(
        &app,
        &format!("/app/v3/api/ai/agents/{agent_id}/sessions"),
        json!({
            "data": {
                "tenantId": "0",
                "organizationId": "0",
                "ownerUserId": "0",
                "sessionId": session_id
            },
            "requestedAt": "2026-06-28T12:00:00Z"
        }),
        StatusCode::CREATED,
    )
    .await;

    let completion = post_json(
        &app,
        &format!("/agent/v3/api/ai/agents/{agent_id}/sessions/{session_id}/messages"),
        json!({
            "tenantId": "100001",
            "content": "Open API hello",
            "requestedAt": "2026-06-28T12:00:01Z"
        }),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(completion["data"]["item"]["assistantMessage"]["role"], "assistant");
}
