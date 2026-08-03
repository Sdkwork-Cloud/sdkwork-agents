//! Bridges managed-agent runtime executions to `sdkwork-agents-runtime-facade`.
//!
//! Preview responses and prompt optimizations must not use deterministic local
//! contract stubs when a canonical code-engine binding is active.

use std::sync::Arc;

use sdkwork_agents_runtime_facade::{
    bootstrap_code_engine, bootstrappable_engine_keys, code_engine_binding_id,
    execute_code_engine_turn, AgentsCodeEngineHost, CodeEngineTurnInput,
};
use sdkwork_utils_rust::string::is_blank;

use crate::domain::AgentProviderBindingRecord;

pub const RUNTIME_MODE_FACADE: &str = "agents-runtime-facade";
pub const RUNTIME_MODE_CONTRACT_FALLBACK: &str = "agents-contract-fallback";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewExecutionOutput {
    pub content: String,
    pub model_id: Option<String>,
    pub runtime_mode: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptOptimizationOutput {
    pub optimized_prompt: String,
    pub runtime_mode: &'static str,
}

pub fn engine_key_for_binding_id(binding_id: &str) -> Option<&'static str> {
    bootstrappable_engine_keys()
        .iter()
        .find(|&engine_key| code_engine_binding_id(engine_key) == Some(binding_id))
        .map(|v| v as _)
}

pub fn engine_key_for_provider_identity(
    binding_id: Option<&str>,
    provider_id: Option<&str>,
) -> Option<&'static str> {
    binding_id.and_then(engine_key_for_binding_id).or_else(|| {
        let provider_id = provider_id?;
        bootstrappable_engine_keys()
            .iter()
            .copied()
            .find(|engine_key| {
                bootstrap_code_engine(engine_key).ok().is_some_and(|slot| {
                    slot.list_model_descriptors()
                        .iter()
                        .any(|descriptor| descriptor.provider_id == provider_id)
                })
            })
    })
}

fn resolve_engine_and_model(
    active_binding: Option<&AgentProviderBindingRecord>,
    requested_model: Option<&str>,
) -> Option<(String, String)> {
    let binding = active_binding?;
    let engine_key = engine_key_for_binding_id(binding.binding_id.as_str())?.to_string();
    let model_id = if is_blank(requested_model) {
        bootstrap_code_engine(engine_key.as_str())
            .ok()?
            .list_model_ids()
            .into_iter()
            .next()?
    } else {
        requested_model.unwrap_or("").to_string()
    };
    Some((engine_key, model_id))
}

pub fn execute_preview_response(
    active_binding: Option<&AgentProviderBindingRecord>,
    content: &str,
    requested_model: Option<&str>,
) -> PreviewExecutionOutput {
    if let Some((engine_key, model_id)) = resolve_engine_and_model(active_binding, requested_model)
    {
        if let Ok(slot) = bootstrap_code_engine(engine_key.as_str()) {
            if let Ok(output) = execute_code_engine_turn(
                &slot,
                &CodeEngineTurnInput {
                    engine_key: engine_key.clone(),
                    model_id: model_id.clone(),
                    prompt: content.to_string(),
                    ..Default::default()
                },
            ) {
                return PreviewExecutionOutput {
                    content: output.assistant_content,
                    model_id: Some(model_id),
                    runtime_mode: RUNTIME_MODE_FACADE,
                };
            }
        }
    }

    PreviewExecutionOutput {
        content: content.to_string(),
        model_id: requested_model.map(str::to_string),
        runtime_mode: RUNTIME_MODE_CONTRACT_FALLBACK,
    }
}

