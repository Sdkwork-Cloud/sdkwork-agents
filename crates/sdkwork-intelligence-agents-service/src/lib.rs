mod api;
mod application;
mod domain;
mod dto;
#[cfg(feature = "http-axum")]
mod http;
mod id;
mod infrastructure;
mod persistence;
mod ports;
#[cfg(feature = "postgres-sync")]
mod postgres_sync_pool;
mod validation;

pub use sdkwork_intelligence_prompts_ai_contract::{
    AgentPromptTemplateKind, AgentPromptTemplateRecord, PromptAiRepository,
};
pub use api::{
    ApiOperation, AGENT_APP_API_OPERATIONS, AGENT_APP_API_PREFIX, AGENT_BACKEND_API_OPERATIONS,
    AGENT_BACKEND_API_PREFIX, AGENT_OPEN_API_OPERATIONS, AGENT_OPEN_API_PREFIX,
};
pub use application::{
    ActivateAgentProviderBindingCommand, AgentCompositionSlotCreateCommand,
    AgentCompositionSlotDeleteCommand, AgentCompositionSlotGetCommand,
    AgentCompositionSlotListCommand, AgentCompositionSlotUpdateCommand, AgentsService,
    AgentPreviewResponseCommand, AgentPromptOptimizationCommand, AgentProviderBindingCommand,
    ChangeAgentStatusCommand, CreateAgentCommand,
    DeleteAgentCommand, GetAgentCommand, ListAgentsCommand, RestoreAgentCommand,
    UpdateAgentCommand,
};

pub use domain::{
    AgentAuditAction, AgentBusinessRecord, AgentBusinessStatus, AgentCompositionSlotKind,
    AgentCompositionSlotRecord, AgentCompositionTargetModule,
    AgentImplementationKind, AgentImplementationType,
    AgentProviderBindingRecord, AgentRuntimeExecutionOperation, AgentRuntimeExecutionRecord,
    AgentRuntimeExecutionStatus, AgentVisibility, DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY,
};
pub use dto::{
    ActivateAgentProviderBindingRequestDto, AgentCompositionSlotCreateRequestDto,
    AgentCompositionSlotDeleteRequestDto, AgentCompositionSlotListResponseDto,
    AgentCompositionSlotRecordDto, AgentCompositionSlotResponseDto,
    AgentCompositionSlotUpdateRequestDto, AgentListResponseDto,
    AgentManagementProfileDto, AgentPreviewResponseRequestDto, AgentPromptOptimizationRequestDto,
    AgentProviderBindingListResponseDto, AgentProviderBindingRecordDto,
    AgentProviderBindingRequestDto, AgentProviderBindingResponseDto,
    AgentRecordDto, AgentResponseDto,
    AgentRuntimeExecutionRecordDto, AgentRuntimeExecutionResponseDto, CreateAgentRequestDto,
    DeleteAgentRequestDto, GetAgentRequestDto, ListAgentsRequestDto, RestoreAgentRequestDto,
    UpdateAgentRequestDto, UpdateAgentStatusRequestDto,
};
#[cfg(feature = "http-axum")]
pub use http::{
    build_app_router, build_app_routes, build_backend_router, build_backend_routes,
    build_combined_router, build_combined_routes, build_open_router, build_open_routes,
    AgentHttpState, AgentRequestContext,
};
pub use id::{AgentBusinessIdGenerator, AgentIdGenerator, AUDIT_SINK_NODE_ID};
pub use infrastructure::{
    AllowAllPolicyProvider, DenyAllPolicyProvider, IamGatedPolicyProvider,
    IAM_PERMISSION_AGENTS_MANAGE, IAM_PERMISSION_AGENTS_READ, InMemoryAgentAuditSink,
    InMemoryAgentRepository, PolicyMode,
};
pub use persistence::{
    AgentAuditEventRow, AgentBusinessRow, AgentCompositionSlotRow,
    AgentProviderBindingRow, PostgresAgentAuditSink, PostgresAgentRepository,
    PostgresAgentRepositoryAdapter, SQL_INSERT_AGENT, SQL_INSERT_AGENT_COMPOSITION_SLOT,
    SQL_INSERT_AGENT_PROVIDER_BINDING, SQL_INSERT_AUDIT_EVENT,
    SQL_LIST_AGENT, SQL_LIST_AGENT_COMPOSITION_SLOTS,
    SQL_LIST_AGENT_PROVIDER_BINDINGS, SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID,
    SQL_SELECT_AGENT_COMPOSITION_SLOT, SQL_SELECT_AGENT_PROVIDER_BINDING, SQL_UPDATE_AGENT,
    SQL_UPDATE_AGENT_COMPOSITION_SLOT, SQL_UPDATE_AGENT_PROVIDER_BINDING,
};
#[cfg(feature = "postgres-sync")]
pub use persistence::{SyncPostgresAdapter, AGENTS_MANAGED_STORE_DATABASE_SERVICE};
pub use ports::{AgentAuditSink, AgentListQuery, AgentRepository};
