use std::path::PathBuf;

use sdkwork_agent_kernel::{
    AgentExecutionProviderOptionValue, KernelResult, ModelRequest, ModelResponse, ModelStreamChunk,
    ModelStreamSink, ToolCall,
};
use sdkwork_agent_provider_spi::SdkRuntimeStreamCompletion;
use sdkwork_utils_rust::string::is_blank;

use crate::code_engines::CodeEngineSlot;
use crate::error::{RuntimeFacadeError, RuntimeFacadeResult};

/// Maximum prompt size accepted by the runtime facade (1 MiB).
pub const MAX_CODE_ENGINE_PROMPT_BYTES: usize = 1_048_576;
/// Maximum stream chunks collected before failing closed.
pub const MAX_CODE_ENGINE_STREAM_CHUNKS: usize = 8_192;
/// Maximum aggregated stream output size (4 MiB).
pub const MAX_CODE_ENGINE_STREAM_OUTPUT_BYTES: usize = 4_194_304;

const WORKING_DIRECTORY_METADATA_KEY: &str = "sdkwork.code_engine.working_directory";
const APPROVAL_POLICY_METADATA_KEY: &str = "sdkwork.code_engine.approval_policy";
const SANDBOX_MODE_METADATA_KEY: &str = "sdkwork.code_engine.sandbox_mode";
const FULL_AUTO_METADATA_KEY: &str = "sdkwork.code_engine.full_auto";
const SKIP_GIT_REPO_CHECK_METADATA_KEY: &str = "sdkwork.code_engine.skip_git_repo_check";
const EPHEMERAL_METADATA_KEY: &str = "sdkwork.code_engine.ephemeral";
const REQUIRE_LIVE_PROVIDER_METADATA_KEY: &str = "sdkwork.code_engine.require_live_provider";
const MAX_OUTPUT_BYTES_METADATA_KEY: &str = "sdkwork.code_engine.max_output_bytes";
const TEMPERATURE_METADATA_KEY: &str = "sdkwork.code_engine.temperature";
const TOP_P_METADATA_KEY: &str = "sdkwork.code_engine.top_p";
const MAX_TOKENS_METADATA_KEY: &str = "sdkwork.code_engine.max_tokens";
const PROVIDER_SESSION_DIAGNOSTIC_KEYS: [&str; 6] = [
    "sdk_runtime_session_id",
    "sdk_runtime_provider_session_id",
    "sdkwork.code_engine.provider_session_id",
    "sdkwork.provider.session_id",
    "provider_session_id",
    "provider_session_id",
];

/// Product-neutral code-engine turn input consumed by the agents runtime facade.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CodeEngineTurnInput {
    pub engine_key: String,
    pub model_id: String,
    pub provider_session_id: Option<String>,
    pub prompt: String,
    pub working_directory: Option<PathBuf>,
    pub timeout_ms: Option<u64>,
    pub approval_policy: Option<String>,
    pub sandbox_mode: Option<String>,
    pub full_auto: bool,
    pub skip_git_repo_check: bool,
    pub ephemeral: bool,
    pub require_live_provider: bool,
    pub max_output_bytes: Option<usize>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<i64>,
    pub access_mode_id: Option<String>,
}

/// Provider-neutral terminal metadata for a streamed code-engine turn.
///
/// The facade constructs this value only after the runtime verifies that the
/// terminal `model_request_id` belongs to the active turn and that the
/// provider supplied a non-empty provider session id. Product callers therefore
/// never inspect provider diagnostics or transport frames to resume a turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeEngineTurnStreamCompletion {
    pub model_request_id: String,
    pub finish_reason: String,
    pub provider_session_id: String,
}

/// Product-neutral code-engine turn output produced by the agents runtime facade.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeEngineTurnOutput {
    pub assistant_content: String,
    pub provider_session_id: Option<String>,
    /// Provider-neutral tool calls returned by the kernel model provider.
    pub tool_calls: Vec<ToolCall>,
    /// Token/word deltas when streaming is available; empty when invoke-only.
    pub stream_deltas: Vec<String>,
    /// Verified terminal metadata for a streamed turn. `None` means the turn
    /// used invoke-only execution or the provider cannot prove completion.
    pub stream_completion: Option<CodeEngineTurnStreamCompletion>,
}

