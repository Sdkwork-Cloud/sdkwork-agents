use sdkwork_agent_kernel::ModelRequest;
use sdkwork_utils_rust::string::is_blank;

use crate::code_engines::CodeEngineSlot;
use crate::error::{RuntimeFacadeError, RuntimeFacadeResult};

/// Maximum prompt size accepted by the runtime facade (1 MiB).
pub const MAX_CODE_ENGINE_PROMPT_BYTES: usize = 1_048_576;
/// Maximum stream chunks collected before failing closed.
pub const MAX_CODE_ENGINE_STREAM_CHUNKS: usize = 8_192;
/// Maximum aggregated stream output size (4 MiB).
pub const MAX_CODE_ENGINE_STREAM_OUTPUT_BYTES: usize = 4_194_304;

/// Product-neutral code-engine turn input consumed by the agents runtime facade.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeEngineTurnInput {
    pub engine_key: String,
    pub model_id: String,
    pub native_session_id: Option<String>,
    pub prompt: String,
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

    Ok(CodeEngineTurnOutput {
        assistant_content: response.messages.join("\n"),
        native_session_id: input.native_session_id.clone(),
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
    model_request
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
            if assistant_content.len() > MAX_CODE_ENGINE_STREAM_OUTPUT_BYTES {
                return Err(RuntimeFacadeError::Kernel(format!(
                    "stream output exceeds maximum size of {MAX_CODE_ENGINE_STREAM_OUTPUT_BYTES} bytes"
                )));
            }
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
                native_session_id: None,
                prompt: "hello agents facade".to_string(),
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
                    native_session_id: None,
                    prompt: format!("ping {engine}"),
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
                native_session_id: None,
                prompt: "   ".to_string(),
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
                native_session_id: None,
                prompt: "hello".to_string(),
            },
        );
        assert!(matches!(
            result,
            Err(RuntimeFacadeError::EngineMismatch { .. })
        ));
    }
}
