mod api;
mod application;
mod turn_runtime;
mod agent_turn;
mod code_engine_catalog;
mod domain;
mod dto;
#[cfg(feature = "http-axum")]
mod http;
mod id;
mod in_memory_pagination;
mod infrastructure;
mod mcp_marketplace;
mod persistence;
mod ports;
#[cfg(feature = "postgres-sync")]
mod postgres_sync_pool;
mod project;
#[cfg(feature = "http-axum")]
pub mod response;
mod runtime_facade_bridge;
#[cfg(feature = "sqlite-sync")]
mod sqlite_sync_pool;
mod validation;

pub use api::{
    ApiOperation, AGENT_APP_API_OPERATIONS, AGENT_APP_API_PREFIX, AGENT_BACKEND_API_OPERATIONS,
    AGENT_BACKEND_API_PREFIX, AGENT_OPEN_API_OPERATIONS, AGENT_OPEN_API_PREFIX,
};
pub use application::{
    ActivateAgentProviderBindingCommand, AgentCompositionSlotCreateCommand,
    AgentCompositionSlotDeleteCommand, AgentCompositionSlotGetCommand,
    AgentCompositionSlotListCommand, AgentCompositionSlotUpdateCommand,
    AgentItemDriveRefInput, AgentSessionItemWithDriveRefs, AgentPreviewResponseCommand,
    AgentPromptOptimizationCommand, AgentProviderBindingCommand, AgentsService,
    AnswerInteractionCommand, ApproveInteractionCommand, ArchiveSessionCommand,
    ChangeSessionCheckpointStatusCommand, ChangeSessionRuntimeBindingStatusCommand,
    ClaimInteractionCommand,
    CancelTurnCommand, ChangeAgentStatusCommand, TurnExecutionResult,
    TurnReconciliationResult, CloseSessionCommand, CreateAgentCommand,
    CreateInteractionCommand, CreateSessionCheckpointCommand, CreateSessionItemCommand,
    CreateSessionRuntimeBindingCommand, CreateProjectCommand,
    CreateProjectCompositionSlotCommand, CreateSessionCommand, DeleteAgentCommand,
    DeleteProjectCompositionSlotCommand, GetAgentCommand, GetTurnByIdempotencyCommand,
    GetTurnCommand, GetInteractionCommand, GetSessionCheckpointCommand, GetSessionItemCommand,
    GetSessionRuntimeBindingCommand, GetProjectCommand,
    GetProjectCompositionSlotCommand, GetSessionCommand, GetSessionUserStateCommand,
    InteractionClaimResult, ListAgentAuditEventsCommand, ListAgentsCommand,
    ListInteractionsCommand, ListSessionCheckpointsCommand,
    ListMcpMarketplaceCommand, ListItemFeedbackCommand, ListSessionItemsCommand,
    ListProjectCompositionSlotsCommand, ListProjectsCommand, ListSessionUserStatesCommand,
    ListSessionRuntimeBindingsCommand, ListSessionsCommand, ListTurnsCommand,
    ItemFeedbackResult, ProjectMutationCommand, ProviderBindingListCommand,
    RestoreAgentCommand, CreateTurnCommand, SessionCheckpointResult,
    SessionRuntimeBindingResult, SessionUserStateResult, UpdateAgentCommand,
    UpdateItemFeedbackCommand, UpdateProjectCommand, UpdateProjectCompositionSlotCommand,
    UpdateSessionRuntimeBindingCommand, UpdateSessionUserStateCommand,
};
pub use turn_runtime::{
    execute_agent_turn, complete_with_timeout, is_inference_error, TurnExecutor,
    TurnExecutionInput, TurnExecutionOutput, ContractTurnExecutor, KernelModelTurnExecutor,
    RuntimeFacadeTurnExecutor, TURN_EXECUTION_TIMEOUT, RUNTIME_MODE_FACADE,
    RUNTIME_MODE_INFERENCE_ERROR,
};
pub use agent_turn::{AgentTurnMode, AgentTurnRecord, AgentTurnStatus};
pub use sdkwork_intelligence_prompts_ai_contract::{
    AgentPromptTemplateKind, AgentPromptTemplateRecord, PromptAiRepository,
};

