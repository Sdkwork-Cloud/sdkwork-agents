use sdkwork_agent_kernel::ModelRequest;
use sdkwork_utils_rust::string::is_blank;

use crate::code_engines::CodeEngineSlot;
use crate::error::{RuntimeFacadeError, RuntimeFacadeResult};

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

    let model_request_id = format!("agents-turn-{}", sdkwork_utils_rust::uuid());
    let mut model_request = ModelRequest::new(model_request_id, vec![input.prompt.clone()]);
    if !is_blank(Some(input.model_id.as_str())) {
        model_request.model_id = Some(input.model_id.clone());
    }
    if let Some(native_session_id) = input.native_session_id.as_ref() {
        model_request.session_id = Some(native_session_id.clone());
    }

    let response = slot
        .invoke_model(model_request)
        .map_err(|error| RuntimeFacadeError::Kernel(error.to_string()))?;

    Ok(CodeEngineTurnOutput {
        assistant_content: response.messages.join("\n"),
        native_session_id: input.native_session_id.clone(),
    })
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
