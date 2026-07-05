mod api;
mod application;
mod chat_runtime;
mod code_engine_catalog;
mod domain;
mod mcp_marketplace;
mod runtime_facade_bridge;
mod dto;
#[cfg(feature = "http-axum")]
mod http;
#[cfg(feature = "http-axum")]
pub mod response;
mod id;
mod in_memory_pagination;
mod infrastructure;
mod persistence;
mod ports;
#[cfg(feature = "postgres-sync")]
mod postgres_sync_pool;
mod validation;

pub use api::{
    ApiOperation, AGENT_APP_API_OPERATIONS, AGENT_APP_API_PREFIX, AGENT_BACKEND_API_OPERATIONS,
    AGENT_BACKEND_API_PREFIX, AGENT_OPEN_API_OPERATIONS, AGENT_OPEN_API_PREFIX,
};
pub use application::{
    ActivateAgentProviderBindingCommand, AgentCompositionSlotCreateCommand,
    AgentCompositionSlotDeleteCommand, AgentCompositionSlotGetCommand,
    AgentCompositionSlotListCommand, AgentCompositionSlotUpdateCommand,
    AgentPreviewResponseCommand, AgentPromptOptimizationCommand, AgentProviderBindingCommand,
    AgentsService, AnswerInteractionCommand, ApproveInteractionCommand, ArchiveSessionCommand,
    ChangeAgentStatusCommand, ChatCompletionResult, CloseSessionCommand, CreateAgentCommand,
    CreateMessageCommand, CreateSessionCommand, DeleteAgentCommand, GetAgentCommand,
    GetInteractionCommand, GetMessageCommand, GetSessionCommand, ListAgentsCommand,
    ListAgentAuditEventsCommand, ListInteractionsCommand, ListMcpMarketplaceCommand,
    ListMessagesCommand, ListSessionsCommand, ProviderBindingListCommand, RestoreAgentCommand,
    SendChatMessageCommand, UpdateAgentCommand,
};
pub use chat_runtime::{
    ChatCompleter, ChatCompletionInput, ChatCompletionOutput, ContractChatCompleter,
    KernelModelChatCompleter, RuntimeFacadeChatCompleter, complete_chat_turn, is_inference_error,
    RUNTIME_MODE_FACADE, RUNTIME_MODE_INFERENCE_ERROR,
};
pub use sdkwork_intelligence_prompts_ai_contract::{
    AgentPromptTemplateKind, AgentPromptTemplateRecord, PromptAiRepository,
};

