mod agent_turn;
mod api;
mod application;
mod code_engine_catalog;
mod domain;
mod dto;
#[cfg(feature = "http-axum")]
mod http;
mod id;
mod in_memory_pagination;
mod infrastructure;
mod mcp_marketplace;
#[cfg(feature = "http-axum")]
mod provider_session_sync;
mod persistence;
mod ports;
#[cfg(feature = "postgres-sync")]
mod postgres_sync_pool;
mod project;
#[cfg(feature = "http-axum")]
pub mod response;
mod runtime_facade_bridge;
mod session_activity;
mod turn_runtime;
mod validation;
mod workspace;

pub use agent_turn::{AgentTurnMode, AgentTurnRecord, AgentTurnStatus};
pub use api::{
    ApiOperation, AGENT_APP_API_OPERATIONS, AGENT_APP_API_PREFIX, AGENT_BACKEND_API_OPERATIONS,
    AGENT_BACKEND_API_PREFIX, AGENT_OPEN_API_OPERATIONS, AGENT_OPEN_API_PREFIX,
};
pub use application::{
    ActivateAgentProviderBindingCommand, AgentCompositionSlotCreateCommand,
    AgentCompositionSlotDeleteCommand, AgentCompositionSlotGetCommand,
    AgentCompositionSlotListCommand, AgentCompositionSlotUpdateCommand, AgentItemDriveRefInput,
    AgentPreviewResponseCommand, AgentPromptOptimizationCommand, AgentProviderBindingCommand,
    AgentSessionItemWithDriveRefs, AgentsService, AnswerInteractionCommand,
    ApproveInteractionCommand, ArchiveSessionCommand, CancelTurnCommand, ChangeAgentStatusCommand,
    ChangeSessionCheckpointStatusCommand, ChangeSessionRuntimeBindingStatusCommand,
    ClaimInteractionCommand, CloseSessionCommand, CreateAgentCommand, CreateInteractionCommand,
    CreateProjectCommand, CreateProjectCompositionSlotCommand, CreateSessionCheckpointCommand,
    CreateSessionCommand, CreateSessionItemCommand, CreateSessionRuntimeBindingCommand,
    CreateTurnCommand, CreateWorkspaceCommand, DeleteAgentCommand,
    DeleteProjectCompositionSlotCommand, EnsureDefaultWorkspaceCommand, GetAgentCommand,
    GetInteractionCommand, GetProjectCommand, GetProjectCompositionSlotCommand,
    GetSessionCheckpointCommand, GetSessionCommand, GetSessionItemCommand,
    GetSessionRuntimeBindingCommand, GetSessionUserStateCommand, GetTurnByIdempotencyCommand,
    GetTurnCommand, GetWorkspaceCommand, ImportProjectCommand, InteractionClaimResult,
    ItemFeedbackResult, ListAgentAuditEventsCommand, ListAgentsCommand, ListInteractionsCommand,
    ListItemFeedbackCommand, ListMcpMarketplaceCommand, ListProjectCompositionSlotsCommand,
    ListProjectsCommand, ListSessionCheckpointsCommand, ListSessionItemsCommand,
    ListSessionRuntimeBindingsCommand, ListSessionUserStatesCommand, ListSessionsCommand,
    ListTurnsCommand, ListWorkspacesCommand, ProjectMutationCommand, ProviderBindingListCommand,
    RestoreAgentCommand, SessionCheckpointResult, SessionRuntimeBindingResult,
    SessionUserStateResult, TurnExecutionResult, TurnReconciliationResult, UpdateAgentCommand,
    UpdateItemFeedbackCommand, UpdateProjectCommand, UpdateProjectCompositionSlotCommand,
    UpdateSessionRuntimeBindingCommand, UpdateSessionUserStateCommand, UpdateWorkspaceCommand,
    WorkspaceMutationCommand,
};
pub use sdkwork_intelligence_prompts_ai_contract::{
    AgentPromptTemplateKind, AgentPromptTemplateRecord, PromptAiRepository,
};
pub use turn_runtime::{
    complete_with_timeout, execute_agent_turn, is_inference_error, ContractTurnExecutor,
    KernelModelTurnExecutor, RuntimeFacadeTurnExecutor, TurnExecutionInput, TurnExecutionOutput,
    TurnExecutor, RUNTIME_MODE_FACADE, RUNTIME_MODE_INFERENCE_ERROR, TURN_EXECUTION_TIMEOUT,
};

