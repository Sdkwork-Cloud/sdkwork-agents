//! Managed-agent inference for durable session turns.
//!
//! Product HTTP APIs call this module to produce assistant replies after a user
//! input item is accepted. Inject a custom [`TurnExecutor`] at service bootstrap
//! for live provider inference; the default [`ContractTurnExecutor`] keeps HTTP
//! contracts stable without a kernel provider registry in-process.

use crate::domain::{AgentSessionItemKind, AgentSessionRecord};
use crate::runtime_facade_bridge::engine_key_for_binding_id;
use sdkwork_agent_kernel::{
    KernelEvent, KernelResult, ModelProvider, ModelRequest, ModelResponse, ModelStatus,
    ModelStreamChunk, ModelStreamSink,
};
use sdkwork_agents_runtime_facade::{
    bootstrap_code_engine, execute_code_engine_turn, execute_code_engine_turn_with_stream,
    execute_code_engine_turn_with_stream_sink, CodeEngineTurnInput,
};
use sdkwork_utils_rust::string::is_blank;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::sync::Semaphore;

/// Runtime mode when managed turn inference succeeds through the code-engine facade.
pub const RUNTIME_MODE_FACADE: &str = "agents-runtime-facade";
/// Runtime mode when inference was attempted but failed (no silent contract fallback).
pub const RUNTIME_MODE_INFERENCE_ERROR: &str = "managed-agent-inference-error";
/// Runtime mode when the bounded provider worker capacity is exhausted.
pub const RUNTIME_MODE_CAPACITY_ERROR: &str = "managed-agent-capacity-error";

pub fn is_inference_error(runtime_mode: &str) -> bool {
    runtime_mode == RUNTIME_MODE_INFERENCE_ERROR
}

pub fn is_capacity_error(runtime_mode: &str) -> bool {
    runtime_mode == RUNTIME_MODE_CAPACITY_ERROR
}

const DEFAULT_PROVIDER_WORKER_LIMIT: usize = 32;

static PROVIDER_WORKER_LIMIT: LazyLock<Arc<Semaphore>> = LazyLock::new(|| {
    let configured = std::env::var("SDKWORK_AGENTS_PROVIDER_WORKER_LIMIT")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| (1..=1024).contains(value))
        .unwrap_or(DEFAULT_PROVIDER_WORKER_LIMIT);
    Arc::new(Semaphore::new(configured))
});

/// Input for one durable turn execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnExecutionInput {
    /// Canonical SDKWork Turn identity established before provider execution.
    pub turn_id: String,
    pub agent_display_name: String,
    pub welcome_message: Option<String>,
    pub session: AgentSessionRecord,
    pub history: Vec<(AgentSessionItemKind, String)>,
    pub user_content: String,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    /// Opaque provider continuation identity. This must never be synthesized
    /// from the canonical SDKWork Session id.
    pub provider_session_id: Option<String>,
    pub access_mode_id: Option<String>,
    /// Active provider binding id (used to resolve canonical code-engine keys).
    pub binding_id: Option<String>,
    /// When true, an active binding exposes `model.chat` and the gateway may
    /// replace this completer with a kernel-backed implementation.
    pub provider_has_model_chat: bool,
}

/// Output from one durable turn execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnExecutionOutput {
    pub content: String,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub provider_session_id: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub runtime_mode: &'static str,
    pub stream_deltas: Vec<String>,
    pub stream_events: Vec<KernelEvent>,
}

/// Provider-neutral observer for one Turn's live output.
///
/// Implementations must remain synchronous because provider SDK callbacks are
/// synchronous. A closed consumer must not fail or cancel durable Turn
/// execution; persistence remains authoritative even after an HTTP disconnect.
pub trait TurnExecutionStreamSink: Send + Sync {
    fn begin_turn(&self, _session_id: &str, _turn_id: &str) {}

    fn push_delta(&self, delta: &str);

    fn push_event(&self, event: &KernelEvent);

