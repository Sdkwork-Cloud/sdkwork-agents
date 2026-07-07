use std::collections::HashMap;

use crate::code_engines::{
    bootstrap_code_engine, bootstrappable_engine_keys, CodeEngineBootstrapError, CodeEngineSlot,
};
use crate::engine_catalog::{build_code_engine_catalog, CodeEngineCatalog};
use crate::error::{RuntimeFacadeError, RuntimeFacadeResult};
use crate::live_interaction::{ApprovalDecision, LiveInteractionRegistry, UserQuestionAnswer};
use crate::turn::{execute_code_engine_turn, CodeEngineTurnInput, CodeEngineTurnOutput};

/// Agents-owned runtime host for canonical code-engine provider slots.
pub struct AgentsCodeEngineHost {
    slots: HashMap<String, CodeEngineSlot>,
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
        for engine_key in bootstrappable_engine_keys() {
            let slot = bootstrap_code_engine(engine_key)?;
            slots.insert(engine_key.to_string(), slot);
        }
        Ok(Self { slots, live })
    }

    pub fn slot(&self, engine_key: &str) -> Option<&CodeEngineSlot> {
        self.slots.get(engine_key)
    }

    pub fn engine_keys(&self) -> impl Iterator<Item = &str> {
        self.slots.keys().map(String::as_str)
    }

    pub fn live_registry(&self) -> &LiveInteractionRegistry {
        &self.live
    }

    pub fn live_registry_mut(&mut self) -> &mut LiveInteractionRegistry {
        &mut self.live
    }

    pub fn catalog(&self) -> CodeEngineCatalog {
        let slots: Vec<&CodeEngineSlot> = self.slots.values().collect();
        build_code_engine_catalog(&slots)
    }

    pub fn execute_turn(
        &self,
        input: &CodeEngineTurnInput,
    ) -> RuntimeFacadeResult<CodeEngineTurnOutput> {
        let slot = self.slots.get(input.engine_key.as_str()).ok_or_else(|| {
            RuntimeFacadeError::UnsupportedEngine {
                engine_key: input.engine_key.clone(),
            }
        })?;
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
        assert_eq!(host.slots.len(), 6);
        assert_eq!(host.catalog().engines.len(), 6);
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
}
