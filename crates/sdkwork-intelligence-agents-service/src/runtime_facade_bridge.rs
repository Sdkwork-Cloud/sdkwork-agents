use sdkwork_agent_kernel::KernelError;
use sdkwork_agents_runtime_facade::{
    bootstrap_code_engine, canonical_code_engine_keys, code_engine_binding_id,
    execute_code_engine_turn, CodeEngineTurnInput,
};
use sdkwork_utils_rust::string::is_blank;

pub fn resolve_code_engine_key_for_binding(binding_id: &str) -> Option<&'static str> {
    canonical_code_engine_keys()
        .iter()
        .copied()
        .find(|engine_key| code_engine_binding_id(engine_key) == Some(binding_id))
}

pub fn invoke_code_engine_prompt(
    binding_id: &str,
    model_id: Option<&str>,
    prompt: &str,
) -> Result<String, KernelError> {
    if is_blank(Some(prompt)) {
        return Err(KernelError::validation("prompt must not be blank"));
    }
    let engine_key = resolve_code_engine_key_for_binding(binding_id).ok_or_else(|| {
        KernelError::validation(format!(
            "provider binding \"{binding_id}\" is not mapped to a canonical code engine"
        ))
    })?;
    let slot = bootstrap_code_engine(engine_key).map_err(|error| {
        KernelError::Internal {
            message: format!("code engine bootstrap failed for {engine_key}: {error}"),
        }
    })?;
    let model = model_id
        .filter(|value| !is_blank(Some(value)))
        .map(str::to_string)
        .or_else(|| slot.list_model_ids().into_iter().next())
        .ok_or_else(|| KernelError::validation("no model available for provider binding"))?;
    let output = execute_code_engine_turn(
        &slot,
        &CodeEngineTurnInput {
            engine_key: engine_key.to_string(),
            model_id: model,
            native_session_id: None,
            prompt: prompt.to_string(),
        },
    )
    .map_err(|error| KernelError::Internal {
        message: error,
    })?;
    Ok(output.assistant_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_codex_binding_to_engine_key() {
        let binding_id = code_engine_binding_id("codex").expect("codex binding");
        assert_eq!(
            resolve_code_engine_key_for_binding(binding_id),
            Some("codex")
        );
    }

    #[test]
    fn invokes_codex_runtime_for_binding() {
        let binding_id = code_engine_binding_id("codex").expect("codex binding");
        let content = invoke_code_engine_prompt(binding_id, None, "hello agents runtime")
            .expect("runtime invoke");
        assert!(!content.trim().is_empty());
    }
}