pub fn execute_prompt_optimization(
    active_binding: Option<&AgentProviderBindingRecord>,
    prompt: &str,
) -> PromptOptimizationOutput {
    let optimization_prompt = format!(
        "Optimize the following agent prompt for clarity and effectiveness. \
Return only the optimized prompt text with no preamble.\n\n{prompt}"
    );

    if let Some((engine_key, model_id)) = resolve_engine_and_model(active_binding, None) {
        if let Ok(slot) = bootstrap_code_engine(engine_key.as_str()) {
            if let Ok(output) = execute_code_engine_turn(
                &slot,
                &CodeEngineTurnInput {
                    engine_key: engine_key.clone(),
                    model_id,
                    prompt: optimization_prompt,
                    ..Default::default()
                },
            ) {
                let optimized = output.assistant_content.trim().to_string();
                if !optimized.is_empty() {
                    return PromptOptimizationOutput {
                        optimized_prompt: optimized,
                        runtime_mode: RUNTIME_MODE_FACADE,
                    };
                }
            }
        }
    }

    PromptOptimizationOutput {
        optimized_prompt: normalize_prompt_text(prompt),
        runtime_mode: RUNTIME_MODE_CONTRACT_FALLBACK,
    }
}

pub fn shared_code_engine_host() -> Option<Arc<AgentsCodeEngineHost>> {
    use std::sync::{Arc, Mutex};
    // A failed bootstrap must not be cached: engine availability can recover
    // (e.g. the provider directory becomes readable again) and a permanently
    // cached None would force a process restart to ever synchronize again.
    static HOST: Mutex<Option<Arc<AgentsCodeEngineHost>>> = Mutex::new(None);
    let mut guard = HOST.lock().expect("provider engine host mutex poisoned");
    if guard.is_none() {
        let host = AgentsCodeEngineHost::bootstrap_selected(
            &bootstrappable_engine_keys(),
            sdkwork_agents_runtime_facade::LiveInteractionRegistry::new(),
        );
        if host.engine_keys().next().is_some() {
            *guard = Some(Arc::new(host));
        }
    }
    guard.clone()
}

fn normalize_prompt_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AgentImplementationKind, AgentProviderBindingRecord};

    fn sample_binding(binding_id: &str) -> AgentProviderBindingRecord {
        AgentProviderBindingRecord {
            id: 1,
            tenant_id: 100001,
            agent_id: "agent.test".to_string(),
            binding_id: binding_id.to_string(),
            provider_id: "provider.test".to_string(),
            implementation_kind: AgentImplementationKind::TypedLocalProvider,
            configuration_profile_id: "profile.test".to_string(),
            capabilities: vec!["model.chat".to_string()],
            active: true,
            version: 1,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn maps_canonical_binding_ids_to_engine_keys() {
        assert_eq!(
            engine_key_for_binding_id("binding.agent-provider.codex"),
            Some("codex")
        );
        assert_eq!(
            engine_key_for_binding_id("binding.agent-provider.opencode"),
            Some("opencode")
        );
        assert!(engine_key_for_binding_id("binding.unknown").is_none());
    }

    #[test]
    fn resolves_engine_from_provider_identity_when_binding_id_is_custom() {
        assert_eq!(
            engine_key_for_provider_identity(Some("binding.custom"), Some("provider.model.codex")),
            Some("codex")
        );
    }

    #[test]
    fn preview_resolves_codex_binding_through_runtime_facade_catalog() {
        let binding = sample_binding("binding.agent-provider.codex");
        let (engine_key, model_id) = resolve_engine_and_model(Some(&binding), None)
            .expect("Codex binding must resolve through the runtime facade catalog");

        assert_eq!(engine_key, "codex");
        assert!(!model_id.trim().is_empty());
    }

    #[test]
    fn preview_falls_back_without_code_engine_binding() {
        let output = execute_preview_response(
            Some(&sample_binding("binding.custom")),
            "hello fallback",
            None,
        );
        assert_eq!(output.runtime_mode, RUNTIME_MODE_CONTRACT_FALLBACK);
        assert_eq!(output.content, "hello fallback");
    }

    #[test]
    fn prompt_optimization_avoids_deterministic_local_contract_mode() {
        let output = execute_prompt_optimization(
            Some(&sample_binding("binding.agent-provider.codex")),
            "  make   this   better  ",
        );
        assert_ne!(output.runtime_mode, "deterministic-local-contract");
    }
}