    fn close(&self) {}
}

/// Pluggable turn execution strategy (Open/Closed: swap at service bootstrap).
pub trait TurnExecutor: Send + Sync {
    fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput;

    fn complete_with_stream_preference(
        &self,
        input: &TurnExecutionInput,
        prefer_stream: bool,
    ) -> TurnExecutionOutput {
        let _ = prefer_stream;
        self.complete(input)
    }

    fn complete_with_stream_sink(
        &self,
        input: &TurnExecutionInput,
        sink: Arc<dyn TurnExecutionStreamSink>,
    ) -> TurnExecutionOutput {
        let output = self.complete_with_stream_preference(input, true);
        replay_turn_execution_stream(&output, sink.as_ref());
        output
    }
}

/// Default managed-agent contract completer used in tests and local deployments.
/// Maximum wall-clock time for one managed turn execution.
pub const TURN_EXECUTION_TIMEOUT: Duration = Duration::from_secs(120);

/// Run turn inference with a hard timeout.
///
/// Uses the tokio bounded blocking pool (when a runtime is available) instead
/// of spawning an unbounded OS thread per request. This prevents thread
/// exhaustion and OOM under high concurrency. The completer runs on a separate
/// blocking worker so the timeout can be enforced; on timeout the detached
/// task remains bounded by the pool size and its result is dropped.
///
/// When no tokio runtime is available (pure unit tests without a runtime
/// context), falls back to inline execution without timeout isolation.
pub fn complete_with_timeout(
    completer: Arc<dyn TurnExecutor>,
    input: &TurnExecutionInput,
    prefer_stream: bool,
    timeout: Duration,
) -> TurnExecutionOutput {
    #[cfg(any(feature = "http-axum", feature = "postgres-sync"))]
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            return run_with_bounded_timeout(&handle, completer, input, prefer_stream, timeout);
        }
        tracing::warn!(
            "no tokio runtime available; running turn inference inline without timeout isolation"
        );
    }
    completer.complete_with_stream_preference(input, prefer_stream)
}

/// Run streaming Turn inference with the same worker bound and timeout as the
/// buffered path while forwarding provider output as it arrives.
pub fn complete_with_timeout_and_sink(
    completer: Arc<dyn TurnExecutor>,
    input: &TurnExecutionInput,
    sink: Arc<dyn TurnExecutionStreamSink>,
    timeout: Duration,
) -> TurnExecutionOutput {
    #[cfg(any(feature = "http-axum", feature = "postgres-sync"))]
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            return run_with_bounded_timeout_and_sink(&handle, completer, input, sink, timeout);
        }
        tracing::warn!(
            "no tokio runtime available; running streamed turn inference inline without timeout isolation"
        );
    }
    completer.complete_with_stream_sink(input, sink)
}

#[cfg(any(feature = "http-axum", feature = "postgres-sync"))]
fn run_with_bounded_timeout(
    handle: &tokio::runtime::Handle,
    completer: Arc<dyn TurnExecutor>,
    input: &TurnExecutionInput,
    prefer_stream: bool,
    timeout: Duration,
) -> TurnExecutionOutput {
    let permit = match PROVIDER_WORKER_LIMIT.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            crate::infrastructure::AgentMetricsRegistry::global()
                .record_provider_worker_rejection();
            return capacity_error(input);
        }
    };
    let input_owned = input.clone();
    let join_handle = handle.spawn_blocking(move || {
        let _permit = permit;
        completer.complete_with_stream_preference(&input_owned, prefer_stream)
    });
    // Safe to block_on here: complete_with_timeout runs inside a spawn_blocking
    // worker (see http.rs handler dispatch), not on an async executor thread.
    match handle.block_on(tokio::time::timeout(timeout, join_handle)) {
        Ok(Ok(output)) => output,
        Ok(Err(_join_error)) => inference_error("turn inference task failed"),
        Err(_elapsed) => inference_error("turn inference timed out"),
    }
}

