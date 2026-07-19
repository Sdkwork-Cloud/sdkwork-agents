use serde::{Deserialize, Serialize};

use crate::{RuntimeFacadeError, RuntimeFacadeResult};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentsChatActor {
    pub subject_id: String,
    pub roles: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolveAgentsChatSessionRequest {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub agent_id: String,
    pub session_id: String,
    pub title: String,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub actor: AgentsChatActor,
    pub requested_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolvedAgentsChatSession {
    pub session_id: String,
    pub created: bool,
    pub version: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompleteAgentsChatTurnRequest {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub agent_id: String,
    pub session_id: String,
    pub content: String,
    pub content_type: String,
    pub idempotency_key: String,
    pub client_request_id: String,
    pub actor: AgentsChatActor,
    pub requested_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompletedAgentsChatTurn {
    pub session_id: String,
    pub turn_id: String,
    pub request_message_id: String,
    pub response_message_id: String,
    pub response_content: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentsChatTurnStatus {
    Requested,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GetAgentsChatTurnByIdempotencyRequest {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub agent_id: String,
    pub session_id: String,
    pub idempotency_key: String,
    pub actor: AgentsChatActor,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentsChatTurnSnapshot {
    pub session_id: String,
    pub turn_id: String,
    pub status: AgentsChatTurnStatus,
    pub request_message_id: String,
    pub response_message_id: Option<String>,
    pub response_content: Option<String>,
    pub error_code: Option<String>,
}

pub trait AgentsChatFacade: Send + Sync {
    fn resolve_or_create_session(
        &self,
        request: ResolveAgentsChatSessionRequest,
    ) -> RuntimeFacadeResult<ResolvedAgentsChatSession>;

    fn complete_turn(
        &self,
        request: CompleteAgentsChatTurnRequest,
    ) -> RuntimeFacadeResult<CompletedAgentsChatTurn>;

    fn get_turn_by_idempotency(
        &self,
        request: GetAgentsChatTurnByIdempotencyRequest,
    ) -> RuntimeFacadeResult<Option<AgentsChatTurnSnapshot>>;
}

pub fn validate_chat_actor(actor: &AgentsChatActor) -> RuntimeFacadeResult<()> {
    if actor.subject_id.trim().is_empty() || actor.roles.iter().any(|role| role.trim().is_empty()) {
        return Err(RuntimeFacadeError::InvalidInput(
            "trusted chat actor is invalid".into(),
        ));
    }
    Ok(())
}