pub use domain::{
    AgentAuditAction, AgentBusinessRecord, AgentBusinessStatus, AgentCompositionSlotKind,
    AgentCompositionSlotRecord, AgentCompositionTargetModule, AgentImplementationKind,
    AgentImplementationType, AgentInteractionKind, AgentInteractionRecord,
    AgentInteractionStatus, AgentMessageRecord, AgentMessageRole, AgentMessageStatus,
    AgentProviderBindingRecord, AgentRuntimeExecutionOperation, AgentRuntimeExecutionRecord,
    AgentRuntimeExecutionStatus, AgentSessionRecord, AgentSessionStatus, AgentVisibility,
    DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY,
};
pub use dto::{
    ActivateAgentProviderBindingRequestDto, AgentCompositionSlotCreateRequestDto,
    AgentCompositionSlotDeleteRequestDto, AgentCompositionSlotListResponseDto,
    AgentCompositionSlotRecordDto, AgentCompositionSlotResponseDto,
    AgentCompositionSlotUpdateRequestDto, AgentListResponseDto, AgentManagementProfileDto,
    AgentMessageListResponseDto, AgentMessageRecordDto, AgentMessageResponseDto,
    AgentPreviewResponseRequestDto, AgentPromptOptimizationRequestDto,
    AgentProviderBindingListResponseDto, AgentProviderBindingRecordDto,
    AgentProviderBindingRequestDto, AgentProviderBindingResponseDto, AgentRecordDto,
    AgentResponseDto, AgentRuntimeExecutionRecordDto, AgentRuntimeExecutionResponseDto,
    AgentSessionListResponseDto, AgentSessionRecordDto, AgentSessionResponseDto,
    ArchiveSessionRequestDto, CloseSessionRequestDto, CreateAgentRequestDto,
    CreateMessageRequestDto, CreateSessionRequestDto, DeleteAgentRequestDto,
    GetAgentRequestDto, ListAgentsRequestDto, ListMessagesRequestDto, ListSessionsRequestDto,
    RestoreAgentRequestDto, UpdateAgentRequestDto, UpdateAgentStatusRequestDto,
};
#[cfg(feature = "http-axum")]
pub use http::{
    build_app_router, build_app_routes, build_backend_router, build_backend_routes,
    build_combined_router, build_combined_routes, build_open_router, build_open_routes,
    serve_agents_metrics, AgentHttpState, AgentRequestContext,
};
#[cfg(feature = "http-axum")]
pub use http::testing;
pub use id::{AgentBusinessIdGenerator, AgentIdGenerator, AUDIT_SINK_NODE_ID};
pub use infrastructure::{
    AllowAllPolicyProvider, AgentMetricsRegistry, AgentServiceMetrics, DenyAllPolicyProvider, IamGatedPolicyProvider,
    InMemoryAgentAuditSink, InMemoryAgentRepository, PolicyMode, validate_production_security_config,
    is_production_environment, ENV_DEV_AUTH_BYPASS, ENV_DEPLOYMENT_ENV,
    IAM_PERMISSION_AGENTS_MANAGE, IAM_PERMISSION_AGENTS_READ,
};
pub use persistence::{
    extract_event_context, PostgresAgentAuditSink, PostgresAgentRepository,
    PostgresAgentRepositoryAdapter, SQL_INSERT_AGENT, SQL_INSERT_AGENT_COMPOSITION_SLOT,
    SQL_INSERT_AGENT_PROVIDER_BINDING, SQL_INSERT_AUDIT_EVENT, SQL_LIST_AGENT, SQL_COUNT_AGENT,
    SQL_LIST_AGENT_COMPOSITION_SLOTS, SQL_COUNT_AGENT_COMPOSITION_SLOTS,
    SQL_LIST_AGENT_PROVIDER_BINDINGS, SQL_COUNT_AGENT_PROVIDER_BINDINGS,
    SQL_LIST_MCP_MARKETPLACE_SLOTS, SQL_COUNT_MCP_MARKETPLACE_SLOTS,
    SQL_COUNT_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID,
    SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID, SQL_SELECT_AGENT_COMPOSITION_SLOT,
    SQL_SELECT_AGENT_PROVIDER_BINDING, SQL_UPDATE_AGENT, SQL_UPDATE_AGENT_COMPOSITION_SLOT,
    SQL_UPDATE_AGENT_PROVIDER_BINDING,
};
#[cfg(feature = "postgres-sync")]
pub use persistence::{
    SQL_INSERT_AGENT_INTERACTION, SQL_INSERT_AGENT_MESSAGE, SQL_INSERT_AGENT_SESSION,
    SQL_LIST_AGENT_INTERACTIONS, SQL_COUNT_AGENT_INTERACTIONS, SQL_LIST_AGENT_MESSAGES, SQL_LIST_AGENT_MESSAGES_RECENT_CONTEXT,
    SQL_LIST_AGENT_SESSIONS, SQL_COUNT_AGENT_MESSAGES, SQL_COUNT_AGENT_SESSIONS,
    SQL_NEXT_MESSAGE_SEQUENCE, SQL_SELECT_AGENT_INTERACTION, SQL_SELECT_AGENT_MESSAGE,
    SQL_SELECT_AGENT_SESSION, SQL_UPDATE_AGENT_INTERACTION, SQL_UPDATE_AGENT_MESSAGE,
    SQL_UPDATE_AGENT_SESSION,
};
#[cfg(feature = "postgres-sync")]
pub use persistence::{SyncPostgresAdapter, AGENTS_MANAGED_STORE_DATABASE_SERVICE};
pub use ports::{
    AgentAuditSink, AgentListQuery, AgentRepository, AuditEventListQuery, CompositionSlotListQuery,
    InteractionListQuery, McpMarketplaceListQuery, MessageListQuery, MessageListSort,
    ProviderBindingListQuery, PaginationParams, PaginatedResult, SessionListQuery,
    CHAT_CONTEXT_MESSAGE_LIMIT, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE, offset_paginated_result,
};