#[cfg(any(feature = "http-axum", feature = "postgres-sync"))]
fn run_with_bounded_timeout_and_sink(
    handle: &tokio::runtime::Handle,
    completer: Arc<dyn TurnExecutor>,
    input: &TurnExecutionInput,
    sink: Arc<dyn TurnExecutionStreamSink>,
    timeout: Duration,
) -> TurnExecutionOutput {
    let permit = match PROVIDER_WORKER_LIMIT.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            crate::infrastructure::AgentMetricsRegistry::global()
                .record_provider_worker_rejection();
            sink.close();
            return capacity_error(input);
        }
    };
    let input_owned = input.clone();
    let worker_sink = Arc::clone(&sink);
    let join_handle = handle.spawn_blocking(move || {
        let _permit = permit;
        completer.complete_with_stream_sink(&input_owned, worker_sink)
    });
    match handle.block_on(tokio::time::timeout(timeout, join_handle)) {
        Ok(Ok(output)) => output,
        Ok(Err(_join_error)) => {
            sink.close();
            inference_error("turn inference task failed")
        }
        Err(_elapsed) => {
            sink.close();
            inference_error("turn inference timed out")
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ContractTurnExecutor;

impl TurnExecutor for ContractTurnExecutor {
    fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
        execute_agent_turn(input)
    }
}

/// Kernel-backed turn executor for production gateway bootstrap.
///
/// Invokes a mounted [`ModelProvider`] when `provider_has_model_chat` is true;
/// otherwise falls back to [`ContractTurnExecutor`] semantics.
pub struct KernelModelTurnExecutor<P> {
    provider: Arc<P>,
    fallback: ContractTurnExecutor,
}

impl<P> KernelModelTurnExecutor<P>
where
    P: ModelProvider + Send + Sync + 'static,
{
    pub fn new(provider: Arc<P>) -> Self {
        Self {
            provider,
            fallback: ContractTurnExecutor,
        }
    }
}

impl<P> TurnExecutor for KernelModelTurnExecutor<P>
where
    P: ModelProvider + Send + Sync + 'static,
{
    fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
        if !input.provider_has_model_chat {
            return self.fallback.complete(input);
        }
        match invoke_kernel_model(self.provider.as_ref(), input) {
            Ok(output) => output,
            Err(error) => {
                tracing::warn!(
                    session_id = %input.session.session_id,
                    error = %error,
                    "kernel model invoke failed"
                );
                inference_error(format!("model inference failed: {error}"))
            }
        }
    }
}

/// Production chat completer: routes active provider bindings through the
/// agents runtime facade (canonical code engines). Never silently echoes user
/// input when `model.chat` is configured.
#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeFacadeTurnExecutor;

impl TurnExecutor for RuntimeFacadeTurnExecutor {
    fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
        execute_runtime_facade_turn(input, false, None)
    }

    fn complete_with_stream_preference(
        &self,
        input: &TurnExecutionInput,
        prefer_stream: bool,
    ) -> TurnExecutionOutput {
        execute_runtime_facade_turn(input, prefer_stream, None)
    }

    fn complete_with_stream_sink(
        &self,
        input: &TurnExecutionInput,
        sink: Arc<dyn TurnExecutionStreamSink>,
    ) -> TurnExecutionOutput {
        execute_runtime_facade_turn(input, true, Some(sink))
    }
}

struct RuntimeFacadeModelStreamSink {
    sink: Arc<dyn TurnExecutionStreamSink>,
}

impl ModelStreamSink for RuntimeFacadeModelStreamSink {
    fn push_chunk(&mut self, chunk: ModelStreamChunk) -> KernelResult<()> {
        self.sink.push_delta(&chunk.content);
        Ok(())
    }

    fn push_event(&mut self, event: KernelEvent) -> KernelResult<()> {
        self.sink.push_event(&event);
        Ok(())
    }
}

