mod api;
mod application;
mod chat_runtime;
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
#[cfg(feature = "http-axum")]
pub mod response;
mod runtime_facade_bridge;
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
    CreateInteractionCommand, CreateMessageCommand, CreateSessionCommand, DeleteAgentCommand,
    GetAgentCommand, GetInteractionCommand, GetMessageCommand, GetSessionCommand,
    ListAgentAuditEventsCommand, ListAgentsCommand, ListInteractionsCommand,
    ListMcpMarketplaceCommand, ListMessagesCommand, ListSessionsCommand,
    ProviderBindingListCommand, RestoreAgentCommand, SendChatMessageCommand, UpdateAgentCommand,
};
pub use chat_runtime::{
    complete_chat_turn, complete_with_timeout, is_inference_error, ChatCompleter,
    ChatCompletionInput, ChatCompletionOutput, ContractChatCompleter, KernelModelChatCompleter,
    RuntimeFacadeChatCompleter, CHAT_COMPLETION_TIMEOUT, RUNTIME_MODE_FACADE,
    RUNTIME_MODE_INFERENCE_ERROR,
};
pub use sdkwork_intelligence_prompts_ai_contract::{
    AgentPromptTemplateKind, AgentPromptTemplateRecord, PromptAiRepository,
};

pub use domain::{
    AgentAuditAction, AgentBusinessRecord, AgentBusinessStatus, AgentCompositionSlotKind,
    AgentCompositionSlotRecord, AgentCompositionTargetModule, AgentImplementationKind,
    AgentImplementationType, AgentInteractionKind, AgentInteractionRecord, AgentInteractionStatus,
    AgentMessageRecord, AgentMessageRole, AgentMessageStatus, AgentProviderBindingRecord,
    AgentRuntimeExecutionOperation, AgentRuntimeExecutionRecord, AgentRuntimeExecutionStatus,
    AgentSessionRecord, AgentSessionStatus, AgentVisibility,
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
    CreateMessageRequestDto, CreateSessionRequestDto, DeleteAgentRequestDto, GetAgentRequestDto,
    ListAgentsRequestDto, ListMessagesRequestDto, ListSessionsRequestDto, RestoreAgentRequestDto,
    UpdateAgentRequestDto, UpdateAgentStatusRequestDto,
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
pub use persistence::{
    extract_event_context, PostgresAgentAuditSink, PostgresAgentRepository,
    PostgresAgentRepositoryAdapter, SQL_COUNT_AGENT, SQL_COUNT_AGENT_COMPOSITION_SLOTS,
    SQL_COUNT_AGENT_PROVIDER_BINDINGS, SQL_COUNT_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID,
    SQL_COUNT_MCP_MARKETPLACE_SLOTS, SQL_INSERT_AGENT, SQL_INSERT_AGENT_COMPOSITION_SLOT,
    SQL_INSERT_AGENT_PROVIDER_BINDING, SQL_INSERT_AUDIT_EVENT, SQL_LIST_AGENT,
    SQL_LIST_AGENT_COMPOSITION_SLOTS, SQL_LIST_AGENT_PROVIDER_BINDINGS,
    SQL_LIST_MCP_MARKETPLACE_SLOTS, SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID,
    SQL_SELECT_AGENT_COMPOSITION_SLOT, SQL_SELECT_AGENT_PROVIDER_BINDING, SQL_UPDATE_AGENT,
    SQL_UPDATE_AGENT_COMPOSITION_SLOT, SQL_UPDATE_AGENT_PROVIDER_BINDING,
};
#[cfg(feature = "postgres-sync")]
pub use persistence::{SyncPostgresAdapter, AGENTS_MANAGED_STORE_DATABASE_SERVICE};
#[cfg(feature = "postgres-sync")]
pub use persistence::{
    SQL_COUNT_AGENT_INTERACTIONS, SQL_COUNT_AGENT_MESSAGES, SQL_COUNT_AGENT_SESSIONS,
    SQL_COUNT_AGENT_TASKS, SQL_INSERT_AGENT_INTERACTION, SQL_INSERT_AGENT_MESSAGE,
    SQL_INSERT_AGENT_SESSION, SQL_INSERT_AGENT_TASK, SQL_LIST_AGENT_INTERACTIONS,
    SQL_LIST_AGENT_MESSAGES, SQL_LIST_AGENT_MESSAGES_RECENT_CONTEXT, SQL_LIST_AGENT_SESSIONS,
    SQL_LIST_AGENT_TASKS, SQL_NEXT_MESSAGE_SEQUENCE, SQL_SELECT_AGENT_INTERACTION,
    SQL_SELECT_AGENT_MESSAGE, SQL_SELECT_AGENT_SESSION, SQL_SELECT_AGENT_TASK,
    SQL_UPDATE_AGENT_INTERACTION, SQL_UPDATE_AGENT_MESSAGE, SQL_UPDATE_AGENT_SESSION,
    SQL_UPDATE_AGENT_TASK,
};
pub use ports::{
    offset_paginated_result, AgentAuditSink, AgentListQuery, AgentRepository, AuditEventListQuery,
    CompositionSlotListQuery, InteractionListQuery, McpMarketplaceListQuery, MessageListQuery,
    MessageListSort, PaginatedResult, PaginationParams, ProviderBindingListQuery, SessionListQuery,
    CHAT_CONTEXT_MESSAGE_LIMIT, DEFAULT_PAGE_SIZE, MAX_CHAT_USER_CONTENT_BYTES, MAX_PAGE_SIZE,
};
