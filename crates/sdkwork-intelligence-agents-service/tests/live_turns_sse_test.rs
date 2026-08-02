use std::sync::{Arc, Mutex};

use sdkwork_agent_kernel::{
    AgentManifest, KernelEvent, KernelResult, PolicySubject,
};
use sdkwork_code_kernel::CodeTaskIntent;
use sdkwork_intelligence_agents_service::{
    ActivateAgentProviderBindingCommand, AgentAuditSink, AgentBusinessStatus,
    AgentImplementationKind, AgentItemDriveRefInput, AgentItemResourceRole, AgentSessionEntrySurface,
    AgentSessionItemKind, AgentSessionKind, AgentTurnMode, AgentVisibility, AgentsService,
    AgentProviderBindingCommand, ChangeAgentStatusCommand, CreateAgentCommand,
    CreateSessionCommand, CreateSessionRuntimeBindingCommand, CreateTurnCommand,
    IamGatedPolicyProvider, InMemoryAgentRepository, RuntimeFacadeTurnExecutor,
    TurnExecutionStreamSink,
};

/// Live end-to-end proof: one real provider turn through the agents business
/// service stream sink (the same path the HTTP turns SSE endpoint uses).
///
/// Requires the `@opencode-ai/sdk` package in the kernel workspace node_modules
/// and a reachable opencode model provider configured through `OPENCODE_CONFIG`
/// (or `OPENCODE_MODEL`). Run with:
/// `OPENCODE_CONFIG=/tmp/opencode-test-config.json cargo test -p sdkwork-intelligence-agents-service --test live_turns_sse_test -- --ignored --nocapture`
#[test]
#[ignore = "requires the @opencode-ai/sdk package and a live opencode model provider"]
fn live_turns_sse_flow_with_real_opencode_provider() {
    std::env::set_var("SDKWORK_KERNEL_ENVIRONMENT", "development");
    std::env::set_var("SDKWORK_AGENT_SDK_WORKSPACE_ROOT", kernel_workspace_root());

    let repository = InMemoryAgentRepository::new();
    let (audit_sink, _events) = RecordingAuditSink::new();
    let policy_provider = IamGatedPolicyProvider::new("policy.agents.test.iam-gated");
    let service = AgentsService::new(repository, audit_sink, policy_provider)
        .with_turn_executor(Arc::new(RuntimeFacadeTurnExecutor));

    let created = service
        .create_agent(create_agent_cmd(
            "agent.live.opencode",
            100_001,
            0,
            100,
            "live-opencode",
            "Live OpenCode",
            "2026-08-01T00:00:00Z",
        ))
        .expect("create should succeed");
    service
        .change_status(ChangeAgentStatusCommand {
            tenant_id: 100_001,
            agent_id: created.agent_id.clone(),
            expected_version: Some(created.version),
            target_status: AgentBusinessStatus::Active,
            requested_by: sample_subject(),
            requested_at: "2026-08-01T00:00:30Z".to_string(),
        })
        .expect("activate agent should succeed");

    let session = service
        .create_session(CreateSessionCommand {
            tenant_id: 100_001,
            organization_id: 0,
            agent_id: created.agent_id.clone(),
            owner_user_id: 100,
            session_id: String::new(),
            project_id: None,
            session_kind: AgentSessionKind::Assistant,
            entry_surface: AgentSessionEntrySurface::Api,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: Some("Live SSE session".to_string()),
            idempotency_key: None,
            payload_hash: None,
            requested_by: sample_subject(),
            requested_at: "2026-08-01T00:01:00Z".to_string(),
        })
        .expect("create session should succeed");

    let provider_binding = service
        .add_provider_binding(AgentProviderBindingCommand {
            tenant_id: 100_001,
            agent_id: created.agent_id.clone(),
            binding_id: "binding.agent-provider.opencode".to_string(),
            provider_id: "provider.model.opencode".to_string(),
            implementation_kind: AgentImplementationKind::ManifestOnly,
            configuration_profile_id: "profile.live.opencode".to_string(),
            capabilities: Vec::new(),
            make_default: true,
            requested_by: sample_subject(),
            requested_at: "2026-08-01T00:01:10Z".to_string(),
        })
        .expect("provider binding should be created");
    service
        .activate_provider_binding(ActivateAgentProviderBindingCommand {
            tenant_id: 100_001,
            agent_id: created.agent_id.clone(),
            binding_id: provider_binding.binding_id.clone(),
            requested_by: sample_subject(),
            requested_at: "2026-08-01T00:01:15Z".to_string(),
        })
        .expect("provider binding should be activated");

    let runtime_binding = service
        .create_session_runtime_binding(CreateSessionRuntimeBindingCommand {
            tenant_id: 100_001,
            organization_id: 0,
            path_agent_id: created.agent_id.clone(),
            session_id: session.session_id.clone(),
            runtime_binding_id: Some("runtime_binding.live.opencode".to_string()),
            runtime_location_id: None,
            host_mode: "managed".to_string(),
            transport_kind: "in_process".to_string(),
            provider_binding_id: provider_binding.binding_id,
            model_id: configured_opencode_model_id(),
            provider_id: "provider.model.opencode".to_string(),
            provider_session_id: None,
            provider_session_tree_id: None,
            provider_parent_session_id: None,
            provider_forked_from_session_id: None,
            provider_directory: None,
            owner_scope: None,
            requested_by: sample_subject(),
            requested_at: "2026-08-01T00:01:20Z".to_string(),
        })
        .expect("session runtime binding should be created");

    let sink = RecordingTurnStreamSink::new();
    let turn_command = CreateTurnCommand {
        tenant_id: 100_001,
        organization_id: 0,
        agent_id: created.agent_id.clone(),
        session_id: session.session_id.clone(),
        turn_id: Some("turn.live.sse.one".to_string()),
        content: "Reply with exactly one word: OK".to_string(),
        content_type: "text/plain".to_string(),
        turn_mode: AgentTurnMode::Interactive,
        runtime_binding_id: Some(runtime_binding.runtime_binding_id),
        requested_model_id: Some(configured_opencode_model_id()),
        access_mode_id: None,
        idempotency_key: "live-sse-idempotency-1".to_string(),
        payload_hash: "sha256:live-sse-payload-1".to_string(),
        client_request_id: Some("request.live.sse.1".to_string()),
        drive_refs: Vec::<AgentItemDriveRefInput>::new(),
        owner_scope: None,
        requested_by: sample_subject(),
        requested_at: "2026-08-01T00:01:30Z".to_string(),
        prefer_stream: true,
    };
    let result = service
        .execute_turn_with_stream_sink(turn_command, Arc::new(sink.clone()))
        .expect("live turn execution should succeed");

    eprintln!(
        "live_turns_sse_phase=stream_sink begin={} deltas={} events={}",
        sink.begin_count(),
        sink.deltas().len(),
        sink.events().len(),
    );
    eprintln!(
        "live_turns_sse_phase=result content={:?} provider_session_id={:?} deltas={} events={}",
        result
            .assistant_output_item
            .content
            .as_deref()
            .map(str::trim),
        result
            .assistant_output_item
            .provider_session_id
            .as_deref(),
        result.stream_deltas.len(),
        result.stream_events.len(),
    );

    let content = result
        .assistant_output_item
        .content
        .as_deref()
        .expect("live turn must produce assistant content")
        .trim()
        .to_string();
    assert!(
        !content.is_empty() && content.to_ascii_uppercase().contains("OK"),
        "live opencode turn must return the real model reply: {content:?}"
    );
    assert!(
        result
            .assistant_output_item
            .provider_session_id
            .as_deref()
            .is_some_and(|id| !id.is_empty()),
        "live turn must return a provider session id for resumption"
    );
    assert!(
        !result.stream_events.is_empty(),
        "live streamed turn must emit kernel events through the stream sink"
    );
    assert!(
        sink.begin_count() >= 1,
        "stream sink must observe the turn begin"
    );
    assert!(
        sink.events().len() >= result.stream_events.len(),
        "stream sink must observe the same kernel events as the turn result"
    );
    let terminal_events = [
        "agent.turn.started",
        "agent.turn.completed",
    ];
    for terminal in terminal_events {
        assert!(
            result
                .stream_events
                .iter()
                .any(|event| event.event_type == terminal),
            "live turn stream must include {terminal}"
        );
    }

    // Second turn resumes the exact provider session through the same service.
    let resumed_sink = RecordingTurnStreamSink::new();
    let resumed = service
        .execute_turn_with_stream_sink(
            CreateTurnCommand {
                turn_id: Some("turn.live.sse.two".to_string()),
                content: "Reply with exactly one word: DONE".to_string(),
                idempotency_key: "live-sse-idempotency-2".to_string(),
                payload_hash: "sha256:live-sse-payload-2".to_string(),
                client_request_id: Some("request.live.sse.2".to_string()),
                requested_at: "2026-08-01T00:02:30Z".to_string(),
                ..turn_command.clone()
            },
            Arc::new(resumed_sink.clone()),
        )
        .expect("live resumed turn should succeed");
    let resumed_content = resumed
        .assistant_output_item
        .content
        .as_deref()
        .expect("resumed live turn must produce assistant content")
        .trim()
        .to_string();
    assert!(
        !resumed_content.is_empty()
            && resumed_content.to_ascii_uppercase().contains("DONE"),
        "live resumed turn must return the real model reply: {resumed_content:?}"
    );
    assert!(
        resumed_sink.events().len() >= 1,
        "resumed live turn must emit kernel events through the stream sink"
    );
    eprintln!(
        "live_turns_sse_phase=resume_ok content={resumed_content:?} deltas={} events={}",
        resumed.stream_deltas.len(),
        resumed.stream_events.len(),
    );
    eprintln!("live_turns_sse_phase=all_ok");
}

