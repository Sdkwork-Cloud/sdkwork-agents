mod context;
mod middleware;
pub mod testing;

pub use context::AgentRequestContext;
use context::RequestScope;

use crate::application::{
    AgentCompositionSlotCreateCommand, AgentCompositionSlotDeleteCommand,
    AgentCompositionSlotGetCommand, AgentCompositionSlotListCommand,
    AgentCompositionSlotUpdateCommand, AgentItemDriveRefInput, AgentsService, CancelTurnCommand,
    CreateProjectCommand, CreateProjectCompositionSlotCommand, CreateSessionCommand,
    CreateTurnCommand, CreateWorkspaceCommand, DeleteProjectCompositionSlotCommand,
    DeleteSessionCommand, EnsureDefaultWorkspaceCommand, GetInteractionCommand, GetProjectCommand,
    GetProjectCompositionSlotCommand, GetSessionCheckpointCommand, GetSessionCommand,
    GetSessionItemCommand, GetSessionRuntimeBindingCommand, GetSessionUserStateCommand,
    GetTaskCommand, GetTurnByIdempotencyCommand, GetTurnCommand, GetWorkspaceCommand,
    ImportProjectCommand, ListAgentAuditEventsCommand, ListItemFeedbackCommand,
    ListMcpMarketplaceCommand, ListProjectCompositionSlotsCommand, ListProjectsCommand,
    ListSessionCheckpointsCommand, ListSessionRuntimeBindingsCommand, ListSessionUserStatesCommand,
    ListTurnsCommand, ListWorkspacesCommand, ProjectMutationCommand, ProviderBindingListCommand,
    UpdateItemFeedbackCommand, UpdateProjectCommand, UpdateProjectCompositionSlotCommand,
    UpdateSessionCommand, UpdateSessionUserStateCommand, UpdateWorkspaceCommand,
    WorkspaceMutationCommand,
};
use crate::domain::{
    AgentCompositionSlotKind, AgentCompositionSlotRecord, AgentCompositionTargetModule,
    AgentItemFeedbackRating, AgentItemResourceRole, AgentProviderBindingRecord,
    AgentSessionEntrySurface, AgentSessionKind, AgentSessionRuntimeBindingRecord,
    AgentSessionRuntimeBindingStatus,
};
use crate::dto::{
    ActivateAgentProviderBindingRequestDto, AgentCompositionSlotCreateRequestDto,
    AgentCompositionSlotRecordDto, AgentCompositionSlotUpdateRequestDto, AgentInteractionRecordDto,
    AgentItemFeedbackRecordDto, AgentManagementProfileDto, AgentPreviewResponseRequestDto,
    AgentPromptOptimizationRequestDto, AgentProviderBindingRecordDto,
    AgentProviderBindingRequestDto, AgentRecordDto, AgentResourceUserStateRecordDto,
    AgentRuntimeExecutionRecordDto, AgentSessionCheckpointRecordDto, AgentSessionItemRecordDto,
    AgentSessionRecordDto, AgentSessionRuntimeBindingRecordDto, AgentTaskRecordDto,
    AgentTurnExecutionDto, AgentTurnRecordDto, AnswerInteractionRequestDto,
    ApproveInteractionRequestDto, ArchiveSessionRequestDto, CancelTaskRequestDto,
    ChangeSessionCheckpointStatusRequestDto, ChangeSessionRuntimeBindingStatusRequestDto,
    ClaimInteractionRequestDto, CloseSessionRequestDto, CreateAgentRequestDto,
    CreateInteractionRequestDto, CreateSessionCheckpointRequestDto, CreateSessionRequestDto,
    CreateSessionRuntimeBindingRequestDto, CreateTaskRequestDto, DeleteAgentRequestDto,
    GetAgentRequestDto, InteractionClaimResultDto, ListAgentsRequestDto,
    ListInteractionsRequestDto, ListSessionItemsRequestDto, ListSessionsRequestDto,
    ListTasksRequestDto, RestoreAgentRequestDto, UpdateAgentRequestDto,
    UpdateAgentStatusRequestDto, UpdateSessionRuntimeBindingRequestDto,
};
use crate::mcp_marketplace::McpServerMarketplaceRecord;
use crate::ports::{
    AgentAuditSink, AgentRepository, AuditEventListQuery, CompositionSlotListQuery,
    ItemFeedbackListQuery, McpMarketplaceListQuery, PaginationParams,
    ProjectCompositionSlotListQuery, ProjectListQuery, ProviderBindingListQuery,
    ResourceUserStateListQuery, SessionCheckpointListQuery, SessionRuntimeBindingListQuery,
    TurnListQuery, WorkspaceListQuery,
};
use crate::project::{
    AgentProjectCompositionSlotRecord, AgentProjectDriveAccessMode, AgentProjectRecord,
    AgentProjectStatus, AgentProjectVisibility,
};
use crate::response::{
    created_json, finish_api_json, finish_created_api_json, no_content, success_json, ApiProblem,
    ApiResult, PageData, PageInfo, PageMode, ResourceData,
};
use crate::turn_runtime::{ContractTurnExecutor, TurnExecutor};
use crate::validation::{
    is_trimmed_blank, parse_expected_version, parse_optional_rfc3339_datetime,
    parse_organization_id, parse_tenant_id, validate_requested_at, validate_standard_id,
};
use crate::workspace::{AgentWorkspaceRecord, AgentWorkspaceStatus};
use axum::body::Body;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{Extension, Path, Query, State};
use axum::http::header::{HeaderName, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
#[cfg(test)]
use sdkwork_agent_kernel::ProviderManifest;
use sdkwork_agent_kernel::{
    AgentManifest, KernelError, KernelErrorKind, KernelResult, PolicyDecision, PolicyProvider,
    PolicyRequest, ProviderHealth,
};
use sdkwork_agents_runtime_facade::CodeEngineCatalog;
use sdkwork_code_kernel::CodeTaskIntent;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, LazyLock};
use time::OffsetDateTime;
use tokio::sync::Semaphore;

const MAX_PAGE_SIZE: usize = 200;
const DEFAULT_SERVICE_WORKER_LIMIT: usize = 128;
pub const ENV_TURN_RECONCILIATION_INTERVAL_SECONDS: &str =
    "SDKWORK_AGENTS_TURN_RECONCILIATION_INTERVAL_SECONDS";
pub const ENV_TURN_STALE_AFTER_SECONDS: &str = "SDKWORK_AGENTS_TURN_STALE_AFTER_SECONDS";
pub const ENV_TURN_RECONCILIATION_BATCH_SIZE: &str =
    "SDKWORK_AGENTS_TURN_RECONCILIATION_BATCH_SIZE";

/// Bounds synchronous repository work before it enters Tokio's blocking pool.
/// A bounded rejection is preferable to an unbounded blocking queue, which can
/// retain request payloads and tenant context until memory is exhausted.
static SERVICE_WORKER_LIMIT: LazyLock<Arc<Semaphore>> = LazyLock::new(|| {
    let configured = std::env::var("SDKWORK_AGENTS_SERVICE_WORKER_LIMIT")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| (1..=4096).contains(value))
        .unwrap_or(DEFAULT_SERVICE_WORKER_LIMIT);
    Arc::new(Semaphore::new(configured))
});
const ALLOWED_AUDIT_ACTIONS: &[&str] = &[
    "created",
    "updated",
    "deleted",
    "restored",
    "status_changed",
    "started",
    "completed",
    "failed",
    "cancelled",
    "runtime_executed",
    "provider_binding_changed",
    "composition_slot_created",
    "composition_slot_updated",
    "composition_slot_deleted",
    "session_created",
    "session_closed",
    "session_archived",
    "session_item_created",
    "session_item_failed",
    "interaction_created",
    "interaction_resolved",
    "interaction_rejected",
    "interaction_expired",
    "interaction_cancelled",
];

pub(crate) struct DynAgentRepository(Box<dyn AgentRepository + Send + Sync>);
pub(crate) struct DynAgentAuditSink(Box<dyn AgentAuditSink + Send + Sync>);
pub(crate) struct DynPolicyProvider(Box<dyn PolicyProvider + Send + Sync>);

impl DynAgentRepository {
    fn new<R>(repository: R) -> Self
    where
        R: AgentRepository + Send + Sync + 'static,
    {
        Self(Box::new(repository))
    }
}

impl DynAgentAuditSink {
    fn new<A>(audit_sink: A) -> Self
    where
        A: AgentAuditSink + Send + Sync + 'static,
    {
        Self(Box::new(audit_sink))
    }
}

impl DynPolicyProvider {
    fn new<P>(policy_provider: P) -> Self
    where
        P: PolicyProvider + Send + Sync + 'static,
    {
        Self(Box::new(policy_provider))
    }
}

impl AgentRepository for DynAgentRepository {
    fn check_readiness(&self) -> KernelResult<()> {
        self.0.check_readiness()
    }

    fn next_id(&self) -> KernelResult<u64> {
        self.0.next_id()
    }

    fn insert(&self, record: crate::domain::AgentBusinessRecord) -> KernelResult<()> {
        self.0.insert(record)
    }

    fn update(&self, record: crate::domain::AgentBusinessRecord) -> KernelResult<()> {
        self.0.update(record)
    }

    fn get(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentBusinessRecord>> {
        self.0.get(tenant_id, agent_id)
    }

    fn list(
        &self,
        query: &crate::ports::AgentListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentBusinessRecord>> {
        self.0.list(query)
    }

    fn count_agents(&self, query: &crate::ports::AgentListQuery) -> KernelResult<u64> {
        self.0.count_agents(query)
    }

    fn insert_workspace(&self, record: crate::workspace::AgentWorkspaceRecord) -> KernelResult<()> {
        self.0.insert_workspace(record)
    }

    fn update_workspace(&self, record: crate::workspace::AgentWorkspaceRecord) -> KernelResult<()> {
        self.0.update_workspace(record)
    }

    fn get_workspace(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
    ) -> KernelResult<Option<crate::workspace::AgentWorkspaceRecord>> {
        self.0
            .get_workspace(tenant_id, organization_id, workspace_id)
    }

    fn get_default_workspace(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
    ) -> KernelResult<Option<crate::workspace::AgentWorkspaceRecord>> {
        self.0
            .get_default_workspace(tenant_id, organization_id, owner_user_id)
    }

    fn list_workspaces(
        &self,
        query: &crate::ports::WorkspaceListQuery,
    ) -> KernelResult<Vec<crate::workspace::AgentWorkspaceRecord>> {
        self.0.list_workspaces(query)
    }

    fn count_workspaces(&self, query: &crate::ports::WorkspaceListQuery) -> KernelResult<u64> {
        self.0.count_workspaces(query)
    }

    fn insert_project(&self, record: crate::project::AgentProjectRecord) -> KernelResult<()> {
        self.0.insert_project(record)
    }

    fn update_project(&self, record: crate::project::AgentProjectRecord) -> KernelResult<()> {
        self.0.update_project(record)
    }

    fn get_project(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
    ) -> KernelResult<Option<crate::project::AgentProjectRecord>> {
        self.0.get_project(tenant_id, organization_id, project_id)
    }

    fn get_project_by_import_source(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        source_kind: &str,
        source_ref: &str,
    ) -> KernelResult<Option<crate::project::AgentProjectRecord>> {
        self.0.get_project_by_import_source(
            tenant_id,
            organization_id,
            owner_user_id,
            source_kind,
            source_ref,
        )
    }

    fn list_projects(
        &self,
        query: &crate::ports::ProjectListQuery,
    ) -> KernelResult<Vec<crate::project::AgentProjectRecord>> {
        self.0.list_projects(query)
    }

    fn count_projects(&self, query: &crate::ports::ProjectListQuery) -> KernelResult<u64> {
        self.0.count_projects(query)
    }

    fn insert_project_composition_slot(
        &self,
        record: crate::project::AgentProjectCompositionSlotRecord,
    ) -> KernelResult<()> {
        self.0.insert_project_composition_slot(record)
    }

    fn update_project_composition_slot(
        &self,
        record: crate::project::AgentProjectCompositionSlotRecord,
    ) -> KernelResult<()> {
        self.0.update_project_composition_slot(record)
    }

    fn get_project_composition_slot(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<crate::project::AgentProjectCompositionSlotRecord>> {
        self.0
            .get_project_composition_slot(tenant_id, organization_id, project_id, slot_id)
    }

    fn list_project_composition_slots(
        &self,
        query: &crate::ports::ProjectCompositionSlotListQuery,
    ) -> KernelResult<Vec<crate::project::AgentProjectCompositionSlotRecord>> {
        self.0.list_project_composition_slots(query)
    }

    fn count_project_composition_slots(
        &self,
        query: &crate::ports::ProjectCompositionSlotListQuery,
    ) -> KernelResult<u64> {
        self.0.count_project_composition_slots(query)
    }

    fn insert_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        self.0.insert_provider_binding(record)
    }

    fn update_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        self.0.update_provider_binding(record)
    }

    fn get_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRecord>> {
        self.0.get_provider_binding(tenant_id, agent_id, binding_id)
    }

    fn get_active_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRecord>> {
        self.0.get_active_provider_binding(tenant_id, agent_id)
    }

    fn list_provider_bindings(
        &self,
        query: &crate::ports::ProviderBindingListQuery,
    ) -> KernelResult<Vec<AgentProviderBindingRecord>> {
        self.0.list_provider_bindings(query)
    }

    fn count_provider_bindings(
        &self,
        query: &crate::ports::ProviderBindingListQuery,
    ) -> KernelResult<u64> {
        self.0.count_provider_bindings(query)
    }

    fn insert_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        self.0.insert_composition_slot(record)
    }

