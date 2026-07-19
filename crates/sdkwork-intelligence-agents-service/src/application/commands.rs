//! Command and result types for the agents application service.
//!
//! Each command is a self-contained value object that carries all the
//! information needed for one service operation: the tenant scope, the
//! target entity, optional optimistic-concurrency version, the requesting
//! subject, and the request timestamp.

use crate::chat_turn::AgentChatTurnRecord;
use crate::domain::{
    AgentAuditAction, AgentBusinessStatus, AgentCompositionSlotKind, AgentCompositionTargetModule,
    AgentImplementationKind, AgentImplementationType, AgentInteractionKind,
    AgentMessageDriveRefRecord, AgentMessageFeedbackRating, AgentMessageFeedbackRecord,
    AgentMessageRecord, AgentMessageRole, AgentResourceUserStateRecord, AgentSessionRecord,
    AgentVisibility,
};
use crate::ports::{
    AgentListQuery, AuditEventListQuery, CompositionSlotListQuery, InteractionListQuery,
    McpMarketplaceListQuery, MessageFeedbackListQuery, MessageListQuery,
    ProjectCompositionSlotListQuery, ProjectListQuery, ProviderBindingListQuery,
    ResourceUserStateListQuery, SessionListQuery, TaskListQuery,
};
use crate::project::{AgentProjectDriveAccessMode, AgentProjectVisibility};
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
pub struct CreateProjectCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub owner_user_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub visibility: AgentProjectVisibility,
    pub drive_access_mode: AgentProjectDriveAccessMode,
    pub default_agent_id: Option<String>,
    pub default_model_id: Option<String>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateProjectCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub owner_scope: Option<u64>,
    pub expected_version: Option<u64>,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub visibility: Option<AgentProjectVisibility>,
    pub drive_access_mode: Option<AgentProjectDriveAccessMode>,
    pub default_agent_id: Option<Option<String>>,
    pub default_model_id: Option<Option<String>>,
    pub requested_user_id: u64,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectMutationCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub owner_scope: Option<u64>,
    pub expected_version: Option<u64>,
    pub requested_user_id: u64,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetProjectCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListProjectsCommand {
    pub query: ProjectListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateProjectCompositionSlotCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub slot_id: String,
    pub slot_kind: AgentCompositionSlotKind,
    pub target_module: AgentCompositionTargetModule,
    pub target_ref: String,
    pub target_version_ref: Option<String>,
    pub priority: i32,
    pub enabled: bool,
    pub policy_json: String,
    pub owner_scope: Option<u64>,
    pub requested_user_id: u64,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateProjectCompositionSlotCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub slot_id: String,
    pub expected_version: Option<u64>,
    pub slot_kind: Option<AgentCompositionSlotKind>,
    pub target_module: Option<AgentCompositionTargetModule>,
    pub target_ref: Option<String>,
    pub target_version_ref: Option<Option<String>>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
    pub policy_json: Option<String>,
    pub owner_scope: Option<u64>,
    pub requested_user_id: u64,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetProjectCompositionSlotCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub slot_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListProjectCompositionSlotsCommand {
    pub query: ProjectCompositionSlotListQuery,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteProjectCompositionSlotCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub slot_id: String,
    pub expected_version: Option<u64>,
    pub owner_scope: Option<u64>,
    pub requested_user_id: u64,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateSessionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub project_id: Option<String>,
    pub session_id: String,
    pub title: Option<String>,
    pub provider_binding_id: Option<String>,
    pub model_id: Option<String>,
    pub metadata_json: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSessionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub title: Option<String>,
    pub project_id: Option<Option<String>>,
    pub expected_version: Option<u64>,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteSessionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub owner_scope: Option<u64>,
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
    /// Agent id from the nested HTTP path; must match the loaded session.
    pub path_agent_id: String,
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
// Task commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTaskCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub task_id: String,
    pub title: Option<String>,
    pub prompt: String,
    pub external_ref: Option<String>,
    pub metadata_json: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelTaskCommand {
    pub tenant_id: u64,
    /// Nested route `{agentId}`; must match the loaded task.
    pub path_agent_id: String,
    pub task_id: String,
    pub expected_version: Option<u64>,
    /// When set, the task must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecuteTaskCommand {
    pub tenant_id: u64,
    pub path_agent_id: String,
    pub task_id: String,
    pub expected_version: Option<u64>,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTaskCommand {
    pub tenant_id: u64,
    /// Agent id from the nested HTTP path; must match the loaded task.
    pub path_agent_id: String,
    pub task_id: String,
    /// When set, the task must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListTasksCommand {
    pub query: TaskListQuery,
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
    /// Agent id from the nested HTTP path; must match the loaded message.
    pub path_agent_id: String,
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

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentMessageMediaResourceInput {
    pub id: String,
    pub kind: String,
    pub source: String,
    pub uri: String,
    pub url: Option<String>,
    pub public_url: Option<String>,
    pub object_blob_id: Option<String>,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<String>,
    pub checksum: Option<serde_json::Value>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_seconds: Option<f64>,
    pub alt_text: Option<String>,
    pub title: Option<String>,
    pub access: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

/// Send a user chat message and produce an assistant reply in one turn.
#[derive(Debug, Clone, PartialEq)]
pub struct SendChatMessageCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub session_id: String,
    pub content: String,
    pub content_type: String,
    pub metadata_json: String,
    pub media_resources: Vec<AgentMessageMediaResourceInput>,
    pub model_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub client_request_id: Option<String>,
    /// When set, the session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
    /// When true, attempt provider stream chunks for SSE `message.delta` events.
    pub prefer_stream: bool,
}

/// Result of a chat completion turn (user message + assistant reply + session).
#[derive(Debug, Clone, PartialEq)]
pub struct ChatCompletionResult {
    pub session: AgentSessionRecord,
    pub user_message: AgentMessageRecord,
    pub assistant_message: AgentMessageRecord,
    pub user_message_drive_refs: Vec<AgentMessageDriveRefRecord>,
    pub stream_deltas: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentMessageWithDriveRefs {
    pub message: AgentMessageRecord,
    pub drive_refs: Vec<AgentMessageDriveRefRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetChatTurnCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetChatTurnByIdempotencyCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub owner_user_id: u64,
    pub idempotency_key: String,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelChatTurnCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub expected_version: Option<u64>,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatTurnReconciliationResult {
    pub examined: usize,
    pub failed: Vec<AgentChatTurnRecord>,
    pub skipped_conflicts: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSessionUserStatesCommand {
    pub query: ResourceUserStateListQuery,
    pub path_agent_id: String,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetSessionUserStateCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSessionUserStateCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub pinned: Option<bool>,
    pub hidden: Option<bool>,
    pub mark_opened: bool,
    pub last_read_message_sequence: Option<u64>,
    pub custom_title: Option<Option<String>>,
    pub expected_version: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

pub type SessionUserStateResult = AgentResourceUserStateRecord;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListMessageFeedbackCommand {
    pub query: MessageFeedbackListQuery,
    pub path_agent_id: String,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMessageFeedbackCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub message_id: String,
    pub rating: Option<AgentMessageFeedbackRating>,
    pub reason_code: Option<String>,
    pub comment: Option<String>,
    pub expected_version: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

pub type MessageFeedbackResult = AgentMessageFeedbackRecord;

// ---------------------------------------------------------------------------
// Interaction commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateInteractionCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub agent_id: String,
    pub interaction_id: String,
    pub engine_key: String,
    pub kind: AgentInteractionKind,
    pub prompt: String,
    pub options_json: String,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListInteractionsCommand {
    pub query: InteractionListQuery,
    /// Agent id from the nested HTTP path; must match the parent session.
    pub path_agent_id: String,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetInteractionCommand {
    pub tenant_id: u64,
    /// Agent id from the nested HTTP path; must match the loaded interaction.
    pub path_agent_id: String,
    pub session_id: String,
    pub interaction_id: String,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApproveInteractionCommand {
    pub tenant_id: u64,
    /// Agent id from the nested HTTP path; must match the parent session.
    pub path_agent_id: String,
    pub session_id: String,
    pub interaction_id: String,
    pub approved: bool,
    pub reason: Option<String>,
    pub expected_version: u64,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerInteractionCommand {
    pub tenant_id: u64,
    /// Agent id from the nested HTTP path; must match the parent session.
    pub path_agent_id: String,
    pub session_id: String,
    pub interaction_id: String,
    pub answer: String,
    pub option_label: Option<String>,
    pub rejected: bool,
    pub expected_version: u64,
    /// When set, the parent session must belong to this owner (app-api scope).
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}