fn kernel_workspace_root() -> String {
    std::env::var("SDKWORK_KERNEL_WORKSPACE_ROOT")
        .unwrap_or_else(|_| {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let candidates = [
                std::path::Path::new(manifest_dir)
                    .ancestors()
                    .nth(3)
                    .map(|path| path.join("sdkwork-kernel")),
            ];
            candidates
                .into_iter()
                .flatten()
                .find(|path| path.join("node_modules").exists())
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string())
        })
}

fn configured_opencode_model_id() -> String {
    std::env::var("OPENCODE_MODEL").unwrap_or_else(|_| {
        std::env::var("OPENCODE_CONFIG")
            .ok()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|json| {
                serde_json::from_str::<serde_json::Value>(&json)
                    .ok()
                    .and_then(|value| value.get("model")?.as_str().map(str::to_string))
            })
            .unwrap_or_else(|| "safeapi/gpt-5.6-sol".to_string())
    })
}

#[derive(Clone, Default)]
struct RecordingAuditSink {
    events: Arc<Mutex<Vec<KernelEvent>>>,
}

impl RecordingAuditSink {
    fn new() -> (Self, Arc<Mutex<Vec<KernelEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        (Self { events: events.clone() }, events)
    }
}

impl AgentAuditSink for RecordingAuditSink {
    fn record(&self, event: KernelEvent) -> KernelResult<()> {
        self.events.lock().expect("audit lock").push(event);
        Ok(())
    }

