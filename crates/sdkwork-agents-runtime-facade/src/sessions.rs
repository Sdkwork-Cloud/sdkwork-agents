use sdkwork_utils_rust::string::is_blank;
use serde::{Deserialize, Serialize};

use crate::{RuntimeFacadeError, RuntimeFacadeResult};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentsSessionActor {
    pub subject_id: String,
    pub roles: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentsSessionKind {
    Assistant,
    Coding,
    Automation,
    ImDispatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentsSessionEntrySurface {
    Pc,
    H5,
    Flutter,
    MiniProgram,
    Api,
    ImDispatch,
    Automation,
}

/// Optional runtime selection attached while resolving an Agents session.
///
/// Provider and native runtime identifiers belong to this bounded binding
/// contract. They must not be persisted directly on the session aggregate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentsSessionRuntimeBindingDescriptor {
    pub runtime_binding_id: String,
    pub runtime_location_id: Option<String>,
    pub host_mode: String,
    pub transport_kind: String,
    pub provider_binding_id: String,
    pub model_id: String,
    pub provider_id: String,
    pub native_session_id: Option<String>,
    pub native_session_tree_id: Option<String>,
    pub native_parent_session_id: Option<String>,
    pub native_forked_from_session_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolveAgentsSessionRequest {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub agent_id: String,
    pub session_id: String,
    pub project_id: Option<String>,
    pub session_kind: AgentsSessionKind,
    pub entry_surface: AgentsSessionEntrySurface,
    pub source_module: Option<String>,
    pub source_context_kind: Option<String>,
    pub source_context_id: Option<String>,
    pub parent_session_id: Option<String>,
    pub forked_from_turn_id: Option<String>,
    pub title: String,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub runtime_binding: Option<AgentsSessionRuntimeBindingDescriptor>,
    pub actor: AgentsSessionActor,
    pub requested_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolvedAgentsSession {
    pub session_id: String,
    pub created: bool,
    pub version: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompleteAgentsTurnRequest {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub agent_id: String,
    pub session_id: String,
    pub content: String,
    pub content_type: String,
    pub idempotency_key: String,
    pub client_request_id: String,
    pub actor: AgentsSessionActor,
    pub requested_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompletedAgentsTurn {
    pub session_id: String,
    pub turn_id: String,
    pub request_item_id: String,
    pub response_item_id: String,
    pub response_content: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentsTurnStatus {
    Requested,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GetAgentsTurnByIdempotencyRequest {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub agent_id: String,
    pub session_id: String,
    pub idempotency_key: String,
    pub actor: AgentsSessionActor,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentsTurnSnapshot {
    pub session_id: String,
    pub turn_id: String,
    pub status: AgentsTurnStatus,
    pub request_item_id: String,
    pub response_item_id: Option<String>,
    pub response_content: Option<String>,
    pub error_code: Option<String>,
}

pub trait AgentsSessionFacade: Send + Sync {
    fn resolve_or_create_session(
        &self,
        request: ResolveAgentsSessionRequest,
    ) -> RuntimeFacadeResult<ResolvedAgentsSession>;

    fn complete_turn(
        &self,
        request: CompleteAgentsTurnRequest,
    ) -> RuntimeFacadeResult<CompletedAgentsTurn>;

    fn get_turn_by_idempotency(
        &self,
        request: GetAgentsTurnByIdempotencyRequest,
    ) -> RuntimeFacadeResult<Option<AgentsTurnSnapshot>>;
}

pub fn validate_session_actor(actor: &AgentsSessionActor) -> RuntimeFacadeResult<()> {
    if is_blank(Some(actor.subject_id.as_str()))
        || actor.roles.iter().any(|role| is_blank(Some(role.as_str())))
    {
        return Err(RuntimeFacadeError::InvalidInput(
            "trusted Agents session actor is invalid".into(),
        ));
    }
    Ok(())
}

pub fn validate_resolve_agents_session_request(
    request: &ResolveAgentsSessionRequest,
) -> RuntimeFacadeResult<()> {
    validate_session_actor(&request.actor)?;
    for (field, value) in [
        ("agentId", request.agent_id.as_str()),
        ("sessionId", request.session_id.as_str()),
        ("title", request.title.as_str()),
        ("idempotencyKey", request.idempotency_key.as_str()),
        ("payloadHash", request.payload_hash.as_str()),
        ("requestedAt", request.requested_at.as_str()),
    ] {
        require_non_blank(field, value)?;
    }
    validate_optional_non_blank("projectId", request.project_id.as_deref())?;

    let source_context = [
        ("sourceModule", request.source_module.as_deref()),
        ("sourceContextKind", request.source_context_kind.as_deref()),
        ("sourceContextId", request.source_context_id.as_deref()),
    ];
    validate_optional_group(
        &source_context,
        "sourceModule, sourceContextKind and sourceContextId must be supplied together",
    )?;

    let fork_lineage = [
        ("parentSessionId", request.parent_session_id.as_deref()),
        ("forkedFromTurnId", request.forked_from_turn_id.as_deref()),
    ];
    validate_optional_group(
        &fork_lineage,
        "parentSessionId and forkedFromTurnId must be supplied together",
    )?;
    if request.parent_session_id.as_deref() == Some(request.session_id.as_str()) {
        return Err(RuntimeFacadeError::InvalidInput(
            "parentSessionId must differ from sessionId".into(),
        ));
    }

    if let Some(binding) = request.runtime_binding.as_ref() {
        validate_runtime_binding_descriptor(binding)?;
    }
    Ok(())
}

fn validate_runtime_binding_descriptor(
    binding: &AgentsSessionRuntimeBindingDescriptor,
) -> RuntimeFacadeResult<()> {
    for (field, value) in [
        (
            "runtimeBinding.runtimeBindingId",
            binding.runtime_binding_id.as_str(),
        ),
        ("runtimeBinding.hostMode", binding.host_mode.as_str()),
        (
            "runtimeBinding.transportKind",
            binding.transport_kind.as_str(),
        ),
        (
            "runtimeBinding.providerBindingId",
            binding.provider_binding_id.as_str(),
        ),
        ("runtimeBinding.modelId", binding.model_id.as_str()),
        ("runtimeBinding.providerId", binding.provider_id.as_str()),
    ] {
        require_non_blank(field, value)?;
    }
    for (field, value) in [
        (
            "runtimeBinding.runtimeLocationId",
            binding.runtime_location_id.as_deref(),
        ),
        (
            "runtimeBinding.nativeSessionId",
            binding.native_session_id.as_deref(),
        ),
        (
            "runtimeBinding.nativeSessionTreeId",
            binding.native_session_tree_id.as_deref(),
        ),
        (
            "runtimeBinding.nativeParentSessionId",
            binding.native_parent_session_id.as_deref(),
        ),
        (
            "runtimeBinding.nativeForkedFromSessionId",
            binding.native_forked_from_session_id.as_deref(),
        ),
    ] {
        validate_optional_non_blank(field, value)?;
    }
    Ok(())
}

fn validate_optional_group(
    fields: &[(&str, Option<&str>)],
    incomplete_message: &str,
) -> RuntimeFacadeResult<()> {
    let supplied = fields.iter().filter(|(_, value)| value.is_some()).count();
    if supplied != 0 && supplied != fields.len() {
        return Err(RuntimeFacadeError::InvalidInput(
            incomplete_message.to_string(),
        ));
    }
    for (field, value) in fields {
        validate_optional_non_blank(field, *value)?;
    }
    Ok(())
}

fn validate_optional_non_blank(field: &str, value: Option<&str>) -> RuntimeFacadeResult<()> {
    if let Some(value) = value {
        require_non_blank(field, value)?;
    }
    Ok(())
}

fn require_non_blank(field: &str, value: &str) -> RuntimeFacadeResult<()> {
    if is_blank(Some(value)) {
        return Err(RuntimeFacadeError::InvalidInput(format!(
            "{field} must not be blank"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> ResolveAgentsSessionRequest {
        ResolveAgentsSessionRequest {
            tenant_id: 100001,
            organization_id: 200001,
            owner_user_id: 300001,
            agent_id: "agent.birdcoder".to_string(),
            session_id: "session.coding-001".to_string(),
            project_id: Some("project.birdcoder-001".to_string()),
            session_kind: AgentsSessionKind::Coding,
            entry_surface: AgentsSessionEntrySurface::Pc,
            source_module: Some("birdcoder".to_string()),
            source_context_kind: Some("coding_project".to_string()),
            source_context_id: Some("workspace-001".to_string()),
            parent_session_id: Some("session.coding-parent".to_string()),
            forked_from_turn_id: Some("turn.parent-001".to_string()),
            title: "Implement session authority".to_string(),
            idempotency_key: "create-session-001".to_string(),
            payload_hash: "sha256:payload-001".to_string(),
            runtime_binding: Some(AgentsSessionRuntimeBindingDescriptor {
                runtime_binding_id: "runtime-binding-001".to_string(),
                runtime_location_id: Some("birdcoder-workspace-001".to_string()),
                host_mode: "desktop".to_string(),
                transport_kind: "process".to_string(),
                provider_binding_id: "binding.agent-provider.codex".to_string(),
                model_id: "model.gpt-5".to_string(),
                provider_id: "provider.openai".to_string(),
                native_session_id: Some("native-session-001".to_string()),
                native_session_tree_id: Some("native-tree-001".to_string()),
                native_parent_session_id: Some("native-session-parent".to_string()),
                native_forked_from_session_id: Some("native-session-origin".to_string()),
            }),
            actor: AgentsSessionActor {
                subject_id: "user:300001".to_string(),
                roles: vec!["agents.session.operator".to_string()],
            },
            requested_at: "2026-07-22T12:00:00Z".to_string(),
        }
    }

    #[test]
    fn session_resolution_contract_serializes_canonical_aggregate_fields() {
        let value = serde_json::to_value(sample_request()).expect("request should serialize");

        assert_eq!(value["sessionKind"], "coding");
        assert_eq!(value["entrySurface"], "pc");
        assert_eq!(value["projectId"], "project.birdcoder-001");
        assert_eq!(value["sourceContextKind"], "coding_project");
        assert_eq!(value["parentSessionId"], "session.coding-parent");
        assert_eq!(
            value["runtimeBinding"]["providerBindingId"],
            "binding.agent-provider.codex"
        );
        assert_eq!(
            value["runtimeBinding"]["nativeSessionTreeId"],
            "native-tree-001"
        );
        assert!(value.get("providerBindingId").is_none());
    }

    #[test]
    fn session_resolution_contract_rejects_unknown_fields() {
        let mut value = serde_json::to_value(sample_request()).expect("request should serialize");
        value
            .as_object_mut()
            .expect("request should be an object")
            .insert("unexpectedField".to_string(), serde_json::json!("invalid"));

        assert!(serde_json::from_value::<ResolveAgentsSessionRequest>(value).is_err());
    }

    #[test]
    fn validation_accepts_complete_session_and_runtime_binding() {
        validate_resolve_agents_session_request(&sample_request())
            .expect("complete request should be valid");
    }

    #[test]
    fn validation_rejects_partial_source_context() {
        let mut request = sample_request();
        request.source_context_id = None;

        let error = validate_resolve_agents_session_request(&request)
            .expect_err("partial source context should fail");
        assert!(error.to_string().contains(
            "sourceModule, sourceContextKind and sourceContextId must be supplied together"
        ));
    }

    #[test]
    fn validation_rejects_partial_or_self_referencing_fork_lineage() {
        let mut partial = sample_request();
        partial.forked_from_turn_id = None;
        assert!(validate_resolve_agents_session_request(&partial).is_err());

        let mut self_referencing = sample_request();
        self_referencing.parent_session_id = Some(self_referencing.session_id.clone());
        assert!(validate_resolve_agents_session_request(&self_referencing).is_err());
    }

    #[test]
    fn validation_rejects_blank_runtime_binding_fields() {
        let mut request = sample_request();
        request
            .runtime_binding
            .as_mut()
            .expect("sample has runtime binding")
            .provider_id = "   ".to_string();

        let error = validate_resolve_agents_session_request(&request)
            .expect_err("blank providerId should fail");
        assert!(error
            .to_string()
            .contains("runtimeBinding.providerId must not be blank"));
    }

    #[test]
    fn validation_accepts_session_without_optional_scopes() {
        let mut request = sample_request();
        request.project_id = None;
        request.source_module = None;
        request.source_context_kind = None;
        request.source_context_id = None;
        request.parent_session_id = None;
        request.forked_from_turn_id = None;
        request.runtime_binding = None;

        validate_resolve_agents_session_request(&request)
            .expect("unscoped session request should remain valid");
    }
}