    fn update_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        self.0.update_composition_slot(record)
    }

    fn get_composition_slot(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentCompositionSlotRecord>> {
        self.0.get_composition_slot(tenant_id, agent_id, slot_id)
    }

    fn list_composition_slots(
        &self,
        query: &crate::ports::CompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>> {
        self.0.list_composition_slots(query)
    }

    fn count_composition_slots(
        &self,
        query: &crate::ports::CompositionSlotListQuery,
    ) -> KernelResult<u64> {
        self.0.count_composition_slots(query)
    }

    fn list_mcp_marketplace_slots(
        &self,
        query: &crate::ports::McpMarketplaceListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>> {
        self.0.list_mcp_marketplace_slots(query)
    }

    fn count_mcp_marketplace_slots(
        &self,
        query: &crate::ports::McpMarketplaceListQuery,
    ) -> KernelResult<u64> {
        self.0.count_mcp_marketplace_slots(query)
    }

    fn insert_session(&self, record: crate::domain::AgentSessionRecord) -> KernelResult<()> {
        self.0.insert_session(record)
    }

    fn update_session(&self, record: crate::domain::AgentSessionRecord) -> KernelResult<()> {
        self.0.update_session(record)
    }

    fn get_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentSessionRecord>> {
        self.0.get_session(tenant_id, organization_id, session_id)
    }

    fn list_sessions(
        &self,
        query: &crate::ports::SessionListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentSessionRecord>> {
        self.0.list_sessions(query)
    }

    fn count_sessions(&self, query: &crate::ports::SessionListQuery) -> KernelResult<u64> {
        self.0.count_sessions(query)
    }

    fn insert_session_runtime_binding(
        &self,
        record: crate::domain::AgentSessionRuntimeBindingRecord,
    ) -> KernelResult<()> {
        self.0.insert_session_runtime_binding(record)
    }

    fn update_session_runtime_binding(
        &self,
        record: crate::domain::AgentSessionRuntimeBindingRecord,
    ) -> KernelResult<()> {
        self.0.update_session_runtime_binding(record)
    }

    fn get_session_runtime_binding(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentSessionRuntimeBindingRecord>> {
        self.0.get_session_runtime_binding(
            tenant_id,
            organization_id,
            session_id,
            runtime_binding_id,
        )
    }

    fn get_current_session_runtime_binding(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentSessionRuntimeBindingRecord>> {
        self.0
            .get_current_session_runtime_binding(tenant_id, organization_id, session_id)
    }

    fn list_session_runtime_bindings(
        &self,
        query: &crate::ports::SessionRuntimeBindingListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentSessionRuntimeBindingRecord>> {
        self.0.list_session_runtime_bindings(query)
    }

    fn count_session_runtime_bindings(
        &self,
        query: &crate::ports::SessionRuntimeBindingListQuery,
    ) -> KernelResult<u64> {
        self.0.count_session_runtime_bindings(query)
    }

    fn activate_session_runtime_binding_atomic(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
        expected_version: u64,
        updated_at: String,
    ) -> KernelResult<crate::domain::AgentSessionRuntimeBindingRecord> {
        self.0.activate_session_runtime_binding_atomic(
            tenant_id,
            organization_id,
            session_id,
            runtime_binding_id,
            expected_version,
            updated_at,
        )
    }

    fn insert_session_checkpoint(
        &self,
        record: crate::domain::AgentSessionCheckpointRecord,
    ) -> KernelResult<()> {
        self.0.insert_session_checkpoint(record)
    }

    fn update_session_checkpoint(
        &self,
        record: crate::domain::AgentSessionCheckpointRecord,
    ) -> KernelResult<()> {
        self.0.update_session_checkpoint(record)
    }

    fn get_session_checkpoint(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        checkpoint_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentSessionCheckpointRecord>> {
        self.0
            .get_session_checkpoint(tenant_id, organization_id, session_id, checkpoint_id)
    }

    fn list_session_checkpoints(
        &self,
        query: &crate::ports::SessionCheckpointListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentSessionCheckpointRecord>> {
        self.0.list_session_checkpoints(query)
    }

    fn count_session_checkpoints(
        &self,
        query: &crate::ports::SessionCheckpointListQuery,
    ) -> KernelResult<u64> {
        self.0.count_session_checkpoints(query)
    }

    fn upsert_resource_user_state(
        &self,
        record: crate::domain::AgentResourceUserStateRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<crate::domain::AgentResourceUserStateRecord> {
        self.0.upsert_resource_user_state(record, expected_version)
    }

    fn get_resource_user_state(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        resource_type: crate::domain::AgentResourceType,
        resource_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentResourceUserStateRecord>> {
        self.0.get_resource_user_state(
            tenant_id,
            organization_id,
            user_id,
            resource_type,
            resource_id,
        )
    }

    fn list_resource_user_states(
        &self,
        query: &crate::ports::ResourceUserStateListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentResourceUserStateRecord>> {
        self.0.list_resource_user_states(query)
    }

    fn count_resource_user_states(
        &self,
        query: &crate::ports::ResourceUserStateListQuery,
    ) -> KernelResult<u64> {
        self.0.count_resource_user_states(query)
    }

    fn append_session_item(
        &self,
        record: crate::domain::AgentSessionItemRecord,
    ) -> KernelResult<(
        crate::domain::AgentSessionRecord,
        crate::domain::AgentSessionItemRecord,
    )> {
        self.0.append_session_item(record)
    }

    fn update_session_item(
        &self,
        record: crate::domain::AgentSessionItemRecord,
    ) -> KernelResult<()> {
        self.0.update_session_item(record)
    }

    fn get_session_item(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        item_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentSessionItemRecord>> {
        self.0
            .get_session_item(tenant_id, organization_id, session_id, item_id)
    }

    fn list_session_items(
        &self,
        query: &crate::ports::SessionItemListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentSessionItemRecord>> {
        self.0.list_session_items(query)
    }

    fn count_session_items(&self, query: &crate::ports::SessionItemListQuery) -> KernelResult<u64> {
        self.0.count_session_items(query)
    }

    fn upsert_item_feedback(
        &self,
        record: crate::domain::AgentItemFeedbackRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<crate::domain::AgentItemFeedbackRecord> {
        self.0.upsert_item_feedback(record, expected_version)
    }

    fn get_item_feedback(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
        user_id: u64,
        include_deleted: bool,
    ) -> KernelResult<Option<crate::domain::AgentItemFeedbackRecord>> {
        self.0.get_item_feedback(
            tenant_id,
            organization_id,
            item_id,
            user_id,
            include_deleted,
        )
    }

    fn list_item_feedback(
        &self,
        query: &crate::ports::ItemFeedbackListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentItemFeedbackRecord>> {
        self.0.list_item_feedback(query)
    }

    fn count_item_feedback(
        &self,
        query: &crate::ports::ItemFeedbackListQuery,
    ) -> KernelResult<u64> {
        self.0.count_item_feedback(query)
    }

    fn get_turn_by_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<crate::agent_turn::AgentTurnRecord>> {
        self.0
            .get_turn_by_idempotency(tenant_id, organization_id, owner_user_id, idempotency_key)
    }

    fn get_turn(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<Option<crate::agent_turn::AgentTurnRecord>> {
        self.0.get_turn(tenant_id, organization_id, turn_id)
    }

    fn list_turns(
        &self,
        query: &crate::ports::TurnListQuery,
    ) -> KernelResult<Vec<crate::agent_turn::AgentTurnRecord>> {
        self.0.list_turns(query)
    }

    fn count_turns(&self, query: &crate::ports::TurnListQuery) -> KernelResult<u64> {
        self.0.count_turns(query)
    }

    fn list_reconcilable_turns(
        &self,
        stale_before: &str,
        limit: usize,
    ) -> KernelResult<Vec<crate::agent_turn::AgentTurnRecord>> {
        self.0.list_reconcilable_turns(stale_before, limit)
    }

    fn insert_turn_request(
        &self,
        turn: crate::agent_turn::AgentTurnRecord,
        request_item: crate::domain::AgentSessionItemRecord,
        drive_refs: Vec<crate::domain::AgentItemDriveRefRecord>,
    ) -> KernelResult<crate::ports::TurnRequestWriteOutcome> {
        self.0.insert_turn_request(turn, request_item, drive_refs)
    }

    fn update_turn_state(
        &self,
        turn: crate::agent_turn::AgentTurnRecord,
        expected_version: u64,
    ) -> KernelResult<crate::agent_turn::AgentTurnRecord> {
        self.0.update_turn_state(turn, expected_version)
    }

    fn complete_turn(
        &self,
        turn: crate::agent_turn::AgentTurnRecord,
        expected_turn_version: u64,
        expected_fencing_token: u64,
        expected_lease_token: Option<String>,
        response_item: crate::domain::AgentSessionItemRecord,
    ) -> KernelResult<(
        crate::domain::AgentSessionRecord,
        crate::domain::AgentSessionItemRecord,
    )> {
        self.0.complete_turn(
            turn,
            expected_turn_version,
            expected_fencing_token,
            expected_lease_token,
            response_item,
        )
    }

    fn list_item_drive_refs(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
    ) -> KernelResult<Vec<crate::domain::AgentItemDriveRefRecord>> {
        self.0
            .list_item_drive_refs(tenant_id, organization_id, item_id)
    }

    fn list_item_drive_refs_batch(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_ids: &[String],
    ) -> KernelResult<Vec<crate::domain::AgentItemDriveRefRecord>> {
        self.0
            .list_item_drive_refs_batch(tenant_id, organization_id, item_ids)
    }

    fn insert_task(&self, record: crate::domain::AgentTaskRecord) -> KernelResult<()> {
        self.0.insert_task(record)
    }

    fn update_task(&self, record: crate::domain::AgentTaskRecord) -> KernelResult<()> {
        self.0.update_task(record)
    }

    fn get_task(
        &self,
        tenant_id: u64,
        task_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentTaskRecord>> {
        self.0.get_task(tenant_id, task_id)
    }

    fn list_tasks(
        &self,
        query: &crate::ports::TaskListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentTaskRecord>> {
        self.0.list_tasks(query)
    }

    fn count_tasks(&self, query: &crate::ports::TaskListQuery) -> KernelResult<u64> {
        self.0.count_tasks(query)
    }

    fn insert_interaction(
        &self,
        record: crate::domain::AgentInteractionRecord,
    ) -> KernelResult<()> {
        self.0.insert_interaction(record)
    }

    fn update_interaction(
        &self,
        record: crate::domain::AgentInteractionRecord,
    ) -> KernelResult<()> {
        self.0.update_interaction(record)
    }

    fn get_interaction(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        interaction_id: &str,
    ) -> KernelResult<Option<crate::domain::AgentInteractionRecord>> {
        self.0
            .get_interaction(tenant_id, organization_id, session_id, interaction_id)
    }

    fn list_interactions(
        &self,
        query: &crate::ports::InteractionListQuery,
    ) -> KernelResult<Vec<crate::domain::AgentInteractionRecord>> {
        self.0.list_interactions(query)
    }

    fn count_interactions(&self, query: &crate::ports::InteractionListQuery) -> KernelResult<u64> {
        self.0.count_interactions(query)
    }

    fn activate_provider_binding_atomic(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
        updated_at: String,
    ) -> KernelResult<AgentProviderBindingRecord> {
        self.0
            .activate_provider_binding_atomic(tenant_id, agent_id, binding_id, updated_at)
    }
}

impl AgentAuditSink for DynAgentAuditSink {
    fn record(&self, event: sdkwork_agent_kernel::KernelEvent) -> KernelResult<()> {
        self.0.record(event)
    }

    fn list_events(
        &self,
        query: &crate::ports::AuditEventListQuery,
    ) -> KernelResult<crate::ports::PaginatedResult<sdkwork_agent_kernel::KernelEvent>> {
        self.0.list_events(query)
    }
}

impl PolicyProvider for DynPolicyProvider {
    fn evaluate(&self, request: PolicyRequest) -> KernelResult<PolicyDecision> {
        self.0.evaluate(request)
    }

    fn health(&self) -> ProviderHealth {
        self.0.health()
    }
}

pub(crate) type HttpService =
    AgentsService<DynAgentRepository, DynAgentAuditSink, DynPolicyProvider>;

#[derive(Clone)]
pub struct AgentHttpState {
    service: Arc<HttpService>,
}

impl AgentHttpState {
    pub fn new<R, A, P>(repository: R, audit_sink: A, policy_provider: P) -> Self
    where
        R: AgentRepository + Send + Sync + 'static,
        A: AgentAuditSink + Send + Sync + 'static,
        P: PolicyProvider + Send + Sync + 'static,
    {
        Self::with_turn_executor(
            repository,
            audit_sink,
            policy_provider,
            Arc::new(ContractTurnExecutor),
        )
    }

    pub fn with_turn_executor<R, A, P>(
        repository: R,
        audit_sink: A,
        policy_provider: P,
        turn_executor: Arc<dyn TurnExecutor>,
    ) -> Self
    where
        R: AgentRepository + Send + Sync + 'static,
        A: AgentAuditSink + Send + Sync + 'static,
        P: PolicyProvider + Send + Sync + 'static,
    {
        let service = AgentsService::new(
            DynAgentRepository::new(repository),
            DynAgentAuditSink::new(audit_sink),
            DynPolicyProvider::new(policy_provider),
        )
        .with_turn_executor(turn_executor);
        Self {
            service: Arc::new(service),
        }
    }

    pub fn session_facade(&self) -> Arc<dyn sdkwork_agents_runtime_facade::AgentsSessionFacade> {
        Arc::new(HttpAgentsSessionFacade {
            service: self.service.clone(),
        })
    }

    /// Verify the repository dependency used by the same HTTP service state.
    pub fn check_readiness(&self) -> KernelResult<()> {
        self.service.check_readiness()
    }

    pub fn spawn_turn_reconciliation_worker(&self) -> Option<tokio::task::JoinHandle<()>> {
        let interval_seconds = env_usize(ENV_TURN_RECONCILIATION_INTERVAL_SECONDS, 30, 0, 3600);
        if interval_seconds == 0 {
            return None;
        }
        let stale_after_seconds = env_usize(ENV_TURN_STALE_AFTER_SECONDS, 300, 30, 86_400);
        let batch_size = env_usize(ENV_TURN_RECONCILIATION_BATCH_SIZE, 100, 1, 200);
        let service = self.service.clone();
        Some(tokio::spawn(async move {
            let mut ticker =
                tokio::time::interval(std::time::Duration::from_secs(interval_seconds as u64));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                ticker.tick().await;
                let now = OffsetDateTime::now_utc();
                let stale_before = now - time::Duration::seconds(stale_after_seconds as i64);
                let occurred_at = format_utc_seconds(now);
                let stale_before = format_utc_seconds(stale_before);
                let service = service.clone();
                let result = tokio::task::spawn_blocking(move || {
                    service.reconcile_stale_turns(&stale_before, &occurred_at, batch_size)
                })
                .await;
                match result {
                    Ok(Ok(summary))
                        if !summary.failed.is_empty() || summary.skipped_conflicts > 0 =>
                    {
                        tracing::info!(
                            target: "sdkwork.agents.turn.reconciliation",
                            examined = summary.examined,
                            failed = summary.failed.len(),
                            skipped_conflicts = summary.skipped_conflicts,
                            "turn reconciliation completed"
                        );
                    }
                    Ok(Ok(_)) => {}
                    Ok(Err(error)) => tracing::error!(
                        target: "sdkwork.agents.turn.reconciliation",
                        error = %error,
                        "turn reconciliation failed"
                    ),
                    Err(error) => tracing::error!(
                        target: "sdkwork.agents.turn.reconciliation",
                        error = %error,
                        "turn reconciliation worker join failed"
                    ),
                }
            }
        }))
    }
}

struct HttpAgentsSessionFacade {
    service: Arc<HttpService>,
}

struct EnsureRuntimeBindingRequest<'a> {
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    agent_id: &'a str,
    session_id: &'a str,
    descriptor: Option<&'a sdkwork_agents_runtime_facade::AgentsSessionRuntimeBindingDescriptor>,
    subject: sdkwork_agent_kernel::PolicySubject,
    requested_at: &'a str,
}

impl HttpAgentsSessionFacade {
    fn ensure_runtime_binding(
        &self,
        request: EnsureRuntimeBindingRequest<'_>,
    ) -> sdkwork_agents_runtime_facade::RuntimeFacadeResult<()> {
        let EnsureRuntimeBindingRequest {
            tenant_id,
            organization_id,
            owner_user_id,
            agent_id,
            session_id,
            descriptor,
            subject,
            requested_at,
        } = request;
        let Some(descriptor) = descriptor else {
            return Ok(());
        };
        let existing = self
            .service
            .get_session_runtime_binding(GetSessionRuntimeBindingCommand {
                tenant_id,
                organization_id,
                path_agent_id: agent_id.to_string(),
                session_id: session_id.to_string(),
                runtime_binding_id: descriptor.runtime_binding_id.clone(),
                owner_scope: Some(owner_user_id),
                requested_by: subject.clone(),
            });
        match existing {
            Ok(existing) => {
                if !runtime_binding_matches_descriptor(&existing, descriptor) {
                    return Err(
                        sdkwork_agents_runtime_facade::RuntimeFacadeError::InvalidInput(
                            "runtime binding descriptor conflicts with the current session binding"
                                .into(),
                        ),
                    );
                }
                return Ok(());
            }
            Err(KernelError::Validation { message })
                if message == "session runtime binding not found" => {}
            Err(error) => {
                return Err(sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                    error.to_string(),
                ));
            }
        }

        self.service
            .create_session_runtime_binding(
                crate::application::CreateSessionRuntimeBindingCommand {
                    tenant_id,
                    organization_id,
                    path_agent_id: agent_id.to_string(),
                    session_id: session_id.to_string(),
                    runtime_binding_id: Some(descriptor.runtime_binding_id.clone()),
                    runtime_location_id: descriptor.runtime_location_id.clone(),
                    host_mode: descriptor.host_mode.clone(),
                    transport_kind: descriptor.transport_kind.clone(),
                    provider_binding_id: descriptor.provider_binding_id.clone(),
                    model_id: descriptor.model_id.clone(),
                    provider_id: descriptor.provider_id.clone(),
                    native_session_id: descriptor.native_session_id.clone(),
                    native_session_tree_id: descriptor.native_session_tree_id.clone(),
                    native_parent_session_id: descriptor.native_parent_session_id.clone(),
                    native_forked_from_session_id: descriptor.native_forked_from_session_id.clone(),
                    owner_scope: Some(owner_user_id),
                    requested_by: subject,
                    requested_at: requested_at.to_string(),
                },
            )
            .map_err(|error| {
                sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(error.to_string())
            })?;
        Ok(())
    }
}

fn runtime_binding_matches_descriptor(
    record: &AgentSessionRuntimeBindingRecord,
    descriptor: &sdkwork_agents_runtime_facade::AgentsSessionRuntimeBindingDescriptor,
) -> bool {
    record.status == AgentSessionRuntimeBindingStatus::Active
        && record.is_current
        && record.runtime_binding_id == descriptor.runtime_binding_id
        && record.runtime_location_id == descriptor.runtime_location_id
        && record.host_mode == descriptor.host_mode
        && record.transport_kind == descriptor.transport_kind
        && record.provider_binding_id == descriptor.provider_binding_id
        && record.model_id == descriptor.model_id
        && record.provider_id == descriptor.provider_id
        && record.native_session_id == descriptor.native_session_id
        && record.native_session_tree_id == descriptor.native_session_tree_id
        && record.native_parent_session_id == descriptor.native_parent_session_id
        && record.native_forked_from_session_id == descriptor.native_forked_from_session_id
}

fn map_facade_session_kind(
    kind: sdkwork_agents_runtime_facade::AgentsSessionKind,
) -> AgentSessionKind {
    match kind {
        sdkwork_agents_runtime_facade::AgentsSessionKind::Assistant => AgentSessionKind::Assistant,
        sdkwork_agents_runtime_facade::AgentsSessionKind::Coding => AgentSessionKind::Coding,
        sdkwork_agents_runtime_facade::AgentsSessionKind::Automation => {
            AgentSessionKind::Automation
        }
        sdkwork_agents_runtime_facade::AgentsSessionKind::ImDispatch => {
            AgentSessionKind::ImDispatch
        }
    }
}

fn map_facade_entry_surface(
    surface: sdkwork_agents_runtime_facade::AgentsSessionEntrySurface,
) -> AgentSessionEntrySurface {
    match surface {
        sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::Pc => {
            AgentSessionEntrySurface::Pc
        }
        sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::H5 => {
            AgentSessionEntrySurface::H5
        }
        sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::Flutter => {
            AgentSessionEntrySurface::Flutter
        }
        sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::MiniProgram => {
            AgentSessionEntrySurface::MiniProgram
        }
        sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::Api => {
            AgentSessionEntrySurface::Api
        }
        sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::ImDispatch => {
            AgentSessionEntrySurface::ImDispatch
        }
        sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::Automation => {
            AgentSessionEntrySurface::Automation
        }
    }
}

impl sdkwork_agents_runtime_facade::AgentsSessionFacade for HttpAgentsSessionFacade {
    fn resolve_or_create_session(
        &self,
        request: sdkwork_agents_runtime_facade::ResolveAgentsSessionRequest,
    ) -> sdkwork_agents_runtime_facade::RuntimeFacadeResult<
        sdkwork_agents_runtime_facade::ResolvedAgentsSession,
    > {
        sdkwork_agents_runtime_facade::validate_resolve_agents_session_request(&request)?;
        let subject = facade_policy_subject(
            request.tenant_id,
            &request.actor.subject_id,
            &request.actor.roles,
        );
        match self.service.get_session(GetSessionCommand {
            tenant_id: request.tenant_id,
            organization_id: request.organization_id,
            path_agent_id: request.agent_id.clone(),
            session_id: request.session_id.clone(),
            owner_scope: Some(request.owner_user_id),
            requested_by: subject.clone(),
        }) {
            Ok(existing) => {
                if existing.organization_id != request.organization_id {
                    return Err(
                        sdkwork_agents_runtime_facade::RuntimeFacadeError::InvalidInput(
                            "session organization mismatch".into(),
                        ),
                    );
                }
                if existing
                    .idempotency_key
                    .as_deref()
                    .is_some_and(|value| value != request.idempotency_key)
                    || existing
                        .payload_hash
                        .as_deref()
                        .is_some_and(|value| value != request.payload_hash)
                {
                    return Err(
                        sdkwork_agents_runtime_facade::RuntimeFacadeError::InvalidInput(
                            "session resolution idempotency payload conflicts with the existing session"
                                .into(),
                        ),
                    );
                }
                self.ensure_runtime_binding(EnsureRuntimeBindingRequest {
                    tenant_id: request.tenant_id,
                    organization_id: request.organization_id,
                    owner_user_id: request.owner_user_id,
                    agent_id: &request.agent_id,
                    session_id: &request.session_id,
                    descriptor: request.runtime_binding.as_ref(),
                    subject,
                    requested_at: &request.requested_at,
                })?;
                return Ok(sdkwork_agents_runtime_facade::ResolvedAgentsSession {
                    session_id: existing.session_id,
                    created: false,
                    version: existing.version,
                });
            }
            Err(KernelError::Validation { message }) if message == "session not found" => {}
            Err(error) => {
                return Err(sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                    error.to_string(),
                ));
            }
        }
        let created = self
            .service
            .create_session(CreateSessionCommand {
                tenant_id: request.tenant_id,
                organization_id: request.organization_id,
                agent_id: request.agent_id.clone(),
                owner_user_id: request.owner_user_id,
                session_id: request.session_id.clone(),
                project_id: request.project_id.clone(),
                session_kind: map_facade_session_kind(request.session_kind),
                entry_surface: map_facade_entry_surface(request.entry_surface),
                source_module: request.source_module.clone(),
                source_context_kind: request.source_context_kind.clone(),
                source_context_id: request.source_context_id.clone(),
                parent_session_id: request.parent_session_id.clone(),
                forked_from_turn_id: request.forked_from_turn_id.clone(),
                title: Some(request.title.clone()),
                idempotency_key: Some(request.idempotency_key.clone()),
                payload_hash: Some(request.payload_hash.clone()),
                requested_by: subject.clone(),
                requested_at: request.requested_at.clone(),
            })
            .map_err(|error| {
                sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(error.to_string())
            })?;
        self.ensure_runtime_binding(EnsureRuntimeBindingRequest {
            tenant_id: request.tenant_id,
            organization_id: request.organization_id,
            owner_user_id: request.owner_user_id,
            agent_id: &request.agent_id,
            session_id: &created.session_id,
            descriptor: request.runtime_binding.as_ref(),
            subject,
            requested_at: &request.requested_at,
        })?;
        Ok(sdkwork_agents_runtime_facade::ResolvedAgentsSession {
            session_id: created.session_id,
            created: true,
            version: created.version,
        })
    }

    fn complete_turn(
        &self,
        request: sdkwork_agents_runtime_facade::CompleteAgentsTurnRequest,
    ) -> sdkwork_agents_runtime_facade::RuntimeFacadeResult<
        sdkwork_agents_runtime_facade::CompletedAgentsTurn,
    > {
        sdkwork_agents_runtime_facade::validate_session_actor(&request.actor)?;
        let subject = facade_policy_subject(
            request.tenant_id,
            &request.actor.subject_id,
            &request.actor.roles,
        );
        let payload_hash = sdkwork_utils_rust::sha256_hash(request.content.as_bytes());
        let result = self
            .service
            .execute_turn(CreateTurnCommand {
                tenant_id: request.tenant_id,
                organization_id: request.organization_id,
                agent_id: request.agent_id,
                session_id: request.session_id,
                turn_id: None,
                content: request.content,
                content_type: request.content_type,
                turn_mode: crate::agent_turn::AgentTurnMode::Interactive,
                runtime_binding_id: None,
                requested_model_id: None,
                idempotency_key: request.idempotency_key.clone(),
                payload_hash,
                client_request_id: Some(request.client_request_id),
                drive_refs: Vec::new(),
                owner_scope: Some(request.owner_user_id),
                requested_by: subject,
                requested_at: request.requested_at,
                prefer_stream: false,
            })
            .map_err(|error| {
                sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(error.to_string())
            })?;
        let turn_id = result
            .user_input_item
            .turn_id
            .clone()
            .or(result.assistant_output_item.turn_id.clone())
            .ok_or_else(|| {
                sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                    "completed Agents turn did not return turnId".into(),
                )
            })?;
        let response_content = result.assistant_output_item.content.ok_or_else(|| {
            sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                "completed Agents turn returned no assistant content".into(),
            )
        })?;
        Ok(sdkwork_agents_runtime_facade::CompletedAgentsTurn {
            session_id: result.session.session_id,
            turn_id,
            request_item_id: result.user_input_item.item_id,
            response_item_id: result.assistant_output_item.item_id,
            response_content,
        })
    }

    fn get_turn_by_idempotency(
        &self,
        request: sdkwork_agents_runtime_facade::GetAgentsTurnByIdempotencyRequest,
    ) -> sdkwork_agents_runtime_facade::RuntimeFacadeResult<
        Option<sdkwork_agents_runtime_facade::AgentsTurnSnapshot>,
    > {
        sdkwork_agents_runtime_facade::validate_session_actor(&request.actor)?;
        let subject = facade_policy_subject(
            request.tenant_id,
            &request.actor.subject_id,
            &request.actor.roles,
        );
        let Some(turn) = self
            .service
            .get_turn_by_idempotency(GetTurnByIdempotencyCommand {
                tenant_id: request.tenant_id,
                organization_id: request.organization_id,
                path_agent_id: request.agent_id.clone(),
                session_id: request.session_id.clone(),
                owner_user_id: request.owner_user_id,
                idempotency_key: request.idempotency_key,
                requested_by: subject.clone(),
            })
            .map_err(|error| {
                sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(error.to_string())
            })?
        else {
            return Ok(None);
        };
        let response_content = match turn.response_item_id.as_deref() {
            Some(item_id) if turn.status == crate::AgentTurnStatus::Completed => {
                self.service
                    .get_session_item(GetSessionItemCommand {
                        tenant_id: request.tenant_id,
                        organization_id: request.organization_id,
                        path_agent_id: request.agent_id,
                        session_id: request.session_id,
                        item_id: item_id.to_owned(),
                        owner_scope: Some(request.owner_user_id),
                        requested_by: subject,
                    })
                    .map_err(|error| {
                        sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(
                            error.to_string(),
                        )
                    })?
                    .content
            }
            _ => None,
        };
        let status = match turn.status {
            crate::AgentTurnStatus::Requested => {
                sdkwork_agents_runtime_facade::AgentsTurnStatus::Requested
            }
            crate::AgentTurnStatus::Running => {
                sdkwork_agents_runtime_facade::AgentsTurnStatus::Running
            }
            crate::AgentTurnStatus::Completed => {
                sdkwork_agents_runtime_facade::AgentsTurnStatus::Completed
            }
            crate::AgentTurnStatus::Failed => {
                sdkwork_agents_runtime_facade::AgentsTurnStatus::Failed
            }
            crate::AgentTurnStatus::Cancelled => {
                sdkwork_agents_runtime_facade::AgentsTurnStatus::Cancelled
            }
        };
        Ok(Some(sdkwork_agents_runtime_facade::AgentsTurnSnapshot {
            session_id: turn.session_id,
            turn_id: turn.turn_id,
            status,
            request_item_id: turn.request_item_id,
            response_item_id: turn.response_item_id,
            response_content,
            error_code: turn.error_code,
        }))
    }
}

fn facade_policy_subject(
    tenant_id: u64,
    subject_id: &str,
    roles: &[String],
) -> sdkwork_agent_kernel::PolicySubject {
    roles.iter().fold(
        sdkwork_agent_kernel::PolicySubject::new(subject_id, tenant_id.to_string()),
        |subject, role| subject.with_role(role.clone()),
    )
}

/// Raw app-api route tree without gateway or web-framework middleware.
pub fn build_app_routes() -> Router<AgentHttpState> {
    Router::new()
        .route(
            "/app/v3/api/ai/workspaces",
            get(app_list_workspaces).post(app_create_workspace),
        )
        .route(
            "/app/v3/api/ai/workspaces/default",
            post(app_ensure_default_workspace),
        )
        .route(
            "/app/v3/api/ai/workspaces/{workspaceId}",
            get(app_get_workspace)
                .patch(app_update_workspace)
                .delete(app_delete_workspace),
        )
        .route(
            "/app/v3/api/ai/workspaces/{workspaceId}/archive",
            post(app_archive_workspace),
        )
        .route("/app/v3/api/ai/projects/import", post(app_import_project))
        .route(
            "/app/v3/api/ai/projects",
            get(app_list_projects).post(app_create_project),
        )
        .route(
            "/app/v3/api/ai/projects/{projectId}",
            get(app_get_project)
                .patch(app_update_project)
                .delete(app_delete_project),
        )
        .route(
            "/app/v3/api/ai/projects/{projectId}/archive",
            post(app_archive_project),
        )
        .route(
            "/app/v3/api/ai/projects/{projectId}/composition_slots",
            get(app_list_project_composition_slots).post(app_create_project_composition_slot),
        )
        .route(
            "/app/v3/api/ai/projects/{projectId}/composition_slots/{slotId}",
            get(app_get_project_composition_slot)
                .patch(app_update_project_composition_slot)
                .delete(app_delete_project_composition_slot),
        )
        .route(
            "/app/v3/api/ai/agents",
            get(app_list_agents).post(app_create_agent),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}",
            get(app_get_agent)
                .patch(app_update_agent)
                .delete(app_delete_agent),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/restore",
            post(app_restore_agent),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/provider_bindings",
            get(app_list_provider_bindings).post(app_add_provider_binding),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
            post(app_activate_provider_binding),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/preview_responses",
            post(app_create_preview_response),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/prompt_optimizations",
            post(app_create_prompt_optimization),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/composition_slots",
            get(app_list_composition_slots).post(app_create_composition_slot),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
            get(app_get_composition_slot)
                .patch(app_update_composition_slot)
                .delete(app_delete_composition_slot),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions",
            get(app_list_sessions).post(app_create_session),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/user_states",
            get(app_list_session_user_states),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
            get(app_get_session)
                .patch(app_update_session)
                .delete(app_delete_session),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/close",
            post(app_close_session),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/user_state",
            get(app_get_session_user_state).patch(app_update_session_user_state),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/item_feedback",
            get(app_list_item_feedback),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items",
            get(app_list_session_items),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items/{itemId}",
            get(app_get_session_item),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items/{itemId}/feedback",
            axum::routing::patch(app_update_item_feedback),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns",
            get(app_list_turns).post(app_create_turn),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}",
            get(app_get_turn),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}/cancel",
            post(app_cancel_turn),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
            get(app_list_interactions).post(app_create_interaction),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}",
            get(app_get_interaction),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/claim",
            post(app_claim_interaction),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/approve",
            post(app_approve_interaction),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/answer",
            post(app_answer_interaction),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints",
            get(app_list_session_checkpoints).post(app_create_session_checkpoint),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}",
            get(app_get_session_checkpoint),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/restore",
            post(app_restore_session_checkpoint),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/invalidate",
            post(app_invalidate_session_checkpoint),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings",
            get(app_list_session_runtime_bindings).post(app_create_session_runtime_binding),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}",
            get(app_get_session_runtime_binding).patch(app_update_session_runtime_binding),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/activate",
            post(app_activate_session_runtime_binding),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/deactivate",
            post(app_deactivate_session_runtime_binding),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks",
            get(app_list_tasks).post(app_create_task),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}",
            get(app_get_task),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/cancel",
            post(app_cancel_task),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/execute",
            post(app_execute_task),
        )
        .route(
            "/app/v3/api/ai/code_engines",
            get(app_list_code_engines),
        )
        .route(
            "/app/v3/api/ai/mcp_servers",
            get(app_list_mcp_servers),
        )
        .layer(axum::middleware::from_fn(
            middleware::reject_client_scope_selectors,
        ))
}

/// Raw open-api route tree without gateway or web-framework middleware.
pub fn build_open_routes() -> Router<AgentHttpState> {
    Router::new()
        .route(
            "/agent/v3/api/ai/agents",
            get(backend_list_agents).post(backend_create_agent),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}",
            get(backend_get_agent)
                .patch(backend_update_agent)
                .delete(open_delete_agent),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/provider_bindings",
            get(backend_list_provider_bindings).post(backend_add_provider_binding),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
            post(backend_activate_provider_binding),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/preview_responses",
            post(open_create_preview_response),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/prompt_optimizations",
            post(open_create_prompt_optimization),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/composition_slots",
            get(backend_list_composition_slots).post(backend_create_composition_slot),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
            get(backend_get_composition_slot)
                .patch(backend_update_composition_slot)
                .delete(backend_delete_composition_slot),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions",
            get(backend_list_sessions).post(backend_create_session),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
            get(backend_get_session),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/close",
            post(backend_close_session),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items",
            get(backend_list_session_items),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items/{itemId}",
            get(backend_get_session_item),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns",
            get(backend_list_turns).post(backend_create_turn),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}",
            get(backend_get_turn),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}/cancel",
            post(backend_cancel_turn),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
            get(backend_list_interactions).post(backend_create_interaction),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}",
            get(backend_get_interaction),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/claim",
            post(backend_claim_interaction),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/approve",
            post(backend_approve_interaction),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/answer",
            post(backend_answer_interaction),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints",
            get(backend_list_session_checkpoints).post(backend_create_session_checkpoint),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}",
            get(backend_get_session_checkpoint),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/restore",
            post(backend_restore_session_checkpoint),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/invalidate",
            post(backend_invalidate_session_checkpoint),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings",
            get(backend_list_session_runtime_bindings).post(backend_create_session_runtime_binding),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}",
            get(backend_get_session_runtime_binding).patch(backend_update_session_runtime_binding),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/activate",
            post(backend_activate_session_runtime_binding),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/deactivate",
            post(backend_deactivate_session_runtime_binding),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks",
            get(backend_list_tasks).post(backend_create_task),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}",
            get(backend_get_task),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/cancel",
            post(backend_cancel_task),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/execute",
            post(backend_execute_task),
        )
        .layer(axum::middleware::from_fn(
            middleware::reject_client_scope_selectors,
        ))
}