    fn list_events(
        &self,
        _query: &sdkwork_intelligence_agents_service::AuditEventListQuery,
    ) -> KernelResult<Vec<KernelEvent>> {
        Ok(self.events.lock().expect("audit lock").clone())
    }
}

#[derive(Clone)]
struct RecordingTurnStreamSink {
    begin: Arc<Mutex<usize>>,
    deltas: Arc<Mutex<Vec<String>>>,
    events: Arc<Mutex<Vec<KernelEvent>>>,
}

impl RecordingTurnStreamSink {
    fn new() -> Self {
        Self {
            begin: Arc::new(Mutex::new(0)),
            deltas: Arc::new(Mutex::new(Vec::new())),
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn begin_count(&self) -> usize {
        *self.begin.lock().expect("sink lock")
    }

    fn deltas(&self) -> Vec<String> {
        self.deltas.lock().expect("sink lock").clone()
    }

    fn events(&self) -> Vec<KernelEvent> {
        self.events.lock().expect("sink lock").clone()
    }
}

impl TurnExecutionStreamSink for RecordingTurnStreamSink {
    fn begin_turn(&self, _session_id: &str, _turn_id: &str) {
        *self.begin.lock().expect("sink lock") += 1;
    }

    fn push_delta(&self, delta: &str) {
        self.deltas.lock().expect("sink lock").push(delta.to_string());
    }

    fn push_event(&self, event: &KernelEvent) -> KernelResult<()> {
        self.events.lock().expect("sink lock").push(event.clone());
        Ok(())
    }
}

fn sample_manifest(agent_id: &str) -> AgentManifest {
    AgentManifest {
        schema_version: "1.0.0".to_string(),
        manifest_type: "agent".to_string(),
        agent_id: agent_id.to_string(),
        name: "live-opencode".to_string(),
        display_name: "Live OpenCode".to_string(),
        description: "live opencode e2e".to_string(),
        version: "0.1.0".to_string(),
        domain: "intelligence".to_string(),
        required_capabilities: vec!["model.chat".to_string()],
        optional_capabilities: vec!["tool.invoke".to_string()],
        required_capability_requirements: vec![],
        optional_capability_requirements: vec![],
        event_families: vec!["agent.lifecycle".to_string()],
        owner_name: "sdkwork".to_string(),
        status: "active".to_string(),
    }
}

fn sample_subject() -> PolicySubject {
    PolicySubject::new("u-1", "100001").with_role("ai.agents.manage")
}

fn create_agent_cmd(
    agent_id: &str,
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    code: &str,
    display_name: &str,
    requested_at: &str,
) -> CreateAgentCommand {
    CreateAgentCommand {
        agent_id: agent_id.to_string(),
        tenant_id,
        organization_id,
        owner_user_id,
        code: code.to_string(),
        display_name: display_name.to_string(),
        description: Some("live opencode e2e".to_string()),
        manifest: sample_manifest(agent_id),
        visibility: AgentVisibility::Organization,
        tags: vec!["starter".to_string()],
        default_code_task_intent: Some(CodeTaskIntent::new("Refactor runtime")),
        implementation_provider_id: None,
        implementation_kind: None,
        implementation_type: None,
        requested_by: sample_subject(),
        requested_at: requested_at.to_string(),
    }
}