fn execute_runtime_facade_turn(
    input: &TurnExecutionInput,
    prefer_stream: bool,
    sink: Option<Arc<dyn TurnExecutionStreamSink>>,
) -> TurnExecutionOutput {
    if !input.provider_has_model_chat {
        return execute_agent_turn(input);
    }

    let binding_id = input
        .binding_id
        .as_deref()
        .filter(|value| !is_blank(Some(*value)))
        .unwrap_or("");
    let Some(engine_key) = engine_key_for_binding_id(binding_id) else {
        return inference_error("active provider binding is not mapped to a canonical code engine");
    };

    let Ok(slot) = bootstrap_code_engine(engine_key) else {
        return inference_error(format!("code engine bootstrap failed for {engine_key}"));
    };

    let model_id = resolve_turn_model_id(input, &slot);
    let prompt = build_managed_chat_prompt(input);
    let turn_input = CodeEngineTurnInput {
        engine_key: engine_key.to_string(),
        model_id: model_id.clone(),
        session_id: Some(input.session.session_id.clone()),
        turn_id: Some(input.turn_id.clone()),
        provider_session_id: input.provider_session_id.clone(),
        prompt,
        access_mode_id: input.access_mode_id.clone(),
        ..Default::default()
    };

    let turn_result = if prefer_stream {
        if let Some(sink) = sink {
            let mut facade_sink = RuntimeFacadeModelStreamSink { sink };
            execute_code_engine_turn_with_stream_sink(&slot, &turn_input, &mut facade_sink)
        } else {
            execute_code_engine_turn_with_stream(&slot, &turn_input)
        }
    } else {
        execute_code_engine_turn(&slot, &turn_input)
    };

    match turn_result {
        Ok(output) => {
            let content = output.assistant_content.trim().to_string();
            if content.is_empty() {
                return inference_error("code engine returned empty assistant content");
            }
            TurnExecutionOutput {
                content,
                model_id: Some(model_id),
                provider_id: input.provider_id.clone(),
                provider_session_id: output.provider_session_id,
                input_tokens: estimate_tokens(input.user_content.as_str()),
                output_tokens: estimate_tokens(output.assistant_content.as_str()),
                runtime_mode: RUNTIME_MODE_FACADE,
                stream_deltas: output.stream_deltas,
                stream_events: output.stream_events,
            }
        }
        Err(error) => inference_error(format!("code engine turn failed: {error}")),
    }
}

fn resolve_turn_model_id(
    input: &TurnExecutionInput,
    slot: &sdkwork_agents_runtime_facade::CodeEngineSlot,
) -> String {
    if let Some(model_id) = input
        .model_id
        .as_deref()
        .filter(|value| !is_blank(Some(*value)))
    {
        return model_id.to_string();
    }
    slot.list_model_ids()
        .into_iter()
        .next()
        .unwrap_or_else(|| "default".to_string())
}

fn build_managed_chat_prompt(input: &TurnExecutionInput) -> String {
    let mut lines = Vec::new();
    if let Some(welcome) = input
        .welcome_message
        .as_deref()
        .filter(|value| !is_blank(Some(*value)))
    {
        lines.push(format!("system: {welcome}"));
    }
    for (role, content) in &input.history {
        lines.push(format!("{}: {content}", role.as_str()));
    }
    lines.push(format!("user: {}", input.user_content));
    lines.join("\n")
}

fn replay_turn_execution_stream(output: &TurnExecutionOutput, sink: &dyn TurnExecutionStreamSink) {
    let mut delta_index = 0usize;
    let mut agent_message_text_by_item = HashMap::new();
    for event in &output.stream_events {
        sink.push_event(event);
        if let Some(expected_delta) =
            buffered_agent_message_delta(event, &mut agent_message_text_by_item)
        {
            if let Some(delta) = output
                .stream_deltas
                .get(delta_index)
                .filter(|delta| delta.as_str() == expected_delta)
            {
                sink.push_delta(delta);
                delta_index += 1;
            }
        }
    }
    for delta in output.stream_deltas.iter().skip(delta_index) {
        sink.push_delta(delta);
    }
}