pub fn execute_code_engine_turn(
    slot: &CodeEngineSlot,
    input: &CodeEngineTurnInput,
) -> RuntimeFacadeResult<CodeEngineTurnOutput> {
    if slot.engine_key() != input.engine_key {
        return Err(RuntimeFacadeError::EngineMismatch {
            slot_engine: slot.engine_key().to_string(),
            input_engine: input.engine_key.clone(),
        });
    }
    if is_blank(Some(input.prompt.as_str())) {
        return Err(RuntimeFacadeError::BlankPrompt);
    }
    if input.prompt.len() > MAX_CODE_ENGINE_PROMPT_BYTES {
        return Err(RuntimeFacadeError::Kernel(format!(
            "prompt exceeds maximum size of {MAX_CODE_ENGINE_PROMPT_BYTES} bytes"
        )));
    }

    let model_request = build_model_request(slot, input)?;

    let response = slot
        .invoke_model(model_request)
        .map_err(|error| RuntimeFacadeError::Kernel(error.to_string()))?;

    let assistant_content = response.messages.join("\n");
    validate_output_size(assistant_content.len(), effective_max_output_bytes(input))?;

    Ok(build_turn_output(response, input))
}

fn build_turn_output(response: ModelResponse, input: &CodeEngineTurnInput) -> CodeEngineTurnOutput {
    let assistant_content = response.messages.join("\n");
    CodeEngineTurnOutput {
        assistant_content,
        provider_session_id: resolve_provider_session_id(&response, input),
        tool_calls: response.tool_calls,
        stream_deltas: Vec::new(),
        stream_completion: None,
    }
}

fn build_model_request(
    slot: &CodeEngineSlot,
    input: &CodeEngineTurnInput,
) -> RuntimeFacadeResult<ModelRequest> {
    let resolved_execution_settings = input
        .access_mode_id
        .as_deref()
        .map(str::trim)
        .filter(|access_mode_id| !access_mode_id.is_empty())
        .map(|access_mode_id| {
            slot.resolve_execution_settings(access_mode_id)
                .map_err(|error| RuntimeFacadeError::Kernel(error.to_string()))
        })
        .transpose()?;
    let model_request_id = format!("agents-turn-{}", sdkwork_utils_rust::uuid());
    let mut model_request = ModelRequest::new(model_request_id, vec![input.prompt.clone()]);
    if !is_blank(Some(input.model_id.as_str())) {
        model_request.model_id = Some(input.model_id.clone());
    }
    if let Some(provider_session_id) = input.provider_session_id.as_ref() {
        model_request.session_id = Some(provider_session_id.clone());
    }
    if let Some(timeout_ms) = input.timeout_ms {
        model_request.timeout_ms = Some(timeout_ms);
    }
    if let Some(working_directory) = input.working_directory.as_ref() {
        let value = working_directory.to_string_lossy();
        if !is_blank(Some(value.as_ref())) {
            model_request = model_request.with_metadata(WORKING_DIRECTORY_METADATA_KEY, value);
        }
    }
    if resolved_execution_settings.is_none() {
        model_request = with_optional_metadata(
            model_request,
            APPROVAL_POLICY_METADATA_KEY,
            input.approval_policy.as_deref(),
        );
        model_request = with_optional_metadata(
            model_request,
            SANDBOX_MODE_METADATA_KEY,
            input.sandbox_mode.as_deref(),
        );
    }
    model_request = model_request
        .with_metadata(FULL_AUTO_METADATA_KEY, input.full_auto.to_string())
        .with_metadata(
            SKIP_GIT_REPO_CHECK_METADATA_KEY,
            input.skip_git_repo_check.to_string(),
        )
        .with_metadata(EPHEMERAL_METADATA_KEY, input.ephemeral.to_string())
        .with_metadata(
            REQUIRE_LIVE_PROVIDER_METADATA_KEY,
            input.require_live_provider.to_string(),
        )
        .with_metadata(
            MAX_OUTPUT_BYTES_METADATA_KEY,
            effective_max_output_bytes(input).to_string(),
        );
    if let Some(temperature) = input.temperature {
        model_request =
            model_request.with_metadata(TEMPERATURE_METADATA_KEY, temperature.to_string());
    }
    if let Some(top_p) = input.top_p {
        model_request = model_request.with_metadata(TOP_P_METADATA_KEY, top_p.to_string());
    }
    if let Some(max_tokens) = input.max_tokens {
        model_request =
            model_request.with_metadata(MAX_TOKENS_METADATA_KEY, max_tokens.to_string());
    }
    if let Some(resolved) = resolved_execution_settings {
        for option in resolved.provider_options {
            let value = match option.value {
                AgentExecutionProviderOptionValue::String(value) => value,
                AgentExecutionProviderOptionValue::Boolean(value) => value.to_string(),
            };
            model_request = model_request.with_metadata(option.key, value);
        }
    }
    Ok(model_request)
}