/// Raw backend-api route tree without gateway or web-framework middleware.
pub fn build_backend_routes() -> Router<AgentHttpState> {
    Router::new()
        .route(
            "/backend/v3/api/ai/agents",
            get(backend_list_agents).post(backend_create_agent),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}",
            get(backend_get_agent).patch(backend_update_agent),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/status",
            post(backend_update_agent_status),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/restore",
            post(backend_restore_agent),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/audit_events",
            get(backend_list_agent_audit_events),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/provider_bindings",
            get(backend_list_provider_bindings).post(backend_add_provider_binding),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
            post(backend_activate_provider_binding),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/composition_slots",
            get(backend_list_composition_slots).post(backend_create_composition_slot),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
            get(backend_get_composition_slot)
                .patch(backend_update_composition_slot)
                .delete(backend_delete_composition_slot),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions",
            get(backend_list_sessions).post(backend_create_session),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
            get(backend_get_session),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/close",
            post(backend_close_session),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/archive",
            post(backend_archive_session),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items",
            get(backend_list_session_items),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/items/{itemId}",
            get(backend_get_session_item),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns",
            get(backend_list_turns).post(backend_create_turn),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}",
            get(backend_get_turn),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/turns/{turnId}/cancel",
            post(backend_cancel_turn),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
            get(backend_list_interactions).post(backend_create_interaction),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}",
            get(backend_get_interaction),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/claim",
            post(backend_claim_interaction),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/approve",
            post(backend_approve_interaction),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/answer",
            post(backend_answer_interaction),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints",
            get(backend_list_session_checkpoints).post(backend_create_session_checkpoint),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}",
            get(backend_get_session_checkpoint),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/restore",
            post(backend_restore_session_checkpoint),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/checkpoints/{checkpointId}/invalidate",
            post(backend_invalidate_session_checkpoint),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings",
            get(backend_list_session_runtime_bindings).post(backend_create_session_runtime_binding),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}",
            get(backend_get_session_runtime_binding).patch(backend_update_session_runtime_binding),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/activate",
            post(backend_activate_session_runtime_binding),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/runtime_bindings/{runtimeBindingId}/deactivate",
            post(backend_deactivate_session_runtime_binding),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks",
            get(backend_list_tasks).post(backend_create_task),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}",
            get(backend_get_task),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/cancel",
            post(backend_cancel_task),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/execute",
            post(backend_execute_task),
        )
        .layer(axum::middleware::from_fn(
            middleware::reject_client_scope_selectors,
        ))
}

/// Prometheus metrics endpoint handler (O-01).
/// Exposes service-level metrics in Prometheus text exposition format.
pub async fn serve_agents_metrics() -> impl IntoResponse {
    let metrics = crate::infrastructure::AgentMetricsRegistry::global().snapshot();
    let prometheus_text = metrics.to_prometheus_text();

    let mut response = prometheus_text.into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );
    response
}