fn buffered_agent_message_delta(
    event: &KernelEvent,
    text_by_item: &mut HashMap<String, String>,
) -> Option<String> {
    if !matches!(
        event.event_type.as_str(),
        "agent.message.started" | "agent.message.updated" | "agent.message.completed"
    ) {
        return None;
    }
    let payload = serde_json::from_str::<serde_json::Value>(&event.payload).ok()?;
    let item = payload.get("item")?;
    if item.get("type")?.as_str()? != "agent_message" {
        return None;
    }
    let item_id = item.get("id")?.as_str()?.to_string();
    let current = item.get("text")?.as_str()?.to_string();
    let previous = text_by_item
        .insert(item_id, current.clone())
        .unwrap_or_default();
    if event.event_type == "agent.message.started"
        || !current.starts_with(&previous)
        || current.len() == previous.len()
    {
        return None;
    }
    Some(current[previous.len()..].to_string())
}

fn inference_error(message: impl Into<String>) -> TurnExecutionOutput {
    TurnExecutionOutput {
        content: message.into(),
        model_id: None,
        provider_id: None,
        provider_session_id: None,
        input_tokens: 0,
        output_tokens: 0,
        runtime_mode: RUNTIME_MODE_INFERENCE_ERROR,
        stream_deltas: Vec::new(),
        stream_events: Vec::new(),
    }
}

fn capacity_error(input: &TurnExecutionInput) -> TurnExecutionOutput {
    TurnExecutionOutput {
        content: "provider concurrency limit reached".to_string(),
        model_id: input.model_id.clone(),
        provider_id: input.provider_id.clone(),
        provider_session_id: input.provider_session_id.clone(),
        input_tokens: 0,
        output_tokens: 0,
        runtime_mode: RUNTIME_MODE_CAPACITY_ERROR,
        stream_deltas: Vec::new(),
        stream_events: Vec::new(),
    }
}

fn build_model_items(input: &TurnExecutionInput) -> Vec<String> {
    let mut items = Vec::new();
    if let Some(welcome) = input
        .welcome_message
        .as_deref()
        .filter(|value| !is_blank(Some(*value)))
    {
        items.push(format!("system: {welcome}"));
    }
    for (role, content) in &input.history {
        items.push(format!("{}: {content}", role.as_str()));
    }
    items.push(format!("user: {}", input.user_content));
    items
}

fn invoke_kernel_model(
    provider: &dyn ModelProvider,
    input: &TurnExecutionInput,
) -> KernelResult<TurnExecutionOutput> {
    let model_request_id = format!(
        "managed-chat.{}.{}",
        input.session.session_id,
        input.history.len() + 1
    );
    let model_id = input.model_id.clone();
    let mut request = ModelRequest::new(model_request_id.clone(), build_model_items(input))
        .for_session(input.session.session_id.clone())
        .for_step(input.turn_id.clone());
    if let Some(provider_session_id) = input.provider_session_id.as_ref() {
        request = request.for_provider_session(provider_session_id.clone());
    }
    if let Some(model_id) = model_id.clone() {
        request = request.with_model_id(model_id);
    }

    let response = provider.invoke(request)?;
    map_model_response(input, model_id, response)
}

