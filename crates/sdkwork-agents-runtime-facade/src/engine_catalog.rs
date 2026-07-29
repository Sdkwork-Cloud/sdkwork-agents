use sdkwork_agent_kernel::{
    AgentExecutionAccessModeDescriptor, AgentExecutionApprovalBehavior,
    AgentExecutionNetworkAccess, AgentExecutionRiskLevel, AgentExecutionWorkspaceAccess,
    ModelDescriptor,
};
use serde::{Deserialize, Serialize};

use crate::code_engines::{
    bootstrap_code_engine, bootstrappable_engine_keys, canonical_code_engine_keys, CodeEngineSlot,
};

/// Engine model catalog entry exposed by the agents runtime facade.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeEngineModelCatalogEntry {
    pub engine_key: String,
    pub model_id: String,
    pub label: String,
    pub description: String,
    pub provider_id: String,
    pub binding_id: String,
    pub default_for_engine: bool,
}

/// Aggregated code-engine catalog for one bootstrapped host.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeEngineCatalog {
    pub engines: Vec<CodeEngineCatalogEngine>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeEngineCatalogEngine {
    pub engine_key: String,
    pub tier: String,
    pub agent_id: String,
    pub binding_id: String,
    pub models: Vec<CodeEngineModelCatalogEntry>,
    pub default_access_mode_id: String,
    pub access_modes: Vec<CodeEngineAccessModeCatalogEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeEngineAccessModeCatalogEntry {
    pub mode_id: String,
    pub display_name: String,
    pub description: String,
    pub approval_behavior: String,
    pub workspace_access: String,
    pub network_access: String,
    pub risk_level: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
}

fn access_mode_to_catalog_entry(
    access_mode: &AgentExecutionAccessModeDescriptor,
) -> CodeEngineAccessModeCatalogEntry {
    CodeEngineAccessModeCatalogEntry {
        mode_id: access_mode.mode_id.clone(),
        display_name: access_mode.display_name.clone(),
        description: access_mode.description.clone(),
        approval_behavior: match access_mode.approval_behavior {
            AgentExecutionApprovalBehavior::UserReview => "user_review",
            AgentExecutionApprovalBehavior::AutomaticReview => "automatic_review",
            AgentExecutionApprovalBehavior::Never => "never",
            AgentExecutionApprovalBehavior::ProviderDefault => "provider_default",
        }
        .to_string(),
        workspace_access: match access_mode.workspace_access {
            AgentExecutionWorkspaceAccess::ReadOnly => "read_only",
            AgentExecutionWorkspaceAccess::WorkspaceWrite => "workspace_write",
            AgentExecutionWorkspaceAccess::FullAccess => "full_access",
            AgentExecutionWorkspaceAccess::ProviderDefault => "provider_default",
        }
        .to_string(),
        network_access: match access_mode.network_access {
            AgentExecutionNetworkAccess::Restricted => "restricted",
            AgentExecutionNetworkAccess::Enabled => "enabled",
            AgentExecutionNetworkAccess::ProviderDefault => "provider_default",
        }
        .to_string(),
        risk_level: match access_mode.risk_level {
            AgentExecutionRiskLevel::Scoped => "scoped",
            AgentExecutionRiskLevel::Elevated => "elevated",
            AgentExecutionRiskLevel::Unrestricted => "unrestricted",
        }
        .to_string(),
        enabled: access_mode.enabled,
        disabled_reason: access_mode.disabled_reason.clone(),
    }
}

pub fn model_descriptor_to_catalog_entry(
    engine_key: &str,
    binding_id: &str,
    descriptor: &ModelDescriptor,
    default_for_engine: bool,
) -> CodeEngineModelCatalogEntry {
    CodeEngineModelCatalogEntry {
        engine_key: engine_key.to_string(),
        model_id: descriptor.model_id.clone(),
        label: descriptor.display_name.clone(),
        description: descriptor.family.clone(),
        provider_id: descriptor.provider_id.clone(),
        binding_id: binding_id.to_string(),
        default_for_engine,
    }
}

pub fn list_slot_catalog_entries(slot: &CodeEngineSlot) -> Vec<CodeEngineModelCatalogEntry> {
    slot.list_model_descriptors()
        .iter()
        .enumerate()
        .map(|(index, descriptor)| {
            model_descriptor_to_catalog_entry(
                slot.engine_key(),
                slot.binding_id(),
                descriptor,
                index == 0,
            )
        })
        .collect()
}

pub fn build_code_engine_catalog(slots: &[&CodeEngineSlot]) -> CodeEngineCatalog {
    let engines = slots
        .iter()
        .filter_map(|slot| {
            let agent_id = super::code_engines::code_engine_agent_id(slot.engine_key())?;
            let tier = super::code_engines::engine_catalog_tier(slot.engine_key())
                .unwrap_or("unknown")
                .to_string();
            let execution_settings = slot.execution_settings_spec().ok();
            Some(CodeEngineCatalogEngine {
                engine_key: slot.engine_key().to_string(),
                tier,
                agent_id: agent_id.to_string(),
                binding_id: slot.binding_id().to_string(),
                models: list_slot_catalog_entries(slot),
                default_access_mode_id: execution_settings
                    .as_ref()
                    .map(|spec| spec.default_access_mode_id.clone())
                    .unwrap_or_default(),
                access_modes: execution_settings
                    .map(|spec| {
                        spec.access_modes
                            .iter()
                            .map(access_mode_to_catalog_entry)
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect();
    CodeEngineCatalog { engines }
}

pub fn bootstrap_canonical_code_engine_catalog(
) -> Result<CodeEngineCatalog, crate::code_engines::CodeEngineBootstrapError> {
    bootstrap_code_engine_catalog(canonical_code_engine_keys())
}

pub fn bootstrap_bootstrappable_code_engine_catalog(
) -> Result<CodeEngineCatalog, crate::code_engines::CodeEngineBootstrapError> {
    bootstrap_code_engine_catalog(&bootstrappable_engine_keys())
}

fn bootstrap_code_engine_catalog(
    engine_keys: &[&str],
) -> Result<CodeEngineCatalog, crate::code_engines::CodeEngineBootstrapError> {
    let mut slots = Vec::new();
    for engine_key in engine_keys {
        let slot = bootstrap_code_engine(engine_key)?;
        slots.push(slot);
    }
    Ok(build_code_engine_catalog(&slots.iter().collect::<Vec<_>>()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_agent_kernel::ModelResponseFormat;

    #[test]
    fn projects_model_descriptor_to_catalog_entry() {
        let descriptor =
            ModelDescriptor::new("codex-1", "provider.model.codex", "Codex 1", "codex")
                .with_response_format(ModelResponseFormat::Text);

        let entry = model_descriptor_to_catalog_entry("codex", "binding.codex", &descriptor, true);
        assert_eq!(entry.engine_key, "codex");
        assert_eq!(entry.model_id, "codex-1");
        assert!(entry.default_for_engine);
    }

    #[test]
    fn canonical_catalog_bootstraps_all_engines() {
        let catalog = bootstrap_canonical_code_engine_catalog().expect("catalog bootstrap");
        assert_eq!(catalog.engines.len(), canonical_code_engine_keys().len());
        assert_eq!(
            catalog
                .engines
                .iter()
                .map(|engine| engine.engine_key.as_str())
                .collect::<Vec<_>>(),
            canonical_code_engine_keys()
        );
        for engine in &catalog.engines {
            assert!(!engine.models.is_empty());
            assert!(!engine.tier.is_empty());
            assert!(!engine.default_access_mode_id.is_empty());
            assert!(!engine.access_modes.is_empty());
        }

        let codex = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("Codex catalog entry");
        assert_eq!(codex.default_access_mode_id, "ask_for_approval");
        assert_eq!(codex.access_modes.len(), 3);

        let gemini = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "gemini")
            .expect("Gemini catalog entry");
        assert_eq!(gemini.default_access_mode_id, "sdk_default");
        assert_eq!(gemini.access_modes.len(), 1);
    }

    #[test]
    fn bootstrappable_catalog_includes_opt_in_engines() {
        let catalog =
            bootstrap_bootstrappable_code_engine_catalog().expect("bootstrappable catalog");
        assert_eq!(catalog.engines.len(), bootstrappable_engine_keys().len());
    }
}