/// Raw combined route tree for served production mounts.
pub fn build_combined_routes() -> Router<AgentHttpState> {
    build_open_routes()
        .merge(build_app_routes())
        .merge(build_backend_routes())
        .route("/metrics/agents", get(serve_agents_metrics))
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ListCompositionSlotsQueryParams {
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ListMcpServersQueryParams {
    q: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListAgentsQueryParams {
    scope: Option<String>,
    include_deleted: Option<bool>,
    q: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppListAgentsQueryParams {
    scope: Option<String>,
    include_deleted: Option<bool>,
    q: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct AppListProjectsQueryParams {
    #[serde(rename = "workspaceId", alias = "workspace_id")]
    workspace_id: Option<String>,
    q: Option<String>,
    status: Option<String>,
    include_deleted: Option<bool>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct AppListWorkspacesQueryParams {
    status: Option<String>,
    include_deleted: Option<bool>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppEnsureDefaultWorkspaceBody {
    name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppCreateWorkspaceBody {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppUpdateWorkspaceBody {
    expected_version: Option<String>,
    name: Option<String>,
    description: Option<Option<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppWorkspaceMutationBody {
    expected_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppDeleteWorkspaceQuery {
    expected_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppCreateProjectBody {
    project_id: Option<String>,
    workspace_id: Option<String>,
    name: String,
    description: Option<String>,
    visibility: Option<String>,
    drive_access_mode: Option<String>,
    default_agent_id: Option<String>,
    default_model_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppImportProjectBody {
    workspace_id: String,
    project_id: Option<String>,
    name: String,
    description: Option<String>,
    source_kind: String,
    source_ref: String,
    drive_space_id: String,
    drive_root_entry_id: String,
    drive_logical_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppUpdateProjectBody {
    expected_version: Option<String>,
    name: Option<String>,
    description: Option<Option<String>>,
    visibility: Option<String>,
    drive_access_mode: Option<String>,
    default_agent_id: Option<Option<String>>,
    default_model_id: Option<Option<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct AppProjectMutationBody {
    expected_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct AppListProjectCompositionSlotsQuery {
    slot_kind: Option<String>,
    enabled: Option<bool>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppCreateProjectCompositionSlotBody {
    slot_id: String,
    slot_kind: String,
    target_module: String,
    target_ref: String,
    target_version_ref: Option<String>,
    priority: Option<i32>,
    enabled: Option<bool>,
    policy_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppUpdateProjectCompositionSlotBody {
    expected_version: Option<String>,
    slot_kind: Option<String>,
    target_module: Option<String>,
    target_ref: Option<String>,
    target_version_ref: Option<String>,
    clear_target_version_ref: Option<bool>,
    priority: Option<i32>,
    enabled: Option<bool>,
    policy_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppDeleteProjectCompositionSlotQuery {
    expected_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppUpdateSessionBody {
    expected_version: Option<String>,
    title: Option<String>,
    project_id: Option<String>,
    clear_project: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppUpdateSessionUserStateBody {
    expected_version: Option<String>,
    pinned: Option<bool>,
    hidden: Option<bool>,
    mark_opened: Option<bool>,
    last_read_item_sequence: Option<String>,
    custom_title: Option<String>,
    clear_custom_title: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppUpdateItemFeedbackBody {
    expected_version: Option<String>,
    rating: Option<String>,
    clear_feedback: Option<bool>,
    reason_code: Option<String>,
    comment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppCancelTurnBody {
    expected_version: String,
    requested_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentProjectRecordResponse {
    id: String,
    project_id: String,
    workspace_id: String,
    tenant_id: String,
    organization_id: String,
    owner_user_id: String,
    name: String,
    description: Option<String>,
    visibility: String,
    status: String,
    drive_access_mode: String,
    default_agent_id: Option<String>,
    default_model_id: Option<String>,
    import_source_kind: Option<String>,
    import_source_ref: Option<String>,
    drive_space_id: Option<String>,
    drive_root_entry_id: Option<String>,
    drive_logical_path: Option<String>,
    version: String,
    created_at: String,
    updated_at: String,
    archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentWorkspaceRecordResponse {
    id: String,
    workspace_id: String,
    tenant_id: String,
    organization_id: String,
    owner_user_id: String,
    name: String,
    description: Option<String>,
    is_default: bool,
    status: String,
    version: String,
    created_at: String,
    updated_at: String,
    archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentProjectCompositionSlotRecordResponse {
    id: String,
    tenant_id: String,
    organization_id: String,
    project_id: String,
    slot_id: String,
    slot_kind: String,
    target_module: String,
    target_ref: String,
    target_version_ref: Option<String>,
    priority: i32,
    enabled: bool,
    policy_json: String,
    created_by: String,
    updated_by: String,
    version: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct AppCreateTurnQueryParams {
    #[serde(default)]
    stream: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct BackendCreateTurnQueryParams {
    #[serde(default)]
    pub(crate) stream: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppListQueryParams {
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TenantAgentPathParams {
    #[serde(rename = "agentId")]
    pub(crate) agent_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TenantAgentBindingPathParams {
    #[serde(rename = "agentId")]
    agent_id: String,
    #[serde(rename = "bindingId")]
    binding_id: String,
}
#[derive(Debug, Clone, Deserialize)]
struct TenantAgentSlotPathParams {
    #[serde(rename = "agentId")]
    agent_id: String,
    #[serde(rename = "slotId")]
    slot_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListSessionsQueryParams {
    pub(crate) status: Option<String>,
    pub(crate) include_archived: Option<bool>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AppListSessionsQueryParams {
    pub(crate) project_id: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) include_archived: Option<bool>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppListSessionUserStatesQueryParams {
    pinned_only: Option<bool>,
    include_hidden: Option<bool>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppListItemFeedbackQueryParams {
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListTasksQueryParams {
    pub(crate) status: Option<String>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AppListTasksQueryParams {
    pub(crate) status: Option<String>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListItemsQueryParams {
    pub(crate) kind: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) sort: Option<String>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AppListItemsQueryParams {
    pub(crate) kind: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) sort: Option<String>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AppListInteractionsQueryParams {
    pub(crate) kind: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListInteractionsQueryParams {
    pub(crate) kind: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentCompositionSlotRecordResponse {
    id: String,
    tenant_id: String,
    organization_id: String,
    agent_id: String,
    slot_id: String,
    slot_kind: String,
    target_module: String,
    target_ref: String,
    target_version_ref: Option<String>,
    priority: i32,
    enabled: bool,
    policy_json: String,
    status: String,
    version: String,
    created_at: String,
    updated_at: String,
    deleted_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuditEventsQueryParams {
    page: Option<usize>,
    page_size: Option<usize>,
    action: Option<String>,
    from: Option<String>,
    to: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateAgentBody {
    agent_id: String,
    code: String,
    display_name: String,
    description: Option<String>,
    manifest: Value,
    default_code_task_intent: Option<CodeTaskIntentBody>,
    management_profile: Option<AgentManagementProfileBody>,
    implementation_provider_id: Option<String>,
    implementation_kind: Option<String>,
    implementation_type: Option<String>,
    visibility: String,
    tags: Option<Vec<String>>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentProviderBindingBody {
    binding_id: String,
    provider_id: String,
    implementation_kind: String,
    configuration_profile_id: String,
    capabilities: Option<Vec<String>>,
    make_default: Option<bool>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivateProviderBindingBody {
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentPreviewResponseBody {
    execution_id: String,
    content: String,
    debug_mode: Option<bool>,
    model: Option<String>,
    temperature: Option<f32>,
    input_payload: Option<Value>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentPromptOptimizationBody {
    execution_id: String,
    prompt: String,
    input_payload: Option<Value>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateAgentBody {
    display_name: Option<String>,
    description: Option<String>,
    manifest: Option<Value>,
    visibility: Option<String>,
    tags: Option<Vec<String>>,
    default_code_task_intent: Option<CodeTaskIntentBody>,
    management_profile: Option<AgentManagementProfileBody>,
    implementation_provider_id: Option<Option<String>>,
    implementation_kind: Option<Option<String>>,
    implementation_type: Option<String>,
    expected_version: Option<String>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateAgentStatusBody {
    target_status: String,
    expected_version: Option<String>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RestoreAgentBody {
    expected_version: Option<String>,
    requested_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentProviderBindingRecordResponse {
    tenant_id: String,
    agent_id: String,
    binding_id: String,
    provider_id: String,
    implementation_kind: String,
    configuration_profile_id: String,
    capabilities: Vec<String>,
    active: bool,
    version: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentRuntimeExecutionRecordResponse {
    tenant_id: String,
    agent_id: String,
    execution_id: String,
    operation: String,
    status: String,
    input_payload: Value,
    output_payload: Value,
    requested_at: String,
    completed_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentRecordResponse {
    id: String,
    agent_id: String,
    tenant_id: String,
    organization_id: String,
    owner_user_id: String,
    code: String,
    display_name: String,
    description: Option<String>,
    manifest: Value,
    default_code_task_intent: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    management_profile: Option<AgentManagementProfileResponse>,
    implementation_provider_id: Option<String>,
    implementation_kind: Option<String>,
    implementation_type: String,
    status: String,
    visibility: String,
    tags: Vec<String>,
    version: String,
    created_at: String,
    updated_at: String,
    deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentManagementProfileResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    debug_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    json_mode: Option<bool>,
    knowledge_base_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    skill_ids: Vec<String>,
    suggested_prompts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    tool_ids: Vec<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    agent_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    users: Option<String>,
    voice_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    welcome_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentAuditEventResponse {
    event_id: String,
    event_type: String,
    severity: String,
    payload: String,
    occurred_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodeTaskIntentBody {
    prompt: String,
    context_paths: Option<Vec<String>>,
    constraints: Option<Vec<String>>,
}

impl From<CodeTaskIntentBody> for CodeTaskIntent {
    fn from(value: CodeTaskIntentBody) -> Self {
        Self {
            prompt: value.prompt,
            context_paths: value.context_paths.unwrap_or_default(),
            constraints: value.constraints.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AgentManagementProfileBody {
    author: Option<String>,
    avatar: Option<String>,
    category_id: Option<String>,
    color: Option<String>,
    debug_mode: Option<bool>,
    icon_name: Option<String>,
    json_mode: Option<bool>,
    knowledge_base_ids: Option<Vec<String>>,
    memory_enabled: Option<bool>,
    model: Option<String>,
    skill_ids: Option<Vec<String>>,
    suggested_prompts: Option<Vec<String>>,
    system_prompt: Option<String>,
    temperature: Option<f64>,
    tool_ids: Option<Vec<String>>,
    #[serde(rename = "type")]
    agent_type: Option<String>,
    users: Option<String>,
    voice_ids: Option<Vec<String>>,
    welcome_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AppCreateTurnBody {
    turn_id: Option<String>,
    content: String,
    content_type: Option<String>,
    turn_mode: String,
    runtime_binding_id: Option<String>,
    requested_model_id: Option<String>,
    idempotency_key: String,
    payload_hash: String,
    client_request_id: Option<String>,
    #[serde(default)]
    drive_refs: Vec<AgentItemDriveRefBody>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateTurnBody {
    turn_id: Option<String>,
    content: String,
    content_type: Option<String>,
    turn_mode: String,
    runtime_binding_id: Option<String>,
    requested_model_id: Option<String>,
    idempotency_key: String,
    payload_hash: String,
    client_request_id: Option<String>,
    #[serde(default)]
    drive_refs: Vec<AgentItemDriveRefBody>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AgentItemDriveRefBody {
    resource_role: String,
    drive_space_id: String,
    drive_node_id: String,
}

impl AgentItemDriveRefBody {
    fn into_input(self) -> Result<AgentItemDriveRefInput, ApiProblem> {
        let resource_role = match self.resource_role.as_str() {
            "attachment" => AgentItemResourceRole::Attachment,
            "image" => AgentItemResourceRole::Image,
            "audio" => AgentItemResourceRole::Audio,
            "artifact" => AgentItemResourceRole::Artifact,
            _ => return Err(ApiProblem::validation("invalid driveRefs.resourceRole")),
        };
        Ok(AgentItemDriveRefInput {
            resource_role,
            drive_space_id: self.drive_space_id,
            drive_node_id: self.drive_node_id,
        })
    }
}

impl From<AgentManagementProfileBody> for AgentManagementProfileDto {
    fn from(value: AgentManagementProfileBody) -> Self {
        Self {
            author: value.author,
            avatar: value.avatar,
            category_id: value.category_id,
            color: value.color,
            debug_mode: value.debug_mode,
            icon_name: value.icon_name,
            json_mode: value.json_mode,
            knowledge_base_ids: value.knowledge_base_ids.unwrap_or_default(),
            memory_enabled: value.memory_enabled,
            model: value.model,
            skill_ids: value.skill_ids.unwrap_or_default(),
            suggested_prompts: value.suggested_prompts.unwrap_or_default(),
            system_prompt: value.system_prompt,
            temperature: value.temperature,
            tool_ids: value.tool_ids.unwrap_or_default(),
            agent_type: value.agent_type,
            users: value.users,
            voice_ids: value.voice_ids.unwrap_or_default(),
            welcome_message: value.welcome_message,
        }
    }
}

impl AgentManagementProfileBody {
    fn into_validated_dto(self) -> Result<AgentManagementProfileDto, ApiProblem> {
        validate_agent_management_profile_body(&self)?;
        Ok(self.into())
    }
}

fn validate_agent_management_profile_body(
    profile: &AgentManagementProfileBody,
) -> Result<(), ApiProblem> {
    validate_optional_profile_string(profile.author.as_deref(), "managementProfile.author", 128)?;
    validate_optional_profile_string(profile.avatar.as_deref(), "managementProfile.avatar", 512)?;
    validate_optional_profile_string(
        profile.category_id.as_deref(),
        "managementProfile.categoryId",
        64,
    )?;
    validate_optional_profile_string(profile.color.as_deref(), "managementProfile.color", 32)?;
    validate_optional_profile_string(
        profile.icon_name.as_deref(),
        "managementProfile.iconName",
        64,
    )?;
    if let Some(model) = profile.model.as_deref() {
        validate_standard_id(model, "managementProfile.model", Some("model."))
            .map_err(ApiProblem::from_kernel_error)?;
    }
    validate_profile_suggested_prompts(profile.suggested_prompts.as_deref().unwrap_or_default())?;
    validate_optional_profile_string(
        profile.system_prompt.as_deref(),
        "managementProfile.systemPrompt",
        32768,
    )?;
    if let Some(temperature) = profile.temperature {
        if temperature < 0.0 {
            return Err(ApiProblem::validation(
                "managementProfile.temperature must be greater than or equal to 0",
            ));
        }
        if temperature > 2.0 {
            return Err(ApiProblem::validation(
                "managementProfile.temperature must be less than or equal to 2",
            ));
        }
    }
    if let Some(agent_type) = profile.agent_type.as_deref() {
        if !matches!(agent_type, "normal" | "independent") {
            return Err(ApiProblem::validation(
                "managementProfile.type must be one of normal, independent",
            ));
        }
    }
    validate_optional_profile_string(profile.users.as_deref(), "managementProfile.users", 128)?;
    validate_optional_profile_string(
        profile.welcome_message.as_deref(),
        "managementProfile.welcomeMessage",
        4096,
    )?;
    validate_profile_standard_id_array(
        profile.knowledge_base_ids.as_deref(),
        "managementProfile.knowledgeBaseIds",
        "knowledge.base.",
        128,
    )?;
    validate_profile_standard_id_array(
        profile.skill_ids.as_deref(),
        "managementProfile.skillIds",
        "skill.",
        128,
    )?;
    validate_profile_standard_id_array(
        profile.tool_ids.as_deref(),
        "managementProfile.toolIds",
        "tool.",
        128,
    )?;
    validate_profile_standard_id_array(
        profile.voice_ids.as_deref(),
        "managementProfile.voiceIds",
        "voice.",
        16,
    )?;
    Ok(())
}

fn validate_profile_standard_id_array(
    values: Option<&[String]>,
    field_name: &str,
    prefix: &str,
    max_items: usize,
) -> Result<(), ApiProblem> {
    let Some(values) = values else {
        return Ok(());
    };
    if values.len() > max_items {
        return Err(ApiProblem::validation(format!(
            "{field_name} must contain at most {max_items} items"
        )));
    }
    for value in values {
        if is_trimmed_blank(value) {
            return Err(ApiProblem::validation(format!(
                "{field_name} items is required"
            )));
        }
        if !value.starts_with(prefix) {
            return Err(ApiProblem::validation(format!(
                "{field_name} items must start with {prefix}"
            )));
        }
        validate_standard_id(value, field_name, Some(prefix))
            .map_err(ApiProblem::from_kernel_error)?;
    }
    Ok(())
}

fn validate_optional_profile_string(
    value: Option<&str>,
    field_name: &str,
    max_length: usize,
) -> Result<(), ApiProblem> {
    let Some(value) = value else {
        return Ok(());
    };
    let length = value.chars().count();
    if length == 0 {
        return Err(ApiProblem::validation(format!("{field_name} is required")));
    }
    if length > max_length {
        return Err(ApiProblem::validation(format!(
            "{field_name} must be at most {max_length} characters"
        )));
    }
    Ok(())
}

fn validate_profile_suggested_prompts(values: &[String]) -> Result<(), ApiProblem> {
    if values.len() > 12 {
        return Err(ApiProblem::validation(
            "managementProfile.suggestedPrompts must contain at most 12 items",
        ));
    }
    for value in values {
        let length = value.chars().count();
        if length == 0 {
            return Err(ApiProblem::validation(
                "managementProfile.suggestedPrompts items is required",
            ));
        }
        if length > 256 {
            return Err(ApiProblem::validation(
                "managementProfile.suggestedPrompts items must be at most 256 characters",
            ));
        }
    }
    Ok(())
}

/// Kernel- and axum-rejection-aware constructors for the canonical
/// `crate::response::ApiProblem`. These live here (not in `response.rs`) because
/// they depend on `sdkwork-agent-kernel` and axum extractors that the generic
/// response module must not import.
impl ApiProblem {
    pub(crate) fn from_kernel_error(error: KernelError) -> Self {
        let safe_message = error.safe_message();
        if safe_message.contains("not found") {
            return Self::not_found(safe_message);
        }
        match error.kind() {
            KernelErrorKind::ValidationError => Self::validation(error.safe_message()),
            KernelErrorKind::Conflict => {
                if safe_message.contains("version mismatch") {
                    Self::version_conflict(safe_message)
                } else {
                    Self::conflict(safe_message)
                }
            }
            KernelErrorKind::PermissionRequired | KernelErrorKind::PolicyDenied => {
                Self::permission(error.safe_message())
            }
            _ => Self::internal(error.safe_message()),
        }
    }

    pub(crate) fn from_json_rejection(rejection: JsonRejection) -> Self {
        Self::validation(format!("invalid json request: {}", rejection.body_text()))
    }

    pub(crate) fn from_query_rejection(rejection: QueryRejection) -> Self {
        Self::validation(format!("invalid query request: {}", rejection.body_text()))
    }

    pub(crate) fn from_path_rejection(rejection: PathRejection) -> Self {
        Self::validation(format!("invalid path request: {}", rejection.body_text()))
    }
}

async fn app_list_agents(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    query: Result<Query<AppListAgentsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentRecordResponse>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let query = ListAgentsQueryParams {
            scope: query.scope,
            include_deleted: query.include_deleted,
            q: query.q,
            page: query.page,
            page_size: query.page_size,
        };
        execute_list(state, query, scope, true).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_list_agents(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    query: Result<Query<ListAgentsQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentRecordResponse>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        execute_list(state, query, RequestScope::from_context(context), false).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<CreateAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create(state, RequestScope::from_context(context), body).await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_create_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<CreateAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create(state, RequestScope::from_context(context), body).await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_get_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        execute_get(state, RequestScope::from_context(context), agent_id).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_get_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        execute_get(state, RequestScope::from_context(context), agent_id).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<UpdateAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_update(state, RequestScope::from_context(context), agent_id, body).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_update_agent(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<UpdateAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_update(state, RequestScope::from_context(context), agent_id, body).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_delete_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        execute_delete(state, RequestScope::from_context(context), agent_id).await
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn open_delete_agent(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        execute_delete(state, RequestScope::from_context(context), agent_id).await
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_restore_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<RestoreAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_restore(state, RequestScope::from_context(context), agent_id, body).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_update_agent_status(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<UpdateAgentStatusBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let subject = scope.subject.clone();
        let command = UpdateAgentStatusRequestDto {
            tenant_id: scope.tenant_id,
            agent_id,
            expected_version: body.expected_version,
            target_status: body.target_status,
            requested_at: body.requested_at,
        }
        .into_command(subject)
        .map_err(ApiProblem::from_kernel_error)?;

        let record = with_service(&state, move |service| service.change_status(command)).await?;
        Ok(ResourceData {
            item: map_agent_record(&AgentRecordDto::from_record(&record))?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_restore_agent(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<RestoreAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        execute_restore(state, scope, agent_id, body).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_list_agent_audit_events(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<AuditEventsQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentAuditEventResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = scope.tenant_id_u64()?;
        let subject = scope.subject.clone();
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        validate_audit_action_filter(query.action.as_deref())?;
        validate_audit_range(query.from.as_deref(), query.to.as_deref())?;
        let audit_query = AuditEventListQuery::for_agent(tenant_id, path.agent_id.clone())
            .with_pagination(
                PaginationParams::default()
                    .with_page_size(page_size)
                    .with_page(page),
            );
        let audit_query = if let Some(action) = query.action.clone() {
            audit_query.with_action(action)
        } else {
            audit_query
        };
        let audit_query = if let Some(from) = query.from.clone() {
            audit_query.with_from(from)
        } else {
            audit_query
        };
        let audit_query = if let Some(to) = query.to.clone() {
            audit_query.with_to(to)
        } else {
            audit_query
        };
        let result = with_service(&state, move |service| {
            service.list_agent_audit_events(ListAgentAuditEventsCommand {
                query: audit_query,
                requested_by: subject,
            })
        })
        .await?;
        let total_items = result.total_count.unwrap_or(0) as usize;
        let total_pages = total_pages(total_items, page_size);
        let items: Vec<AgentAuditEventResponse> = result
            .items
            .into_iter()
            .map(|event| AgentAuditEventResponse {
                event_id: event.event_id,
                event_type: event.event_type,
                severity: kernel_event_severity(event.severity).to_string(),
                payload: event.payload,
                occurred_at: event.occurred_at.unwrap_or_default(),
            })
            .collect();

        Ok(PageData {
            items,
            page_info: PageInfo {
                mode: PageMode::Offset,
                page: Some(page as i32),
                page_size: Some(page_size as i32),
                total_items: Some(total_items.to_string()),
                total_pages: Some(total_pages as i32),
                next_cursor: None,
                has_more: Some(result.has_more),
            },
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_provider_bindings(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        execute_list_provider_bindings(
            state,
            RequestScope::from_context(context),
            query.page,
            query.page_size,
            path.agent_id,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_list_provider_bindings(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        execute_list_provider_bindings(
            state,
            RequestScope::from_context(context),
            query.page,
            query.page_size,
            path.agent_id,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_add_provider_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    body: Result<Json<AgentProviderBindingBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_add_provider_binding(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_add_provider_binding(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentProviderBindingBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_add_provider_binding(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_activate_provider_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentBindingPathParams>, PathRejection>,
    body: Result<Json<ActivateProviderBindingBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_activate_provider_binding(state, RequestScope::from_context(context), path, body)
            .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_activate_provider_binding(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentBindingPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ActivateProviderBindingBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_activate_provider_binding(state, RequestScope::from_context(context), path, body)
            .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_preview_response(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    body: Result<Json<AgentPreviewResponseBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRuntimeExecutionRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create_preview_response(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn app_create_prompt_optimization(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    body: Result<Json<AgentPromptOptimizationBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRuntimeExecutionRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create_prompt_optimization(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn open_create_preview_response(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentPreviewResponseBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRuntimeExecutionRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create_preview_response(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn open_create_prompt_optimization(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentPromptOptimizationBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRuntimeExecutionRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create_prompt_optimization(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn app_list_composition_slots(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<ListCompositionSlotsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        execute_list_composition_slots(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            query.page,
            query.page_size,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_code_engines(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<CodeEngineCatalog>> = async {
        let scope = RequestScope::from_context(context);
        let subject = scope.subject().clone();
        let catalog =
            with_service(&state, |service| service.list_code_engine_catalog(subject)).await?;
        Ok(ResourceData { item: catalog })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_mcp_servers(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    query: Result<Query<ListMcpServersQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<McpServerMarketplaceRecord>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = scope.tenant_id_u64()?;
        let subject = scope.subject().clone();
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut list_query = McpMarketplaceListQuery::for_tenant(tenant_id).with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        if let Some(q) = query.q {
            list_query = list_query.with_q(q);
        }
        let result = with_service(&state, move |service| {
            service.list_mcp_marketplace(ListMcpMarketplaceCommand {
                query: list_query,
                requested_by: subject,
            })
        })
        .await?;
        let total_items = result.total_count.unwrap_or(0) as usize;
        Ok(PageData {
            items: result.items,
            page_info: PageInfo {
                mode: PageMode::Offset,
                page: Some(page as i32),
                page_size: Some(page_size as i32),
                total_items: Some(total_items.to_string()),
                total_pages: Some(total_pages(total_items, page_size) as i32),
                next_cursor: None,
                has_more: Some(result.has_more),
            },
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    body: Result<Json<AgentCompositionSlotCreateRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_get_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        execute_get_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            path.slot_id,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
    body: Result<Json<AgentCompositionSlotUpdateRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_update_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            path.slot_id,
            body,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_delete_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        execute_delete_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            path.slot_id,
        )
        .await
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_list_composition_slots(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        execute_list_composition_slots(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            query.page,
            query.page_size,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_create_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentCompositionSlotCreateRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_create_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            body,
        )
        .await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_get_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        execute_get_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            path.slot_id,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_update_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentCompositionSlotUpdateRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        execute_update_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            path.slot_id,
            body,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_delete_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        execute_delete_composition_slot(
            state,
            RequestScope::from_context(context),
            path.agent_id,
            path.slot_id,
        )
        .await
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}
async fn execute_list_composition_slots(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    page: Option<usize>,
    page_size: Option<usize>,
) -> ApiResult<PageData<AgentCompositionSlotRecordResponse>> {
    let (page, page_size) = normalized_pagination(page, page_size)?;
    let tenant_id = scope.tenant_id_u64()?;
    let subject = scope.subject.clone();
    let result = with_service(&state, move |service| {
        service.list_composition_slots(AgentCompositionSlotListCommand {
            query: CompositionSlotListQuery::for_agent(tenant_id, agent_id).with_pagination(
                PaginationParams::default()
                    .with_page_size(page_size)
                    .with_page(page),
            ),
            requested_by: subject,
        })
    })
    .await?;
    let items: Vec<AgentCompositionSlotRecordResponse> = result
        .items
        .iter()
        .map(|record| {
            map_composition_slot_record(&AgentCompositionSlotRecordDto::from_record(record))
        })
        .collect();
    let total_items = result.total_count.unwrap_or(0) as usize;
    Ok(PageData {
        items,
        page_info: PageInfo {
            mode: PageMode::Offset,
            page: Some(page as i32),
            page_size: Some(page_size as i32),
            total_items: Some(total_items.to_string()),
            total_pages: Some(total_pages(total_items, page_size) as i32),
            next_cursor: None,
            has_more: Some(result.has_more),
        },
    })
}

async fn execute_create_composition_slot(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: AgentCompositionSlotCreateRequestDto,
) -> ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> {
    let tenant_id = scope.tenant_id_u64()?;
    let organization_id =
        parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
    validate_requested_at(body.requested_at.as_str()).map_err(ApiProblem::from_kernel_error)?;
    let slot_kind = AgentCompositionSlotKind::try_from_str(body.slot_kind.as_str())
        .ok_or_else(|| ApiProblem::bad_request("invalid slotKind"))?;
    let target_module = AgentCompositionTargetModule::try_from_str(body.target_module.as_str())
        .ok_or_else(|| ApiProblem::bad_request("invalid targetModule"))?;
    let command = AgentCompositionSlotCreateCommand {
        tenant_id,
        organization_id,
        agent_id,
        slot_id: body.slot_id,
        slot_kind,
        target_module,
        target_ref: body.target_ref,
        target_version_ref: body.target_version_ref,
        priority: body.priority.unwrap_or(0),
        enabled: body.enabled.unwrap_or(true),
        policy_json: body.policy_json.unwrap_or_else(|| "{}".to_string()),
        requested_by: scope.subject,
        requested_at: body.requested_at,
    };
    let record = with_service(&state, move |service| {
        service.create_composition_slot(command)
    })
    .await?;
    Ok(ResourceData {
        item: map_composition_slot_record(&AgentCompositionSlotRecordDto::from_record(&record)),
    })
}

async fn execute_get_composition_slot(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    slot_id: String,
) -> ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> {
    let tenant_id = scope.tenant_id_u64()?;
    let command = AgentCompositionSlotGetCommand {
        tenant_id,
        agent_id,
        slot_id,
        requested_by: scope.subject,
    };
    let record = with_service(&state, move |service| service.get_composition_slot(command)).await?;
    Ok(ResourceData {
        item: map_composition_slot_record(&AgentCompositionSlotRecordDto::from_record(&record)),
    })
}

async fn execute_update_composition_slot(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    slot_id: String,
    body: AgentCompositionSlotUpdateRequestDto,
) -> ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> {
    let tenant_id = scope.tenant_id_u64()?;
    validate_requested_at(body.requested_at.as_str()).map_err(ApiProblem::from_kernel_error)?;
    let expected_version = body
        .expected_version
        .as_deref()
        .map(parse_expected_version)
        .transpose()
        .map_err(ApiProblem::from_kernel_error)?;
    let slot_kind = body
        .slot_kind
        .as_deref()
        .map(|value| {
            AgentCompositionSlotKind::try_from_str(value)
                .ok_or_else(|| KernelError::validation("invalid slotKind"))
        })
        .transpose()
        .map_err(ApiProblem::from_kernel_error)?;
    let target_module = body
        .target_module
        .as_deref()
        .map(|value| {
            AgentCompositionTargetModule::try_from_str(value)
                .ok_or_else(|| KernelError::validation("invalid targetModule"))
        })
        .transpose()
        .map_err(ApiProblem::from_kernel_error)?;
    let command = AgentCompositionSlotUpdateCommand {
        tenant_id,
        agent_id,
        slot_id,
        expected_version,
        slot_kind,
        target_module,
        target_ref: body.target_ref,
        target_version_ref: body.target_version_ref,
        priority: body.priority,
        enabled: body.enabled,
        policy_json: body.policy_json,
        requested_by: scope.subject,
        requested_at: body.requested_at,
    };
    let record = with_service(&state, move |service| {
        service.update_composition_slot(command)
    })
    .await?;
    Ok(ResourceData {
        item: map_composition_slot_record(&AgentCompositionSlotRecordDto::from_record(&record)),
    })
}

async fn execute_delete_composition_slot(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    slot_id: String,
) -> ApiResult<()> {
    let tenant_id = scope.tenant_id_u64()?;
    let command = AgentCompositionSlotDeleteCommand {
        tenant_id,
        agent_id,
        slot_id,
        expected_version: None,
        requested_by: scope.subject,
        requested_at: server_requested_at(),
    };
    with_service(&state, move |service| {
        service.delete_composition_slot(command)
    })
    .await?;
    Ok(())
}

fn map_composition_slot_record(
    record: &AgentCompositionSlotRecordDto,
) -> AgentCompositionSlotRecordResponse {
    AgentCompositionSlotRecordResponse {
        id: record.id.clone(),
        tenant_id: record.tenant_id.clone(),
        organization_id: record.organization_id.clone(),
        agent_id: record.agent_id.clone(),
        slot_id: record.slot_id.clone(),
        slot_kind: record.slot_kind.clone(),
        target_module: record.target_module.clone(),
        target_ref: record.target_ref.clone(),
        target_version_ref: record.target_version_ref.clone(),
        priority: record.priority,
        enabled: record.enabled,
        policy_json: record.policy_json.clone(),
        status: record.status.clone(),
        version: record.version.clone(),
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
        deleted_at: record.deleted_at.clone(),
    }
}

// ===========================================================================
// Project handlers - App API
// ===========================================================================

async fn app_ensure_default_workspace(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<AppEnsureDefaultWorkspaceBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentWorkspaceRecordResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = EnsureDefaultWorkspaceCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            owner_user_id,
            default_name: body.name,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| {
            service.ensure_default_workspace(command)
        })
        .await?;
        Ok(ResourceData {
            item: workspace_response(&record),
        })
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn app_create_workspace(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<AppCreateWorkspaceBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentWorkspaceRecordResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = CreateWorkspaceCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            owner_user_id,
            name: body.name,
            description: body.description,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| service.create_workspace(command)).await?;
        Ok(ResourceData {
            item: workspace_response(&record),
        })
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn app_get_workspace(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    workspace_id: Result<Path<String>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentWorkspaceRecordResponse>> = async {
        let Path(workspace_id) = workspace_id.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = GetWorkspaceCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            workspace_id,
            owner_user_id,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_workspace(command)).await?;
        Ok(ResourceData {
            item: workspace_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_workspace(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    workspace_id: Result<Path<String>, PathRejection>,
    body: Result<Json<AppUpdateWorkspaceBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentWorkspaceRecordResponse>> = async {
        let Path(workspace_id) = workspace_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = UpdateWorkspaceCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            workspace_id,
            owner_user_id,
            expected_version: body
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            name: body.name,
            description: body.description,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| service.update_workspace(command)).await?;
        Ok(ResourceData {
            item: workspace_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_archive_workspace(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    workspace_id: Result<Path<String>, PathRejection>,
    body: Result<Json<AppWorkspaceMutationBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentWorkspaceRecordResponse>> = async {
        let Path(workspace_id) = workspace_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = WorkspaceMutationCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            workspace_id,
            owner_user_id,
            expected_version: body
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record =
            with_service(&state, move |service| service.archive_workspace(command)).await?;
        Ok(ResourceData {
            item: workspace_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_delete_workspace(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    workspace_id: Result<Path<String>, PathRejection>,
    query: Result<Query<AppDeleteWorkspaceQuery>, QueryRejection>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path(workspace_id) = workspace_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = WorkspaceMutationCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            workspace_id,
            owner_user_id,
            expected_version: query
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        with_service(&state, move |service| service.delete_workspace(command)).await?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_list_workspaces(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    query: Result<Query<AppListWorkspacesQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentWorkspaceRecordResponse>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut workspace_query = WorkspaceListQuery::for_owner(
            scope.tenant_id_u64()?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            owner_user_id,
        )
        .with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        workspace_query.status = query
            .status
            .as_deref()
            .map(parse_workspace_status)
            .transpose()?;
        workspace_query.include_deleted = query.include_deleted.unwrap_or(false);
        let records = with_service(&state, move |service| {
            service.list_workspaces(ListWorkspacesCommand {
                query: workspace_query,
                requested_by: scope.subject,
            })
        })
        .await?;
        Ok(PageData {
            items: records.items.iter().map(workspace_response).collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_projects(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    query: Result<Query<AppListProjectsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentProjectRecordResponse>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = scope.tenant_id_u64()?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut project_query = ProjectListQuery::for_organization(tenant_id, organization_id)
            .for_owner(owner_user_id)
            .with_pagination(
                PaginationParams::default()
                    .with_page_size(page_size)
                    .with_page(page),
            );
        if let Some(workspace_id) = query.workspace_id {
            project_query = project_query.for_workspace(workspace_id);
        }
        if let Some(search) = query.q {
            project_query = project_query.with_search(search);
        }
        if let Some(status) = query.status {
            project_query = project_query.with_status(parse_project_status(&status)?);
        }
        project_query.include_deleted = query.include_deleted.unwrap_or(false);
        let records = with_service(&state, move |service| {
            service.list_projects(ListProjectsCommand {
                query: project_query,
                requested_by: scope.subject,
            })
        })
        .await?;
        Ok(PageData {
            items: records.items.iter().map(project_response).collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_project(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<AppCreateProjectBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectRecordResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = CreateProjectCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id: body.project_id.unwrap_or_default(),
            workspace_id: body.workspace_id,
            owner_user_id,
            name: body.name,
            description: body.description,
            visibility: parse_project_visibility(body.visibility.as_deref().unwrap_or("private"))?,
            drive_access_mode: parse_project_drive_access(
                body.drive_access_mode.as_deref().unwrap_or("owner_library"),
            )?,
            default_agent_id: body.default_agent_id,
            default_model_id: body.default_model_id,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| service.create_project(command)).await?;
        Ok(ResourceData {
            item: project_response(&record),
        })
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn app_import_project(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    body: Result<Json<AppImportProjectBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectRecordResponse>> = async {
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = ImportProjectCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            workspace_id: body.workspace_id,
            project_id: body.project_id.unwrap_or_default(),
            owner_user_id,
            name: body.name,
            description: body.description,
            source_kind: body.source_kind,
            source_ref: body.source_ref,
            drive_space_id: body.drive_space_id,
            drive_root_entry_id: body.drive_root_entry_id,
            drive_logical_path: body.drive_logical_path.unwrap_or_default(),
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| service.import_project(command)).await?;
        Ok(ResourceData {
            item: project_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_get_project(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectRecordResponse>> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetProjectCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_project(command)).await?;
        Ok(ResourceData {
            item: project_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_project(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
    body: Result<Json<AppUpdateProjectBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectRecordResponse>> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let requested_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = UpdateProjectCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            owner_scope: Some(requested_user_id),
            expected_version: body
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            name: body.name,
            description: body.description,
            visibility: body
                .visibility
                .as_deref()
                .map(parse_project_visibility)
                .transpose()?,
            drive_access_mode: body
                .drive_access_mode
                .as_deref()
                .map(parse_project_drive_access)
                .transpose()?,
            default_agent_id: body.default_agent_id,
            default_model_id: body.default_model_id,
            requested_user_id,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| service.update_project(command)).await?;
        Ok(ResourceData {
            item: project_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_archive_project(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
    body: Result<Json<AppProjectMutationBody>, JsonRejection>,
) -> Response {
    app_mutate_project(state, context, web_ctx, project_id, body, false).await
}

async fn app_delete_project(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let requested_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = ProjectMutationCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            owner_scope: Some(requested_user_id),
            expected_version: None,
            requested_user_id,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        with_service(&state, move |service| service.delete_project(command)).await?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_mutate_project(
    state: AgentHttpState,
    context: AgentRequestContext,
    web_ctx: sdkwork_web_core::WebRequestContext,
    project_id: Result<Path<String>, PathRejection>,
    body: Result<Json<AppProjectMutationBody>, JsonRejection>,
    _delete_project: bool,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectRecordResponse>> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let requested_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = ProjectMutationCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            owner_scope: Some(requested_user_id),
            expected_version: body
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            requested_user_id,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| service.archive_project(command)).await?;
        Ok(ResourceData {
            item: project_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

fn project_response(record: &AgentProjectRecord) -> AgentProjectRecordResponse {
    AgentProjectRecordResponse {
        id: record.id.to_string(),
        project_id: record.project_id.clone(),
        workspace_id: record.workspace_id.clone(),
        tenant_id: record.tenant_id.to_string(),
        organization_id: record.organization_id.to_string(),
        owner_user_id: record.owner_user_id.to_string(),
        name: record.name.clone(),
        description: record.description.clone(),
        visibility: record.visibility.as_str().to_string(),
        status: record.status.as_str().to_string(),
        drive_access_mode: record.drive_access_mode.as_str().to_string(),
        default_agent_id: record.default_agent_id.clone(),
        default_model_id: record.default_model_id.clone(),
        import_source_kind: record.import_source_kind.clone(),
        import_source_ref: record.import_source_ref.clone(),
        drive_space_id: record.drive_space_id.clone(),
        drive_root_entry_id: record.drive_root_entry_id.clone(),
        drive_logical_path: record.drive_logical_path.clone(),
        version: record.version.to_string(),
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
        archived_at: record.archived_at.clone(),
    }
}

fn workspace_response(record: &AgentWorkspaceRecord) -> AgentWorkspaceRecordResponse {
    AgentWorkspaceRecordResponse {
        id: record.id.to_string(),
        workspace_id: record.workspace_id.clone(),
        tenant_id: record.tenant_id.to_string(),
        organization_id: record.organization_id.to_string(),
        owner_user_id: record.owner_user_id.to_string(),
        name: record.name.clone(),
        description: record.description.clone(),
        is_default: record.is_default,
        status: record.status.as_str().to_string(),
        version: record.version.to_string(),
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
        archived_at: record.archived_at.clone(),
    }
}

async fn app_list_project_composition_slots(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
    query: Result<Query<AppListProjectCompositionSlotsQuery>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentProjectCompositionSlotRecordResponse>> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut slot_query = ProjectCompositionSlotListQuery::for_project(
            scope.tenant_id_u64()?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            project_id,
        )
        .with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        slot_query.slot_kind = query
            .slot_kind
            .as_deref()
            .map(parse_project_composition_slot_kind)
            .transpose()?;
        slot_query.enabled = query.enabled;
        let records = with_service(&state, move |service| {
            service.list_project_composition_slots(ListProjectCompositionSlotsCommand {
                query: slot_query,
                owner_scope: Some(owner_user_id),
                requested_by: scope.subject,
            })
        })
        .await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(project_composition_slot_response)
                .collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_project_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    project_id: Result<Path<String>, PathRejection>,
    body: Result<Json<AppCreateProjectCompositionSlotBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectCompositionSlotRecordResponse>> = async {
        let Path(project_id) = project_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let requested_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = CreateProjectCompositionSlotCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            slot_id: body.slot_id,
            slot_kind: parse_project_composition_slot_kind(&body.slot_kind)?,
            target_module: parse_project_composition_target_module(&body.target_module)?,
            target_ref: body.target_ref,
            target_version_ref: body.target_version_ref,
            priority: body.priority.unwrap_or(0),
            enabled: body.enabled.unwrap_or(true),
            policy_json: body.policy_json.unwrap_or_else(|| "{}".to_string()),
            owner_scope: Some(requested_user_id),
            requested_user_id,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| {
            service.create_project_composition_slot(command)
        })
        .await?;
        Ok(ResourceData {
            item: project_composition_slot_response(&record),
        })
    }
    .await;
    finish_created_api_json(&web_ctx, result)
}

async fn app_get_project_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectCompositionSlotRecordResponse>> = async {
        let Path((project_id, slot_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetProjectCompositionSlotCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            slot_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| {
            service.get_project_composition_slot(command)
        })
        .await?;
        Ok(ResourceData {
            item: project_composition_slot_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_project_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<AppUpdateProjectCompositionSlotBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProjectCompositionSlotRecordResponse>> = async {
        let Path((project_id, slot_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        if body.target_version_ref.is_some() && body.clear_target_version_ref.unwrap_or(false) {
            return Err(ApiProblem::validation(
                "targetVersionRef and clearTargetVersionRef cannot be supplied together",
            ));
        }
        let scope = RequestScope::from_context(context);
        let requested_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let target_version_ref = if body.clear_target_version_ref.unwrap_or(false) {
            Some(None)
        } else {
            body.target_version_ref.map(Some)
        };
        let command = UpdateProjectCompositionSlotCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            slot_id,
            expected_version: body
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            slot_kind: body
                .slot_kind
                .as_deref()
                .map(parse_project_composition_slot_kind)
                .transpose()?,
            target_module: body
                .target_module
                .as_deref()
                .map(parse_project_composition_target_module)
                .transpose()?,
            target_ref: body.target_ref,
            target_version_ref,
            priority: body.priority,
            enabled: body.enabled,
            policy_json: body.policy_json,
            owner_scope: Some(requested_user_id),
            requested_user_id,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| {
            service.update_project_composition_slot(command)
        })
        .await?;
        Ok(ResourceData {
            item: project_composition_slot_response(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_delete_project_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppDeleteProjectCompositionSlotQuery>, QueryRejection>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path((project_id, slot_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let requested_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = DeleteProjectCompositionSlotCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            project_id,
            slot_id,
            expected_version: query
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            owner_scope: Some(requested_user_id),
            requested_user_id,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        with_service(&state, move |service| {
            service.delete_project_composition_slot(command)
        })
        .await?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

fn project_composition_slot_response(
    record: &AgentProjectCompositionSlotRecord,
) -> AgentProjectCompositionSlotRecordResponse {
    AgentProjectCompositionSlotRecordResponse {
        id: record.id.to_string(),
        tenant_id: record.tenant_id.to_string(),
        organization_id: record.organization_id.to_string(),
        project_id: record.project_id.clone(),
        slot_id: record.slot_id.clone(),
        slot_kind: record.slot_kind.as_str().to_string(),
        target_module: record.target_module.as_str().to_string(),
        target_ref: record.target_ref.clone(),
        target_version_ref: record.target_version_ref.clone(),
        priority: record.priority,
        enabled: record.enabled,
        policy_json: record.policy_json.clone(),
        created_by: record.created_by.to_string(),
        updated_by: record.updated_by.to_string(),
        version: record.version.to_string(),
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
    }
}

fn parse_project_composition_slot_kind(value: &str) -> ApiResult<AgentCompositionSlotKind> {
    AgentCompositionSlotKind::try_from_str(value)
        .ok_or_else(|| ApiProblem::validation("invalid project composition slot kind"))
}

fn parse_project_composition_target_module(value: &str) -> ApiResult<AgentCompositionTargetModule> {
    AgentCompositionTargetModule::try_from_str(value)
        .ok_or_else(|| ApiProblem::validation("invalid project composition target module"))
}

fn parse_project_status(value: &str) -> ApiResult<AgentProjectStatus> {
    match value {
        "active" => Ok(AgentProjectStatus::Active),
        "archived" => Ok(AgentProjectStatus::Archived),
        "deleted" => Ok(AgentProjectStatus::Deleted),
        _ => Err(ApiProblem::validation("invalid project status")),
    }
}

fn parse_workspace_status(value: &str) -> ApiResult<AgentWorkspaceStatus> {
    match value {
        "active" => Ok(AgentWorkspaceStatus::Active),
        "archived" => Ok(AgentWorkspaceStatus::Archived),
        "deleted" => Ok(AgentWorkspaceStatus::Deleted),
        _ => Err(ApiProblem::validation("invalid workspace status")),
    }
}

fn parse_project_visibility(value: &str) -> ApiResult<AgentProjectVisibility> {
    match value {
        "private" => Ok(AgentProjectVisibility::Private),
        "organization" => Ok(AgentProjectVisibility::Organization),
        "shared" => Ok(AgentProjectVisibility::Shared),
        _ => Err(ApiProblem::validation("invalid project visibility")),
    }
}

fn parse_project_drive_access(value: &str) -> ApiResult<AgentProjectDriveAccessMode> {
    match value {
        "disabled" => Ok(AgentProjectDriveAccessMode::Disabled),
        "owner_library" => Ok(AgentProjectDriveAccessMode::OwnerLibrary),
        "explicit_resources" => Ok(AgentProjectDriveAccessMode::ExplicitResources),
        _ => Err(ApiProblem::validation("invalid project drive access mode")),
    }
}

// ===========================================================================
// Session handlers  - App API
// ===========================================================================

async fn app_list_sessions(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<AppListSessionsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentSessionRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut command = ListSessionsRequestDto {
            tenant_id: scope.tenant_id,
            owner_user_id: Some(scope.owner_user_id),
            status: query.status,
            include_archived: query.include_archived.unwrap_or(false),
        }
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.query = command
            .query
            .for_organization(
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
            )
            .for_agent(agent_id)
            .with_pagination(
                PaginationParams::default()
                    .with_page_size(page_size)
                    .with_page(page),
            );
        if let Some(project_id) = query.project_id {
            command.query = command.query.for_project(project_id);
        }
        let records = with_service(&state, move |service| service.list_sessions(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentSessionRecordDto::from_record)
                .collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_session_user_states(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<AppListSessionUserStatesQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentResourceUserStateRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut state_query = ResourceUserStateListQuery::for_user_sessions(
            scope.tenant_id_u64()?,
            organization_id,
            owner_user_id,
        )
        .for_agent(agent_id.clone())
        .with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        state_query.pinned_only = query.pinned_only.unwrap_or(false);
        state_query.include_hidden = query.include_hidden.unwrap_or(false);
        let records = with_service(&state, move |service| {
            service.list_session_user_states(ListSessionUserStatesCommand {
                query: state_query,
                path_agent_id: agent_id,
                requested_by: scope.subject,
            })
        })
        .await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentResourceUserStateRecordDto::from_record)
                .collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_get_session_user_state(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentResourceUserStateRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = GetSessionUserStateCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            user_id,
            path_agent_id: agent_id,
            session_id,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| {
            service.get_session_user_state(command)
        })
        .await?;
        Ok(ResourceData {
            item: AgentResourceUserStateRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_session_user_state(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<AppUpdateSessionUserStateBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentResourceUserStateRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        if body.custom_title.is_some() && body.clear_custom_title.unwrap_or(false) {
            return Err(ApiProblem::validation(
                "customTitle and clearCustomTitle cannot be supplied together",
            ));
        }
        let custom_title = if body.clear_custom_title.unwrap_or(false) {
            Some(None)
        } else {
            body.custom_title.map(Some)
        };
        let last_read_item_sequence = body
            .last_read_item_sequence
            .map(|value| {
                value.parse::<u64>().map_err(|_| {
                    ApiProblem::validation("lastReadItemSequence must be an unsigned integer")
                })
            })
            .transpose()?;
        let expected_version = body
            .expected_version
            .as_deref()
            .map(parse_expected_version)
            .transpose()
            .map_err(ApiProblem::from_kernel_error)?;
        let scope = RequestScope::from_context(context);
        let user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = UpdateSessionUserStateCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            user_id,
            path_agent_id: agent_id,
            session_id,
            pinned: body.pinned,
            hidden: body.hidden,
            mark_opened: body.mark_opened.unwrap_or(false),
            last_read_item_sequence,
            custom_title,
            expected_version,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| {
            service.update_session_user_state(command)
        })
        .await?;
        Ok(ResourceData {
            item: AgentResourceUserStateRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_item_feedback(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListItemFeedbackQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentItemFeedbackRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let feedback_query = ItemFeedbackListQuery::for_user_session(
            scope.tenant_id_u64()?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            user_id,
            session_id,
        )
        .with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        let records = with_service(&state, move |service| {
            service.list_item_feedback(ListItemFeedbackCommand {
                query: feedback_query,
                path_agent_id: agent_id,
                requested_by: scope.subject,
            })
        })
        .await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentItemFeedbackRecordDto::from_record)
                .collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_item_feedback(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<AppUpdateItemFeedbackBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentItemFeedbackRecordDto>> = async {
        let Path((agent_id, session_id, item_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let clear_feedback = body.clear_feedback.unwrap_or(false);
        if clear_feedback && body.rating.is_some() {
            return Err(ApiProblem::validation(
                "rating and clearFeedback cannot be supplied together",
            ));
        }
        let rating = if clear_feedback {
            None
        } else {
            match body.rating.as_deref() {
                Some("up") => Some(AgentItemFeedbackRating::Up),
                Some("down") => Some(AgentItemFeedbackRating::Down),
                Some(_) => return Err(ApiProblem::validation("rating must be up or down")),
                None => {
                    return Err(ApiProblem::validation(
                        "rating or clearFeedback is required",
                    ))
                }
            }
        };
        let scope = RequestScope::from_context(context);
        let user_id = scope
            .owner_scope()?
            .ok_or_else(|| ApiProblem::validation("owner user id is required"))?;
        let command = UpdateItemFeedbackCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            user_id,
            path_agent_id: agent_id,
            session_id,
            item_id,
            rating,
            reason_code: body.reason_code,
            comment: body.comment,
            expected_version: body
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record =
            with_service(&state, move |service| service.update_item_feedback(command)).await?;
        Ok(ResourceData {
            item: AgentItemFeedbackRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<CreateSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                scope
                    .owner_scope()?
                    .ok_or_else(|| ApiProblem::validation("owner user id is required"))?,
                agent_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.create_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_get_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let command = GetSessionCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            owner_scope,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<AppUpdateSessionBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((_agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        if body.title.is_none() && body.project_id.is_none() && !body.clear_project.unwrap_or(false)
        {
            return Err(ApiProblem::validation(
                "session update requires a changed field",
            ));
        }
        if body.project_id.is_some() && body.clear_project.unwrap_or(false) {
            return Err(ApiProblem::validation(
                "projectId and clearProject cannot be supplied together",
            ));
        }
        let scope = RequestScope::from_context(context);
        let project_id = if body.clear_project.unwrap_or(false) {
            Some(None)
        } else {
            body.project_id.map(Some)
        };
        let command = UpdateSessionCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            session_id,
            title: body.title,
            project_id,
            expected_version: body
                .expected_version
                .as_deref()
                .map(parse_expected_version)
                .transpose()
                .map_err(ApiProblem::from_kernel_error)?,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        let record = with_service(&state, move |service| service.update_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_delete_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path((_agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = DeleteSessionCommand {
            tenant_id: scope.tenant_id_u64()?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            session_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
            requested_at: server_requested_at(),
        };
        with_service(&state, move |service| service.delete_session(command)).await?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => {
            no_content(&web_ctx).unwrap_or_else(|problem| problem.into_response_for(&web_ctx))
        }
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_close_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CloseSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((_agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                session_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record = with_service(&state, move |service| service.close_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Task handlers  - App API
// ===========================================================================

async fn app_list_tasks(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<AppListTasksQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentTaskRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut command = ListTasksRequestDto {
            tenant_id: scope.tenant_id,
            owner_user_id: Some(scope.owner_user_id),
            status: query.status,
        }
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.query = command.query.for_agent(agent_id).with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        let records = with_service(&state, move |service| service.list_tasks(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentTaskRecordDto::from_record)
                .collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_task(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<CreateTaskRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                scope
                    .owner_scope()?
                    .ok_or_else(|| ApiProblem::validation("owner user id is required"))?,
                agent_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.create_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_get_task(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let command = GetTaskCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            task_id,
            owner_scope,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_cancel_task(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CancelTaskRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(scope.tenant_id_u64()?, agent_id, task_id, scope.subject)
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record = with_service(&state, move |service| service.cancel_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_execute_task(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CancelTaskRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_execute_command(scope.tenant_id_u64()?, agent_id, task_id, scope.subject)
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record = with_service(&state, move |service| service.execute_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Interaction handlers  - App API
// ===========================================================================

async fn app_list_interactions(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListInteractionsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut command = ListInteractionsRequestDto {
            tenant_id: scope.tenant_id,
            organization_id: scope.organization_id,
            kind: query.kind,
            status: query.status,
        }
        .into_command(session_id, scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.path_agent_id = agent_id;
        command.owner_scope = owner_scope;
        command.query = command.query.with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        let records =
            with_service(&state, move |service| service.list_interactions(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentInteractionRecordDto::from_record)
                .collect::<KernelResult<Vec<_>>>()
                .map_err(ApiProblem::from_kernel_error)?,
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CreateInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record =
            with_service(&state, move |service| service.create_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_get_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let command = GetInteractionCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            interaction_id,
            owner_scope,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_claim_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<ClaimInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<InteractionClaimResultDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                interaction_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let claim = with_service(&state, move |service| service.claim_interaction(command)).await?;
        Ok(ResourceData {
            item: InteractionClaimResultDto::from_result(&claim)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_approve_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<ApproveInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                interaction_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record =
            with_service(&state, move |service| service.approve_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_answer_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<AnswerInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                interaction_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record =
            with_service(&state, move |service| service.answer_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Session item and turn handlers  - App API
// ===========================================================================

async fn app_list_session_items(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListItemsQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentSessionItemRecordDto>> = async {
        let Path((_agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut command = ListSessionItemsRequestDto {
            tenant_id: scope.tenant_id,
            organization_id: scope.organization_id,
            kind: query.kind,
            status: query.status,
            sort: query.sort,
        }
        .into_command(session_id, scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        command.query = command.query.with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        let records = with_service(&state, move |service| {
            service.list_session_items_with_drive_refs(command)
        })
        .await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(|item| {
                    AgentSessionItemRecordDto::from_record_with_drive_refs(
                        &item.item,
                        &item.drive_refs,
                    )
                    .map_err(ApiProblem::from_kernel_error)
                })
                .collect::<Result<Vec<_>, _>>()?,
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_turns(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentTurnRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let command = ListTurnsCommand {
            query: TurnListQuery::for_session(
                parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                session_id,
            )
            .with_pagination(
                PaginationParams::default()
                    .with_page_size(page_size)
                    .with_page(page),
            ),
            path_agent_id: agent_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
        };
        let records = with_service(&state, move |service| service.list_turns(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentTurnRecordDto::from_record)
                .collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_turn(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppCreateTurnQueryParams>, QueryRejection>,
    body: Result<Json<AppCreateTurnBody>, JsonRejection>,
) -> Response {
    let result: Result<Response, ApiProblem> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        validate_requested_at(body.requested_at.as_str()).map_err(ApiProblem::from_kernel_error)?;
        let drive_refs = body
            .drive_refs
            .into_iter()
            .map(AgentItemDriveRefBody::into_input)
            .collect::<Result<Vec<_>, _>>()?;
        let command = CreateTurnCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            turn_id: body.turn_id,
            content: body.content,
            content_type: body
                .content_type
                .unwrap_or_else(|| "text/plain".to_string()),
            turn_mode: crate::agent_turn::AgentTurnMode::from_code(&body.turn_mode)
                .ok_or_else(|| ApiProblem::validation("invalid turnMode"))?,
            runtime_binding_id: body.runtime_binding_id,
            requested_model_id: body.requested_model_id,
            idempotency_key: body.idempotency_key,
            payload_hash: body.payload_hash,
            client_request_id: body.client_request_id,
            drive_refs,
            owner_scope,
            requested_by: scope.subject,
            requested_at: body.requested_at,
            prefer_stream: query.stream.unwrap_or(false),
        };
        let turn_result =
            with_service(&state, move |service| service.execute_turn(command)).await?;
        turn_execution_http_response(&web_ctx, &turn_result, query.stream.unwrap_or(false))
    }
    .await;
    crate::response::finish_api_response(&web_ctx, result)
}

async fn app_get_session_item(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionItemRecordDto>> = async {
        let Path((agent_id, session_id, item_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let command = GetSessionItemCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            item_id,
            owner_scope,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| {
            service.get_session_item_with_drive_refs(command)
        })
        .await?;
        Ok(ResourceData {
            item: AgentSessionItemRecordDto::from_record_with_drive_refs(
                &record.item,
                &record.drive_refs,
            )
            .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_get_turn(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTurnRecordDto>> = async {
        let Path((agent_id, session_id, turn_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetTurnCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            turn_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_turn(command)).await?;
        Ok(ResourceData {
            item: AgentTurnRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_cancel_turn(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<AppCancelTurnBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTurnRecordDto>> = async {
        let Path((agent_id, session_id, turn_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        validate_requested_at(&body.requested_at).map_err(ApiProblem::from_kernel_error)?;
        let scope = RequestScope::from_context(context);
        let command = CancelTurnCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            turn_id,
            expected_version: Some(
                parse_expected_version(&body.expected_version)
                    .map_err(ApiProblem::from_kernel_error)?,
            ),
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
            requested_at: body.requested_at,
        };
        let record = with_service(&state, move |service| service.cancel_turn(command)).await?;
        Ok(ResourceData {
            item: AgentTurnRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn list_session_checkpoints_data(
    state: &AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    owner_scope: Option<u64>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> ApiResult<PageData<AgentSessionCheckpointRecordDto>> {
    let (page, page_size) = normalized_pagination(page, page_size)?;
    let command = ListSessionCheckpointsCommand {
        query: SessionCheckpointListQuery::for_session(
            parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            session_id,
        )
        .with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        ),
        path_agent_id: agent_id,
        owner_scope,
        requested_by: scope.subject,
    };
    let records = with_service(state, move |service| {
        service.list_session_checkpoints(command)
    })
    .await?;
    Ok(PageData {
        items: records
            .items
            .iter()
            .map(AgentSessionCheckpointRecordDto::from_record)
            .collect(),
        page_info: offset_page_info(
            page,
            page_size,
            records.total_count.unwrap_or(0),
            records.has_more,
        ),
    })
}

async fn create_session_checkpoint_data(
    state: &AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    owner_scope: Option<u64>,
    body: CreateSessionCheckpointRequestDto,
) -> ApiResult<ResourceData<AgentSessionCheckpointRecordDto>> {
    let mut command = body
        .into_command(
            parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            scope.subject,
        )
        .map_err(ApiProblem::from_kernel_error)?;
    command.owner_scope = owner_scope;
    let record = with_service(state, move |service| {
        service.create_session_checkpoint(command)
    })
    .await?;
    Ok(ResourceData {
        item: AgentSessionCheckpointRecordDto::from_record(&record),
    })
}

async fn get_session_checkpoint_data(
    state: &AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    checkpoint_id: String,
    owner_scope: Option<u64>,
) -> ApiResult<ResourceData<AgentSessionCheckpointRecordDto>> {
    let command = GetSessionCheckpointCommand {
        tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
        organization_id: parse_organization_id(&scope.organization_id)
            .map_err(ApiProblem::from_kernel_error)?,
        path_agent_id: agent_id,
        session_id,
        checkpoint_id,
        owner_scope,
        requested_by: scope.subject,
    };
    let record = with_service(state, move |service| {
        service.get_session_checkpoint(command)
    })
    .await?;
    Ok(ResourceData {
        item: AgentSessionCheckpointRecordDto::from_record(&record),
    })
}

enum SessionCheckpointTransition {
    Restore,
    Invalidate,
}

struct ChangeSessionCheckpointInput {
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    checkpoint_id: String,
    owner_scope: Option<u64>,
    body: ChangeSessionCheckpointStatusRequestDto,
    transition: SessionCheckpointTransition,
}

async fn change_session_checkpoint_data(
    state: &AgentHttpState,
    input: ChangeSessionCheckpointInput,
) -> ApiResult<ResourceData<AgentSessionCheckpointRecordDto>> {
    let ChangeSessionCheckpointInput {
        scope,
        agent_id,
        session_id,
        checkpoint_id,
        owner_scope,
        body,
        transition,
    } = input;
    let mut command = body
        .into_command(
            parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            checkpoint_id,
            scope.subject,
        )
        .map_err(ApiProblem::from_kernel_error)?;
    command.owner_scope = owner_scope;
    let record = with_service(state, move |service| match transition {
        SessionCheckpointTransition::Restore => service.restore_session_checkpoint(command),
        SessionCheckpointTransition::Invalidate => service.invalidate_session_checkpoint(command),
    })
    .await?;
    Ok(ResourceData {
        item: AgentSessionCheckpointRecordDto::from_record(&record),
    })
}

async fn list_session_runtime_bindings_data(
    state: &AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    owner_scope: Option<u64>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> ApiResult<PageData<AgentSessionRuntimeBindingRecordDto>> {
    let (page, page_size) = normalized_pagination(page, page_size)?;
    let command = ListSessionRuntimeBindingsCommand {
        query: SessionRuntimeBindingListQuery::for_session(
            parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            session_id,
        )
        .with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        ),
        path_agent_id: agent_id,
        owner_scope,
        requested_by: scope.subject,
    };
    let records = with_service(state, move |service| {
        service.list_session_runtime_bindings(command)
    })
    .await?;
    Ok(PageData {
        items: records
            .items
            .iter()
            .map(AgentSessionRuntimeBindingRecordDto::from_record)
            .collect(),
        page_info: offset_page_info(
            page,
            page_size,
            records.total_count.unwrap_or(0),
            records.has_more,
        ),
    })
}

async fn create_session_runtime_binding_data(
    state: &AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    owner_scope: Option<u64>,
    body: CreateSessionRuntimeBindingRequestDto,
) -> ApiResult<ResourceData<AgentSessionRuntimeBindingRecordDto>> {
    let mut command = body
        .into_command(
            parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            scope.subject,
        )
        .map_err(ApiProblem::from_kernel_error)?;
    command.owner_scope = owner_scope;
    let record = with_service(state, move |service| {
        service.create_session_runtime_binding(command)
    })
    .await?;
    Ok(ResourceData {
        item: AgentSessionRuntimeBindingRecordDto::from_record(&record),
    })
}

async fn get_session_runtime_binding_data(
    state: &AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    runtime_binding_id: String,
    owner_scope: Option<u64>,
) -> ApiResult<ResourceData<AgentSessionRuntimeBindingRecordDto>> {
    let command = GetSessionRuntimeBindingCommand {
        tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
        organization_id: parse_organization_id(&scope.organization_id)
            .map_err(ApiProblem::from_kernel_error)?,
        path_agent_id: agent_id,
        session_id,
        runtime_binding_id,
        owner_scope,
        requested_by: scope.subject,
    };
    let record = with_service(state, move |service| {
        service.get_session_runtime_binding(command)
    })
    .await?;
    Ok(ResourceData {
        item: AgentSessionRuntimeBindingRecordDto::from_record(&record),
    })
}

async fn update_session_runtime_binding_data(
    state: &AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    runtime_binding_id: String,
    owner_scope: Option<u64>,
    body: UpdateSessionRuntimeBindingRequestDto,
) -> ApiResult<ResourceData<AgentSessionRuntimeBindingRecordDto>> {
    let mut command = body
        .into_command(
            parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            runtime_binding_id,
            scope.subject,
        )
        .map_err(ApiProblem::from_kernel_error)?;
    command.owner_scope = owner_scope;
    let record = with_service(state, move |service| {
        service.update_session_runtime_binding(command)
    })
    .await?;
    Ok(ResourceData {
        item: AgentSessionRuntimeBindingRecordDto::from_record(&record),
    })
}

enum SessionRuntimeBindingTransition {
    Activate,
    Deactivate,
}

struct ChangeSessionRuntimeBindingInput {
    scope: RequestScope,
    agent_id: String,
    session_id: String,
    runtime_binding_id: String,
    owner_scope: Option<u64>,
    body: ChangeSessionRuntimeBindingStatusRequestDto,
    transition: SessionRuntimeBindingTransition,
}

async fn change_session_runtime_binding_data(
    state: &AgentHttpState,
    input: ChangeSessionRuntimeBindingInput,
) -> ApiResult<ResourceData<AgentSessionRuntimeBindingRecordDto>> {
    let ChangeSessionRuntimeBindingInput {
        scope,
        agent_id,
        session_id,
        runtime_binding_id,
        owner_scope,
        body,
        transition,
    } = input;
    let mut command = body
        .into_command(
            parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            runtime_binding_id,
            scope.subject,
        )
        .map_err(ApiProblem::from_kernel_error)?;
    command.owner_scope = owner_scope;
    let record = with_service(state, move |service| match transition {
        SessionRuntimeBindingTransition::Activate => {
            service.activate_session_runtime_binding(command)
        }
        SessionRuntimeBindingTransition::Deactivate => {
            service.deactivate_session_runtime_binding(command)
        }
    })
    .await?;
    Ok(ResourceData {
        item: AgentSessionRuntimeBindingRecordDto::from_record(&record),
    })
}

async fn app_list_session_checkpoints(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        list_session_checkpoints_data(
            &state,
            scope,
            agent_id,
            session_id,
            owner_scope,
            query.page,
            query.page_size,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CreateSessionCheckpointRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        create_session_checkpoint_data(&state, scope, agent_id, session_id, owner_scope, body).await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_get_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, checkpoint_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        get_session_checkpoint_data(
            &state,
            scope,
            agent_id,
            session_id,
            checkpoint_id,
            owner_scope,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_restore_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<ChangeSessionCheckpointStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, checkpoint_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        change_session_checkpoint_data(
            &state,
            ChangeSessionCheckpointInput {
                scope,
                agent_id,
                session_id,
                checkpoint_id,
                owner_scope,
                body,
                transition: SessionCheckpointTransition::Restore,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_invalidate_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<ChangeSessionCheckpointStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, checkpoint_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        change_session_checkpoint_data(
            &state,
            ChangeSessionCheckpointInput {
                scope,
                agent_id,
                session_id,
                checkpoint_id,
                owner_scope,
                body,
                transition: SessionCheckpointTransition::Invalidate,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_list_session_runtime_bindings(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        list_session_runtime_bindings_data(
            &state,
            scope,
            agent_id,
            session_id,
            owner_scope,
            query.page,
            query.page_size,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_create_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CreateSessionRuntimeBindingRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        create_session_runtime_binding_data(&state, scope, agent_id, session_id, owner_scope, body)
            .await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn app_get_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        get_session_runtime_binding_data(
            &state,
            scope,
            agent_id,
            session_id,
            runtime_binding_id,
            owner_scope,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_update_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<UpdateSessionRuntimeBindingRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        update_session_runtime_binding_data(
            &state,
            scope,
            agent_id,
            session_id,
            runtime_binding_id,
            owner_scope,
            body,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_activate_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<ChangeSessionRuntimeBindingStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        change_session_runtime_binding_data(
            &state,
            ChangeSessionRuntimeBindingInput {
                scope,
                agent_id,
                session_id,
                runtime_binding_id,
                owner_scope,
                body,
                transition: SessionRuntimeBindingTransition::Activate,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn app_deactivate_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    body: Result<Json<ChangeSessionRuntimeBindingStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        change_session_runtime_binding_data(
            &state,
            ChangeSessionRuntimeBindingInput {
                scope,
                agent_id,
                session_id,
                runtime_binding_id,
                owner_scope,
                body,
                transition: SessionRuntimeBindingTransition::Deactivate,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Session handlers  - Backend API
// ===========================================================================

async fn backend_list_sessions(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<ListSessionsQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentSessionRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut command = ListSessionsRequestDto {
            tenant_id: scope.tenant_id,
            owner_user_id: None,
            status: query.status,
            include_archived: query.include_archived.unwrap_or(false),
        }
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.query = command
            .query
            .for_organization(
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
            )
            .for_agent(agent_id)
            .with_pagination(
                PaginationParams::default()
                    .with_page_size(page_size)
                    .with_page(page),
            );
        let records = with_service(&state, move |service| service.list_sessions(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentSessionRecordDto::from_record)
                .collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_create_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<CreateSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                scope
                    .owner_scope()?
                    .ok_or_else(|| ApiProblem::validation("owner user id is required"))?,
                agent_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.create_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_get_session(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetSessionCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            owner_scope: None,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_close_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CloseSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((_agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                session_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.close_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_archive_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<ArchiveSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((_agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                session_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.archive_session(command)).await?;
        Ok(ResourceData {
            item: AgentSessionRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Session item and turn handlers  - Backend API
// ===========================================================================

async fn backend_list_session_items(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<ListItemsQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentSessionItemRecordDto>> = async {
        let Path((_agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut command = ListSessionItemsRequestDto {
            tenant_id: scope.tenant_id,
            organization_id: scope.organization_id,
            kind: query.kind,
            status: query.status,
            sort: query.sort,
        }
        .into_command(session_id, scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = None;
        command.query = command.query.with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        let records = with_service(&state, move |service| {
            service.list_session_items_with_drive_refs(command)
        })
        .await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(|item| {
                    AgentSessionItemRecordDto::from_record_with_drive_refs(
                        &item.item,
                        &item.drive_refs,
                    )
                    .map_err(ApiProblem::from_kernel_error)
                })
                .collect::<Result<Vec<_>, _>>()?,
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_list_turns(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentTurnRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let command = ListTurnsCommand {
            query: TurnListQuery::for_session(
                parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                session_id,
            )
            .with_pagination(
                PaginationParams::default()
                    .with_page_size(page_size)
                    .with_page(page),
            ),
            path_agent_id: agent_id,
            owner_scope: None,
            requested_by: scope.subject,
        };
        let records = with_service(&state, move |service| service.list_turns(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentTurnRecordDto::from_record)
                .collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_create_turn(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<BackendCreateTurnQueryParams>, QueryRejection>,
    body: Result<Json<CreateTurnBody>, JsonRejection>,
) -> Response {
    let result: Result<Response, ApiProblem> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        validate_requested_at(body.requested_at.as_str()).map_err(ApiProblem::from_kernel_error)?;
        let drive_refs = body
            .drive_refs
            .into_iter()
            .map(AgentItemDriveRefBody::into_input)
            .collect::<Result<Vec<_>, _>>()?;
        let command = CreateTurnCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            turn_id: body.turn_id,
            content: body.content,
            content_type: body
                .content_type
                .unwrap_or_else(|| "text/plain".to_string()),
            turn_mode: crate::agent_turn::AgentTurnMode::from_code(&body.turn_mode)
                .ok_or_else(|| ApiProblem::validation("invalid turnMode"))?,
            runtime_binding_id: body.runtime_binding_id,
            requested_model_id: body.requested_model_id,
            idempotency_key: body.idempotency_key,
            payload_hash: body.payload_hash,
            client_request_id: body.client_request_id,
            drive_refs,
            owner_scope: None,
            requested_by: scope.subject,
            requested_at: body.requested_at,
            prefer_stream: query.stream.unwrap_or(false),
        };
        let turn_result =
            with_service(&state, move |service| service.execute_turn(command)).await?;
        turn_execution_http_response(&web_ctx, &turn_result, query.stream.unwrap_or(false))
    }
    .await;
    crate::response::finish_api_response(&web_ctx, result)
}

async fn backend_get_session_item(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionItemRecordDto>> = async {
        let Path((agent_id, session_id, item_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetSessionItemCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            item_id,
            owner_scope: None,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| {
            service.get_session_item_with_drive_refs(command)
        })
        .await?;
        Ok(ResourceData {
            item: AgentSessionItemRecordDto::from_record_with_drive_refs(
                &record.item,
                &record.drive_refs,
            )
            .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_get_turn(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTurnRecordDto>> = async {
        let Path((agent_id, session_id, turn_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetTurnCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            turn_id,
            owner_scope: None,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_turn(command)).await?;
        Ok(ResourceData {
            item: AgentTurnRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_cancel_turn(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AppCancelTurnBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTurnRecordDto>> = async {
        let Path((agent_id, session_id, turn_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = CancelTurnCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            turn_id,
            expected_version: Some(
                parse_expected_version(&body.expected_version)
                    .map_err(ApiProblem::from_kernel_error)?,
            ),
            owner_scope: None,
            requested_by: scope.subject,
            requested_at: body.requested_at,
        };
        let record = with_service(&state, move |service| service.cancel_turn(command)).await?;
        Ok(ResourceData {
            item: AgentTurnRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_list_session_checkpoints(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        list_session_checkpoints_data(
            &state,
            scope,
            agent_id,
            session_id,
            None,
            query.page,
            query.page_size,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_create_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<CreateSessionCheckpointRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        create_session_checkpoint_data(&state, scope, agent_id, session_id, None, body).await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_get_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, checkpoint_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        get_session_checkpoint_data(&state, scope, agent_id, session_id, checkpoint_id, None).await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_restore_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ChangeSessionCheckpointStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, checkpoint_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        change_session_checkpoint_data(
            &state,
            ChangeSessionCheckpointInput {
                scope,
                agent_id,
                session_id,
                checkpoint_id,
                owner_scope: None,
                body,
                transition: SessionCheckpointTransition::Restore,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_invalidate_session_checkpoint(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ChangeSessionCheckpointStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, checkpoint_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        change_session_checkpoint_data(
            &state,
            ChangeSessionCheckpointInput {
                scope,
                agent_id,
                session_id,
                checkpoint_id,
                owner_scope: None,
                body,
                transition: SessionCheckpointTransition::Invalidate,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_list_session_runtime_bindings(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        list_session_runtime_bindings_data(
            &state,
            scope,
            agent_id,
            session_id,
            None,
            query.page,
            query.page_size,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_create_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<CreateSessionRuntimeBindingRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        create_session_runtime_binding_data(&state, scope, agent_id, session_id, None, body).await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_get_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        get_session_runtime_binding_data(
            &state,
            scope,
            agent_id,
            session_id,
            runtime_binding_id,
            None,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_update_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<UpdateSessionRuntimeBindingRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        update_session_runtime_binding_data(
            &state,
            scope,
            agent_id,
            session_id,
            runtime_binding_id,
            None,
            body,
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_activate_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ChangeSessionRuntimeBindingStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        change_session_runtime_binding_data(
            &state,
            ChangeSessionRuntimeBindingInput {
                scope,
                agent_id,
                session_id,
                runtime_binding_id,
                owner_scope: None,
                body,
                transition: SessionRuntimeBindingTransition::Activate,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_deactivate_session_runtime_binding(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ChangeSessionRuntimeBindingStatusRequestDto>, JsonRejection>,
) -> Response {
    let result = async {
        let Path((agent_id, session_id, runtime_binding_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        change_session_runtime_binding_data(
            &state,
            ChangeSessionRuntimeBindingInput {
                scope,
                agent_id,
                session_id,
                runtime_binding_id,
                owner_scope: None,
                body,
                transition: SessionRuntimeBindingTransition::Deactivate,
            },
        )
        .await
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Task handlers  - Backend API
// ===========================================================================

async fn backend_list_tasks(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<ListTasksQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentTaskRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut command = ListTasksRequestDto {
            tenant_id: scope.tenant_id,
            owner_user_id: Some(scope.owner_user_id),
            status: query.status,
        }
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.query = command.query.for_agent(agent_id).with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        let records = with_service(&state, move |service| service.list_tasks(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentTaskRecordDto::from_record)
                .collect(),
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_create_task(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<CreateTaskRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = body
            .into_command(
                scope.tenant_id_u64()?,
                parse_organization_id(&scope.organization_id)
                    .map_err(ApiProblem::from_kernel_error)?,
                scope
                    .owner_scope()?
                    .ok_or_else(|| ApiProblem::validation("owner user id is required"))?,
                agent_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.create_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_get_task(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetTaskCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            task_id,
            owner_scope: scope.owner_scope()?,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_cancel_task(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<CancelTaskRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(scope.tenant_id_u64()?, agent_id, task_id, scope.subject)
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record = with_service(&state, move |service| service.cancel_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_execute_task(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<CancelTaskRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_execute_command(scope.tenant_id_u64()?, agent_id, task_id, scope.subject)
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record = with_service(&state, move |service| service.execute_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Interaction handlers  - Backend API
// ===========================================================================

async fn backend_list_interactions(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<ListInteractionsQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut command = ListInteractionsRequestDto {
            tenant_id: scope.tenant_id,
            organization_id: scope.organization_id,
            kind: query.kind,
            status: query.status,
        }
        .into_command(session_id, scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.path_agent_id = agent_id;
        command.query = command.query.with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        let records =
            with_service(&state, move |service| service.list_interactions(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentInteractionRecordDto::from_record)
                .collect::<KernelResult<Vec<_>>>()
                .map_err(ApiProblem::from_kernel_error)?,
            page_info: offset_page_info(
                page,
                page_size,
                records.total_count.unwrap_or(0),
                records.has_more,
            ),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_create_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CreateInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record =
            with_service(&state, move |service| service.create_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_get_interaction(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let command = GetInteractionCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            organization_id: parse_organization_id(&scope.organization_id)
                .map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            interaction_id,
            owner_scope: None,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_claim_interaction(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ClaimInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<InteractionClaimResultDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                interaction_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let claim = with_service(&state, move |service| service.claim_interaction(command)).await?;
        Ok(ResourceData {
            item: InteractionClaimResultDto::from_result(&claim)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_approve_interaction(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ApproveInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                interaction_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record =
            with_service(&state, move |service| service.approve_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

async fn backend_answer_interaction(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AnswerInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let tenant_id = parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?;
        let organization_id =
            parse_organization_id(&scope.organization_id).map_err(ApiProblem::from_kernel_error)?;
        let command = body
            .into_command(
                tenant_id,
                organization_id,
                agent_id,
                session_id,
                interaction_id,
                scope.subject,
            )
            .map_err(ApiProblem::from_kernel_error)?;
        let record =
            with_service(&state, move |service| service.answer_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record)
                .map_err(ApiProblem::from_kernel_error)?,
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

/// Invoke a service operation without Mutex locking.
///
/// Since `AgentsService`, `AgentRepository`, and `AgentAuditSink` are all
/// `&self`-based (Send + Sync), there is no need for a global Mutex. This
/// function simply calls the action directly, enabling true concurrent request
/// processing.
pub(crate) async fn with_service<T>(
    state: &AgentHttpState,
    action: impl FnOnce(&HttpService) -> KernelResult<T> + Send + 'static,
) -> Result<T, ApiProblem>
where
    T: Send + 'static,
{
    let permit = SERVICE_WORKER_LIMIT
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            crate::infrastructure::AgentMetricsRegistry::global().record_service_worker_rejection();
            ApiProblem::too_many_requests("agents service concurrency limit reached", Some(1))
        })?;
    let service = Arc::clone(&state.service);
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        action(service.as_ref())
    })
    .await
    .map_err(|error| ApiProblem::internal(format!("agents service worker failed: {error}")))?
    .map_err(ApiProblem::from_kernel_error)
}
async fn execute_list(
    state: AgentHttpState,
    query: ListAgentsQueryParams,
    scope: RequestScope,
    owner_scoped: bool,
) -> ApiResult<PageData<AgentRecordResponse>> {
    let include_deleted = query.include_deleted.unwrap_or(false);
    let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
    let visibility = if matches!(
        query.scope.as_deref(),
        Some("market" | "public" | "published")
    ) {
        Some("public".to_string())
    } else {
        None
    };
    let request_dto = ListAgentsRequestDto {
        tenant_id: scope.tenant_id,
        organization_id: Some(scope.organization_id),
        owner_user_id: if owner_scoped && visibility.is_none() {
            Some(scope.owner_user_id)
        } else {
            None
        },
        include_deleted,
        search_query: query.q,
        visibility,
        pagination: PaginationParams::default()
            .with_page_size(page_size)
            .with_page(page),
    };
    let command = request_dto
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;

    let result = with_service(&state, move |service| service.list_agents(command)).await?;
    let total_items = result.total_count.unwrap_or(0) as usize;
    let total_pages = total_pages(total_items, page_size);

    let items: Vec<AgentRecordResponse> = result
        .items
        .iter()
        .map(|record| map_agent_record(&AgentRecordDto::from_record(record)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(PageData {
        items,
        page_info: PageInfo {
            mode: PageMode::Offset,
            page: Some(page as i32),
            page_size: Some(page_size as i32),
            total_items: Some(total_items.to_string()),
            total_pages: Some(total_pages as i32),
            next_cursor: None,
            has_more: Some(result.has_more),
        },
    })
}

async fn execute_create(
    state: AgentHttpState,
    scope: RequestScope,
    body: CreateAgentBody,
) -> ApiResult<ResourceData<AgentRecordResponse>> {
    let manifest = parse_manifest(body.manifest)?;
    let mut default_code_task_intent = body.default_code_task_intent.map(Into::into);
    if let Some(management_profile) = body.management_profile {
        default_code_task_intent = management_profile
            .into_validated_dto()?
            .merge_into_default_code_task_intent(default_code_task_intent)
            .map_err(ApiProblem::from_kernel_error)?;
    }

    let command = CreateAgentRequestDto {
        agent_id: body.agent_id,
        tenant_id: scope.tenant_id,
        organization_id: scope.organization_id,
        owner_user_id: scope.owner_user_id,
        code: body.code,
        display_name: body.display_name,
        description: body.description,
        manifest,
        visibility: body.visibility,
        tags: body.tags.unwrap_or_default(),
        default_code_task_intent,
        implementation_provider_id: body.implementation_provider_id,
        implementation_kind: body.implementation_kind,
        implementation_type: body.implementation_type,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| service.create_agent(command)).await?;
    Ok(ResourceData {
        item: map_agent_record(&AgentRecordDto::from_record(&record))?,
    })
}

async fn execute_get(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
) -> ApiResult<ResourceData<AgentRecordResponse>> {
    let command = GetAgentRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| service.get_agent(command)).await?;
    Ok(ResourceData {
        item: map_agent_record(&AgentRecordDto::from_record(&record))?,
    })
}

async fn execute_update(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: UpdateAgentBody,
) -> ApiResult<ResourceData<AgentRecordResponse>> {
    let mut default_code_task_intent = body.default_code_task_intent.map(Into::into);
    if let Some(management_profile) = body.management_profile {
        let base_intent = match default_code_task_intent.take() {
            Some(intent) => Some(intent),
            None => {
                let command = GetAgentRequestDto {
                    tenant_id: scope.tenant_id.clone(),
                    agent_id: agent_id.clone(),
                }
                .into_command(scope.subject.clone())
                .map_err(ApiProblem::from_kernel_error)?;
                let current =
                    with_service(&state, move |service| service.get_agent(command)).await?;
                current.default_code_task_intent
            }
        };
        default_code_task_intent = management_profile
            .into_validated_dto()?
            .merge_into_default_code_task_intent(base_intent)
            .map_err(ApiProblem::from_kernel_error)?;
    }
    let command = UpdateAgentRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
        expected_version: body.expected_version,
        display_name: body.display_name,
        description: body.description,
        manifest: body.manifest.map(parse_manifest).transpose()?,
        visibility: body.visibility,
        tags: body.tags,
        default_code_task_intent,
        implementation_provider_id: body.implementation_provider_id,
        implementation_kind: body.implementation_kind,
        implementation_type: body.implementation_type,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| service.update_agent(command)).await?;
    Ok(ResourceData {
        item: map_agent_record(&AgentRecordDto::from_record(&record))?,
    })
}

async fn execute_delete(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
) -> ApiResult<()> {
    let command = DeleteAgentRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
        expected_version: None,
        requested_at: server_requested_at(),
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    with_service(&state, move |service| service.delete_agent(command)).await?;
    Ok(())
}

async fn execute_restore(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: RestoreAgentBody,
) -> ApiResult<ResourceData<AgentRecordResponse>> {
    let command = RestoreAgentRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
        expected_version: body.expected_version,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| service.restore_agent(command)).await?;
    Ok(ResourceData {
        item: map_agent_record(&AgentRecordDto::from_record(&record))?,
    })
}

async fn execute_list_provider_bindings(
    state: AgentHttpState,
    scope: RequestScope,
    page: Option<usize>,
    page_size: Option<usize>,
    agent_id: String,
) -> ApiResult<PageData<AgentProviderBindingRecordResponse>> {
    let (page, page_size) = normalized_pagination(page, page_size)?;
    let tenant_id = scope.tenant_id_u64()?;
    let subject = scope.subject.clone();
    let result = with_service(&state, move |service| {
        service.list_provider_bindings(ProviderBindingListCommand {
            query: ProviderBindingListQuery::for_agent(tenant_id, agent_id).with_pagination(
                PaginationParams::default()
                    .with_page_size(page_size)
                    .with_page(page),
            ),
            requested_by: subject,
        })
    })
    .await?;
    let total_items = result.total_count.unwrap_or(0) as usize;
    let items = result
        .items
        .iter()
        .map(|record| {
            map_provider_binding_record(&AgentProviderBindingRecordDto::from_record(record))
        })
        .collect();
    let total_pages = total_pages(total_items, page_size);

    Ok(PageData {
        items,
        page_info: PageInfo {
            mode: PageMode::Offset,
            page: Some(page as i32),
            page_size: Some(page_size as i32),
            total_items: Some(total_items.to_string()),
            total_pages: Some(total_pages as i32),
            next_cursor: None,
            has_more: Some(result.has_more),
        },
    })
}

async fn execute_add_provider_binding(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: AgentProviderBindingBody,
) -> ApiResult<ResourceData<AgentProviderBindingRecordResponse>> {
    let command = AgentProviderBindingRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
        binding_id: body.binding_id,
        provider_id: body.provider_id,
        implementation_kind: body.implementation_kind,
        configuration_profile_id: body.configuration_profile_id,
        capabilities: body.capabilities.unwrap_or_default(),
        make_default: body.make_default.unwrap_or(false),
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| service.add_provider_binding(command)).await?;
    Ok(ResourceData {
        item: map_provider_binding_record(&AgentProviderBindingRecordDto::from_record(&record)),
    })
}

async fn execute_activate_provider_binding(
    state: AgentHttpState,
    scope: RequestScope,
    path: TenantAgentBindingPathParams,
    body: ActivateProviderBindingBody,
) -> ApiResult<ResourceData<AgentProviderBindingRecordResponse>> {
    let command = ActivateAgentProviderBindingRequestDto {
        tenant_id: scope.tenant_id,
        agent_id: path.agent_id,
        binding_id: path.binding_id,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| {
        service.activate_provider_binding(command)
    })
    .await?;
    Ok(ResourceData {
        item: map_provider_binding_record(&AgentProviderBindingRecordDto::from_record(&record)),
    })
}

async fn execute_create_preview_response(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: AgentPreviewResponseBody,
) -> ApiResult<ResourceData<AgentRuntimeExecutionRecordResponse>> {
    let input_payload_json = json_value_to_string(
        body.input_payload
            .unwrap_or_else(|| json!({ "content": body.content })),
        "inputPayload",
    )?;
    let command = AgentPreviewResponseRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
        execution_id: body.execution_id,
        content: body.content,
        debug_mode: body.debug_mode.unwrap_or(false),
        model: body.model,
        temperature: body.temperature,
        input_payload_json,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| {
        service.create_preview_response(command)
    })
    .await?;
    Ok(ResourceData {
        item: map_runtime_execution_record(&AgentRuntimeExecutionRecordDto::from_record(&record))?,
    })
}

async fn execute_create_prompt_optimization(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: AgentPromptOptimizationBody,
) -> ApiResult<ResourceData<AgentRuntimeExecutionRecordResponse>> {
    let input_payload_json = json_value_to_string(
        body.input_payload
            .unwrap_or_else(|| json!({ "prompt": body.prompt })),
        "inputPayload",
    )?;
    let command = AgentPromptOptimizationRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
        execution_id: body.execution_id,
        prompt: body.prompt,
        input_payload_json,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service(&state, move |service| {
        service.create_prompt_optimization(command)
    })
    .await?;
    Ok(ResourceData {
        item: map_runtime_execution_record(&AgentRuntimeExecutionRecordDto::from_record(&record))?,
    })
}

fn parse_manifest(value: Value) -> Result<AgentManifest, ApiProblem> {
    let json_string = serde_json::to_string(&value)
        .map_err(|error| ApiProblem::validation(format!("manifest json encode failed: {error}")))?;
    AgentManifest::from_json(json_string.as_str()).map_err(ApiProblem::from_kernel_error)
}

fn map_agent_record(record: &AgentRecordDto) -> Result<AgentRecordResponse, ApiProblem> {
    let manifest_value = manifest_to_value(&record.manifest)?;
    let default_code_task_intent = record
        .default_code_task_intent
        .as_ref()
        .map(intent_to_value);

    Ok(AgentRecordResponse {
        id: record.id.clone(),
        agent_id: record.agent_id.clone(),
        tenant_id: record.tenant_id.clone(),
        organization_id: record.organization_id.clone(),
        owner_user_id: record.owner_user_id.clone(),
        code: record.code.clone(),
        display_name: record.display_name.clone(),
        description: record.description.clone(),
        manifest: manifest_value,
        default_code_task_intent,
        management_profile: record
            .management_profile
            .as_ref()
            .map(map_agent_management_profile),
        implementation_provider_id: record.implementation_provider_id.clone(),
        implementation_kind: record.implementation_kind.clone(),
        implementation_type: record.implementation_type.clone(),
        status: record.status.clone(),
        visibility: record.visibility.clone(),
        tags: record.tags.clone(),
        version: record.version.clone(),
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
        deleted_at: record.deleted_at.clone(),
    })
}

fn map_agent_management_profile(
    profile: &AgentManagementProfileDto,
) -> AgentManagementProfileResponse {
    AgentManagementProfileResponse {
        author: profile.author.clone(),
        avatar: profile.avatar.clone(),
        category_id: profile.category_id.clone(),
        color: profile.color.clone(),
        debug_mode: profile.debug_mode,
        icon_name: profile.icon_name.clone(),
        json_mode: profile.json_mode,
        knowledge_base_ids: profile.knowledge_base_ids.clone(),
        memory_enabled: profile.memory_enabled,
        model: profile.model.clone(),
        skill_ids: profile.skill_ids.clone(),
        suggested_prompts: profile.suggested_prompts.clone(),
        system_prompt: profile.system_prompt.clone(),
        temperature: profile.temperature,
        tool_ids: profile.tool_ids.clone(),
        agent_type: profile.agent_type.clone(),
        users: profile.users.clone(),
        voice_ids: profile.voice_ids.clone(),
        welcome_message: profile.welcome_message.clone(),
    }
}

fn map_provider_binding_record(
    record: &AgentProviderBindingRecordDto,
) -> AgentProviderBindingRecordResponse {
    AgentProviderBindingRecordResponse {
        tenant_id: record.tenant_id.clone(),
        agent_id: record.agent_id.clone(),
        binding_id: record.binding_id.clone(),
        provider_id: record.provider_id.clone(),
        implementation_kind: record.implementation_kind.clone(),
        configuration_profile_id: record.configuration_profile_id.clone(),
        capabilities: record.capabilities.clone(),
        active: record.active,
        version: record.version.clone(),
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
    }
}

fn map_runtime_execution_record(
    record: &AgentRuntimeExecutionRecordDto,
) -> Result<AgentRuntimeExecutionRecordResponse, ApiProblem> {
    Ok(AgentRuntimeExecutionRecordResponse {
        tenant_id: record.tenant_id.clone(),
        agent_id: record.agent_id.clone(),
        execution_id: record.execution_id.clone(),
        operation: record.operation.clone(),
        status: record.status.clone(),
        input_payload: json_string_to_value(record.input_payload_json.as_str(), "inputPayload")?,
        output_payload: json_string_to_value(record.output_payload_json.as_str(), "outputPayload")?,
        requested_at: record.requested_at.clone(),
        completed_at: record.completed_at.clone(),
    })
}

fn manifest_to_value(manifest: &AgentManifest) -> Result<Value, ApiProblem> {
    let value = json!({
        "schema_version": manifest.schema_version,
        "manifest_type": manifest.manifest_type,
        "agent_id": manifest.agent_id,
        "name": manifest.name,
        "display_name": manifest.display_name,
        "description": manifest.description,
        "version": manifest.version,
        "domain": manifest.domain,
        "required_capabilities": manifest.required_capabilities,
        "optional_capabilities": manifest.optional_capabilities,
        "event_families": manifest.event_families,
        "owner": {
            "name": manifest.owner_name,
        },
        "status": manifest.status,
    });
    serde_json::from_value(value)
        .map_err(|error| ApiProblem::internal(format!("manifest json decode failed: {error}")))
}

fn intent_to_value(intent: &CodeTaskIntent) -> Value {
    json!({
        "prompt": intent.prompt,
        "contextPaths": intent.context_paths,
        "constraints": intent.constraints,
    })
}

fn json_value_to_string(value: Value, field_name: &str) -> Result<String, ApiProblem> {
    serde_json::to_string(&value).map_err(|error| {
        ApiProblem::validation(format!("{field_name} json encode failed: {error}"))
    })
}

fn json_string_to_value(value: &str, field_name: &str) -> Result<Value, ApiProblem> {
    serde_json::from_str(value)
        .map_err(|error| ApiProblem::internal(format!("{field_name} json decode failed: {error}")))
}

fn kernel_event_severity(severity: sdkwork_agent_kernel::KernelEventSeverity) -> &'static str {
    match severity {
        sdkwork_agent_kernel::KernelEventSeverity::Debug => "debug",
        sdkwork_agent_kernel::KernelEventSeverity::Info => "info",
        sdkwork_agent_kernel::KernelEventSeverity::Warn => "warn",
        sdkwork_agent_kernel::KernelEventSeverity::Error => "error",
    }
}

fn validate_audit_action_filter(action: Option<&str>) -> Result<(), ApiProblem> {
    let Some(action) = action else {
        return Ok(());
    };
    if !ALLOWED_AUDIT_ACTIONS.contains(&action) {
        return Err(ApiProblem::validation(format!(
            "action must be one of {}",
            ALLOWED_AUDIT_ACTIONS.join(", ")
        )));
    }
    Ok(())
}

fn validate_audit_range(from: Option<&str>, to: Option<&str>) -> Result<(), ApiProblem> {
    let from = parse_optional_query_datetime("from", from)?;
    let to = parse_optional_query_datetime("to", to)?;
    if let (Some(from_value), Some(to_value)) = (from.as_ref(), to.as_ref()) {
        if from_value > to_value {
            return Err(ApiProblem::validation(
                "from must be less than or equal to to",
            ));
        }
    }
    Ok(())
}

fn parse_optional_query_datetime(
    field_name: &str,
    value: Option<&str>,
) -> Result<Option<OffsetDateTime>, ApiProblem> {
    parse_optional_rfc3339_datetime(value, field_name).map_err(ApiProblem::from_kernel_error)
}

fn server_requested_at() -> String {
    format_utc_seconds(OffsetDateTime::now_utc())
}

fn format_utc_seconds(value: OffsetDateTime) -> String {
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        value.year(),
        u8::from(value.month()),
        value.day(),
        value.hour(),
        value.minute(),
        value.second()
    )
}

fn env_usize(key: &str, default: usize, minimum: usize, maximum: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| (minimum..=maximum).contains(value))
        .unwrap_or(default)
}

pub(crate) fn normalized_pagination(
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<(usize, usize), ApiProblem> {
    if page == Some(0) {
        return Err(ApiProblem::validation(
            "page must be greater than or equal to 1",
        ));
    }
    if page_size == Some(0) {
        return Err(ApiProblem::validation(
            "page_size must be greater than or equal to 1",
        ));
    }
    if let Some(size) = page_size {
        if size > MAX_PAGE_SIZE {
            return Err(ApiProblem::validation(format!(
                "page_size must be less than or equal to {MAX_PAGE_SIZE}"
            )));
        }
    }

    let params = sdkwork_utils_rust::http_api::OffsetListPageParams::parse(
        page.map(|value| value as i64),
        page_size.map(|value| value as i64),
    );
    Ok((params.page as usize, params.page_size as usize))
}

/// Build the durable turn execution response.
/// Non-streaming returns `200 OK` with the SDKWork response envelope.
/// Streaming returns ordered delta events followed by a completion envelope.
fn turn_execution_http_response(
    ctx: &sdkwork_web_core::WebRequestContext,
    result: &crate::application::TurnExecutionResult,
    stream_requested: bool,
) -> Result<Response, ApiProblem> {
    let execution =
        AgentTurnExecutionDto::from_result(result).map_err(ApiProblem::from_kernel_error)?;
    let trace_id = ctx.resolved_trace_id();

    if stream_requested {
        let envelope = sdkwork_utils_rust::SdkWorkApiResponse::success(
            ResourceData { item: execution },
            trace_id.clone(),
        );
        let completion_payload = serde_json::to_string(&json!({
            "eventType": "completion",
            "response": envelope,
        }))
        .map_err(|error| {
            ApiProblem::internal(format!("failed to encode turn completion: {error}"))
        })?;
        let mut body = String::new();
        for (index, delta) in result.stream_deltas.iter().enumerate() {
            let payload = serde_json::to_string(&json!({
                "eventType": "delta",
                "index": index,
                "delta": delta,
            }))
            .map_err(|error| {
                ApiProblem::internal(format!("failed to encode turn delta: {error}"))
            })?;
            body.push_str("event: delta\n");
            body.push_str("data: ");
            body.push_str(&payload);
            body.push_str("\n\n");
        }
        body.push_str("event: completion\n");
        body.push_str("data: ");
        body.push_str(&completion_payload);
        body.push_str("\n\n");
        let mut response = Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/event-stream")
            .body(Body::from(body))
            .map_err(|error| {
                ApiProblem::internal(format!("failed to build SSE response: {error}"))
            })?;
        if let Ok(value) = HeaderValue::from_str(&trace_id) {
            response
                .headers_mut()
                .insert(HeaderName::from_static("x-sdkwork-trace-id"), value);
        }
        return Ok(response);
    }

    success_json(ctx, ResourceData { item: execution })
}

fn total_pages(total_items: usize, page_size: usize) -> usize {
    if total_items == 0 {
        0
    } else {
        total_items.div_ceil(page_size)
    }
}

fn offset_page_info(page: usize, page_size: usize, total_count: u64, has_more: bool) -> PageInfo {
    let params = sdkwork_utils_rust::http_api::OffsetListPageParams {
        page: page as i64,
        page_size: page_size as i64,
        offset: ((page as i64) - 1) * (page_size as i64),
    };
    let mut info = sdkwork_utils_rust::http_api::offset_list_page_info(total_count as i64, params);
    info.has_more = Some(has_more);
    info
}

#[cfg(test)]
mod tests {
    use super::testing::test_web_context;
    use super::*;
    use crate::infrastructure::{
        IamGatedPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
    };
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use axum::Extension;
    use tower::ServiceExt;

    fn test_manifest() -> Value {
        json!({
            "schema_version": "1.0.0",
            "manifest_type": "agent",
            "agent_id": "agent.alpha",
            "name": "sample-agent",
            "display_name": "Sample Agent",
            "description": "sample",
            "version": "0.1.0",
            "domain": "intelligence",
            "required_capabilities": ["model.chat"],
            "optional_capabilities": ["tool.invoke"],
            "event_families": ["agent.lifecycle"],
            "owner": { "name": "sdkwork" },
            "status": "active"
        })
    }

    fn test_policy_provider() -> IamGatedPolicyProvider {
        IamGatedPolicyProvider::new("policy.agents.test.iam-gated")
    }

    fn build_test_router(state: AgentHttpState) -> axum::Router {
        build_test_router_with_context(state, test_agent_context())
    }

    fn build_test_router_with_context(
        state: AgentHttpState,
        context: AgentRequestContext,
    ) -> axum::Router {
        build_combined_routes()
            .with_state(state)
            .layer(Extension(context))
            .layer(Extension(test_web_context()))
    }

    fn create_agent_body(agent_id: &str, code: &str) -> Value {
        json!({
            "agentId": agent_id,
            "code": code,
            "displayName": code,
            "description": "contract test agent",
            "manifest": test_manifest(),
            "visibility": "organization",
            "requestedAt": "2026-06-01T00:00:00Z"
        })
    }

    async fn create_app_agent(app: &axum::Router, agent_id: &str, code: &str) {
        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(create_agent_body(agent_id, code).to_string()))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("create request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    async fn create_app_provider_binding(app: &axum::Router, agent_id: &str) {
        let binding_body = json!({
            "bindingId": "binding.agent-provider.codex",
            "providerId": "provider.model.codex",
            "implementationKind": "process-adapter",
            "configurationProfileId": "profile.codex.default",
            "capabilities": ["agent.runtime.preview", "agent.runtime.prompt_optimization"],
            "makeDefault": true,
            "requestedAt": "2026-06-01T00:01:00Z"
        });
        let request = Request::builder()
            .method("POST")
            .uri(format!(
                "/app/v3/api/ai/agents/{agent_id}/provider_bindings"
            ))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(binding_body.to_string()))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("provider binding request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    fn test_agent_context() -> AgentRequestContext {
        AgentRequestContext::new("100001", "100")
            .with_organization_id("0")
            .with_subject_id("100")
            .with_roles(["ai.agents.manage"])
            .with_trace_id("trace-test-fixed")
            .with_request_id("req-test-fixed")
    }

    fn facade_actor() -> sdkwork_agents_runtime_facade::AgentsSessionActor {
        sdkwork_agents_runtime_facade::AgentsSessionActor {
            subject_id: "user:100".to_string(),
            roles: vec!["ai.agents.manage".to_string()],
        }
    }

    fn facade_runtime_binding(
        runtime_binding_id: &str,
    ) -> sdkwork_agents_runtime_facade::AgentsSessionRuntimeBindingDescriptor {
        sdkwork_agents_runtime_facade::AgentsSessionRuntimeBindingDescriptor {
            runtime_binding_id: runtime_binding_id.to_string(),
            runtime_location_id: Some("birdcoder-workspace-001".to_string()),
            host_mode: "desktop".to_string(),
            transport_kind: "process".to_string(),
            provider_binding_id: "binding.agent-provider.codex".to_string(),
            model_id: "model.gpt-5".to_string(),
            provider_id: "provider.model.codex".to_string(),
            native_session_id: Some(format!("native-{runtime_binding_id}")),
            native_session_tree_id: Some("native-tree-001".to_string()),
            native_parent_session_id: Some("native-parent-001".to_string()),
            native_forked_from_session_id: Some("native-origin-001".to_string()),
        }
    }

    fn facade_session_request(
        session_id: &str,
        runtime_binding_id: Option<&str>,
    ) -> sdkwork_agents_runtime_facade::ResolveAgentsSessionRequest {
        sdkwork_agents_runtime_facade::ResolveAgentsSessionRequest {
            tenant_id: 100_001,
            organization_id: 0,
            owner_user_id: 100,
            agent_id: "agent.facade".to_string(),
            session_id: session_id.to_string(),
            project_id: None,
            session_kind: sdkwork_agents_runtime_facade::AgentsSessionKind::Coding,
            entry_surface: sdkwork_agents_runtime_facade::AgentsSessionEntrySurface::Pc,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: format!("Facade session {session_id}"),
            idempotency_key: format!("create-{session_id}"),
            payload_hash: format!("sha256:create-{session_id}"),
            runtime_binding: runtime_binding_id.map(facade_runtime_binding),
            actor: facade_actor(),
            requested_at: "2026-07-22T12:00:00Z".to_string(),
        }
    }

    #[derive(Clone)]
    struct SessionRetrieveFailurePolicy;

    impl PolicyProvider for SessionRetrieveFailurePolicy {
        fn provider_manifest(&self) -> ProviderManifest {
            ProviderManifest::new(
                "policy.agents.test.session-retrieve-failure",
                "policy",
                "session-retrieve-failure-policy",
                "0.1.0",
                vec!["policy.evaluate".to_string()],
            )
        }

        fn evaluate(&self, request: PolicyRequest) -> KernelResult<PolicyDecision> {
            if request.action.as_deref() == Some("session.retrieve") {
                return Err(KernelError::Internal {
                    message: "injected session retrieval failure".to_string(),
                });
            }
            Ok(PolicyDecision::allow(
                format!("decision.{}", request.policy_request_id),
                request.policy_request_id,
                "policy.agents.test.session-retrieve-failure",
            ))
        }

        fn health(&self) -> ProviderHealth {
            ProviderHealth::available()
        }
    }

    #[tokio::test]
    async fn session_facade_persists_canonical_session_lineage_and_runtime_binding() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state.clone());
        create_app_agent(&app, "agent.facade", "facade").await;
        create_app_provider_binding(&app, "agent.facade").await;

        let subject = sdkwork_agent_kernel::PolicySubject::new("user:100", "100001")
            .with_role("ai.agents.manage");
        state
            .service
            .create_project(CreateProjectCommand {
                tenant_id: 100_001,
                organization_id: 0,
                project_id: "project.facade".to_string(),
                workspace_id: None,
                owner_user_id: 100,
                name: "Facade project".to_string(),
                description: Some("Canonical session ownership".to_string()),
                visibility: AgentProjectVisibility::Private,
                drive_access_mode: AgentProjectDriveAccessMode::OwnerLibrary,
                default_agent_id: Some("agent.facade".to_string()),
                default_model_id: Some("model.gpt-5".to_string()),
                requested_by: subject.clone(),
                requested_at: "2026-07-22T11:59:00Z".to_string(),
            })
            .expect("facade project should be created");

        let facade = state.session_facade();
        let parent_request = facade_session_request(
            "session.facade.parent",
            Some("runtime_binding.facade.parent"),
        );
        facade
            .resolve_or_create_session(parent_request)
            .expect("parent session should resolve");
        let parent_turn_facade = facade.clone();
        let parent_turn = tokio::task::spawn_blocking(move || {
            parent_turn_facade.complete_turn(
                sdkwork_agents_runtime_facade::CompleteAgentsTurnRequest {
                    tenant_id: 100_001,
                    organization_id: 0,
                    owner_user_id: 100,
                    agent_id: "agent.facade".to_string(),
                    session_id: "session.facade.parent".to_string(),
                    content: "Create a fork point".to_string(),
                    content_type: "text/plain".to_string(),
                    idempotency_key: "turn.facade.parent".to_string(),
                    client_request_id: "request.facade.parent".to_string(),
                    actor: facade_actor(),
                    requested_at: "2026-07-22T12:00:01Z".to_string(),
                },
            )
        })
        .await
        .expect("parent turn worker should complete")
        .expect("parent turn should complete");

        let mut child_request =
            facade_session_request("session.facade.child", Some("runtime_binding.facade.child"));
        child_request.project_id = Some("project.facade".to_string());
        child_request.source_module = Some("birdcoder".to_string());
        child_request.source_context_kind = Some("coding_project".to_string());
        child_request.source_context_id = Some("workspace-001".to_string());
        child_request.parent_session_id = Some("session.facade.parent".to_string());
        child_request.forked_from_turn_id = Some(parent_turn.turn_id.clone());

        let created = facade
            .resolve_or_create_session(child_request.clone())
            .expect("child session should resolve");
        assert!(created.created);

        let stored = state
            .service
            .get_session(GetSessionCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: "agent.facade".to_string(),
                session_id: "session.facade.child".to_string(),
                owner_scope: Some(100),
                requested_by: subject.clone(),
            })
            .expect("child session should be persisted");
        assert_eq!(stored.project_id.as_deref(), Some("project.facade"));
        assert_eq!(stored.session_kind, AgentSessionKind::Coding);
        assert_eq!(stored.entry_surface, AgentSessionEntrySurface::Pc);
        assert_eq!(stored.source_module.as_deref(), Some("birdcoder"));
        assert_eq!(
            stored.source_context_kind.as_deref(),
            Some("coding_project")
        );
        assert_eq!(stored.source_context_id.as_deref(), Some("workspace-001"));
        assert_eq!(
            stored.parent_session_id.as_deref(),
            Some("session.facade.parent")
        );
        assert_eq!(
            stored.forked_from_turn_id.as_deref(),
            Some(parent_turn.turn_id.as_str())
        );

        let binding = state
            .service
            .get_session_runtime_binding(GetSessionRuntimeBindingCommand {
                tenant_id: 100_001,
                organization_id: 0,
                path_agent_id: "agent.facade".to_string(),
                session_id: "session.facade.child".to_string(),
                runtime_binding_id: "runtime_binding.facade.child".to_string(),
                owner_scope: Some(100),
                requested_by: subject,
            })
            .expect("child runtime binding should be persisted");
        let descriptor = child_request
            .runtime_binding
            .as_ref()
            .expect("child request has a runtime binding");
        assert!(runtime_binding_matches_descriptor(&binding, descriptor));

        let replay = facade
            .resolve_or_create_session(child_request.clone())
            .expect("same descriptor should resolve idempotently");
        assert!(!replay.created);
        assert_eq!(replay.session_id, created.session_id);

        child_request
            .runtime_binding
            .as_mut()
            .expect("child request has a runtime binding")
            .model_id = "model.conflict".to_string();
        let error = facade
            .resolve_or_create_session(child_request)
            .expect_err("conflicting descriptor must be rejected");
        assert!(matches!(
            error,
            sdkwork_agents_runtime_facade::RuntimeFacadeError::InvalidInput(message)
                if message.contains("runtime binding descriptor conflicts")
        ));
    }

    #[tokio::test]
    async fn session_facade_turn_requires_current_runtime_binding() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state.clone());
        create_app_agent(&app, "agent.facade", "facade-no-binding").await;
        let facade = state.session_facade();
        facade
            .resolve_or_create_session(facade_session_request("session.facade.unbound", None))
            .expect("session without a runtime binding should still resolve");

        let error = facade
            .complete_turn(sdkwork_agents_runtime_facade::CompleteAgentsTurnRequest {
                tenant_id: 100_001,
                organization_id: 0,
                owner_user_id: 100,
                agent_id: "agent.facade".to_string(),
                session_id: "session.facade.unbound".to_string(),
                content: "This must not fall back to an agent binding".to_string(),
                content_type: "text/plain".to_string(),
                idempotency_key: "turn.facade.unbound".to_string(),
                client_request_id: "request.facade.unbound".to_string(),
                actor: facade_actor(),
                requested_at: "2026-07-22T12:01:00Z".to_string(),
            })
            .expect_err("turn without a current runtime binding must fail");
        assert!(matches!(
            error,
            sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(message)
                if message.contains("active session runtime binding not found")
        ));
    }

    #[tokio::test]
    async fn session_facade_does_not_create_after_non_not_found_read_error() {
        let audit_sink = InMemoryAgentAuditSink::default();
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            audit_sink.clone(),
            SessionRetrieveFailurePolicy,
        );
        let app = build_test_router(state.clone());
        create_app_agent(&app, "agent.facade", "facade-read-error").await;
        let audit_count_before = audit_sink.events().len();

        let error = state
            .session_facade()
            .resolve_or_create_session(facade_session_request("session.facade.read-error", None))
            .expect_err("non-not-found read errors must fail closed");
        assert!(matches!(
            error,
            sdkwork_agents_runtime_facade::RuntimeFacadeError::Handler(message)
                if message.contains("injected session retrieval failure")
        ));
        assert_eq!(audit_sink.events().len(), audit_count_before);
    }

    #[tokio::test]
    async fn app_create_and_retrieve_agent_should_work() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);

        let create_body = json!({
            "agentId": "agent.alpha",
            "code": "alpha",
            "displayName": "Alpha",
            "description": "first",
            "manifest": test_manifest(),
            "defaultCodeTaskIntent": {
                "prompt": "Refactor runtime",
                "contextPaths": ["src/lib.rs"],
                "constraints": ["safe"]
            },
            "visibility": "organization",
            "tags": ["starter"],
            "requestedAt": "2026-06-01T00:00:00Z"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(create_body.to_string()))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("create request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);

        let request = Request::builder()
            .method("GET")
            .uri("/app/v3/api/ai/agents/agent.alpha")
            .body(Body::empty())
            .expect("request should be built");
        let response = app
            .oneshot(request)
            .await
            .expect("get request should succeed");
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let body_json: Value =
            serde_json::from_slice(&body_bytes).expect("response body should be valid json");
        // 信封形状：{ code: 0, data: { item: { agentId, ... } }, traceId }
        assert_eq!(body_json["code"], 0);
        assert_eq!(body_json["data"]["item"]["agentId"], "agent.alpha");
    }

    #[tokio::test]
    async fn open_api_create_and_retrieve_agent_should_work() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);

        let mut manifest = test_manifest();
        manifest["agent_id"] = json!("agent.open");
        let create_body = json!({
            "agentId": "agent.open",
            "code": "open",
            "displayName": "Open Agent",
            "description": "developer api",
            "manifest": manifest,
            "visibility": "organization",
            "tags": ["developer"],
            "requestedAt": "2026-06-01T00:00:00Z"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/agent/v3/api/ai/agents")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(create_body.to_string()))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("create request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);

        let request = Request::builder()
            .method("GET")
            .uri("/agent/v3/api/ai/agents/agent.open")
            .body(Body::empty())
            .expect("request should be built");
        let response = app
            .oneshot(request)
            .await
            .expect("get request should succeed");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn app_delete_agent_uses_204_without_json_body() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);

        create_app_agent(&app, "agent.delete", "delete").await;

        let request = Request::builder()
            .method("DELETE")
            .uri("/app/v3/api/ai/agents/agent.delete")
            .body(Body::empty())
            .expect("request should be built");
        let response = app
            .oneshot(request)
            .await
            .expect("delete request should succeed");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let body_bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        assert!(body_bytes.is_empty());
    }

    #[tokio::test]
    async fn app_runtime_create_operations_use_201_status() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);

        create_app_agent(&app, "agent.runtime", "runtime").await;
        create_app_provider_binding(&app, "agent.runtime").await;

        let preview_body = json!({
            "executionId": "execution.preview",
            "content": "Summarize the repository state",
            "debugMode": false,
            "model": "codex",
            "temperature": 0.2,
            "requestedAt": "2026-06-01T00:02:00Z"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents/agent.runtime/preview_responses")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(preview_body.to_string()))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("preview request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);

        let optimization_body = json!({
            "executionId": "execution.prompt",
            "prompt": "Make this prompt concise",
            "requestedAt": "2026-06-01T00:03:00Z"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents/agent.runtime/prompt_optimizations")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(optimization_body.to_string()))
            .expect("request should be built");
        let response = app
            .oneshot(request)
            .await
            .expect("prompt optimization request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn app_delete_composition_slot_uses_204_without_json_body() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);

        create_app_agent(&app, "agent.slot-delete", "slot-delete").await;
        let slot_body = json!({
            "slotId": "slot.skill.primary",
            "slotKind": "skill",
            "targetModule": "skills",
            "targetRef": "skill.primary",
            "priority": 10,
            "enabled": true,
            "policyJson": "{}",
            "requestedAt": "2026-06-01T00:04:00Z"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents/agent.slot-delete/composition_slots")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(slot_body.to_string()))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("composition slot create request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);

        let request = Request::builder()
            .method("DELETE")
            .uri("/app/v3/api/ai/agents/agent.slot-delete/composition_slots/slot.skill.primary?requestedAt=2026-06-01T00:05:00Z")
            .body(Body::empty())
            .expect("request should be built");
        let response = app
            .oneshot(request)
            .await
            .expect("composition slot delete request should succeed");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let body_bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        assert!(body_bytes.is_empty());
    }

    #[tokio::test]
    async fn backend_status_update_should_work() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state);

        let create_body = json!({
            "agentId": "agent.beta",
            "code": "beta",
            "displayName": "Beta",
            "description": null,
            "manifest": {
                "schema_version": "1.0.0",
                "manifest_type": "agent",
                "agent_id": "agent.beta",
                "name": "sample-agent",
                "display_name": "Sample Agent",
                "description": "sample",
                "version": "0.1.0",
                "domain": "intelligence",
                "required_capabilities": ["model.chat"],
                "optional_capabilities": ["tool.invoke"],
                "event_families": ["agent.lifecycle"],
                "owner": { "name": "sdkwork" },
                "status": "active"
            },
            "visibility": "private",
            "requestedAt": "2026-06-01T01:00:00Z"
        });

        let create_request = Request::builder()
            .method("POST")
            .uri("/backend/v3/api/ai/agents")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(create_body.to_string()))
            .expect("request should be built");
        let create_response = app
            .clone()
            .oneshot(create_request)
            .await
            .expect("create request should succeed");
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let update_status_body = json!({
            "targetStatus": "active",
            "expectedVersion": "1",
            "requestedAt": "2026-06-01T01:05:00Z"
        });

        let status_request = Request::builder()
            .method("POST")
            .uri("/backend/v3/api/ai/agents/agent.beta/status")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(update_status_body.to_string()))
            .expect("request should be built");
        let status_response = app
            .oneshot(status_request)
            .await
            .expect("status request should succeed");

        assert_eq!(status_response.status(), StatusCode::OK);
        let body_bytes = to_bytes(status_response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let body_json: Value =
            serde_json::from_slice(&body_bytes).expect("response body should be valid json");
        // 信封形状：{ code: 0, data: { item: { status, ... } }, traceId }
        assert_eq!(body_json["code"], 0);
        assert_eq!(body_json["data"]["item"]["status"], "active");
    }

    #[tokio::test]
    async fn app_project_and_code_engine_lists_allow_read_only_subjects() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let read_context = AgentRequestContext::new("100001", "100")
            .with_organization_id("0")
            .with_subject_id("100")
            .with_roles(["ai.agents.read"]);
        let app = build_test_router_with_context(state, read_context);

        for uri in [
            "/app/v3/api/ai/code_engines",
            "/app/v3/api/ai/projects?page=1&page_size=20",
        ] {
            let request = Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .expect("request should be built");
            let response = app
                .clone()
                .oneshot(request)
                .await
                .expect("list request should succeed");
            assert_eq!(response.status(), StatusCode::OK, "GET {uri}");
        }
    }

    #[tokio::test]
    async fn app_user_can_create_project_with_use_permission() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app_user_context = AgentRequestContext::new("100001", "100")
            .with_organization_id("0")
            .with_subject_id("100")
            .with_roles(["ai.agents.read", "ai.agents.use"]);
        let app = build_test_router_with_context(state, app_user_context);
        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/projects")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "projectId": "project.app-user",
                    "name": "BirdCoder project"
                })
                .to_string(),
            ))
            .expect("request should be built");

        let response = app
            .oneshot(request)
            .await
            .expect("project create request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn read_only_subject_cannot_create_project() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let read_context = AgentRequestContext::new("100001", "100")
            .with_organization_id("0")
            .with_subject_id("100")
            .with_roles(["ai.agents.read"]);
        let app = build_test_router_with_context(state, read_context);
        let request = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/projects")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "name": "Denied project" }).to_string()))
            .expect("request should be built");

        let response = app
            .oneshot(request)
            .await
            .expect("project create request should complete");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn request_scope_rejects_zero_tenant_context() {
        let scope = RequestScope::from_context(AgentRequestContext::new("0", "100"));
        let err = scope
            .tenant_id_u64()
            .expect_err("tenant_id 0 must be rejected at the service scope boundary");
        assert!(err.message.contains("greater than 0"));
    }

    #[tokio::test]
    async fn app_session_user_state_routes_use_trusted_scope_and_optimistic_version() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            test_policy_provider(),
        );
        let app = build_test_router(state.clone());
        create_app_agent(&app, "agent.alpha", "alpha").await;

        let create_session = Request::builder()
            .method("POST")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "sessionId": "session.user-state",
                    "sessionKind": "assistant",
                    "entrySurface": "api",
                    "title": "User state contract",
                    "idempotencyKey": "create-session.user-state",
                    "payloadHash": "sha256:create-session.user-state",
                    "requestedAt": "2026-07-19T00:00:00Z"
                })
                .to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(create_session).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        with_service(&state, |service| {
            service.create_session_item(crate::application::CreateSessionItemCommand {
                tenant_id: 100_001,
                organization_id: 0,
                session_id: "session.user-state".to_string(),
                item_id: "item.assistant".to_string(),
                kind: crate::domain::AgentSessionItemKind::AssistantOutput,
                content: "Assistant answer".to_string(),
                content_type: "text/plain".to_string(),
                input_tokens: 0,
                output_tokens: 2,
                model_id: None,
                provider_id: None,
                parent_item_id: None,
                requested_by: sdkwork_agent_kernel::PolicySubject::new("u-1", "100001")
                    .with_role("ai.agents.manage"),
                requested_at: "2026-07-19T00:00:01Z".to_string(),
            })
        })
        .await
        .expect("assistant item fixture should be created");

        let pin = Request::builder()
            .method("PATCH")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/session.user-state/user_state")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "pinned": true }).to_string()))
            .unwrap();
        let response = app.clone().oneshot(pin).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["item"]["resourceId"], "session.user-state");
        assert_eq!(payload["data"]["item"]["version"], "0");
        assert!(payload["data"]["item"]["pinnedAt"].is_string());
        assert!(payload["data"]["item"].get("hiddenAt").is_none());

        let list = Request::builder()
            .method("GET")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/user_states?pinned_only=true&page=1")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(list).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["items"].as_array().unwrap().len(), 1);

        let missing_version = Request::builder()
            .method("PATCH")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/session.user-state/user_state")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "pinned": false }).to_string()))
            .unwrap();
        let response = app.clone().oneshot(missing_version).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let unpin = Request::builder()
            .method("PATCH")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/session.user-state/user_state")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({ "pinned": false, "expectedVersion": "0" }).to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(unpin).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["item"]["version"], "1");
        assert!(payload["data"]["item"].get("pinnedAt").is_none());

        let feedback = Request::builder()
            .method("PATCH")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/session.user-state/items/item.assistant/feedback")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "rating": "up" }).to_string()))
            .unwrap();
        let response = app.clone().oneshot(feedback).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["item"]["rating"], "up");
        assert_eq!(payload["data"]["item"]["version"], "0");

        let feedback_list = Request::builder()
            .method("GET")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/session.user-state/item_feedback")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(feedback_list).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["items"].as_array().unwrap().len(), 1);

        let clear_feedback = Request::builder()
            .method("PATCH")
            .uri("/app/v3/api/ai/agents/agent.alpha/sessions/session.user-state/items/item.assistant/feedback")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({ "clearFeedback": true, "expectedVersion": "0" }).to_string(),
            ))
            .unwrap();
        let response = app.clone().oneshot(clear_feedback).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["data"]["item"]["version"], "1");
        assert!(payload["data"]["item"]["deletedAt"].is_string());
    }
}
