use std::collections::HashMap;

use crate::agent_engines::{
    bootstrap_agent_engine, bootstrappable_engine_keys, AgentEngineBootstrapError,
    AgentEngineInteractionResolution, AgentEngineSlot,
};
use crate::agent_engine_catalog::{
    build_complete_agent_engine_catalog, AgentEngineCatalog,
};
use crate::error::{RuntimeFacadeError, RuntimeFacadeResult};
use crate::live_interaction::{ApprovalDecision, LiveInteractionRegistry, UserQuestionAnswer};
use crate::provider_sessions::{
    discover_provider_sessions, load_provider_session_children, load_provider_session_messages,
    load_provider_session_messages_for_directory, ProviderSessionInventorySelector,
    ProviderSessionInventorySnapshot,
};
use crate::turn::{
    cancel_agent_engine_turn, execute_agent_engine_turn, AgentEngineTurnCancellation,
    AgentEngineTurnInput, AgentEngineTurnOutput,
};

/// Agents-owned runtime host for canonical agent-engine provider slots.
pub struct AgentsAgentEngineHost {
    slots: HashMap<String, AgentEngineSlot>,
    unavailable: HashMap<String, AgentEngineBootstrapError>,
    engine_order: Vec<String>,
    live: LiveInteractionRegistry,
}

impl AgentsAgentEngineHost {
    pub fn bootstrap() -> Result<Self, AgentEngineBootstrapError> {
        Self::bootstrap_with_live(LiveInteractionRegistry::new())
    }

    pub fn bootstrap_with_live(
        live: LiveInteractionRegistry,
    ) -> Result<Self, AgentEngineBootstrapError> {
        let mut slots = HashMap::new();
        let mut engine_order = Vec::new();
        for engine_key in bootstrappable_engine_keys() {
            let slot = bootstrap_agent_engine(engine_key)?;
            slots.insert(engine_key.to_string(), slot);
            engine_order.push(engine_key.to_string());
        }
        Ok(Self {
            slots,
            unavailable: HashMap::new(),
            engine_order,
            live,
        })
    }

    /// Bootstrap only the requested engines, retaining per-engine failures.
    ///
    /// Unlike [`Self::bootstrap_with_live`], one unavailable provider does not
    /// prevent successfully bootstrapped providers from serving requests.
    pub fn bootstrap_selected(engine_keys: &[&str], live: LiveInteractionRegistry) -> Self {
        let mut slots = HashMap::new();
        let mut unavailable = HashMap::new();
        let mut engine_order = Vec::new();

        for engine_key in engine_keys {
            if engine_order
                .iter()
                .any(|registered| registered == engine_key)
            {
                continue;
            }
            engine_order.push((*engine_key).to_string());
            match bootstrap_agent_engine(engine_key) {
                Ok(slot) => {
                    slots.insert((*engine_key).to_string(), slot);
                }
                Err(error) => {
                    unavailable.insert((*engine_key).to_string(), error);
                }
            }
        }

        Self {
            slots,
            unavailable,
            engine_order,
            live,
        }
    }

    pub fn slot(&self, engine_key: &str) -> Option<&AgentEngineSlot> {
        self.slots.get(engine_key)
    }

    /// Like [`Self::bootstrap_selected`], but the Rig (simple agent) engine is
    /// bootstrapped with a materialized model configuration and a host secret
    /// surface so a live OpenAI-compatible backend replaces the fail-closed
    /// default. Used to apply provider configuration at runtime without
    /// restarting the host; other engines bootstrap exactly as
    /// [`Self::bootstrap_selected`].
    pub fn bootstrap_selected_with_rig(
        engine_keys: &[&str],
        rig_configuration: Option<&sdkwork_agent_kernel::AgentConfiguration>,
        rig_host: std::sync::Arc<dyn sdkwork_agent_kernel::HostProvider + Send + Sync>,
        live: LiveInteractionRegistry,
    ) -> Self {
        let mut slots = HashMap::new();
        let mut unavailable = HashMap::new();
        let mut engine_order = Vec::new();

        for engine_key in engine_keys {
            if engine_order
                .iter()
                .any(|registered| registered == engine_key)
            {
                continue;
            }
            engine_order.push((*engine_key).to_string());
            let bootstrapped = if *engine_key == "rig" {
                crate::agent_engines::bootstrap_rig_agent_engine(rig_configuration, rig_host.clone())
            } else {
                bootstrap_agent_engine(engine_key)
            };
            match bootstrapped {
                Ok(slot) => {
                    slots.insert((*engine_key).to_string(), slot);
                }
                Err(error) => {
                    unavailable.insert((*engine_key).to_string(), error);
                }
            }
        }

        Self {
            slots,
            unavailable,
            engine_order,
            live,
        }
    }

