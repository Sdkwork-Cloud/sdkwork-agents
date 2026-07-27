use std::collections::HashMap;

use crate::code_engines::{
    bootstrap_code_engine, canonical_code_engine_keys, CodeEngineBootstrapError, CodeEngineSlot,
};
use crate::engine_catalog::{build_code_engine_catalog, CodeEngineCatalog};
use crate::error::{RuntimeFacadeError, RuntimeFacadeResult};
use crate::live_interaction::{ApprovalDecision, LiveInteractionRegistry, UserQuestionAnswer};
use crate::provider_sessions::{
    discover_provider_sessions, load_provider_session_messages, ProviderSessionInventoryItem,
    ProviderSessionInventorySelector,
};
use crate::turn::{execute_code_engine_turn, CodeEngineTurnInput, CodeEngineTurnOutput};

/// Agents-owned runtime host for canonical code-engine provider slots.
pub struct AgentsCodeEngineHost {
    slots: HashMap<String, CodeEngineSlot>,
    unavailable: HashMap<String, CodeEngineBootstrapError>,
    engine_order: Vec<String>,
    live: LiveInteractionRegistry,
}

impl AgentsCodeEngineHost {
    pub fn bootstrap() -> Result<Self, CodeEngineBootstrapError> {
        Self::bootstrap_with_live(LiveInteractionRegistry::new())
    }

    pub fn bootstrap_with_live(
        live: LiveInteractionRegistry,
    ) -> Result<Self, CodeEngineBootstrapError> {
        let mut slots = HashMap::new();
        let mut engine_order = Vec::new();
        for engine_key in canonical_code_engine_keys() {
            let slot = bootstrap_code_engine(engine_key)?;
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
            match bootstrap_code_engine(engine_key) {
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

    pub fn slot(&self, engine_key: &str) -> Option<&CodeEngineSlot> {
        self.slots.get(engine_key)
    }

    pub fn engine_keys(&self) -> impl Iterator<Item = &str> {
        self.engine_order
            .iter()
            .filter(|engine_key| self.slots.contains_key(*engine_key))
            .map(String::as_str)
    }

    pub fn unavailable_engine(&self, engine_key: &str) -> Option<&CodeEngineBootstrapError> {
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

    pub fn catalog(&self) -> CodeEngineCatalog {
        let slots: Vec<&CodeEngineSlot> = self
            .engine_order
            .iter()
            .filter_map(|engine_key| self.slots.get(engine_key))
            .collect();
        build_code_engine_catalog(&slots)
    }

    pub fn discover_provider_sessions(
        &self,
        selector: &ProviderSessionInventorySelector,
    ) -> RuntimeFacadeResult<Vec<ProviderSessionInventoryItem>> {
        discover_provider_sessions(&self.slots, selector)
    }

    pub fn load_provider_session_messages(
        &self,
        engine_key: &str,
        provider_session_id: &str,
    ) -> RuntimeFacadeResult<Vec<sdkwork_agent_kernel::AgentMessage>> {
        load_provider_session_messages(&self.slots, engine_key, provider_session_id)
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
        input: &CodeEngineTurnInput,
    ) -> RuntimeFacadeResult<CodeEngineTurnOutput> {
        self.validate_engine_key(input.engine_key.as_str())?;
        let slot = self
            .slots
            .get(input.engine_key.as_str())
            .expect("validated engine slot must exist");
        execute_code_engine_turn(slot, input)
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
    fn host_bootstraps_all_canonical_engines() {
        let host = AgentsCodeEngineHost::bootstrap().expect("host bootstrap");
        assert_eq!(host.slots.len(), canonical_code_engine_keys().len());
        assert_eq!(
            host.engine_keys().collect::<Vec<_>>(),
            canonical_code_engine_keys()
        );
        assert_eq!(
            host.catalog()
                .engines
                .iter()
                .map(|engine| engine.engine_key.as_str())
                .collect::<Vec<_>>(),
            canonical_code_engine_keys()
        );
    }

    #[test]
    fn validate_engine_key_returns_typed_error_for_unknown() {
        let host = AgentsCodeEngineHost::bootstrap().expect("host bootstrap");
        let result = host.validate_engine_key("nonexistent");
        assert!(matches!(
            result,
            Err(RuntimeFacadeError::UnsupportedEngine { ref engine_key })
                if engine_key == "nonexistent"
        ));
    }

    #[test]
    fn selected_bootstrap_starts_only_requested_canonical_engines() {
        let host = AgentsCodeEngineHost::bootstrap_selected(
            crate::code_engines::canonical_code_engine_keys(),
            LiveInteractionRegistry::new(),
        );

        assert_eq!(host.slots.len(), 4);
        assert_eq!(host.catalog().engines.len(), 4);
        for engine_key in crate::code_engines::canonical_code_engine_keys() {
            assert!(host.slot(engine_key).is_some(), "missing slot {engine_key}");
        }
        assert!(host.slot("openclaw").is_none());
        assert_eq!(host.unavailable_engine_keys().count(), 0);
    }

    #[test]
    fn selected_bootstrap_retains_failure_without_dropping_working_slot() {
        let host = AgentsCodeEngineHost::bootstrap_selected(
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
            Some(CodeEngineBootstrapError::UnsupportedEngine(engine_key))
                if engine_key == "missing-provider"
        ));
        assert!(matches!(
            host.validate_engine_key("missing-provider"),
            Err(RuntimeFacadeError::EngineUnavailable { ref engine_key, .. })
                if engine_key == "missing-provider"
        ));
        assert!(matches!(
            host.execute_turn(&CodeEngineTurnInput {
                engine_key: "missing-provider".to_string(),
                prompt: "test unavailable slot".to_string(),
                ..Default::default()
            }),
            Err(RuntimeFacadeError::EngineUnavailable { ref engine_key, .. })
                if engine_key == "missing-provider"
        ));
    }
}
