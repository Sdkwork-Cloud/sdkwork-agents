use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Live approval decision routed through the agents runtime facade.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalDecision {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_session_id: Option<String>,
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
    pub native_session_id: Option<String>,
    pub question_id: String,
    pub answer: String,
    pub rejected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_label: Option<String>,
}

/// Engine-specific live interaction handler registered by product bridges.
pub trait EngineLiveInteraction: Send + Sync {
    fn submit_approval(&self, decision: &ApprovalDecision) -> Result<(), String>;
    fn submit_user_question(&self, answer: &UserQuestionAnswer) -> Result<(), String>;
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
    ) -> Result<(), String> {
        let handler = self.handlers.get(engine_key).ok_or_else(|| {
            format!(
                "engineId \"{engine_key}\" does not support live approval replies through agents runtime facade yet."
            )
        })?;
        handler.submit_approval(decision)
    }

    pub fn submit_user_question(
        &self,
        engine_key: &str,
        answer: &UserQuestionAnswer,
    ) -> Result<(), String> {
        let handler = self.handlers.get(engine_key).ok_or_else(|| {
            format!(
                "engineId \"{engine_key}\" does not support live user-question replies through agents runtime facade yet."
            )
        })?;
        handler.submit_user_question(answer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubHandler;

    impl EngineLiveInteraction for StubHandler {
        fn submit_approval(&self, _decision: &ApprovalDecision) -> Result<(), String> {
            Ok(())
        }

        fn submit_user_question(&self, _answer: &UserQuestionAnswer) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn unsupported_engine_returns_clear_error() {
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
        assert!(err.contains("codex"));
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