    pub fn engine_keys(&self) -> impl Iterator<Item = &str> {
        self.engine_order
            .iter()
            .filter(|engine_key| self.slots.contains_key(*engine_key))
            .map(String::as_str)
    }

    pub fn unavailable_engine(&self, engine_key: &str) -> Option<&AgentEngineBootstrapError> {
        self.unavailable.get(engine_key)
    }

    pub fn unavailable_engine_keys(&self) -> impl Iterator<Item = &str> {
        self.engine_order
            .iter()
            .filter(|engine_key| self.unavailable.contains_key(*engine_key))
            .map(String::as_str)
    }

    pub fn live_registry(&self) -> &LiveInteractionRegistry {
        &self.live
    }

    pub fn live_registry_mut(&mut self) -> &mut LiveInteractionRegistry {
        &mut self.live
    }

    pub fn catalog(&self) -> AgentEngineCatalog {
        let slots: Vec<&AgentEngineSlot> = self
            .engine_order
            .iter()
            .filter_map(|engine_key| self.slots.get(engine_key))
            .collect();
        build_complete_agent_engine_catalog(&slots, &self.unavailable)
    }

    pub fn discover_provider_sessions(
        &self,
        selector: &ProviderSessionInventorySelector,
    ) -> RuntimeFacadeResult<ProviderSessionInventorySnapshot> {
        discover_provider_sessions(&self.slots, selector)
    }

    pub fn load_provider_session_messages(
        &self,
        engine_key: &str,
        provider_session_id: &str,
    ) -> RuntimeFacadeResult<Vec<sdkwork_agent_kernel::AgentMessage>> {
        load_provider_session_messages(&self.slots, engine_key, provider_session_id)
    }

    pub fn load_provider_session_messages_for_directory(
        &self,
        engine_key: &str,
        provider_session_id: &str,
        working_directory: Option<&str>,
    ) -> RuntimeFacadeResult<Vec<sdkwork_agent_kernel::AgentMessage>> {
        load_provider_session_messages_for_directory(
            &self.slots,
            engine_key,
            provider_session_id,
            working_directory,
        )
    }

    pub fn load_provider_session_children(
        &self,
        engine_key: &str,
        provider_session_id: &str,
        working_directory: Option<&str>,
    ) -> RuntimeFacadeResult<Vec<String>> {
        load_provider_session_children(
            &self.slots,
            engine_key,
            provider_session_id,
            working_directory,
        )
    }

    pub fn get_provider_session_activity(
        &self,
        engine_key: &str,
        provider_session_id: &str,
    ) -> RuntimeFacadeResult<sdkwork_agent_kernel::SessionActivitySnapshot> {
        self.validate_engine_key(engine_key)?;
        self.slots
            .get(engine_key)
            .expect("validated engine slot must exist")
            .get_provider_session_activity(provider_session_id)
            .map_err(|error| RuntimeFacadeError::Kernel(error.to_string()))
    }

    pub fn execute_turn(
        &self,
        input: &AgentEngineTurnInput,
    ) -> RuntimeFacadeResult<AgentEngineTurnOutput> {
        self.validate_engine_key(input.engine_key.as_str())?;
        let slot = self
            .slots
            .get(input.engine_key.as_str())
            .expect("validated engine slot must exist");
        execute_agent_engine_turn(slot, input)
    }

    pub fn cancel_turn(
        &self,
        engine_key: &str,
        model_request_id: &str,
    ) -> RuntimeFacadeResult<AgentEngineTurnCancellation> {
        self.validate_engine_key(engine_key)?;
        let slot = self
            .slots
            .get(engine_key)
            .expect("validated engine slot must exist");
        cancel_agent_engine_turn(slot, model_request_id)
    }

    pub fn submit_approval_decision(
        &self,
        engine_key: &str,
        decision: &ApprovalDecision,
    ) -> RuntimeFacadeResult<()> {
        self.validate_engine_key(engine_key)?;
        self.live.submit_approval(engine_key, decision)
    }