fn map_model_response(
    input: &TurnExecutionInput,
    model_id: Option<String>,
    response: ModelResponse,
) -> KernelResult<TurnExecutionOutput> {
    if response.status != ModelStatus::Succeeded {
        return Err(sdkwork_agent_kernel::KernelError::validation(format!(
            "model invoke returned status {:?}",
            response.status
        )));
    }
    let content = response
        .messages
        .into_iter()
        .next()
        .filter(|message| !is_blank(Some(message.as_str())))
        .unwrap_or_else(|| self_fallback_content(input));
    let (input_tokens, output_tokens) = response
        .usage
        .map(|usage| (usage.input_tokens as u64, usage.output_tokens as u64))
        .unwrap_or_else(|| {
            (
                estimate_tokens(input.user_content.as_str()),
                estimate_tokens(content.as_str()),
            )
        });
    Ok(TurnExecutionOutput {
        content,
        model_id,
        provider_id: Some(response.provider_id),
        provider_session_id: input.provider_session_id.clone(),
        input_tokens,
        output_tokens,
        runtime_mode: "managed-agent-kernel-model-v1",
        stream_deltas: Vec::new(),
        stream_events: Vec::new(),
    })
}

fn self_fallback_content(input: &TurnExecutionInput) -> String {
    format!(
        "Hello! I'm {}. I received your message:\n\n> {}",
        input.agent_display_name, input.user_content
    )
}

fn estimate_tokens(text: &str) -> u64 {
    (text.chars().count() as u64).div_ceil(4)
}

fn build_transcript(history: &[(AgentSessionItemKind, String)]) -> String {
    let mut transcript = String::new();
    for (role, message) in history {
        transcript.push_str(role.as_str());
        transcript.push_str(": ");
        transcript.push_str(message);
        transcript.push('\n');
    }
    transcript
}

fn runtime_mode_for(input: &TurnExecutionInput) -> &'static str {
    if input.provider_has_model_chat {
        "managed-agent-provider-bound-v1"
    } else {
        "managed-agent-contract-v1"
    }
}

