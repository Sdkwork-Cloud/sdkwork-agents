//! Command and result types for the agents application service.
//!
//! Each command is a self-contained value object that carries all the
//! information needed for one service operation: the tenant scope, the
//! target entity, optional optimistic-concurrency version, the requesting
//! subject, and the request timestamp.

use crate::domain::{
    AgentAuditAction, AgentBusinessStatus, AgentCompositionSlotKind,
    AgentCompositionTargetModule, AgentImplementationKind, AgentImplementationType,
    AgentMessageRole, AgentSessionRecord, AgentMessageRecord, AgentVisibility,
};
use crate::ports::{
    AgentListQuery, AuditEventListQuery, CompositionSlotListQuery, InteractionListQuery,
    McpMarketplaceListQuery, MessageListQuery, ProviderBindingListQuery, SessionListQuery,
};
use sdkwork_agent_kernel::{AgentManifest, PolicySubject};
use sdkwork_code_kernel::CodeTaskIntent;

// ---------------------------------------------------------------------------
// Audit event input (internal)
// ---------------------------------------------------------------------------

/// Input for marketplace/composition-slot audit events.
///
/// This is an internal type used by [`super::AgentsService`] to pass
/// audit metadata to `emit_marketplace_audit_event`.
pub(super) struct AgentBusinessAuditEventInput<'a> {
    pub action: AgentAuditAction,
    pub item_kind: &'a str,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub item_id: &'a str,
    pub status: AgentBusinessStatus,
    pub visibility: AgentVisibility,
    pub version: u64,
    pub subject: PolicySubject,
    pub occurred_at: String,
}

// ---------------------------------------------------------------------------
// Agent business commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAgentCommand {
    pub agent_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub manifest: AgentManifest,
    pub visibility: AgentVisibility,
    pub tags: Vec<String>,
    pub default_code_task_intent: Option<CodeTaskIntent>,
    pub implementation_provider_id: Option<String>,
    pub implementation_kind: Option<AgentImplementationKind>,
    pub implementation_type: Option<AgentImplementationType>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAgentCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub expected_version: Option<u64>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub manifest: Option<AgentManifest>,
    pub visibility: Option<AgentVisibility>,
    pub tags: Option<Vec<String>>,
    pub default_code_task_intent: Option<CodeTaskIntent>,
    pub implementation_provider_id: Option<Option<String>>,
    pub implementation_kind: Option<Option<AgentImplementationKind>>,
    pub implementation_type: Option<AgentImplementationType>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeAgentStatusCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub expected_version: Option<u64>,
    pub target_status: AgentBusinessStatus,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteAgentCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub expected_version: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreAgentCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub expected_version: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetAgentCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListAgentsCommand {
    pub query: AgentListQuery,
    pub requested_by: PolicySubject,
}

// ---------------------------------------------------------------------------
// Provider binding commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderBindingCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub binding_id: String,
    pub provider_id: String,
    pub implementation_kind: AgentImplementationKind,
    pub configuration_profile_id: String,
    pub capabilities: Vec<String>,
    pub make_default: bool,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivateAgentProviderBindingCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub binding_id: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

// ---------------------------------------------------------------------------
// Runtime execution commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct AgentPreviewResponseCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub execution_id: String,
    pub content: String,
    pub debug_mode: bool,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub input_payload_json: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentPromptOptimizationCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub execution_id: String,
    pub prompt: String,
    pub input_payload_json: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

// ---------------------------------------------------------------------------
// Composition slot commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotCreateCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub slot_id: String,
    pub slot_kind: AgentCompositionSlotKind,
    pub target_module: AgentCompositionTargetModule,
    pub target_ref: String,
    pub target_version_ref: Option<String>,
    pub priority: i32,
    pub enabled: bool,
    pub policy_json: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotUpdateCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub slot_id: String,
    pub expected_version: Option<u64>,
    pub slot_kind: Option<AgentCompositionSlotKind>,
    pub target_module: Option<AgentCompositionTargetModule>,
    pub target_ref: Option<String>,
    pub target_version_ref: Option<Option<String>>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
    pub policy_json: Option<String>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotDeleteCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub slot_id: String,
    pub expected_version: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderBindingListCommand {
    pub query: ProviderBindingListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListAgentAuditEventsCommand {
    pub query: AuditEventListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListMcpMarketplaceCommand {
    pub query: McpMarketplaceListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotListCommand {
    pub query: CompositionSlotListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotGetCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub slot_id: String,
    pub requested_by: PolicySubject,
}

// ---------------------------------------------------------------------------
// Session commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateSessionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub session_id: String,
    pub title: Option<String>,
    pub provider_binding_id: Option<String>,
    pub model_id: Option<String>,
    pub metadata_json: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseSessionCommand {
    pub tenant_id: u64,
    pub session_id: String,
    pub expected_version: Option<u64>,
    /// When set, the session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveSessionCommand {
    pub tenant_id: u64,
    pub session_id: String,
    pub expected_version: Option<u64>,
    /// When set, the session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetSessionCommand {
    pub tenant_id: u64,
    pub session_id: String,
    /// When set, the session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSessionsCommand {
    pub query: SessionListQuery,
    pub requested_by: PolicySubject,
}

// ---------------------------------------------------------------------------
// Message commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateMessageCommand {
    pub tenant_id: u64,
    pub session_id: String,
    pub message_id: String,
    pub role: AgentMessageRole,
    pub content: String,
    pub content_type: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub artifacts_json: String,
    pub metadata_json: String,
    pub parent_message_id: Option<String>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetMessageCommand {
    pub tenant_id: u64,
    pub session_id: String,
    pub message_id: String,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListMessagesCommand {
    pub query: MessageListQuery,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

/// Send a user chat message and produce an assistant reply in one turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendChatMessageCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub session_id: String,
    pub content: String,
    pub content_type: String,
    pub metadata_json: String,
    pub model_id: Option<String>,
    /// When set, the session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

/// Result of a chat completion turn (user message + assistant reply + session).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatCompletionResult {
    pub session: AgentSessionRecord,
    pub user_message: AgentMessageRecord,
    pub assistant_message: AgentMessageRecord,
}

// ---------------------------------------------------------------------------
// Interaction commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListInteractionsCommand {
    pub query: InteractionListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetInteractionCommand {
    pub tenant_id: u64,
    pub session_id: String,
    pub interaction_id: String,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApproveInteractionCommand {
    pub tenant_id: u64,
    pub session_id: String,
    pub interaction_id: String,
    pub approved: bool,
    pub reason: Option<String>,
    pub expected_version: u64,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerInteractionCommand {
    pub tenant_id: u64,
    pub session_id: String,
    pub interaction_id: String,
    pub answer: String,
    pub option_label: Option<String>,
    pub rejected: bool,
    pub expected_version: u64,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}