    pub fn submit_user_question_answer(
        &self,
        engine_key: &str,
        answer: &UserQuestionAnswer,
    ) -> RuntimeFacadeResult<()> {
        self.validate_engine_key(engine_key)?;
        self.live.submit_user_question(engine_key, answer)
    }

    pub fn resolve_interaction(
        &self,
        engine_key: &str,
        resolution: &AgentEngineInteractionResolution,
    ) -> RuntimeFacadeResult<serde_json::Value> {
        self.validate_engine_key(engine_key)?;
        self.slots
            .get(engine_key)
            .expect("validated engine slot must exist")
            .resolve_interaction(resolution)
    }

    pub fn validate_engine_key(&self, engine_key: &str) -> RuntimeFacadeResult<()> {
        if self.slots.contains_key(engine_key) {
            return Ok(());
        }
        if let Some(error) = self.unavailable.get(engine_key) {
            return Err(RuntimeFacadeError::EngineUnavailable {
                engine_key: engine_key.to_string(),
                reason: error.to_string(),
            });
        }
        Err(RuntimeFacadeError::UnsupportedEngine {
            engine_key: engine_key.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_bootstraps_all_available_engines() {
        let host = AgentsAgentEngineHost::bootstrap().expect("host bootstrap");
        let expected = bootstrappable_engine_keys();
        assert_eq!(host.slots.len(), expected.len());
        assert_eq!(host.engine_keys().collect::<Vec<_>>(), expected);
        assert_eq!(
            host.catalog()
                .engines
                .iter()
                .map(|engine| engine.engine_key.as_str())
                .collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    fn validate_engine_key_returns_typed_error_for_unknown() {
        let host = AgentsAgentEngineHost::bootstrap().expect("host bootstrap");
        let result = host.validate_engine_key("nonexistent");
        assert!(matches!(
            result,
            Err(RuntimeFacadeError::UnsupportedEngine { ref engine_key })
                if engine_key == "nonexistent"
        ));
    }

    #[test]
    fn selected_bootstrap_starts_only_requested_canonical_engines() {
        let host = AgentsAgentEngineHost::bootstrap_selected(
            crate::agent_engines::canonical_agent_engine_keys(),
            LiveInteractionRegistry::new(),
        );

        assert_eq!(host.slots.len(), 4);
        // The catalog is the full platform inventory: requested engines are
        // available, engines outside the requested set stay listed as
        // unavailable instead of disappearing from settings surfaces.
        let catalog = host.catalog();
        assert_eq!(catalog.engines.len(), crate::agent_engines::bootstrappable_engine_keys().len());
        for engine_key in crate::agent_engines::canonical_agent_engine_keys() {
            assert!(host.slot(engine_key).is_some(), "missing slot {engine_key}");
        }
        assert!(host.slot("openclaw").is_none());
        assert_eq!(host.unavailable_engine_keys().count(), 0);
        let openclaw = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "openclaw")
            .expect("openclaw must stay listed");
        assert!(!openclaw.available);
        assert!(openclaw.unavailable_reason.is_some());
    }

    #[test]
    fn selected_bootstrap_retains_failure_without_dropping_working_slot() {
        let host = AgentsAgentEngineHost::bootstrap_selected(
            &["missing-provider", "codex"],
            LiveInteractionRegistry::new(),
        );

        assert!(host.slot("codex").is_some());
        assert_eq!(host.engine_keys().collect::<Vec<_>>(), vec!["codex"]);
        assert_eq!(
            host.unavailable_engine_keys().collect::<Vec<_>>(),
            vec!["missing-provider"]
        );
        assert!(matches!(
            host.unavailable_engine("missing-provider"),
            Some(AgentEngineBootstrapError::UnsupportedEngine(engine_key))
                if engine_key == "missing-provider"
        ));
        assert!(matches!(
            host.validate_engine_key("missing-provider"),
            Err(RuntimeFacadeError::EngineUnavailable { ref engine_key, .. })
                if engine_key == "missing-provider"
        ));
        assert!(matches!(
            host.execute_turn(&AgentEngineTurnInput {
                engine_key: "missing-provider".to_string(),
                prompt: "test unavailable slot".to_string(),
                ..Default::default()
            }),
            Err(RuntimeFacadeError::EngineUnavailable { ref engine_key, .. })
                if engine_key == "missing-provider"
        ));
    }
}
