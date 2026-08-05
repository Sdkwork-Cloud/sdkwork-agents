use sdkwork_agent_kernel::{
    AgentExecutionAccessModeDescriptor, AgentExecutionApprovalBehavior,
    AgentExecutionNetworkAccess, AgentExecutionRiskLevel, AgentExecutionWorkspaceAccess,
    ModelDescriptor,
};
use serde::{Deserialize, Serialize};

use crate::agent_engines::{
    bootstrap_agent_engine, bootstrappable_engine_keys, canonical_agent_engine_keys,
    engine_catalog_kind, AgentEngineSlot,
};

/// Engine model catalog entry exposed by the agents runtime facade.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEngineModelCatalogEntry {
    pub engine_key: String,
    pub model_id: String,
    pub label: String,
    pub description: String,
    pub provider_id: String,
    pub binding_id: String,
    pub default_for_engine: bool,
}

/// Aggregated agent-engine catalog for one bootstrapped host.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEngineCatalog {
    pub engines: Vec<AgentEngineCatalogEngine>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEngineCatalogEngine {
    pub engine_key: String,
    pub engine_kind: String,
    pub tier: String,
    pub agent_id: String,
    pub binding_id: String,
    pub models: Vec<AgentEngineModelCatalogEntry>,
    pub default_access_mode_id: String,
    pub access_modes: Vec<AgentEngineAccessModeCatalogEntry>,
    /// Whether the engine provider is bootstrapped and usable in this runtime.
    /// Unavailable engines stay in the catalog so clients always see the full
    /// platform engine inventory instead of silently losing entries.
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEngineAccessModeCatalogEntry {
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
) -> AgentEngineAccessModeCatalogEntry {
    AgentEngineAccessModeCatalogEntry {
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
) -> AgentEngineModelCatalogEntry {
    AgentEngineModelCatalogEntry {
        engine_key: engine_key.to_string(),
        model_id: descriptor.model_id.clone(),
        label: descriptor.display_name.clone(),
        description: descriptor.family.clone(),
        provider_id: descriptor.provider_id.clone(),
        binding_id: binding_id.to_string(),
        default_for_engine,
    }
}

pub fn list_slot_catalog_entries(slot: &AgentEngineSlot) -> Vec<AgentEngineModelCatalogEntry> {
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

pub fn build_agent_engine_catalog(slots: &[&AgentEngineSlot]) -> AgentEngineCatalog {
    let engines = slots
        .iter()
        .filter_map(|slot| {
            let agent_id = super::agent_engines::agent_engine_agent_id(slot.engine_key())?;
            let tier = super::agent_engines::engine_catalog_tier(slot.engine_key())
                .unwrap_or("unknown")
                .to_string();
            let execution_settings = slot.execution_settings_spec().ok();
            Some(AgentEngineCatalogEngine {
                engine_key: slot.engine_key().to_string(),
                engine_kind: engine_catalog_kind(slot.engine_key())
                    .unwrap_or("unknown")
                    .to_string(),
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
                available: true,
                unavailable_reason: None,
            })
        })
        .collect();
    AgentEngineCatalog { engines }
}

/// Builds the full platform engine inventory: bootstrapped slots plus engines
/// whose provider failed to bootstrap. Unavailable engines keep their stable
/// identity metadata (engine key, kind, tier, agent and binding ids) with an
/// empty model set and an explicit `available: false` reason so settings and
/// session surfaces can render the complete engine list with runtime state.
pub fn build_complete_agent_engine_catalog(
    slots: &[&AgentEngineSlot],
    unavailable: &std::collections::HashMap<String, crate::agent_engines::AgentEngineBootstrapError>,
) -> AgentEngineCatalog {
    let mut engines = build_agent_engine_catalog(slots).engines;
    let mut order: Vec<&str> = slots.iter().map(|slot| slot.engine_key()).collect();
    for engine_key in crate::agent_engines::bootstrappable_engine_keys() {
        if engines.iter().any(|engine| engine.engine_key == engine_key) {
            continue;
        }
        let Some(agent_id) = crate::agent_engines::agent_engine_agent_id(engine_key) else {
            continue;
        };
        engines.push(AgentEngineCatalogEngine {
            engine_key: engine_key.to_string(),
            engine_kind: engine_catalog_kind(engine_key).unwrap_or("unknown").to_string(),
            tier: crate::agent_engines::engine_catalog_tier(engine_key)
                .unwrap_or("unknown")
                .to_string(),
            agent_id: agent_id.to_string(),
            binding_id: crate::agent_engines::agent_engine_binding_id(engine_key)
                .unwrap_or("")
                .to_string(),
            models: Vec::new(),
            default_access_mode_id: String::new(),
            access_modes: Vec::new(),
            available: false,
            unavailable_reason: Some(
                unavailable
                    .get(engine_key)
                    .map(ToString::to_string)
                    .unwrap_or_else(|| {
                        format!("{engine_key} is not bootstrapped in this runtime profile")
                    }),
            ),
        });
        order.push(engine_key);
    }
    engines.sort_by_key(|engine| {
        order
            .iter()
            .position(|key| key == &engine.engine_key.as_str())
            .unwrap_or(usize::MAX)
    });
    AgentEngineCatalog { engines }
}

pub fn bootstrap_canonical_agent_engine_catalog(
) -> Result<AgentEngineCatalog, crate::agent_engines::AgentEngineBootstrapError> {
    bootstrap_agent_engine_catalog(canonical_agent_engine_keys())
}

fn bootstrap_agent_engine_catalog(
    engine_keys: &[&str],
) -> Result<AgentEngineCatalog, crate::agent_engines::AgentEngineBootstrapError> {
    let mut slots = Vec::new();
    for engine_key in engine_keys {
        let slot = bootstrap_agent_engine(engine_key)?;
        slots.push(slot);
    }
    Ok(build_agent_engine_catalog(&slots.iter().collect::<Vec<_>>()))
}

/// Bootstraps every bootstrappable engine, tolerating individual provider
/// failures exactly like the production host (`bootstrap_selected`): engines
/// whose provider transport cannot resolve (e.g. an external SDK package is
/// not installed) are skipped instead of failing the whole catalog.
pub fn bootstrap_bootstrappable_agent_engine_catalog() -> Result<AgentEngineCatalog, crate::agent_engines::AgentEngineBootstrapError> {
    let mut slots = Vec::new();
    for engine_key in bootstrappable_engine_keys() {
        if let Ok(slot) = bootstrap_agent_engine(engine_key) {
            slots.push(slot);
        }
    }
    Ok(build_agent_engine_catalog(&slots.iter().collect::<Vec<_>>()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_agent_kernel::ModelResponseFormat;

    #[test]
    fn projects_model_descriptor_to_catalog_entry() {
        let descriptor = ModelDescriptor::new("codex-1", "provider.codex", "Codex 1", "codex")
            .with_response_format(ModelResponseFormat::Text);

        let entry = model_descriptor_to_catalog_entry("codex", "binding.codex", &descriptor, true);
        assert_eq!(entry.engine_key, "codex");
        assert_eq!(entry.model_id, "codex-1");
        assert!(entry.default_for_engine);
    }

    #[test]
    fn canonical_catalog_bootstraps_all_engines() {
        let catalog = bootstrap_canonical_agent_engine_catalog().expect("catalog bootstrap");
        assert_eq!(catalog.engines.len(), canonical_agent_engine_keys().len());
        assert_eq!(
            catalog
                .engines
                .iter()
                .map(|engine| engine.engine_key.as_str())
                .collect::<Vec<_>>(),
            canonical_agent_engine_keys()
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
            bootstrap_bootstrappable_agent_engine_catalog().expect("bootstrappable catalog");
        // The full inventory always lists every bootstrappable engine, even
        // when a provider failed to bootstrap: unavailable engines stay in
        // the catalog with `available: false` so clients never lose entries.
        assert_eq!(
            catalog.engines.len(),
            bootstrappable_engine_keys().len(),
            "every bootstrappable engine must be listed"
        );
        for engine_key in bootstrappable_engine_keys() {
            let engine = catalog
                .engines
                .iter()
                .find(|engine| engine.engine_key == engine_key)
                .unwrap_or_else(|| panic!("bootstrappable engine {engine_key} missing from catalog"));
            assert_eq!(
                engine.available,
                !engine.models.is_empty(),
                "engine {} availability must match its model inventory",
                engine_key
            );
            if engine.available {
                assert!(!engine.tier.is_empty());
                assert!(!engine.agent_id.is_empty());
            } else {
                assert!(
                    engine.unavailable_reason.is_some(),
                    "unavailable engine {} must carry a reason",
                    engine_key
                );
            }
        }
    }

    #[test]
    fn bootstrappable_catalog_classifies_every_engine_kind() {
        let catalog =
            bootstrap_bootstrappable_agent_engine_catalog().expect("bootstrappable catalog");
        for engine in &catalog.engines {
            assert!(
                matches!(engine.engine_kind.as_str(), "code" | "work" | "simple"),
                "engine {} has unknown kind {}",
                engine.engine_key,
                engine.engine_kind
            );
        }
        let kind = |engine_key: &str| -> &str {
            catalog
                .engines
                .iter()
                .find(|engine| engine.engine_key == engine_key)
                .map(|engine| engine.engine_kind.as_str())
                .unwrap_or("missing")
        };
        assert_eq!(kind("codex"), "code");
        assert_eq!(kind("mimo-code"), "code");
        assert_eq!(kind("openclaw"), "work");
        assert_eq!(kind("hermes"), "work");
        assert_eq!(kind("rig"), "simple");
    }
}
