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
    AgentMcpServerCreateCommand, AgentMcpServerUpdateCommand, AgentPreviewResponseCommand,
    AgentPromptOptimizationCommand, AgentProviderBindingCommand, AgentProviderDeploymentCommand,
    ChangeAgentStatusCommand, CreateAgentCommand, DeleteAgentCommand,
    DeleteAgentMarketplaceItemCommand, GetAgentCommand, GetAgentMarketplaceItemCommand,
    ListAgentsCommand, RestoreAgentCommand, RestoreAgentMarketplaceItemCommand,
    UpdateAgentCommand,
};
pub use domain::{
    AgentAuditAction, AgentBusinessRecord, AgentBusinessStatus, AgentCompositionSlotKind,
    AgentCompositionSlotRecord, AgentCompositionTargetModule, AgentDeploymentRecord,
    AgentDeploymentStatus, AgentImplementationKind, AgentImplementationType, AgentMcpAuthKind,
    AgentMcpServerRecord, AgentMcpTransportKind, AgentProviderBindingRecord,
    AgentRuntimeExecutionOperation, AgentRuntimeExecutionRecord, AgentRuntimeExecutionStatus,
    AgentVisibility, DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY,
};
pub use dto::{
    ActivateAgentProviderBindingRequestDto, AgentCompositionSlotCreateRequestDto,
    AgentCompositionSlotDeleteRequestDto, AgentCompositionSlotListResponseDto,
    AgentCompositionSlotRecordDto, AgentCompositionSlotResponseDto,
    AgentCompositionSlotUpdateRequestDto, AgentDeploymentListResponseDto,
    AgentDeploymentRecordDto, AgentDeploymentResponseDto, AgentListResponseDto,
    AgentManagementProfileDto, AgentPreviewResponseRequestDto, AgentPromptOptimizationRequestDto,
    AgentProviderBindingListResponseDto, AgentProviderBindingRecordDto,
    AgentProviderBindingRequestDto, AgentProviderBindingResponseDto,
    AgentProviderDeploymentRequestDto, AgentRecordDto, AgentResponseDto,
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
pub use id::{AgentBusinessIdGenerator, AgentIdGenerator};
pub use infrastructure::{
    AllowAllPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository, PolicyMode,
};
pub use persistence::{
    AgentAuditEventRow, AgentBusinessRow, AgentCompositionSlotRow, AgentDeploymentRow,
    AgentMcpServerRow, AgentProviderBindingRow, PostgresAgentAuditSink, PostgresAgentRepository,
    PostgresAgentRepositoryAdapter, SQL_INSERT_AGENT, SQL_INSERT_AGENT_COMPOSITION_SLOT,
    SQL_INSERT_AGENT_DEPLOYMENT, SQL_INSERT_AGENT_MCP_SERVER, SQL_INSERT_AGENT_PROVIDER_BINDING,
    SQL_INSERT_AUDIT_EVENT, SQL_LIST_AGENT, SQL_LIST_AGENT_COMPOSITION_SLOTS,
    SQL_LIST_AGENT_DEPLOYMENTS, SQL_LIST_AGENT_MCP_SERVERS, SQL_LIST_AGENT_PROVIDER_BINDINGS,
    SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID, SQL_SELECT_AGENT_COMPOSITION_SLOT,
    SQL_SELECT_AGENT_MCP_SERVER, SQL_SELECT_AGENT_PROVIDER_BINDING, SQL_UPDATE_AGENT,
    SQL_UPDATE_AGENT_COMPOSITION_SLOT, SQL_UPDATE_AGENT_MCP_SERVER,
    SQL_UPDATE_AGENT_PROVIDER_BINDING,
};
#[cfg(feature = "postgres-sync")]
pub use persistence::{SyncPostgresAdapter, AGENTS_MANAGED_STORE_DATABASE_SERVICE};
pub use ports::{AgentAuditSink, AgentListQuery, AgentMarketplaceListQuery, AgentRepository};