pub use domain::{
    AgentAuditAction, AgentBusinessRecord, AgentBusinessStatus, AgentCompositionSlotKind,
    AgentCompositionSlotRecord, AgentCompositionTargetModule, AgentImplementationKind,
    AgentImplementationType, AgentInteractionKind, AgentInteractionRecord, AgentInteractionStatus,
    AgentItemDriveRefRecord, AgentItemFeedbackRating, AgentItemFeedbackRecord,
    AgentItemResourceRole, AgentProviderBindingRecord, AgentResourceType,
    AgentResourceUserStateRecord, AgentRuntimeExecutionOperation, AgentRuntimeExecutionRecord,
    AgentRuntimeExecutionStatus, AgentSessionCheckpointRecord, AgentSessionCheckpointStatus,
    AgentSessionEntrySurface, AgentSessionItemKind, AgentSessionItemRecord, AgentSessionItemStatus,
    AgentSessionKind, AgentSessionRecord, AgentSessionRuntimeBindingRecord,
    AgentSessionRuntimeBindingStatus, AgentSessionStatus, AgentVisibility,
    DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY,
};
pub use dto::{
    ActivateAgentProviderBindingRequestDto, AgentCompositionSlotCreateRequestDto,
    AgentCompositionSlotDeleteRequestDto, AgentCompositionSlotListResponseDto,
    AgentCompositionSlotRecordDto, AgentCompositionSlotResponseDto,
    AgentCompositionSlotUpdateRequestDto, AgentItemFeedbackRecordDto, AgentListResponseDto,
    AgentManagementProfileDto, AgentPreviewResponseRequestDto, AgentPromptOptimizationRequestDto,
    AgentProviderBindingListResponseDto, AgentProviderBindingRecordDto,
    AgentProviderBindingRequestDto, AgentProviderBindingResponseDto, AgentRecordDto,
    AgentResourceUserStateRecordDto, AgentResponseDto, AgentRuntimeExecutionRecordDto,
    AgentRuntimeExecutionResponseDto, AgentSessionItemListResponseDto, AgentSessionItemRecordDto,
    AgentSessionItemResponseDto, AgentSessionListResponseDto, AgentSessionRecordDto,
    AgentSessionResponseDto, ArchiveSessionRequestDto, CloseSessionRequestDto,
    CreateAgentRequestDto, CreateSessionItemRequestDto, CreateSessionRequestDto,
    DeleteAgentRequestDto, GetAgentRequestDto, ListAgentsRequestDto, ListSessionItemsRequestDto,
    ListSessionsRequestDto, RestoreAgentRequestDto, UpdateAgentRequestDto,
    UpdateAgentStatusRequestDto,
};
#[cfg(feature = "http-axum")]
pub use http::testing;
#[cfg(feature = "http-axum")]
pub use http::{
    build_app_routes, build_backend_routes, build_combined_routes, build_open_routes,
    serve_agents_metrics, AgentHttpState, AgentRequestContext,
};
pub use id::{AgentBusinessIdGenerator, AgentIdGenerator, AUDIT_SINK_NODE_ID};
pub use infrastructure::{
    is_production_environment, validate_production_security_config, AgentMetricsRegistry,
    AgentServiceMetrics, AllowAllPolicyProvider, DenyAllPolicyProvider, IamGatedPolicyProvider,
    InMemoryAgentAuditSink, InMemoryAgentRepository, PolicyMode, ENV_DEPLOYMENT_ENV,
    ENV_DEV_AUTH_BYPASS, IAM_PERMISSION_AGENTS_MANAGE, IAM_PERMISSION_AGENTS_READ,
};
pub use persistence::{
    extract_event_context, AgentAuditAdapter, AgentRepositoryAdapter, SqlAgentAuditSink,
    SqlAgentRepository, SQL_COUNT_AGENT, SQL_COUNT_AGENT_COMPOSITION_SLOTS,
    SQL_COUNT_AGENT_PROVIDER_BINDINGS, SQL_COUNT_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID,
    SQL_COUNT_MCP_MARKETPLACE_SLOTS, SQL_INSERT_AGENT, SQL_INSERT_AGENT_COMPOSITION_SLOT,
    SQL_INSERT_AGENT_PROVIDER_BINDING, SQL_INSERT_AUDIT_EVENT, SQL_LIST_AGENT,
    SQL_LIST_AGENT_COMPOSITION_SLOTS, SQL_LIST_AGENT_PROVIDER_BINDINGS,
    SQL_LIST_MCP_MARKETPLACE_SLOTS, SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID,
    SQL_SELECT_AGENT_COMPOSITION_SLOT, SQL_SELECT_AGENT_PROVIDER_BINDING, SQL_UPDATE_AGENT,
    SQL_UPDATE_AGENT_COMPOSITION_SLOT, SQL_UPDATE_AGENT_PROVIDER_BINDING,
};
#[cfg(feature = "postgres-sync")]
pub use persistence::{SyncPostgresAdapter, AGENTS_DATABASE_SERVICE};
#[cfg(feature = "postgres-sync")]
pub use persistence::{
    SQL_COMPLETE_AGENT_TURN_STATE, SQL_COUNT_AGENT_INTERACTIONS, SQL_COUNT_AGENT_ITEM_FEEDBACK,
    SQL_COUNT_AGENT_PROJECTS, SQL_COUNT_AGENT_PROJECT_COMPOSITION_SLOTS,
    SQL_COUNT_AGENT_RESOURCE_USER_STATES, SQL_COUNT_AGENT_SESSIONS, SQL_COUNT_AGENT_SESSION_ITEMS,
    SQL_COUNT_AGENT_TASKS, SQL_INSERT_AGENT_INTERACTION, SQL_INSERT_AGENT_ITEM_DRIVE_REF,
    SQL_INSERT_AGENT_PROJECT, SQL_INSERT_AGENT_PROJECT_COMPOSITION_SLOT, SQL_INSERT_AGENT_SESSION,
    SQL_INSERT_AGENT_SESSION_ITEM, SQL_INSERT_AGENT_TASK, SQL_INSERT_AGENT_TURN,
    SQL_LIST_AGENT_INTERACTIONS, SQL_LIST_AGENT_ITEM_DRIVE_REFS,
    SQL_LIST_AGENT_ITEM_DRIVE_REFS_BATCH, SQL_LIST_AGENT_ITEM_FEEDBACK, SQL_LIST_AGENT_PROJECTS,
    SQL_LIST_AGENT_PROJECT_COMPOSITION_SLOTS, SQL_LIST_AGENT_RESOURCE_USER_STATES,
    SQL_LIST_AGENT_SESSIONS, SQL_LIST_AGENT_SESSION_ACTIVITY_HEADS, SQL_LIST_AGENT_SESSION_ITEMS,
    SQL_LIST_AGENT_SESSION_ITEMS_DESC, SQL_LIST_AGENT_SESSION_ITEMS_RECENT_CONTEXT,
    SQL_LIST_AGENT_TASKS, SQL_LIST_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID,
    SQL_RECORD_AGENT_SESSION_ITEM, SQL_SELECT_AGENT_INTERACTION, SQL_SELECT_AGENT_ITEM_FEEDBACK,
    SQL_SELECT_AGENT_PROJECT, SQL_SELECT_AGENT_PROJECT_BY_WORKSPACE_NAME,
    SQL_SELECT_AGENT_PROJECT_COMPOSITION_SLOT, SQL_SELECT_AGENT_RESOURCE_USER_STATE,
    SQL_SELECT_AGENT_SESSION, SQL_SELECT_AGENT_SESSION_ITEM, SQL_SELECT_AGENT_TASK,
    SQL_SELECT_AGENT_TURN, SQL_SELECT_AGENT_TURN_BY_IDEMPOTENCY, SQL_UPDATE_AGENT_INTERACTION,
    SQL_UPDATE_AGENT_PROJECT, SQL_UPDATE_AGENT_PROJECT_COMPOSITION_SLOT, SQL_UPDATE_AGENT_SESSION,
    SQL_UPDATE_AGENT_SESSION_ITEM, SQL_UPDATE_AGENT_TASK, SQL_UPDATE_AGENT_TURN_STATE,
    SQL_UPDATE_AGENT_WORKSPACE, SQL_UPSERT_AGENT_ITEM_FEEDBACK,
    SQL_UPSERT_AGENT_RESOURCE_USER_STATE,
};
pub use ports::WorkspaceListQuery;
pub use ports::{
    offset_paginated_result, AgentAuditSink, AgentListQuery, AgentRepository, AuditEventListQuery,
    CompositionSlotListQuery, InteractionListQuery, ItemFeedbackListQuery, McpMarketplaceListQuery,
    PaginatedResult, PaginationParams, ProjectCompositionSlotListQuery, ProjectListQuery,
    ProviderBindingListQuery, ResourceUserStateListQuery, SessionActivitySummaryListQuery,
    SessionItemListQuery, SessionItemListSort, SessionListQuery, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE,
    MAX_TURN_INPUT_CONTENT_BYTES, TURN_CONTEXT_ITEM_LIMIT,
};
pub use ports::{SessionCheckpointListQuery, SessionRuntimeBindingListQuery, TurnListQuery};
pub use project::{
    AgentProjectCompositionSlotRecord, AgentProjectDriveAccessMode, AgentProjectRecord,
    AgentProjectStatus, AgentProjectVisibility,
};
pub use session_activity::{
    SessionActivityCursor, SessionActivityFreshness, SessionActivitySource,
    SessionActivitySummaryRecord, SessionProviderActivityEvidenceKind,
    SessionProviderActivityFreshness, SessionProviderActivityInteractionHint,
    SessionProviderActivityObservation, SessionProviderActivityState, SessionPresentationPhase,
    SessionProviderIdentity,
};
pub use workspace::{default_workspace_id, AgentWorkspaceRecord, AgentWorkspaceStatus};