pub use domain::{
    AgentAuditAction, AgentBusinessRecord, AgentBusinessStatus, AgentCompositionSlotKind,
    AgentCompositionSlotRecord, AgentCompositionTargetModule, AgentImplementationKind,
    AgentImplementationType, AgentInteractionKind, AgentInteractionRecord, AgentInteractionStatus,
    AgentItemDriveRefRecord, AgentItemFeedbackRating, AgentItemFeedbackRecord,
    AgentItemResourceRole, AgentProviderBindingRecord, AgentResourceType, AgentResourceUserStateRecord,
    AgentRuntimeExecutionOperation, AgentRuntimeExecutionRecord, AgentRuntimeExecutionStatus,
    AgentSessionCheckpointRecord, AgentSessionCheckpointStatus,
    AgentSessionEntrySurface, AgentSessionItemKind, AgentSessionItemRecord,
    AgentSessionItemStatus, AgentSessionKind, AgentSessionRecord,
    AgentSessionRuntimeBindingRecord, AgentSessionRuntimeBindingStatus, AgentSessionStatus,
    AgentVisibility,
    DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY,
};
pub use dto::{
    ActivateAgentProviderBindingRequestDto, AgentCompositionSlotCreateRequestDto,
    AgentCompositionSlotDeleteRequestDto, AgentCompositionSlotListResponseDto,
    AgentCompositionSlotRecordDto, AgentCompositionSlotResponseDto,
    AgentCompositionSlotUpdateRequestDto, AgentListResponseDto, AgentManagementProfileDto,
    AgentItemFeedbackRecordDto, AgentPreviewResponseRequestDto, AgentPromptOptimizationRequestDto,
    AgentSessionItemListResponseDto, AgentSessionItemRecordDto, AgentSessionItemResponseDto,
    AgentProviderBindingListResponseDto, AgentProviderBindingRecordDto,
    AgentProviderBindingRequestDto, AgentProviderBindingResponseDto, AgentRecordDto,
    AgentResourceUserStateRecordDto, AgentResponseDto, AgentRuntimeExecutionRecordDto,
    AgentRuntimeExecutionResponseDto, AgentSessionListResponseDto, AgentSessionRecordDto,
    AgentSessionResponseDto, ArchiveSessionRequestDto, CloseSessionRequestDto,
    CreateAgentRequestDto, CreateSessionItemRequestDto, CreateSessionRequestDto, DeleteAgentRequestDto,
    GetAgentRequestDto, ListAgentsRequestDto, ListSessionItemsRequestDto, ListSessionsRequestDto,
    RestoreAgentRequestDto, UpdateAgentRequestDto, UpdateAgentStatusRequestDto,
};
#[cfg(feature = "http-axum")]
pub use http::testing;
#[cfg(feature = "http-axum")]
pub use http::{
    build_app_router, build_app_routes, build_backend_router, build_backend_routes,
    build_combined_router, build_combined_routes, build_open_router, build_open_routes,
    serve_agents_metrics, AgentHttpState, AgentRequestContext,
};
pub use id::{AgentBusinessIdGenerator, AgentIdGenerator, AUDIT_SINK_NODE_ID};
pub use infrastructure::{
    is_production_environment, validate_production_security_config, AgentMetricsRegistry,
    AgentServiceMetrics, AllowAllPolicyProvider, DenyAllPolicyProvider, IamGatedPolicyProvider,
    InMemoryAgentAuditSink, InMemoryAgentRepository, PolicyMode, ENV_DEPLOYMENT_ENV,
    ENV_DEV_AUTH_BYPASS, IAM_PERMISSION_AGENTS_MANAGE, IAM_PERMISSION_AGENTS_READ,
};
#[cfg(feature = "sqlite-sync")]
pub use persistence::sqlite_sql;
#[cfg(feature = "sqlite-sync")]
pub use persistence::SyncSqliteAdapter;
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
pub use ports::{SessionCheckpointListQuery, SessionRuntimeBindingListQuery, TurnListQuery};
#[cfg(feature = "postgres-sync")]
pub use persistence::{SyncPostgresAdapter, AGENTS_MANAGED_STORE_DATABASE_SERVICE};
#[cfg(feature = "postgres-sync")]
pub use persistence::{
    SQL_COUNT_AGENT_INTERACTIONS, SQL_COUNT_AGENT_ITEM_FEEDBACK,
    SQL_COUNT_AGENT_SESSION_ITEMS,
    SQL_COUNT_AGENT_PROJECTS, SQL_COUNT_AGENT_PROJECT_COMPOSITION_SLOTS,
    SQL_COUNT_AGENT_RESOURCE_USER_STATES, SQL_COUNT_AGENT_SESSIONS, SQL_COUNT_AGENT_TASKS,
    SQL_INSERT_AGENT_INTERACTION, SQL_INSERT_AGENT_ITEM_DRIVE_REF, SQL_INSERT_AGENT_PROJECT,
    SQL_INSERT_AGENT_PROJECT_COMPOSITION_SLOT, SQL_INSERT_AGENT_SESSION,
    SQL_INSERT_AGENT_SESSION_ITEM, SQL_INSERT_AGENT_TASK, SQL_INSERT_AGENT_TURN,
    SQL_LIST_AGENT_INTERACTIONS, SQL_LIST_AGENT_ITEM_DRIVE_REFS,
    SQL_LIST_AGENT_ITEM_DRIVE_REFS_BATCH, SQL_LIST_AGENT_ITEM_FEEDBACK,
    SQL_LIST_AGENT_PROJECTS, SQL_LIST_AGENT_PROJECT_COMPOSITION_SLOTS,
    SQL_LIST_AGENT_RESOURCE_USER_STATES, SQL_LIST_AGENT_SESSION_ITEMS,
    SQL_LIST_AGENT_SESSION_ITEMS_RECENT_CONTEXT, SQL_LIST_AGENT_SESSIONS, SQL_LIST_AGENT_TASKS,
    SQL_NEXT_SESSION_ITEM_SEQUENCE, SQL_SELECT_AGENT_INTERACTION,
    SQL_SELECT_AGENT_ITEM_FEEDBACK,
    SQL_SELECT_AGENT_PROJECT, SQL_SELECT_AGENT_PROJECT_COMPOSITION_SLOT,
    SQL_SELECT_AGENT_RESOURCE_USER_STATE, SQL_SELECT_AGENT_SESSION, SQL_SELECT_AGENT_SESSION_ITEM,
    SQL_SELECT_AGENT_TASK, SQL_SELECT_AGENT_TURN, SQL_SELECT_AGENT_TURN_BY_IDEMPOTENCY,
    SQL_UPDATE_AGENT_INTERACTION, SQL_UPDATE_AGENT_PROJECT,
    SQL_UPDATE_AGENT_PROJECT_COMPOSITION_SLOT, SQL_UPDATE_AGENT_SESSION,
    SQL_UPDATE_AGENT_SESSION_ITEM, SQL_UPDATE_AGENT_TASK, SQL_UPDATE_AGENT_TURN_STATE,
    SQL_UPSERT_AGENT_ITEM_FEEDBACK, SQL_UPSERT_AGENT_RESOURCE_USER_STATE,
};
pub use ports::{
    offset_paginated_result, AgentAuditSink, AgentListQuery, AgentRepository, AuditEventListQuery,
    CompositionSlotListQuery, InteractionListQuery, McpMarketplaceListQuery,
    ItemFeedbackListQuery, SessionItemListQuery, SessionItemListSort, PaginatedResult, PaginationParams,
    ProjectCompositionSlotListQuery, ProjectListQuery, ProviderBindingListQuery,
    ResourceUserStateListQuery, SessionListQuery, CHAT_CONTEXT_MESSAGE_LIMIT, DEFAULT_PAGE_SIZE,
    MAX_CHAT_USER_CONTENT_BYTES, MAX_PAGE_SIZE,
};
pub use project::{
    AgentProjectCompositionSlotRecord, AgentProjectDriveAccessMode, AgentProjectRecord,
    AgentProjectStatus, AgentProjectVisibility,
};
#[cfg(feature = "sqlite-sync")]
pub use sqlite_sync_pool::{BlockingSqlitePool, SQLITE_MANAGED_STORE_DATABASE_SERVICE};