/// Complete one user turn using managed-agent contract semantics.
pub fn execute_agent_turn(input: &TurnExecutionInput) -> TurnExecutionOutput {
    let input_tokens = estimate_tokens(input.user_content.as_str());
    let model_id = input.model_id.clone();
    let provider_id = input.provider_id.clone();
    let runtime_mode = runtime_mode_for(input);

    let content = if input.history.is_empty() {
        if let Some(welcome) = input
            .welcome_message
            .as_deref()
            .filter(|value| !is_blank(Some(*value)))
        {
            format!(
                "{welcome}\n\nHow can I help you today?\n\n> {}",
                input.user_content
            )
        } else if input.provider_has_model_chat {
            format!(
                "Hello! I'm {}. I received your message:\n\n> {}\n\nLive model inference requires a canonical code-engine provider binding.",
                input.agent_display_name, input.user_content
            )
        } else {
            format!(
                "Hello! I'm {}. I received your message:\n\n> {}\n\nActivate a provider binding with model.chat for live model inference.",
                input.agent_display_name, input.user_content
            )
        }
    } else {
        let transcript = build_transcript(&input.history);
        format!(
            "{transcript}user: {}\n\nassistant: Thanks for the context. Based on our conversation, here is my response to your latest message:\n\n> {}",
            input.user_content, input.user_content
        )
    };

    let output_tokens = estimate_tokens(content.as_str());
    TurnExecutionOutput {
        content,
        model_id,
        provider_id,
        provider_session_id: input.provider_session_id.clone(),
        input_tokens,
        output_tokens,
        runtime_mode,
        stream_deltas: Vec::new(),
        stream_events: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        AgentSessionEntrySurface, AgentSessionKind, AgentSessionRecord, AgentSessionStatus,
        AgentSessionTitleSource,
    };

    fn sample_session() -> AgentSessionRecord {
        AgentSessionRecord {
            id: 1,
            session_id: "session.test".to_string(),
            tenant_id: 100001,
            organization_id: 0,
            agent_id: "agent.test".to_string(),
            owner_user_id: 42,
            project_id: None,
            session_kind: AgentSessionKind::Assistant,
            entry_surface: AgentSessionEntrySurface::Api,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: Some("Test".to_string()),
            title_source: AgentSessionTitleSource::System,
            status: AgentSessionStatus::Active,
            item_count: 0,
            last_item_sequence: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            idempotency_key: None,
            payload_hash: None,
            created_by: 42,
            updated_by: 42,
            version: 0,
            created_at: "2026-06-28T00:00:00Z".to_string(),
            updated_at: "2026-06-28T00:00:00Z".to_string(),
            last_item_at: None,
            closed_at: None,
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        }
    }

    #[test]
    fn execute_agent_turn_returns_assistant_content() {
        let output = execute_agent_turn(&TurnExecutionInput {
            turn_id: "turn.test".to_string(),
            agent_display_name: "Demo Agent".to_string(),
            welcome_message: Some("Welcome".to_string()),
            session: sample_session(),
            history: Vec::new(),
            user_content: "Hello".to_string(),
            model_id: None,
            provider_id: None,
            provider_session_id: None,
            binding_id: None,
            access_mode_id: None,
            provider_has_model_chat: false,
        });
        assert!(output.content.contains("Hello"));
        assert!(output.content.contains("Welcome"));
        assert_eq!(output.runtime_mode, "managed-agent-contract-v1");
        assert!(output.output_tokens > 0);
    }

    #[test]
    fn execute_agent_turn_marks_provider_bound_runtime_mode() {
        let output = execute_agent_turn(&TurnExecutionInput {
            turn_id: "turn.test".to_string(),
            agent_display_name: "Demo Agent".to_string(),
            welcome_message: None,
            session: sample_session(),
            history: Vec::new(),
            user_content: "Hello".to_string(),
            model_id: None,
            provider_id: Some("provider.model.rig".to_string()),
            provider_session_id: None,
            binding_id: None,
            access_mode_id: None,
            provider_has_model_chat: true,
        });
        assert_eq!(output.runtime_mode, "managed-agent-provider-bound-v1");
        assert!(output.content.contains("canonical code-engine"));
    }

    struct FakeKernelModelProvider;

    impl ModelProvider for FakeKernelModelProvider {
        fn provider_manifest(&self) -> sdkwork_agent_kernel::ProviderManifest {
            sdkwork_agent_kernel::ProviderManifest::new(
                "provider.model.fake",
                "model",
                "sdkwork-fake-model",
                "0.1.0",
                vec!["model.chat".to_string()],
            )
        }

        fn health(&self) -> sdkwork_agent_kernel::ProviderHealth {
            sdkwork_agent_kernel::ProviderHealth::available()
        }

        fn invoke(&self, request: ModelRequest) -> KernelResult<ModelResponse> {
            assert_eq!(request.session_id.as_deref(), Some("session.test"));
            assert_eq!(request.step_id.as_deref(), Some("turn.test"));
            assert_eq!(
                request.provider_session_id.as_deref(),
                Some("provider-session.fake")
            );
            Ok(ModelResponse::text(
                request.model_request_id,
                "provider.model.fake",
                "kernel reply",
            )
            .with_usage(sdkwork_agent_kernel::ModelUsage::new(3, 5)))
        }
    }

    #[test]
    fn kernel_model_turn_executor_preserves_session_identities() {
        let completer = KernelModelTurnExecutor::new(Arc::new(FakeKernelModelProvider));
        let output = completer.complete(&TurnExecutionInput {
            turn_id: "turn.test".to_string(),
            agent_display_name: "Demo Agent".to_string(),
            welcome_message: None,
            session: sample_session(),
            history: Vec::new(),
            user_content: "Hello".to_string(),
            model_id: Some("model.fake".to_string()),
            provider_id: Some("provider.model.fake".to_string()),
            provider_session_id: Some("provider-session.fake".to_string()),
            binding_id: None,
            access_mode_id: None,
            provider_has_model_chat: true,
        });
        assert_eq!(output.content, "kernel reply");
        assert_eq!(output.runtime_mode, "managed-agent-kernel-model-v1");
        assert_eq!(output.provider_id.as_deref(), Some("provider.model.fake"));
        assert_eq!(
            output.provider_session_id.as_deref(),
            Some("provider-session.fake")
        );
    }
}