fn with_optional_metadata(request: ModelRequest, key: &str, value: Option<&str>) -> ModelRequest {
    match value.filter(|candidate| !is_blank(Some(*candidate))) {
        Some(candidate) => request.with_metadata(key, candidate.trim()),
        None => request,
    }
}

fn effective_max_output_bytes(input: &CodeEngineTurnInput) -> usize {
    input
        .max_output_bytes
        .unwrap_or(MAX_CODE_ENGINE_STREAM_OUTPUT_BYTES)
        .min(MAX_CODE_ENGINE_STREAM_OUTPUT_BYTES)
}

fn validate_output_size(output_bytes: usize, max_output_bytes: usize) -> RuntimeFacadeResult<()> {
    if output_bytes > max_output_bytes {
        return Err(RuntimeFacadeError::Kernel(format!(
            "code-engine output exceeds maximum size of {max_output_bytes} bytes"
        )));
    }
    Ok(())
}

fn resolve_provider_session_id(
    response: &ModelResponse,
    input: &CodeEngineTurnInput,
) -> Option<String> {
    response
        .diagnostics
        .iter()
        .find_map(|diagnostic| {
            let (key, value) = diagnostic.split_once('=')?;
            PROVIDER_SESSION_DIAGNOSTIC_KEYS
                .contains(&key.trim())
                .then(|| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .or_else(|| input.provider_session_id.clone())
}

/// Execute a turn preferring provider stream chunks when supported.
pub fn execute_code_engine_turn_with_stream(
    slot: &CodeEngineSlot,
    input: &CodeEngineTurnInput,
) -> RuntimeFacadeResult<CodeEngineTurnOutput> {
    let mut sink = DiscardingModelStreamSink;
    execute_code_engine_turn_with_stream_sink(slot, input, &mut sink)
}

/// Execute a turn and forward each provider-neutral model chunk as it arrives.
/// The caller owns its product representation; this facade owns the kernel SPI
/// boundary, ordering, output budget enforcement, and provider-session proof.
///
/// Codex initial turns can stream only because its runtime terminal frame
/// carries a correlated provider session id. Other engines remain invoke-only
/// until they offer the same proof. Once a chunk is delivered, this function
/// never invokes the provider again as a fallback.
pub fn execute_code_engine_turn_with_stream_sink(
    slot: &CodeEngineSlot,
    input: &CodeEngineTurnInput,
    sink: &mut dyn ModelStreamSink,
) -> RuntimeFacadeResult<CodeEngineTurnOutput> {
    if slot.engine_key() != input.engine_key {
        return Err(RuntimeFacadeError::EngineMismatch {
            slot_engine: slot.engine_key().to_string(),
            input_engine: input.engine_key.clone(),
        });
    }
    if is_blank(Some(input.prompt.as_str())) {
        return Err(RuntimeFacadeError::BlankPrompt);
    }
    if input.prompt.len() > MAX_CODE_ENGINE_PROMPT_BYTES {
        return Err(RuntimeFacadeError::Kernel(format!(
            "prompt exceeds maximum size of {MAX_CODE_ENGINE_PROMPT_BYTES} bytes"
        )));
    }
    if input.provider_session_id.is_none() {
        if slot.supports_first_turn_streaming_completion() {
            return execute_first_turn_with_stream_completion(slot, input, sink);
        }
        return execute_code_engine_turn(slot, input);
    }

    let model_request = build_model_request(slot, input)?;
    let mut collector = ForwardingStreamCollector::new(sink);
    match slot.stream_model_into(model_request, &mut collector) {
        Ok(()) if !collector.is_empty() => {
            return build_streamed_turn_output(input, collector.into_deltas(), None);
        }
        Ok(()) => {
            return Err(RuntimeFacadeError::Kernel(
                "provider stream completed without output; invoke fallback is unsafe after execution"
                    .to_string(),
            ));
        }
        Err(_error) if collector.is_empty() => {}
        Err(error) => {
            return Err(RuntimeFacadeError::Kernel(error.to_string()));
        }
    }

    execute_code_engine_turn(slot, input)
}

fn execute_first_turn_with_stream_completion(
    slot: &CodeEngineSlot,
    input: &CodeEngineTurnInput,
    sink: &mut dyn ModelStreamSink,
) -> RuntimeFacadeResult<CodeEngineTurnOutput> {
    let model_request = build_model_request(slot, input)?;
    let mut collector = ForwardingStreamCollector::new(sink);
    let runtime_completion = slot
        .stream_first_turn_model_into(model_request, &mut collector)
        .map_err(|error| RuntimeFacadeError::Kernel(error.to_string()))?;
    let completion = code_engine_stream_completion(runtime_completion)?;

    build_streamed_turn_output(input, collector.into_deltas(), Some(completion))
}

fn code_engine_stream_completion(
    runtime_completion: SdkRuntimeStreamCompletion,
) -> RuntimeFacadeResult<CodeEngineTurnStreamCompletion> {
    let provider_session_id = runtime_completion
        .provider_session_id
        .filter(|value| !is_blank(Some(value.as_str())))
        .ok_or_else(|| {
            RuntimeFacadeError::Kernel(
                "provider stream completed without a verified provider session id".to_string(),
            )
        })?;

    Ok(CodeEngineTurnStreamCompletion {
        model_request_id: runtime_completion.model_request_id,
        finish_reason: runtime_completion.finish_reason,
        provider_session_id,
    })
}

fn build_streamed_turn_output(
    input: &CodeEngineTurnInput,
    stream_deltas: Vec<String>,
    stream_completion: Option<CodeEngineTurnStreamCompletion>,
) -> RuntimeFacadeResult<CodeEngineTurnOutput> {
    if stream_deltas.is_empty() {
        return Err(RuntimeFacadeError::Kernel(
            "provider stream completed without output".to_string(),
        ));
    }

    let assistant_content = stream_deltas.join("");
    validate_output_size(assistant_content.len(), effective_max_output_bytes(input))?;
    if assistant_content.trim().is_empty() {
        return Err(RuntimeFacadeError::Kernel(
            "provider stream completed with blank output".to_string(),
        ));
    }

    let provider_session_id = stream_completion
        .as_ref()
        .map(|completion| completion.provider_session_id.clone())
        .or_else(|| input.provider_session_id.clone())
        .ok_or_else(|| {
            RuntimeFacadeError::Kernel(
                "provider stream completed without a provider session id".to_string(),
            )
        })?;

    Ok(CodeEngineTurnOutput {
        assistant_content,
        provider_session_id: Some(provider_session_id),
        tool_calls: Vec::new(),
        stream_deltas,
        stream_completion,
    })
}

struct ForwardingStreamCollector<'a> {
    inner: &'a mut dyn ModelStreamSink,
    deltas: Vec<String>,
    bytes: usize,
}

impl<'a> ForwardingStreamCollector<'a> {
    fn new(inner: &'a mut dyn ModelStreamSink) -> Self {
        Self {
            inner,
            deltas: Vec::new(),
            bytes: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.deltas.is_empty()
    }

    fn into_deltas(self) -> Vec<String> {
        self.deltas
    }
}

impl ModelStreamSink for ForwardingStreamCollector<'_> {
    fn push_chunk(&mut self, chunk: ModelStreamChunk) -> KernelResult<()> {
        if chunk.content.is_empty() {
            return Ok(());
        }
        if self.deltas.len() >= MAX_CODE_ENGINE_STREAM_CHUNKS {
            return Err(sdkwork_agent_kernel::KernelError::resource_exhausted(
                "code-engine stream chunk limit exceeded",
            ));
        }
        self.bytes = self.bytes.checked_add(chunk.content.len()).ok_or_else(|| {
            sdkwork_agent_kernel::KernelError::resource_exhausted(
                "code-engine stream output byte count overflow",
            )
        })?;
        if self.bytes > MAX_CODE_ENGINE_STREAM_OUTPUT_BYTES {
            return Err(sdkwork_agent_kernel::KernelError::resource_exhausted(
                "code-engine stream output exceeds maximum size",
            ));
        }

        self.inner.push_chunk(chunk.clone())?;
        self.deltas.push(chunk.content);
        Ok(())
    }
}

struct DiscardingModelStreamSink;

impl ModelStreamSink for DiscardingModelStreamSink {
    fn push_chunk(&mut self, _chunk: ModelStreamChunk) -> KernelResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code_engines::{bootstrap_code_engine, canonical_code_engine_keys};
    use sdkwork_agent_kernel::{ModelProvider, ProviderHealth, ProviderManifest};
    use sdkwork_agent_provider_codex::CodexSdkIntegration;
    use sdkwork_agent_provider_spi::{
        NegotiatedCapability, SdkBackendKind, SdkBackendRuntime, SdkCapabilityNegotiation,
        SdkDriverHealth, SdkRuntimeBackedModelProvider, SdkRuntimeError, SdkRuntimeOperationKind,
        SdkRuntimeRequest, SdkRuntimeResponse, SdkRuntimeRouter, SDK_CAPABILITY_MODEL_CHAT,
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[derive(Default)]
    struct RecordingStreamSink {
        contents: Vec<String>,
    }

    impl ModelStreamSink for RecordingStreamSink {
        fn push_chunk(&mut self, chunk: ModelStreamChunk) -> KernelResult<()> {
            self.contents.push(chunk.content);
            Ok(())
        }
    }

    struct NeverInvokeFallback;

    impl ModelProvider for NeverInvokeFallback {
        fn provider_manifest(&self) -> ProviderManifest {
            ProviderManifest::new(
                "provider.model.codex.test-fallback",
                "model",
                "Codex test fallback",
                "0.1.0",
                Vec::new(),
            )
        }

        fn health(&self) -> ProviderHealth {
            ProviderHealth::available()
        }

        fn invoke(&self, _request: ModelRequest) -> KernelResult<ModelResponse> {
            panic!("the controlled Codex stream test must not invoke a fallback provider")
        }
    }

    struct ControlledCodexStreamingRuntime {
        invoke_count: Arc<AtomicUsize>,
        stream_count: Arc<AtomicUsize>,
    }

    impl SdkBackendRuntime for ControlledCodexStreamingRuntime {
        fn backend_kind(&self) -> SdkBackendKind {
            SdkBackendKind::RustNative
        }

        fn health(&self) -> SdkDriverHealth {
            SdkDriverHealth::healthy()
        }

        fn invoke(
            &self,
            request: &SdkRuntimeRequest,
        ) -> Result<SdkRuntimeResponse, SdkRuntimeError> {
            self.invoke_count.fetch_add(1, Ordering::SeqCst);
            Ok(SdkRuntimeResponse::success(
                SdkBackendKind::RustNative,
                &request.capability_id,
                serde_json::json!({
                    "ok": true,
                    "items": ["unexpected invoke"],
                }),
            ))
        }

        fn invoke_streaming(
            &self,
            request: &SdkRuntimeRequest,
            sink: &mut dyn FnMut(serde_json::Value) -> Result<bool, SdkRuntimeError>,
        ) -> Result<(), SdkRuntimeError> {
            self.stream_count.fetch_add(1, Ordering::SeqCst);
            let model_request_id = request
                .operation
                .request_id()
                .expect("model streaming operation has a request id");
            if !sink(serde_json::json!({
                "event": "stream.chunk",
                "model_request_id": model_request_id,
                "sequence": 0,
                "content": "streamed response",
            }))? {
                return Ok(());
            }
            sink(serde_json::json!({
                "event": "stream.done",
                "model_request_id": model_request_id,
                "finish_reason": "stop",
                "provider_session_id": "thread-controlled",
            }))?;
            Ok(())
        }
    }

    fn controlled_codex_slot(
        invoke_count: Arc<AtomicUsize>,
        stream_count: Arc<AtomicUsize>,
    ) -> CodeEngineSlot {
        let mut integration = CodexSdkIntegration::bootstrap().expect("codex bootstrap");
        let negotiation = SdkCapabilityNegotiation {
            agent_id: "agent.facade-test".to_string(),
            binding_id: "binding.facade-test".to_string(),
            binding_version: "0.1.0".to_string(),
            selected: vec![NegotiatedCapability {
                capability_id: SDK_CAPABILITY_MODEL_CHAT.to_string(),
                backend_kind: SdkBackendKind::RustNative,
                driver_id: "driver.facade-test".to_string(),
                runtime_operations: vec![
                    SdkRuntimeOperationKind::ModelChat,
                    SdkRuntimeOperationKind::ModelChatStream,
                ],
            }],
            missing_required: Vec::new(),
            degraded_optional: Vec::new(),
        };
        let runtime = Arc::new(
            SdkRuntimeRouter::new(negotiation).with_rust_runtime(Arc::new(
                ControlledCodexStreamingRuntime {
                    invoke_count,
                    stream_count,
                },
            )),
        );
        integration.model = SdkRuntimeBackedModelProvider::new(
            runtime,
            Arc::new(NeverInvokeFallback),
            SDK_CAPABILITY_MODEL_CHAT,
            "provider.model.codex",
        );
        CodeEngineSlot::Codex(integration)
    }

    #[test]
    fn forwarding_stream_collector_preserves_chunk_order_for_product_output() {
        let mut sink = RecordingStreamSink::default();
        let mut collector = ForwardingStreamCollector::new(&mut sink);

        collector
            .push_chunk(ModelStreamChunk::output("request-1", 0, "Hello"))
            .expect("first chunk");
        collector
            .push_chunk(ModelStreamChunk::output("request-1", 1, " world"))
            .expect("second chunk");

        let deltas = collector.into_deltas();
        assert_eq!(sink.contents, ["Hello", " world"]);
        assert_eq!(deltas, ["Hello", " world"]);
    }

    #[test]
    fn forwarding_stream_collector_discards_empty_transport_chunks() {
        let mut sink = RecordingStreamSink::default();
        let mut collector = ForwardingStreamCollector::new(&mut sink);

        collector
            .push_chunk(ModelStreamChunk::output("request-1", 0, ""))
            .expect("empty chunk is ignored");
        collector
            .push_chunk(ModelStreamChunk::output("request-1", 1, "content"))
            .expect("content chunk");

        let deltas = collector.into_deltas();
        assert_eq!(sink.contents, ["content"]);
        assert_eq!(deltas, ["content"]);
    }

    #[test]
    fn verified_runtime_completion_becomes_provider_neutral_turn_completion() {
        let completion = code_engine_stream_completion(SdkRuntimeStreamCompletion {
            model_request_id: "request-1".to_string(),
            finish_reason: "stop".to_string(),
            provider_session_id: Some("thread-1".to_string()),
        })
        .expect("provider session completion");

        assert_eq!(completion.model_request_id, "request-1");
        assert_eq!(completion.finish_reason, "stop");
        assert_eq!(completion.provider_session_id, "thread-1");
    }

    #[test]
    fn incomplete_runtime_stream_cannot_create_a_first_turn_session() {
        let result = code_engine_stream_completion(SdkRuntimeStreamCompletion {
            model_request_id: "request-1".to_string(),
            finish_reason: "stop".to_string(),
            provider_session_id: None,
        });

        assert!(matches!(result, Err(RuntimeFacadeError::Kernel(_))));
    }

    #[test]
    fn streamed_turn_output_uses_verified_completion_and_ordered_deltas() {
        let input = CodeEngineTurnInput {
            engine_key: "codex".to_string(),
            prompt: "implement this".to_string(),
            ..Default::default()
        };
        let completion = CodeEngineTurnStreamCompletion {
            model_request_id: "request-1".to_string(),
            finish_reason: "stop".to_string(),
            provider_session_id: "thread-1".to_string(),
        };

        let output = build_streamed_turn_output(
            &input,
            vec!["first ".to_string(), "second".to_string()],
            Some(completion.clone()),
        )
        .expect("streamed output");

        assert_eq!(output.assistant_content, "first second");
        assert_eq!(output.provider_session_id.as_deref(), Some("thread-1"));
        assert_eq!(output.stream_deltas, ["first ", "second"]);
        assert_eq!(output.stream_completion, Some(completion));
    }

    #[test]
    fn codex_first_turn_completion_binds_provider_session_and_resume_does_not_invoke() {
        let invoke_count = Arc::new(AtomicUsize::new(0));
        let stream_count = Arc::new(AtomicUsize::new(0));
        let slot = controlled_codex_slot(invoke_count.clone(), stream_count.clone());

        let mut first_sink = RecordingStreamSink::default();
        let first = execute_code_engine_turn_with_stream_sink(
            &slot,
            &CodeEngineTurnInput {
                engine_key: "codex".to_string(),
                model_id: "gpt-test".to_string(),
                prompt: "first turn".to_string(),
                require_live_provider: true,
                ..Default::default()
            },
            &mut first_sink,
        )
        .expect("first streamed turn");
        assert_eq!(first_sink.contents, ["streamed response"]);
        assert_eq!(first.assistant_content, "streamed response");
        assert_eq!(
            first.provider_session_id.as_deref(),
            Some("thread-controlled")
        );
        assert_eq!(
            first
                .stream_completion
                .as_ref()
                .map(|completion| completion.provider_session_id.as_str()),
            Some("thread-controlled")
        );

        let mut resumed_sink = RecordingStreamSink::default();
        let resumed = execute_code_engine_turn_with_stream_sink(
            &slot,
            &CodeEngineTurnInput {
                engine_key: "codex".to_string(),
                model_id: "gpt-test".to_string(),
                provider_session_id: first.provider_session_id.clone(),
                prompt: "resumed turn".to_string(),
                require_live_provider: true,
                ..Default::default()
            },
            &mut resumed_sink,
        )
        .expect("resumed streamed turn");
        assert_eq!(resumed_sink.contents, ["streamed response"]);
        assert_eq!(
            resumed.provider_session_id.as_deref(),
            Some("thread-controlled")
        );
        assert_eq!(resumed.stream_completion, None);
        assert_eq!(invoke_count.load(Ordering::SeqCst), 0);
        assert_eq!(stream_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn executes_turn_for_canonical_codex_engine() {
        let slot = bootstrap_code_engine("codex").expect("codex bootstrap");
        let model_id = slot.list_model_ids().into_iter().next().expect("model id");
        let output = execute_code_engine_turn(
            &slot,
            &CodeEngineTurnInput {
                engine_key: "codex".to_string(),
                model_id,
                prompt: "hello agents facade".to_string(),
                ..Default::default()
            },
        )
        .expect("turn execution");
        assert!(!output.assistant_content.trim().is_empty());
    }

    #[test]
    fn all_canonical_engines_execute_turn() {
        for engine in canonical_code_engine_keys() {
            let slot = bootstrap_code_engine(engine).expect("bootstrap");
            let model_id = slot.list_model_ids().into_iter().next().expect("model id");
            let output = execute_code_engine_turn(
                &slot,
                &CodeEngineTurnInput {
                    engine_key: (*engine).to_string(),
                    model_id,
                    prompt: format!("ping {engine}"),
                    ..Default::default()
                },
            )
            .unwrap_or_else(|error| panic!("turn failed for {engine}: {error}"));
            assert!(!output.assistant_content.trim().is_empty());
        }
    }

    #[test]
    fn blank_prompt_returns_typed_error() {
        let slot = bootstrap_code_engine("codex").expect("codex bootstrap");
        let result = execute_code_engine_turn(
            &slot,
            &CodeEngineTurnInput {
                engine_key: "codex".to_string(),
                model_id: "model".to_string(),
                prompt: "   ".to_string(),
                ..Default::default()
            },
        );
        assert!(matches!(result, Err(RuntimeFacadeError::BlankPrompt)));
    }

    #[test]
    fn engine_mismatch_returns_typed_error() {
        let slot = bootstrap_code_engine("codex").expect("codex bootstrap");
        let result = execute_code_engine_turn(
            &slot,
            &CodeEngineTurnInput {
                engine_key: "gemini".to_string(),
                model_id: "model".to_string(),
                prompt: "hello".to_string(),
                ..Default::default()
            },
        );
        assert!(matches!(
            result,
            Err(RuntimeFacadeError::EngineMismatch { .. })
        ));
    }

    #[test]
    fn build_model_request_preserves_execution_context_and_budget() {
        let slot = bootstrap_code_engine("codex").expect("codex bootstrap");
        let request = build_model_request(&slot, &CodeEngineTurnInput {
            engine_key: "codex".to_string(),
            model_id: "gpt-5-codex".to_string(),
            provider_session_id: Some("session-existing".to_string()),
            prompt: "implement the change".to_string(),
            working_directory: Some(PathBuf::from("C:/workspace/project")),
            timeout_ms: Some(90_000),
            approval_policy: Some("on-request".to_string()),
            sandbox_mode: Some("workspace-write".to_string()),
            full_auto: true,
            skip_git_repo_check: true,
            ephemeral: true,
            require_live_provider: true,
            max_output_bytes: Some(65_536),
            temperature: Some(0.2),
            top_p: Some(0.9),
            max_tokens: Some(4_096),
            access_mode_id: None,
        })
        .expect("model request");

        assert_eq!(request.model_id.as_deref(), Some("gpt-5-codex"));
        assert_eq!(request.session_id.as_deref(), Some("session-existing"));
        assert_eq!(request.timeout_ms, Some(90_000));
        assert_eq!(
            request.metadata_value(WORKING_DIRECTORY_METADATA_KEY),
            Some("C:/workspace/project")
        );
        assert_eq!(
            request.metadata_value(APPROVAL_POLICY_METADATA_KEY),
            Some("on-request")
        );
        assert_eq!(
            request.metadata_value(SANDBOX_MODE_METADATA_KEY),
            Some("workspace-write")
        );
        assert_eq!(request.metadata_value(FULL_AUTO_METADATA_KEY), Some("true"));
        assert_eq!(
            request.metadata_value(SKIP_GIT_REPO_CHECK_METADATA_KEY),
            Some("true")
        );
        assert_eq!(request.metadata_value(EPHEMERAL_METADATA_KEY), Some("true"));
        assert_eq!(
            request.metadata_value(REQUIRE_LIVE_PROVIDER_METADATA_KEY),
            Some("true")
        );
        assert_eq!(
            request.metadata_value(MAX_OUTPUT_BYTES_METADATA_KEY),
            Some("65536")
        );
        assert_eq!(
            request.metadata_value(TEMPERATURE_METADATA_KEY),
            Some("0.2")
        );
        assert_eq!(request.metadata_value(TOP_P_METADATA_KEY), Some("0.9"));
        assert_eq!(
            request.metadata_value(MAX_TOKENS_METADATA_KEY),
            Some("4096")
        );
    }

    #[test]
    fn access_mode_resolution_overrides_legacy_execution_fields() {
        let slot = bootstrap_code_engine("codex").expect("codex bootstrap");
        let request = build_model_request(
            &slot,
            &CodeEngineTurnInput {
                engine_key: "codex".to_string(),
                model_id: "gpt-5-codex".to_string(),
                prompt: "implement the change".to_string(),
                approval_policy: Some("never".to_string()),
                sandbox_mode: Some("danger-full-access".to_string()),
                access_mode_id: Some("approve_for_me".to_string()),
                ..Default::default()
            },
        )
        .expect("model request");

        assert_eq!(
            request.metadata_value(APPROVAL_POLICY_METADATA_KEY),
            Some("on-request")
        );
        assert_eq!(
            request.metadata_value(SANDBOX_MODE_METADATA_KEY),
            Some("workspace-write")
        );
        assert_eq!(
            request.metadata_value("sdkwork.code_engine.approvals_reviewer"),
            Some("auto_review")
        );
    }

    #[test]
    fn provider_session_diagnostic_overrides_input_session() {
        let response = ModelResponse::text("request-1", "provider.model.codex", "done")
            .with_diagnostic("sdk_runtime_session_id=session-provider");
        let input = CodeEngineTurnInput {
            provider_session_id: Some("session-input".to_string()),
            ..Default::default()
        };

        assert_eq!(
            resolve_provider_session_id(&response, &input).as_deref(),
            Some("session-provider")
        );
    }

    #[test]
    fn provider_session_resolution_falls_back_to_input_session() {
        let response = ModelResponse::text("request-1", "provider.model.codex", "done")
            .with_diagnostic("sdk_runtime_mode=sdk_live");
        let input = CodeEngineTurnInput {
            provider_session_id: Some("session-input".to_string()),
            ..Default::default()
        };

        assert_eq!(
            resolve_provider_session_id(&response, &input).as_deref(),
            Some("session-input")
        );
    }

    #[test]
    fn turn_output_preserves_kernel_tool_calls() {
        let response =
            ModelResponse::text("request-1", "provider.model.codex", "done").with_tool_call(
                ToolCall::new("call-1", "codex.shell", r#"{"command":"cargo test"}"#),
            );
        let output = build_turn_output(response, &CodeEngineTurnInput::default());

        assert_eq!(output.tool_calls.len(), 1);
        assert_eq!(output.tool_calls[0].tool_call_id, "call-1");
        assert_eq!(output.tool_calls[0].tool_id, "codex.shell");
        assert_eq!(output.stream_completion, None);
    }

    #[test]
    fn output_budget_is_bounded_by_the_facade_limit() {
        let input = CodeEngineTurnInput {
            max_output_bytes: Some(MAX_CODE_ENGINE_STREAM_OUTPUT_BYTES + 1),
            ..Default::default()
        };

        assert_eq!(
            effective_max_output_bytes(&input),
            MAX_CODE_ENGINE_STREAM_OUTPUT_BYTES
        );
        assert!(validate_output_size(
            MAX_CODE_ENGINE_STREAM_OUTPUT_BYTES,
            effective_max_output_bytes(&input)
        )
        .is_ok());
        assert!(validate_output_size(
            MAX_CODE_ENGINE_STREAM_OUTPUT_BYTES + 1,
            effective_max_output_bytes(&input)
        )
        .is_err());
    }
}
