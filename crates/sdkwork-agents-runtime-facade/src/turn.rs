use std::path::PathBuf;

use sdkwork_agent_kernel::{ModelRequest, ModelResponse};
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
const NATIVE_SESSION_DIAGNOSTIC_KEYS: [&str; 6] = [
    "sdk_runtime_session_id",
    "sdk_runtime_native_session_id",
    "sdkwork.code_engine.native_session_id",
    "sdkwork.provider.session_id",
    "provider_session_id",
    "native_session_id",
];

/// Product-neutral code-engine turn input consumed by the agents runtime facade.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CodeEngineTurnInput {
    pub engine_key: String,
    pub model_id: String,
    pub native_session_id: Option<String>,
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
}

/// Product-neutral code-engine turn output produced by the agents runtime facade.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeEngineTurnOutput {
    pub assistant_content: String,
    pub native_session_id: Option<String>,
    /// Token/word deltas when streaming is available; empty when invoke-only.
    pub stream_deltas: Vec<String>,
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

    let model_request = build_model_request(input);

    let response = slot
        .invoke_model(model_request)
        .map_err(|error| RuntimeFacadeError::Kernel(error.to_string()))?;

    let assistant_content = response.messages.join("\n");
    validate_output_size(assistant_content.len(), effective_max_output_bytes(input))?;

    Ok(CodeEngineTurnOutput {
        assistant_content,
        native_session_id: resolve_native_session_id(&response, input),
        stream_deltas: Vec::new(),
    })
}

fn build_model_request(input: &CodeEngineTurnInput) -> ModelRequest {
    let model_request_id = format!("agents-turn-{}", sdkwork_utils_rust::uuid());
    let mut model_request = ModelRequest::new(model_request_id, vec![input.prompt.clone()]);
    if !is_blank(Some(input.model_id.as_str())) {
        model_request.model_id = Some(input.model_id.clone());
    }
    if let Some(native_session_id) = input.native_session_id.as_ref() {
        model_request.session_id = Some(native_session_id.clone());
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
    model_request
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

fn resolve_native_session_id(
    response: &ModelResponse,
    input: &CodeEngineTurnInput,
) -> Option<String> {
    response
        .diagnostics
        .iter()
        .find_map(|diagnostic| {
            let (key, value) = diagnostic.split_once('=')?;
            NATIVE_SESSION_DIAGNOSTIC_KEYS
                .contains(&key.trim())
                .then(|| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .or_else(|| input.native_session_id.clone())
}

/// Execute a turn preferring provider stream chunks when supported.
pub fn execute_code_engine_turn_with_stream(
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

    let model_request = build_model_request(input);
    if let Ok(chunks) = slot.stream_model(model_request.clone()) {
        if !chunks.is_empty() {
            if chunks.len() > MAX_CODE_ENGINE_STREAM_CHUNKS {
                return Err(RuntimeFacadeError::Kernel(format!(
                    "stream exceeded maximum chunk count of {MAX_CODE_ENGINE_STREAM_CHUNKS}"
                )));
            }
            let stream_deltas: Vec<String> =
                chunks.into_iter().map(|chunk| chunk.content).collect();
            let assistant_content = stream_deltas.join("");
            validate_output_size(assistant_content.len(), effective_max_output_bytes(input))?;
            if !assistant_content.trim().is_empty() {
                return Ok(CodeEngineTurnOutput {
                    assistant_content,
                    native_session_id: input.native_session_id.clone(),
                    stream_deltas,
                });
            }
        }
    }

    execute_code_engine_turn(slot, input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code_engines::{bootstrap_code_engine, canonical_code_engine_keys};

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
        let request = build_model_request(&CodeEngineTurnInput {
            engine_key: "codex".to_string(),
            model_id: "gpt-5-codex".to_string(),
            native_session_id: Some("session-existing".to_string()),
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
        });

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
    fn provider_native_session_diagnostic_overrides_input_session() {
        let response = ModelResponse::text("request-1", "provider.model.codex", "done")
            .with_diagnostic("sdk_runtime_session_id=session-provider");
        let input = CodeEngineTurnInput {
            native_session_id: Some("session-input".to_string()),
            ..Default::default()
        };

        assert_eq!(
            resolve_native_session_id(&response, &input).as_deref(),
            Some("session-provider")
        );
    }

    #[test]
    fn native_session_resolution_falls_back_to_input_session() {
        let response = ModelResponse::text("request-1", "provider.model.codex", "done")
            .with_diagnostic("sdk_runtime_mode=sdk_live");
        let input = CodeEngineTurnInput {
            native_session_id: Some("session-input".to_string()),
            ..Default::default()
        };

        assert_eq!(
            resolve_native_session_id(&response, &input).as_deref(),
            Some("session-input")
        );
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
