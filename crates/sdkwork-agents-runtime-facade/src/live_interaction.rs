use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error::{RuntimeFacadeError, RuntimeFacadeResult};

/// Live approval decision routed through the agents runtime facade.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalDecision {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_session_id: Option<String>,
    pub approval_id: String,
    pub decision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Live user-question answer routed through the agents runtime facade.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserQuestionAnswer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_session_id: Option<String>,
    pub question_id: String,
    pub answer: String,
    pub rejected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_label: Option<String>,
}

/// Engine-specific live interaction handler registered by product bridges.
pub trait EngineLiveInteraction: Send + Sync {
    fn submit_approval(&self, decision: &ApprovalDecision) -> RuntimeFacadeResult<()>;
    fn submit_user_question(&self, answer: &UserQuestionAnswer) -> RuntimeFacadeResult<()>;
}

/// Registry of per-engine live interaction handlers.
#[derive(Default)]
pub struct LiveInteractionRegistry {
    handlers: HashMap<String, Arc<dyn EngineLiveInteraction>>,
}

impl LiveInteractionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        engine_key: impl Into<String>,
        handler: Arc<dyn EngineLiveInteraction>,
    ) {
        self.handlers.insert(engine_key.into(), handler);
    }

    pub fn submit_approval(
        &self,
        engine_key: &str,
        decision: &ApprovalDecision,
    ) -> RuntimeFacadeResult<()> {
        let handler = self.handlers.get(engine_key).ok_or_else(|| {
            RuntimeFacadeError::UnsupportedLiveInteraction {
                engine_key: engine_key.to_string(),
                interaction_type: "approval",
            }
        })?;
        handler.submit_approval(decision)
    }

    pub fn submit_user_question(
        &self,
        engine_key: &str,
        answer: &UserQuestionAnswer,
    ) -> RuntimeFacadeResult<()> {
        let handler = self.handlers.get(engine_key).ok_or_else(|| {
            RuntimeFacadeError::UnsupportedLiveInteraction {
                engine_key: engine_key.to_string(),
                interaction_type: "user-question",
            }
        })?;
        handler.submit_user_question(answer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubHandler;

    impl EngineLiveInteraction for StubHandler {
        fn submit_approval(&self, _decision: &ApprovalDecision) -> RuntimeFacadeResult<()> {
            Ok(())
        }

        fn submit_user_question(&self, _answer: &UserQuestionAnswer) -> RuntimeFacadeResult<()> {
            Ok(())
        }
    }

    #[test]
    fn unsupported_engine_returns_typed_error() {
        let registry = LiveInteractionRegistry::new();
        let err = registry
            .submit_approval(
                "codex",
                &ApprovalDecision {
                    approval_id: "perm-1".to_string(),
                    decision: "approve".to_string(),
                    ..Default::default()
                },
            )
            .expect_err("codex should not be registered");
        assert!(matches!(
            err,
            RuntimeFacadeError::UnsupportedLiveInteraction { ref engine_key, .. }
                if engine_key == "codex"
        ));
    }

    #[test]
    fn registered_engine_routes_approval() {
        let mut registry = LiveInteractionRegistry::new();
        registry.register("opencode", Arc::new(StubHandler));
        registry
            .submit_approval(
                "opencode",
                &ApprovalDecision {
                    approval_id: "perm-1".to_string(),
                    decision: "approve".to_string(),
                    ..Default::default()
                },
            )
            .expect("registered handler");
    }
}
