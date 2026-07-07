use crate::application::{
    AgentCompositionSlotCreateCommand, AgentCompositionSlotDeleteCommand,
    AgentCompositionSlotGetCommand, AgentCompositionSlotListCommand,
    AgentCompositionSlotUpdateCommand, AgentsService, ChatCompletionResult, GetInteractionCommand,
    GetMessageCommand, GetSessionCommand, GetTaskCommand, ListAgentAuditEventsCommand,
    ListMcpMarketplaceCommand, ProviderBindingListCommand, SendChatMessageCommand,
};
use crate::chat_runtime::{ChatCompleter, ContractChatCompleter};
use crate::domain::{
    AgentCompositionSlotKind, AgentCompositionSlotRecord, AgentCompositionTargetModule,
    AgentProviderBindingRecord,
};
use crate::dto::{
    ActivateAgentProviderBindingRequestDto, AgentCompositionSlotCreateRequestDto,
    AgentCompositionSlotRecordDto, AgentCompositionSlotUpdateRequestDto, AgentInteractionRecordDto,
    AgentManagementProfileDto, AgentMessageRecordDto, AgentPreviewResponseRequestDto,
    AgentPromptOptimizationRequestDto, AgentProviderBindingRecordDto,
    AgentProviderBindingRequestDto, AgentRecordDto, AgentRuntimeExecutionRecordDto,
    AgentSessionRecordDto, AgentTaskRecordDto, AnswerInteractionRequestDto,
    ApproveInteractionRequestDto, ArchiveSessionRequestDto, CancelTaskRequestDto,
    CloseSessionRequestDto, CreateAgentRequestDto, CreateInteractionRequestDto,
    CreateSessionRequestDto, CreateTaskRequestDto, DeleteAgentRequestDto, GetAgentRequestDto,
    ListAgentsRequestDto, ListInteractionsRequestDto, ListMessagesRequestDto,
    ListSessionsRequestDto, ListTasksRequestDto, RestoreAgentRequestDto, UpdateAgentRequestDto,
    UpdateAgentStatusRequestDto,
};
use crate::mcp_marketplace::McpServerMarketplaceRecord;
use crate::ports::{
    AgentAuditSink, AgentRepository, AuditEventListQuery, CompositionSlotListQuery,
    McpMarketplaceListQuery, PaginationParams, ProviderBindingListQuery,
};
use crate::response::{
    created_json, finish_api_json, no_content, ApiProblem, ApiResult, PageData, PageInfo, PageMode,
    ResourceData,
};
use crate::validation::{
    is_trimmed_blank, parse_expected_version, parse_optional_rfc3339_datetime,
    parse_organization_id, parse_owner_user_id, parse_tenant_id, validate_requested_at,
    validate_standard_id,
};
use axum::body::Body;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{DefaultBodyLimit, Extension, Path, Query, State};
use axum::http::header::{HeaderName, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{middleware, Json, Router};
use sdkwork_agent_kernel::{
    AgentManifest, KernelError, KernelErrorKind, KernelResult, PolicyDecision, PolicyProvider,
    PolicyRequest, PolicySubject, ProviderHealth,
};
use sdkwork_agents_runtime_facade::CodeEngineCatalog;
use sdkwork_code_kernel::CodeTaskIntent;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use time::OffsetDateTime;

const HEADER_SUBJECT_ID: &str = "x-subject-id";
const HEADER_SUBJECT_TENANT_ID: &str = "x-subject-tenant-id";
const HEADER_SUBJECT_ROLES: &str = "x-subject-roles";
const HEADER_SDKWORK_USER_ID: &str = "x-sdkwork-user-id";
const HEADER_SDKWORK_ACTOR_ID: &str = "x-sdkwork-actor-id";
const HEADER_SDKWORK_TENANT_ID: &str = "x-sdkwork-tenant-id";
const HEADER_SDKWORK_PERMISSION_SCOPE: &str = "x-sdkwork-permission-scope";
const HEADER_SDKWORK_TRACE_ID: &str = "x-sdkwork-trace-id";
const HEADER_SDKWORK_REQUEST_ID: &str = "x-sdkwork-request-id";
const MAX_PAGE_SIZE: usize = 200;
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
    "message_created",
    "message_failed",
    "interaction_created",
    "interaction_resolved",
    "interaction_rejected",
    "interaction_expired",
    "interaction_cancelled",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRequestContext {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub subject_id: String,
    pub roles: Vec<String>,
    /// W3C trace id resolved from `traceparent` (or `x-sdkwork-trace-id`) at the
    /// gateway boundary. Propagated to every success/error envelope so responses
    /// stay correlatable end-to-end (`API_SPEC.md` §15.1.1, OBSERVABILITY_SPEC §1).
    pub trace_id: Option<String>,
    /// Server request id used as the Problem+json fallback correlation when no
    /// trace id is present.
    pub request_id: Option<String>,
}

impl AgentRequestContext {
    pub fn new(tenant_id: impl Into<String>, owner_user_id: impl Into<String>) -> Self {
        let owner_user_id = owner_user_id.into();
        Self {
            tenant_id: tenant_id.into(),
            organization_id: None,
            subject_id: owner_user_id.clone(),
            owner_user_id,
            roles: Vec::new(),
            trace_id: None,
            request_id: None,
        }
    }

    pub fn with_organization_id(mut self, organization_id: impl Into<String>) -> Self {
        self.organization_id = Some(organization_id.into());
        self
    }

    pub fn with_subject_id(mut self, subject_id: impl Into<String>) -> Self {
        self.subject_id = subject_id.into();
        self
    }

    pub fn with_roles(mut self, roles: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.roles = roles.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Build trusted context from gateway-injected subject headers before resource tenant reconciliation.
    pub(crate) fn from_gateway_subject_headers(headers: &HeaderMap) -> Result<Self, ApiProblem> {
        let subject_id = required_header_any(
            headers,
            &[
                HEADER_SUBJECT_ID,
                HEADER_SDKWORK_USER_ID,
                HEADER_SDKWORK_ACTOR_ID,
            ],
        )?;
        // tenant_id is the mandatory multi-tenant isolation key. The gateway
        // must always inject either `x-subject-tenant-id` or
        // `x-sdkwork-tenant-id`. Falling back to an empty string (the previous
        // behavior) allowed a request with a missing tenant header to enter the
        // application layer as tenant_id="", which downstream code then parsed
        // as 0 or used to bypass tenant-scoped filters. Reject at the edge.
        let tenant_id = required_header_any(
            headers,
            &[HEADER_SUBJECT_TENANT_ID, HEADER_SDKWORK_TENANT_ID],
        )?;
        let mut roles = Vec::new();
        if let Some(roles_header) = optional_header_any(
            headers,
            &[HEADER_SUBJECT_ROLES, HEADER_SDKWORK_PERMISSION_SCOPE],
        ) {
            for role in roles_header
                .split([',', ' '])
                .map(str::trim)
                .filter(|role| !role.is_empty())
            {
                roles.push(role.to_string());
            }
        }
        // Resolve W3C trace id for envelope correlation. Prefer an explicit
        // `x-sdkwork-trace-id` header (set by the gateway), then `traceparent`,
        // then synthesize from the request id so every response stays
        // correlatable even when upstream proxies strip tracing headers.
        let request_id = optional_header_any(headers, &[HEADER_SDKWORK_REQUEST_ID])
            .unwrap_or_else(synthesize_request_id);
        let trace_id = optional_header_any(headers, &[HEADER_SDKWORK_TRACE_ID])
            .or_else(|| {
                sdkwork_web_core::trace::resolve_trace_context(headers, &request_id)
                    .traceparent
                    .as_str()
                    .split('-')
                    .nth(1)
                    .map(str::to_owned)
            })
            .or_else(|| Some(request_id.clone()));
        Ok(Self {
            tenant_id,
            organization_id: None,
            owner_user_id: subject_id.clone(),
            subject_id,
            roles,
            trace_id,
            request_id: Some(request_id),
        })
    }

    fn subject(&self) -> PolicySubject {
        let mut subject = PolicySubject::new(self.subject_id.clone(), self.tenant_id.clone());
        for role in &self.roles {
            subject = subject.with_role(role.clone());
        }
        subject
    }
}

/// Build a minimal framework request context for envelope correlation.
///
/// The agents service owns its own lightweight middleware, so we construct a
/// `WebRequestContext` carrying only the fields required by
/// `sdkwork_web_core::problem_response` and `SdkWorkApiResponse::success`:
/// `request_id`, `api_surface`, `transport` facts, and the resolved `trace_id`.
/// Richer principal/auth fields are populated by the gateway web-framework
/// layer when this service is mounted behind `sdkwork-web-bootstrap`.
fn build_web_request_context(
    agent_context: &AgentRequestContext,
    request: &Request<axum::body::Body>,
    api_surface: sdkwork_web_core::WebApiSurface,
) -> sdkwork_web_core::WebRequestContext {
    let transport = sdkwork_web_core::WebTransportFacts {
        path: request.uri().path().to_owned(),
        method: request.method().as_str().to_owned(),
        auth_token_present: request.headers().get("authorization").is_some(),
        access_token_present: request.headers().get("x-sdkwork-access-token").is_some(),
        api_key_present: request.headers().get("x-sdkwork-api-key").is_some(),
        oauth_bearer_present: request.headers().get("x-sdkwork-oauth-bearer").is_some(),
        agent_token_present: request.headers().get("x-sdkwork-agent-token").is_some(),
    };
    sdkwork_web_core::WebRequestContext {
        request_id: sdkwork_web_core::ServerRequestId(
            agent_context
                .request_id
                .clone()
                .unwrap_or_else(synthesize_request_id),
        ),
        api_surface,
        auth_mode: sdkwork_web_core::WebAuthMode::DualToken,
        transport,
        principal: None,
        locale: None,
        client_kind: None,
        operation: None,
        trace_id: agent_context.trace_id.clone(),
    }
}

fn synthesize_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("ag-{nanos:032x}")
}

#[derive(Debug, Clone)]
pub(crate) struct RequestScope {
    tenant_id: String,
    organization_id: String,
    owner_user_id: String,
    subject: PolicySubject,
}

impl RequestScope {
    pub(crate) fn from_context(context: AgentRequestContext) -> Self {
        let subject = context.subject();
        Self {
            tenant_id: context.tenant_id.clone(),
            organization_id: context.organization_id.unwrap_or_else(|| "0".to_string()),
            owner_user_id: context.owner_user_id.clone(),
            subject,
        }
    }

    fn owner_scope(&self) -> Result<Option<u64>, ApiProblem> {
        parse_owner_user_id(&self.owner_user_id)
            .map(Some)
            .map_err(ApiProblem::from_kernel_error)
    }

    pub(crate) fn from_trusted_extension(
        mut context: AgentRequestContext,
        resource_tenant_id: String,
        organization_id: Option<String>,
        owner_user_id: Option<String>,
    ) -> Result<Self, ApiProblem> {
        let header_tenant = if context.tenant_id.is_empty() {
            None
        } else {
            Some(context.tenant_id.clone())
        };
        let tenant_id = reconcile_resource_tenant_with_subject_header(
            resource_tenant_id.as_str(),
            header_tenant,
        )?;
        context.tenant_id = tenant_id;
        if let Some(organization_id) = organization_id {
            context.organization_id = Some(organization_id);
        }
        if let Some(owner_user_id) = owner_user_id {
            context.owner_user_id = owner_user_id;
        }
        Ok(Self::from_context(context))
    }

    pub(crate) fn subject(&self) -> &PolicySubject {
        &self.subject
    }

    pub(crate) fn tenant_id_u64(&self) -> Result<u64, ApiProblem> {
        parse_tenant_id(self.tenant_id.as_str()).map_err(ApiProblem::from_kernel_error)
    }
}

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
    fn next_id(&self) -> KernelResult<u64> {
        self.0.next_id()
    }

    fn insert(&self, record: crate::domain::AgentBusinessRecord) -> KernelResult<()> {
        self.0.insert(record)
    }

    fn update(&self, record: crate::domain::AgentBusinessRecord) -> KernelResult<()> {
        self.0.update(record)
    }

    fn get(&self, tenant_id: u64, agent_id: &str) -> Option<crate::domain::AgentBusinessRecord> {
        self.0.get(tenant_id, agent_id)
    }

    fn list(
        &self,
        query: &crate::ports::AgentListQuery,
    ) -> Vec<crate::domain::AgentBusinessRecord> {
        self.0.list(query)
    }

    fn count_agents(&self, query: &crate::ports::AgentListQuery) -> u64 {
        self.0.count_agents(query)
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
    ) -> Option<AgentProviderBindingRecord> {
        self.0.get_provider_binding(tenant_id, agent_id, binding_id)
    }

    fn list_provider_bindings(
        &self,
        query: &crate::ports::ProviderBindingListQuery,
    ) -> Vec<AgentProviderBindingRecord> {
        self.0.list_provider_bindings(query)
    }

    fn count_provider_bindings(&self, query: &crate::ports::ProviderBindingListQuery) -> u64 {
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
    ) -> Option<AgentCompositionSlotRecord> {
        self.0.get_composition_slot(tenant_id, agent_id, slot_id)
    }

    fn list_composition_slots(
        &self,
        query: &crate::ports::CompositionSlotListQuery,
    ) -> Vec<AgentCompositionSlotRecord> {
        self.0.list_composition_slots(query)
    }

    fn count_composition_slots(&self, query: &crate::ports::CompositionSlotListQuery) -> u64 {
        self.0.count_composition_slots(query)
    }

    fn list_mcp_marketplace_slots(
        &self,
        query: &crate::ports::McpMarketplaceListQuery,
    ) -> Vec<AgentCompositionSlotRecord> {
        self.0.list_mcp_marketplace_slots(query)
    }

    fn count_mcp_marketplace_slots(&self, query: &crate::ports::McpMarketplaceListQuery) -> u64 {
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
        session_id: &str,
    ) -> Option<crate::domain::AgentSessionRecord> {
        self.0.get_session(tenant_id, session_id)
    }

    fn list_sessions(
        &self,
        query: &crate::ports::SessionListQuery,
    ) -> Vec<crate::domain::AgentSessionRecord> {
        self.0.list_sessions(query)
    }

    fn count_sessions(&self, query: &crate::ports::SessionListQuery) -> u64 {
        self.0.count_sessions(query)
    }

    fn insert_message(&self, record: crate::domain::AgentMessageRecord) -> KernelResult<()> {
        self.0.insert_message(record)
    }

    fn update_message(&self, record: crate::domain::AgentMessageRecord) -> KernelResult<()> {
        self.0.update_message(record)
    }

    fn get_message(
        &self,
        tenant_id: u64,
        session_id: &str,
        message_id: &str,
    ) -> Option<crate::domain::AgentMessageRecord> {
        self.0.get_message(tenant_id, session_id, message_id)
    }

    fn list_messages(
        &self,
        query: &crate::ports::MessageListQuery,
    ) -> Vec<crate::domain::AgentMessageRecord> {
        self.0.list_messages(query)
    }

    fn count_messages(&self, query: &crate::ports::MessageListQuery) -> u64 {
        self.0.count_messages(query)
    }

    fn next_message_sequence(&self, tenant_id: u64, session_id: &str) -> KernelResult<u64> {
        self.0.next_message_sequence(tenant_id, session_id)
    }

    fn insert_chat_turn(
        &self,
        session: crate::domain::AgentSessionRecord,
        user_message: crate::domain::AgentMessageRecord,
        assistant_message: crate::domain::AgentMessageRecord,
    ) -> KernelResult<(
        crate::domain::AgentSessionRecord,
        crate::domain::AgentMessageRecord,
        crate::domain::AgentMessageRecord,
    )> {
        self.0
            .insert_chat_turn(session, user_message, assistant_message)
    }

    fn insert_task(&self, record: crate::domain::AgentTaskRecord) -> KernelResult<()> {
        self.0.insert_task(record)
    }

    fn update_task(&self, record: crate::domain::AgentTaskRecord) -> KernelResult<()> {
        self.0.update_task(record)
    }

    fn get_task(&self, tenant_id: u64, task_id: &str) -> Option<crate::domain::AgentTaskRecord> {
        self.0.get_task(tenant_id, task_id)
    }

    fn list_tasks(
        &self,
        query: &crate::ports::TaskListQuery,
    ) -> Vec<crate::domain::AgentTaskRecord> {
        self.0.list_tasks(query)
    }

    fn count_tasks(&self, query: &crate::ports::TaskListQuery) -> u64 {
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
        session_id: &str,
        interaction_id: &str,
    ) -> Option<crate::domain::AgentInteractionRecord> {
        self.0
            .get_interaction(tenant_id, session_id, interaction_id)
    }

    fn list_interactions(
        &self,
        query: &crate::ports::InteractionListQuery,
    ) -> Vec<crate::domain::AgentInteractionRecord> {
        self.0.list_interactions(query)
    }

    fn count_interactions(&self, query: &crate::ports::InteractionListQuery) -> u64 {
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
        Self::with_chat_completer(
            repository,
            audit_sink,
            policy_provider,
            Arc::new(ContractChatCompleter),
        )
    }

    pub fn with_chat_completer<R, A, P>(
        repository: R,
        audit_sink: A,
        policy_provider: P,
        chat_completer: Arc<dyn ChatCompleter>,
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
        .with_chat_completer(chat_completer);
        Self {
            service: Arc::new(service),
        }
    }
}

async fn inject_gateway_agent_context(
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let api_surface = classify_api_surface(request.uri().path());
    match AgentRequestContext::from_gateway_subject_headers(request.headers()) {
        Ok(context) => {
            let web_context = build_web_request_context(&context, &request, api_surface);
            request.extensions_mut().insert(context);
            request.extensions_mut().insert(web_context);
            next.run(request).await
        }
        // 中间件运行在 `WebRequestContext` 注入之前，只能走 fallback 路径
        // （内部会生成一个新的服务端 trace id 用于支持排障关联）。
        Err(problem) => problem.into_response_fallback(),
    }
}

/// Classify the request path into a framework API surface so the
/// `WebRequestContext` carries accurate telemetry for Problem+json correlation.
fn classify_api_surface(path: &str) -> sdkwork_web_core::WebApiSurface {
    use sdkwork_web_core::WebApiSurface;
    if path.starts_with("/app/") {
        WebApiSurface::AppApi
    } else if path.starts_with("/backend/") {
        WebApiSurface::BackendApi
    } else if path.starts_with("/agent/") || path.starts_with("/open/") {
        WebApiSurface::OpenApi
    } else if path.starts_with("/gateway/") {
        WebApiSurface::GatewayApi
    } else {
        WebApiSurface::Unknown
    }
}

/// Lightweight request tracing middleware.
///
/// Emits a single structured log line per request with method, path, status
/// and latency. This is the minimal observability baseline required by P1-3
/// without introducing `tower-http` (which would pull in additional build
/// scripts that are currently broken on the Windows toolchain). Throughput
/// limiting (rate limiting) and CORS are intentionally deferred to the
/// sdkwork-web-framework layer where they can be configured uniformly across
/// all managed-store surfaces.
async fn trace_request(request: Request<axum::body::Body>, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let started = std::time::Instant::now();
    let response = next.run(request).await;
    let elapsed = started.elapsed();
    let status = response.status().as_u16();
    crate::infrastructure::AgentMetricsRegistry::global().record_http_request(status);
    tracing::info!(
        method = %method,
        path = %path,
        status = response.status().as_u16(),
        elapsed_ms = elapsed.as_millis() as u64,
        "agents.managed_store.request"
    );
    response
}

/// Maximum JSON request body size for managed-agent HTTP APIs (2 MiB).
pub const MAX_HTTP_REQUEST_BODY_BYTES: usize = 2 * 1024 * 1024;

fn with_gateway_trusted_context(router: Router<AgentHttpState>) -> Router<AgentHttpState> {
    router
        .layer(DefaultBodyLimit::max(MAX_HTTP_REQUEST_BODY_BYTES))
        .layer(middleware::from_fn(trace_request))
        .layer(middleware::from_fn(inject_gateway_agent_context))
}

/// Raw app-api route tree without gateway or web-framework middleware.
pub fn build_app_routes() -> Router<AgentHttpState> {
    Router::new()
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
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
            get(app_get_session),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/close",
            post(app_close_session),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages",
            get(app_list_messages).post(app_create_message),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages/{messageId}",
            get(app_get_message),
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
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/approve",
            post(app_approve_interaction),
        )
        .route(
            "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/answer",
            post(app_answer_interaction),
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
}

pub fn build_app_router() -> Router<AgentHttpState> {
    build_app_routes()
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
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages",
            get(backend_list_messages).post(backend_create_message),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages/{messageId}",
            get(backend_get_message),
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
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/approve",
            post(backend_approve_interaction),
        )
        .route(
            "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/answer",
            post(backend_answer_interaction),
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
}

/// Legacy gateway-trusted open-api router for contract tests.
///
/// Production mounts must use `sdkwork-routes-agents-open-api::build_served_router` instead.
pub fn build_open_router() -> Router<AgentHttpState> {
    with_gateway_trusted_context(build_open_routes())
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
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages",
            get(backend_list_messages).post(backend_create_message),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages/{messageId}",
            get(backend_get_message),
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
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/approve",
            post(backend_approve_interaction),
        )
        .route(
            "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/answer",
            post(backend_answer_interaction),
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
}

/// Legacy gateway-trusted backend-api router for contract tests.
///
/// Production mounts must use `sdkwork-routes-agents-backend-api::build_served_router` instead.
pub fn build_backend_router() -> Router<AgentHttpState> {
    with_gateway_trusted_context(build_backend_routes())
}

/// Legacy combined router for gateway-trusted contract tests (`http_axum_contracts.rs`).
///
/// Production mounts must merge raw route builders and wrap with
/// `sdkwork-routes-agents-http-shared::build_served_combined_router`.
pub fn build_combined_router(state: AgentHttpState) -> Router {
    build_open_router()
        .merge(build_app_router())
        .merge(build_backend_router())
        .route("/metrics", get(serve_metrics))
        .with_state(state)
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

async fn serve_metrics() -> impl IntoResponse {
    serve_agents_metrics().await
}

/// Raw combined route tree for served production mounts.
pub fn build_combined_routes() -> Router<AgentHttpState> {
    build_open_routes()
        .merge(build_app_routes())
        .merge(build_backend_routes())
        .route("/metrics/agents", get(serve_agents_metrics))
}

/// Testing helpers for integrating with `WebRequestContext` in contract tests.
///
/// Production mounts inject `WebRequestContext` via `sdkwork-web-framework`'s
/// 18-stage pipeline. Tests that exercise `build_combined_router` directly must
/// inject a minimal context manually via `Extension(test_web_context())`.
pub mod testing {
    use sdkwork_web_core::{
        ServerRequestId, WebApiSurface, WebAuthMode, WebRequestContext, WebTransportFacts,
    };

    /// Construct a minimal `WebRequestContext` for app-api contract tests.
    pub fn test_web_context() -> WebRequestContext {
        WebRequestContext {
            request_id: ServerRequestId("req-test-fixed".to_owned()),
            api_surface: WebApiSurface::AppApi,
            auth_mode: WebAuthMode::DualToken,
            principal: None,
            transport: WebTransportFacts {
                path: "/app/v3/api/ai/agents".to_owned(),
                method: "POST".to_owned(),
                auth_token_present: true,
                access_token_present: true,
                api_key_present: false,
                oauth_bearer_present: false,
                agent_token_present: false,
            },
            locale: None,
            client_kind: None,
            operation: None,
            trace_id: Some("trace-test-fixed".to_owned()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ListCompositionSlotsQueryParams {
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ListMcpServersQueryParams {
    q: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct ListAgentsQueryParams {
    tenant_id: String,
    organization_id: Option<String>,
    owner_user_id: Option<String>,
    scope: Option<String>,
    include_deleted: Option<bool>,
    q: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct AppListAgentsQueryParams {
    scope: Option<String>,
    include_deleted: Option<bool>,
    q: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct AppCreateMessageQueryParams {
    #[serde(default)]
    stream: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct BackendCreateMessageQueryParams {
    #[serde(default)]
    pub(crate) tenant_id: String,
    #[serde(default)]
    pub(crate) stream: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TenantQueryParams {
    pub(crate) tenant_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TenantListQueryParams {
    tenant_id: String,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
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
#[serde(rename_all = "camelCase")]
pub(crate) struct ListSessionsQueryParams {
    pub(crate) tenant_id: String,
    pub(crate) agent_id: Option<String>,
    pub(crate) owner_user_id: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) include_archived: Option<bool>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppListSessionsQueryParams {
    pub(crate) agent_id: Option<String>,
    pub(crate) owner_user_id: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) include_archived: Option<bool>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListTasksQueryParams {
    pub(crate) tenant_id: String,
    pub(crate) agent_id: Option<String>,
    pub(crate) owner_user_id: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppListTasksQueryParams {
    pub(crate) agent_id: Option<String>,
    pub(crate) owner_user_id: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListMessagesQueryParams {
    pub(crate) tenant_id: String,
    pub(crate) role: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppListMessagesQueryParams {
    pub(crate) role: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppListInteractionsQueryParams {
    pub(crate) status: Option<String>,
    pub(crate) page: Option<usize>,
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListInteractionsQueryParams {
    pub(crate) tenant_id: String,
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
    priority: String,
    enabled: bool,
    policy_json: String,
    status: String,
    version: String,
    created_at: String,
    updated_at: String,
    deleted_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct AuditEventsQueryParams {
    tenant_id: String,
    page: Option<usize>,
    page_size: Option<usize>,
    action: Option<String>,
    from: Option<String>,
    to: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateAgentBody {
    agent_id: String,
    organization_id: Option<String>,
    owner_user_id: Option<String>,
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
#[serde(rename_all = "camelCase")]
struct AppSendChatMessageBody {
    content: String,
    content_type: Option<String>,
    metadata_json: Option<String>,
    model_id: Option<String>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SendChatMessageBody {
    tenant_id: String,
    content: String,
    content_type: Option<String>,
    metadata_json: Option<String>,
    model_id: Option<String>,
    requested_at: String,
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
        let owner_user_id = match query.scope.as_deref() {
            Some("market" | "public" | "published") => None,
            _ => Some(scope.owner_user_id.clone()),
        };
        let query = ListAgentsQueryParams {
            tenant_id: scope.tenant_id.clone(),
            organization_id: Some(scope.organization_id.clone()),
            owner_user_id,
            scope: query.scope,
            include_deleted: query.include_deleted,
            q: query.q,
            page: query.page,
            page_size: query.page_size,
        };
        execute_list(state, query, scope).await
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
        let scope = RequestScope::from_trusted_extension(
            context,
            query.tenant_id.clone(),
            query.organization_id.clone(),
            query.owner_user_id.clone(),
        )?;
        execute_list(state, query, scope).await
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
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    body: Result<Json<CreateAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_trusted_extension(
            context,
            query.tenant_id,
            body.organization_id.clone(),
            body.owner_user_id.clone(),
        )?;
        execute_create(state, scope, body).await
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
    query: Result<Query<TenantQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
        execute_get(state, scope, agent_id).await
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
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<UpdateAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
        execute_update(state, scope, agent_id, body).await
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
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<()> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
        execute_delete(state, scope, agent_id).await
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
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<UpdateAgentStatusBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
        let subject = scope.subject.clone();
        let command = UpdateAgentStatusRequestDto {
            tenant_id: query.tenant_id,
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
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn backend_restore_agent(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<RestoreAgentBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRecordResponse>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
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
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
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
    query: Result<Query<TenantListQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
        execute_list_provider_bindings(state, scope, query.page, query.page_size, path.agent_id)
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
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentProviderBindingBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
        execute_add_provider_binding(state, scope, path.agent_id, body).await
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
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ActivateProviderBindingBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentProviderBindingRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
        execute_activate_provider_binding(state, scope, path, body).await
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
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
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
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn open_create_preview_response(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentPreviewResponseBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRuntimeExecutionRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
        execute_create_preview_response(state, scope, path.agent_id, body).await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
}

async fn open_create_prompt_optimization(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentPromptOptimizationBody>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentRuntimeExecutionRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
        execute_create_prompt_optimization(state, scope, path.agent_id, body).await
    }
    .await;
    match result {
        Ok(data) => created_json(&web_ctx, data)
            .unwrap_or_else(|problem| problem.into_response_for(&web_ctx)),
        Err(problem) => problem.into_response_for(&web_ctx),
    }
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
    query: Result<Query<TenantListQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
        execute_list_composition_slots(state, scope, path.agent_id, query.page, query.page_size)
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
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<ResourceData<AgentCompositionSlotRecordResponse>> = async {
        let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
        execute_get_composition_slot(state, scope, path.agent_id, path.slot_id).await
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
    let tenant_id =
        parse_tenant_id(body.data.tenant_id.as_str()).map_err(ApiProblem::from_kernel_error)?;
    let organization_id = parse_organization_id(body.data.organization_id.as_str())
        .map_err(ApiProblem::from_kernel_error)?;
    validate_requested_at(body.requested_at.as_str()).map_err(ApiProblem::from_kernel_error)?;
    let slot_kind = AgentCompositionSlotKind::try_from_str(body.data.slot_kind.as_str())
        .ok_or_else(|| ApiProblem::bad_request("invalid slotKind"))?;
    let target_module =
        AgentCompositionTargetModule::try_from_str(body.data.target_module.as_str())
            .ok_or_else(|| ApiProblem::bad_request("invalid targetModule"))?;
    let priority = body
        .data
        .priority
        .as_deref()
        .map(|s| {
            s.parse::<i32>()
                .map_err(|_| KernelError::validation("invalid priority"))
        })
        .transpose()
        .map_err(ApiProblem::from_kernel_error)?
        .unwrap_or(0);
    let command = AgentCompositionSlotCreateCommand {
        tenant_id,
        organization_id,
        agent_id,
        slot_id: body.data.slot_id,
        slot_kind,
        target_module,
        target_ref: body.data.target_ref,
        target_version_ref: body.data.target_version_ref,
        priority,
        enabled: body.data.enabled.unwrap_or(true),
        policy_json: body.data.policy_json.unwrap_or_else(|| "{}".to_string()),
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
    let tenant_id =
        parse_tenant_id(body.data.tenant_id.as_str()).map_err(ApiProblem::from_kernel_error)?;
    validate_requested_at(body.requested_at.as_str()).map_err(ApiProblem::from_kernel_error)?;
    let expected_version = body
        .data
        .expected_version
        .as_deref()
        .map(parse_expected_version)
        .transpose()
        .map_err(ApiProblem::from_kernel_error)?;
    let slot_kind = body
        .data
        .slot_kind
        .as_deref()
        .map(|value| {
            AgentCompositionSlotKind::try_from_str(value)
                .ok_or_else(|| KernelError::validation("invalid slotKind"))
        })
        .transpose()
        .map_err(ApiProblem::from_kernel_error)?;
    let target_module = body
        .data
        .target_module
        .as_deref()
        .map(|value| {
            AgentCompositionTargetModule::try_from_str(value)
                .ok_or_else(|| KernelError::validation("invalid targetModule"))
        })
        .transpose()
        .map_err(ApiProblem::from_kernel_error)?;
    let priority = body
        .data
        .priority
        .as_deref()
        .map(|s| {
            s.parse::<i32>()
                .map_err(|_| KernelError::validation("invalid priority"))
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
        target_ref: body.data.target_ref,
        target_version_ref: body.data.target_version_ref,
        priority,
        enabled: body.data.enabled,
        policy_json: body.data.policy_json,
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
        priority: record.priority.clone(),
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
// Session handlers — App API
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
        let owner_user_id = query
            .owner_user_id
            .unwrap_or_else(|| scope.owner_user_id.clone());
        let mut command = ListSessionsRequestDto {
            tenant_id: scope.tenant_id,
            owner_user_id: Some(owner_user_id),
            status: query.status,
            include_archived: query.include_archived.unwrap_or(false),
        }
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.query = command
            .query
            .for_agent(query.agent_id.unwrap_or(agent_id))
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

async fn app_create_session(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<CreateSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Json(mut body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        body.data.tenant_id = scope.tenant_id.clone();
        body.data.owner_user_id = scope.owner_user_id.clone();
        let command = body
            .into_command(agent_id, scope.subject)
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
            .into_command(session_id, scope.subject)
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
// Task handlers — App API
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
        let owner_user_id = query
            .owner_user_id
            .unwrap_or_else(|| scope.owner_user_id.clone());
        let mut command = ListTasksRequestDto {
            tenant_id: scope.tenant_id,
            owner_user_id: Some(owner_user_id),
            status: query.status,
        }
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.query = command
            .query
            .for_agent(query.agent_id.unwrap_or(agent_id))
            .with_pagination(
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
        let Json(mut body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        body.data.tenant_id = scope.tenant_id.clone();
        body.data.owner_user_id = scope.owner_user_id.clone();
        let command = body
            .into_command(agent_id, scope.subject)
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
            .into_command(agent_id, task_id, scope.subject)
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
            .into_execute_command(agent_id, task_id, scope.subject)
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
// Interaction handlers — App API
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

async fn app_create_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    body: Result<Json<CreateInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Json(mut body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        body.data.tenant_id = scope.tenant_id.clone();
        body.data.organization_id = scope.organization_id.clone();
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(agent_id.clone(), session_id, scope.subject)
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record =
            with_service(&state, move |service| service.create_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record),
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
            path_agent_id: agent_id,
            session_id,
            interaction_id,
            owner_scope,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record),
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
        let Json(mut body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        body.tenant_id = scope.tenant_id.clone();
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(agent_id, session_id, interaction_id, scope.subject)
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record =
            with_service(&state, move |service| service.approve_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record),
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
        let Json(mut body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        body.tenant_id = scope.tenant_id.clone();
        let owner_scope = scope.owner_scope()?;
        let mut command = body
            .into_command(agent_id, session_id, interaction_id, scope.subject)
            .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        let record =
            with_service(&state, move |service| service.answer_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Message handlers — App API
// ===========================================================================

async fn app_list_messages(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppListMessagesQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<PageData<AgentMessageRecordDto>> = async {
        let Path((_agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut command = ListMessagesRequestDto {
            tenant_id: scope.tenant_id,
            role: query.role,
            status: query.status,
        }
        .into_command(session_id, scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = owner_scope;
        command.query = command.query.with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        let records = with_service(&state, move |service| service.list_messages(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentMessageRecordDto::from_record)
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

async fn app_create_message(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<AppCreateMessageQueryParams>, QueryRejection>,
    body: Result<Json<AppSendChatMessageBody>, JsonRejection>,
) -> Response {
    let result: Result<Response, ApiProblem> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        validate_requested_at(body.requested_at.as_str()).map_err(ApiProblem::from_kernel_error)?;
        let command = SendChatMessageCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            content: body.content,
            content_type: body
                .content_type
                .unwrap_or_else(|| "text/plain".to_string()),
            metadata_json: body.metadata_json.unwrap_or_else(|| "{}".to_string()),
            model_id: body.model_id,
            owner_scope,
            requested_by: scope.subject,
            requested_at: body.requested_at,
            prefer_stream: query.stream.unwrap_or(false),
        };
        let chat_result =
            with_service(&state, move |service| service.send_chat_message(command)).await?;
        chat_completion_http_response(&web_ctx, &chat_result, query.stream.unwrap_or(false))
    }
    .await;
    crate::response::finish_api_response(&web_ctx, result)
}

async fn app_get_message(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentMessageRecordDto>> = async {
        let Path((agent_id, session_id, message_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let scope = RequestScope::from_context(context);
        let owner_scope = scope.owner_scope()?;
        let command = GetMessageCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            message_id,
            owner_scope,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_message(command)).await?;
        Ok(ResourceData {
            item: AgentMessageRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Session handlers — Backend API
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
        let scope = RequestScope::from_trusted_extension(
            context,
            query.tenant_id.clone(),
            None,
            query.owner_user_id.clone(),
        )?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut command = ListSessionsRequestDto {
            tenant_id: scope.tenant_id,
            owner_user_id: query.owner_user_id,
            status: query.status,
            include_archived: query.include_archived.unwrap_or(false),
        }
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.query = command
            .query
            .for_agent(query.agent_id.unwrap_or(agent_id))
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
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    body: Result<Json<CreateSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(mut body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_trusted_extension(
            context,
            query.tenant_id,
            Some(body.data.organization_id.clone()),
            Some(body.data.owner_user_id.clone()),
        )?;
        body.data.tenant_id = scope.tenant_id.clone();
        let command = body
            .into_command(agent_id, scope.subject)
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
    query: Result<Query<TenantQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_trusted_extension(context, query.tenant_id, None, None)?;
        let command = GetSessionCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
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
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    body: Result<Json<CloseSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((_agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_trusted_extension(context, query.tenant_id, None, None)?;
        let command = body
            .into_command(session_id, scope.subject)
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
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    body: Result<Json<ArchiveSessionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentSessionRecordDto>> = async {
        let Path((_agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_trusted_extension(context, query.tenant_id, None, None)?;
        let command = body
            .into_command(session_id, scope.subject)
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
// Message handlers — Backend API
// ===========================================================================

async fn backend_list_messages(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<ListMessagesQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
) -> Response {
    let result: ApiResult<PageData<AgentMessageRecordDto>> = async {
        let Path((_agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut command = ListMessagesRequestDto {
            tenant_id: scope.tenant_id,
            role: query.role,
            status: query.status,
        }
        .into_command(session_id, scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.owner_scope = None;
        command.query = command.query.with_pagination(
            PaginationParams::default()
                .with_page_size(page_size)
                .with_page(page),
        );
        let records = with_service(&state, move |service| service.list_messages(command)).await?;
        Ok(PageData {
            items: records
                .items
                .iter()
                .map(AgentMessageRecordDto::from_record)
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

async fn backend_create_message(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<BackendCreateMessageQueryParams>, QueryRejection>,
    body: Result<Json<SendChatMessageBody>, JsonRejection>,
) -> Response {
    let result: Result<Response, ApiProblem> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
        let tenant_id =
            resolve_tenant_from_query_or_body(query.tenant_id.as_str(), body.tenant_id.as_str())?;
        let scope = RequestScope::from_trusted_extension(context, tenant_id, None, None)?;
        validate_requested_at(body.requested_at.as_str()).map_err(ApiProblem::from_kernel_error)?;
        let command = SendChatMessageCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            agent_id,
            session_id,
            content: body.content,
            content_type: body
                .content_type
                .unwrap_or_else(|| "text/plain".to_string()),
            metadata_json: body.metadata_json.unwrap_or_else(|| "{}".to_string()),
            model_id: body.model_id,
            owner_scope: None,
            requested_by: scope.subject,
            requested_at: body.requested_at,
            prefer_stream: query.stream.unwrap_or(false),
        };
        let chat_result =
            with_service(&state, move |service| service.send_chat_message(command)).await?;
        chat_completion_http_response(&web_ctx, &chat_result, query.stream.unwrap_or(false))
    }
    .await;
    crate::response::finish_api_response(&web_ctx, result)
}

async fn backend_get_message(
    State(state): State<AgentHttpState>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String, String)>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentMessageRecordDto>> = async {
        let Path((agent_id, session_id, message_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_trusted_extension(context, query.tenant_id, None, None)?;
        let command = GetMessageCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            message_id,
            owner_scope: None,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_message(command)).await?;
        Ok(ResourceData {
            item: AgentMessageRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Task handlers — Backend API
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
        let scope = RequestScope::from_trusted_extension(
            context,
            query.tenant_id.clone(),
            None,
            query.owner_user_id.clone(),
        )?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut command = ListTasksRequestDto {
            tenant_id: scope.tenant_id,
            owner_user_id: query.owner_user_id,
            status: query.status,
        }
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;
        command.query = command
            .query
            .for_agent(query.agent_id.unwrap_or(agent_id))
            .with_pagination(
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
        let scope = RequestScope::from_trusted_extension(
            context,
            body.data.tenant_id.clone(),
            None,
            Some(body.data.owner_user_id.clone()),
        )?;
        let command = body
            .into_command(agent_id, scope.subject)
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
    query: Result<Query<TenantQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentTaskRecordDto>> = async {
        let Path((agent_id, task_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_trusted_extension(context, query.tenant_id, None, None)?;
        let command = GetTaskCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            task_id,
            owner_scope: None,
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
        let scope =
            RequestScope::from_trusted_extension(context, body.tenant_id.clone(), None, None)?;
        let command = body
            .into_command(agent_id, task_id, scope.subject)
            .map_err(ApiProblem::from_kernel_error)?;
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
        let scope =
            RequestScope::from_trusted_extension(context, body.tenant_id.clone(), None, None)?;
        let command = body
            .into_execute_command(agent_id, task_id, scope.subject)
            .map_err(ApiProblem::from_kernel_error)?;
        let record = with_service(&state, move |service| service.execute_task(command)).await?;
        Ok(ResourceData {
            item: AgentTaskRecordDto::from_record(&record),
        })
    }
    .await;
    finish_api_json(&web_ctx, result)
}

// ===========================================================================
// Interaction handlers — Backend API
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
        let scope =
            RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
        let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
        let mut command = ListInteractionsRequestDto {
            tenant_id: scope.tenant_id,
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

async fn backend_create_interaction(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    Extension(web_ctx): Extension<sdkwork_web_core::WebRequestContext>,
    path: Result<Path<(String, String)>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    body: Result<Json<CreateInteractionRequestDto>, JsonRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id)) = path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let Json(mut body) = body.map_err(ApiProblem::from_json_rejection)?;
        let scope = RequestScope::from_trusted_extension(
            context,
            query.tenant_id,
            Some(body.data.organization_id.clone()),
            None,
        )?;
        body.data.tenant_id = scope.tenant_id.clone();
        let command = body
            .into_command(agent_id, session_id, scope.subject)
            .map_err(ApiProblem::from_kernel_error)?;
        let record =
            with_service(&state, move |service| service.create_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record),
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
    query: Result<Query<TenantQueryParams>, QueryRejection>,
) -> Response {
    let result: ApiResult<ResourceData<AgentInteractionRecordDto>> = async {
        let Path((agent_id, session_id, interaction_id)) =
            path.map_err(ApiProblem::from_path_rejection)?;
        let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
        let scope = RequestScope::from_trusted_extension(context, query.tenant_id, None, None)?;
        let command = GetInteractionCommand {
            tenant_id: parse_tenant_id(&scope.tenant_id).map_err(ApiProblem::from_kernel_error)?,
            path_agent_id: agent_id,
            session_id,
            interaction_id,
            owner_scope: None,
            requested_by: scope.subject,
        };
        let record = with_service(&state, move |service| service.get_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record),
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
        let scope =
            RequestScope::from_trusted_extension(context, body.tenant_id.clone(), None, None)?;
        let command = body
            .into_command(agent_id, session_id, interaction_id, scope.subject)
            .map_err(ApiProblem::from_kernel_error)?;
        let record =
            with_service(&state, move |service| service.approve_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record),
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
        let scope =
            RequestScope::from_trusted_extension(context, body.tenant_id.clone(), None, None)?;
        let command = body
            .into_command(agent_id, session_id, interaction_id, scope.subject)
            .map_err(ApiProblem::from_kernel_error)?;
        let record =
            with_service(&state, move |service| service.answer_interaction(command)).await?;
        Ok(ResourceData {
            item: AgentInteractionRecordDto::from_record(&record),
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
    let service = Arc::clone(&state.service);
    tokio::task::spawn_blocking(move || action(service.as_ref()))
        .await
        .map_err(|error| ApiProblem::internal(format!("agents service worker failed: {error}")))?
        .map_err(ApiProblem::from_kernel_error)
}
async fn execute_list(
    state: AgentHttpState,
    query: ListAgentsQueryParams,
    scope: RequestScope,
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
        organization_id: query.organization_id,
        owner_user_id: query.owner_user_id,
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
    let now = OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

fn reconcile_resource_tenant_with_subject_header(
    resource_tenant_id: &str,
    header_tenant_id: Option<String>,
) -> Result<String, ApiProblem> {
    let resource_tenant =
        parse_tenant_id(resource_tenant_id).map_err(ApiProblem::from_kernel_error)?;
    let Some(header_tenant_id) = header_tenant_id else {
        // Defense in depth: the gateway middleware (`from_gateway_subject_headers`)
        // already rejects requests without a tenant header, but a backend route
        // must never trust the resource-supplied tenant_id on its own. Without
        // a subject tenant header to cross-check, a caller could address any
        // tenant's resource by simply omitting the header. Reject explicitly.
        return Err(ApiProblem::validation(
            "subject tenant header is required for backend resource access",
        ));
    };
    let header_tenant = parse_tenant_id(header_tenant_id.as_str())
        .map_err(|_| ApiProblem::permission("subject tenant does not match resource tenant"))?;
    if header_tenant != resource_tenant {
        return Err(ApiProblem::permission(
            "subject tenant does not match resource tenant",
        ));
    }
    Ok(resource_tenant_id.to_string())
}

fn required_header_any(headers: &HeaderMap, keys: &[&str]) -> Result<String, ApiProblem> {
    optional_header_any(headers, keys).ok_or_else(|| {
        ApiProblem::validation(format!("required header missing: {}", keys.join(" or ")))
    })
}

fn optional_header_any(headers: &HeaderMap, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| optional_header(headers, key))
}

fn optional_header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
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

fn resolve_tenant_from_query_or_body(
    query_tenant_id: &str,
    body_tenant_id: &str,
) -> Result<String, ApiProblem> {
    let query_tenant_id = query_tenant_id.trim();
    let body_tenant_id = body_tenant_id.trim();
    match (query_tenant_id.is_empty(), body_tenant_id.is_empty()) {
        (false, false) if query_tenant_id != body_tenant_id => Err(ApiProblem::validation(
            "tenant_id mismatch between query and request body",
        )),
        (false, _) => Ok(query_tenant_id.to_string()),
        (_, false) => Ok(body_tenant_id.to_string()),
        (true, true) => Err(ApiProblem::validation(
            "tenant_id is required in query or request body",
        )),
    }
}

/// 聊天完成响应的内部数据形状：`{ session, userMessage, assistantMessage }`。
///
/// 该结构序列化后会作为 `ResourceData.item` 被包装进 `SdkWorkApiResponse` 信封。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ChatCompletionData {
    session: AgentSessionRecordDto,
    user_message: AgentMessageRecordDto,
    assistant_message: AgentMessageRecordDto,
}

impl ChatCompletionData {
    fn from_result(result: &ChatCompletionResult) -> Self {
        Self {
            session: AgentSessionRecordDto::from_record(&result.session),
            user_message: AgentMessageRecordDto::from_record(&result.user_message),
            assistant_message: AgentMessageRecordDto::from_record(&result.assistant_message),
        }
    }
}

/// 构造聊天完成响应：
/// - 非流式：`201 Created` + `SdkWorkApiResponse { code: 0, data: { item: ChatCompletionData }, traceId }`。
/// - 流式（SSE）：`201 Created` + `text/event-stream`，`data:` 行为完整 `SdkWorkApiResponse` 信封 JSON。
///
/// 调用方必须传入 `WebRequestContext` 以便信封和 Problem+json 都能携带真实 `traceId`。
fn chat_completion_http_response(
    ctx: &sdkwork_web_core::WebRequestContext,
    result: &ChatCompletionResult,
    stream_requested: bool,
) -> Result<Response, ApiProblem> {
    let chat_data = ChatCompletionData::from_result(result);
    let trace_id = ctx.resolved_trace_id();

    if stream_requested {
        // SSE 路径：将完整信封作为 `data:` 行的 JSON 负载。
        let envelope = sdkwork_utils_rust::SdkWorkApiResponse::success(
            ResourceData { item: chat_data },
            trace_id.clone(),
        );
        let payload = serde_json::to_string(&envelope).map_err(|error| {
            ApiProblem::internal(format!("failed to encode chat completion: {error}"))
        })?;
        let body = format!("event: completion\ndata: {payload}\n\n");
        let mut response = Response::builder()
            .status(StatusCode::CREATED)
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

    // 非流式 JSON 路径：复用 `created_json` 生成 `201 + 信封 + trace 头`。
    created_json(ctx, ResourceData { item: chat_data })
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
    use super::*;
    use crate::infrastructure::{
        AllowAllPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
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

    fn auth_headers(mut request: Request<Body>) -> Request<Body> {
        let headers = request.headers_mut();
        headers.insert("x-subject-id", HeaderValue::from_static("u-1"));
        headers.insert("x-subject-tenant-id", HeaderValue::from_static("100001"));
        request
    }

    struct EnvVarRestore {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarRestore {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarRestore {
        fn drop(&mut self) {
            match self.previous.as_ref() {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    fn create_agent_body(agent_id: &str, code: &str) -> Value {
        json!({
            "agentId": agent_id,
            "organizationId": "0",
            "ownerUserId": "100",
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
            .oneshot(auth_headers(request))
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
            .oneshot(auth_headers(request))
            .await
            .expect("provider binding request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    fn test_agent_context() -> AgentRequestContext {
        AgentRequestContext::new("100001", "100")
            .with_organization_id("0")
            .with_subject_id("u-1")
            .with_roles(["agent.write", "agent.read"])
            .with_trace_id("trace-test-fixed")
            .with_request_id("req-test-fixed")
    }

    /// 构造测试用 `WebRequestContext`，供 app-api 路由（未挂载网关中间件）的测试注入。
    fn test_web_context() -> sdkwork_web_core::WebRequestContext {
        sdkwork_web_core::WebRequestContext {
            request_id: sdkwork_web_core::ServerRequestId("req-test-fixed".to_owned()),
            api_surface: sdkwork_web_core::WebApiSurface::AppApi,
            auth_mode: sdkwork_web_core::WebAuthMode::DualToken,
            principal: None,
            transport: sdkwork_web_core::WebTransportFacts {
                path: "/app/v3/api/ai/agents".to_owned(),
                method: "POST".to_owned(),
                auth_token_present: true,
                access_token_present: true,
                api_key_present: false,
                oauth_bearer_present: false,
                agent_token_present: false,
            },
            locale: None,
            client_kind: None,
            operation: None,
            trace_id: Some("trace-test-fixed".to_owned()),
        }
    }

    #[tokio::test]
    async fn app_create_and_retrieve_agent_should_work() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            AllowAllPolicyProvider::allow("policy.memory"),
        );
        let app = build_combined_router(state)
            .layer(Extension(test_agent_context()))
            .layer(Extension(test_web_context()));

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
            .oneshot(auth_headers(request))
            .await
            .expect("create request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);

        let request = Request::builder()
            .method("GET")
            .uri("/app/v3/api/ai/agents/agent.alpha")
            .body(Body::empty())
            .expect("request should be built");
        let response = app
            .oneshot(auth_headers(request))
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
            AllowAllPolicyProvider::allow("policy.memory"),
        );
        let app = build_combined_router(state);

        let create_body = json!({
            "agentId": "agent.open",
            "organizationId": "0",
            "ownerUserId": "100",
            "code": "open",
            "displayName": "Open Agent",
            "description": "developer api",
            "manifest": test_manifest(),
            "visibility": "organization",
            "tags": ["developer"],
            "requestedAt": "2026-06-01T00:00:00Z"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/agent/v3/api/ai/agents?tenant_id=100001")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(create_body.to_string()))
            .expect("request should be built");
        let response = app
            .clone()
            .oneshot(auth_headers(request))
            .await
            .expect("create request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);

        let request = Request::builder()
            .method("GET")
            .uri("/agent/v3/api/ai/agents/agent.open?tenant_id=100001")
            .body(Body::empty())
            .expect("request should be built");
        let response = app
            .oneshot(auth_headers(request))
            .await
            .expect("get request should succeed");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn app_delete_agent_uses_204_without_json_body() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            AllowAllPolicyProvider::allow("policy.memory"),
        );
        let app = build_combined_router(state)
            .layer(Extension(test_agent_context()))
            .layer(Extension(test_web_context()));

        create_app_agent(&app, "agent.delete", "delete").await;

        let request = Request::builder()
            .method("DELETE")
            .uri("/app/v3/api/ai/agents/agent.delete")
            .body(Body::empty())
            .expect("request should be built");
        let response = app
            .oneshot(auth_headers(request))
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
        let _env_lock = sdkwork_agents_contract::env_test_lock();
        let _deployment_env = EnvVarRestore::set("SDKWORK_DEPLOYMENT_ENV", "development");
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            AllowAllPolicyProvider::allow("policy.memory"),
        );
        let app = build_combined_router(state)
            .layer(Extension(test_agent_context()))
            .layer(Extension(test_web_context()));

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
            .oneshot(auth_headers(request))
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
            .oneshot(auth_headers(request))
            .await
            .expect("prompt optimization request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn app_delete_composition_slot_uses_204_without_json_body() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            AllowAllPolicyProvider::allow("policy.memory"),
        );
        let app = build_combined_router(state)
            .layer(Extension(test_agent_context()))
            .layer(Extension(test_web_context()));

        create_app_agent(&app, "agent.slot-delete", "slot-delete").await;
        let slot_body = json!({
            "data": {
                "tenantId": "100001",
                "organizationId": "0",
                "slotId": "slot.skill.primary",
                "slotKind": "skill",
                "targetModule": "skills",
                "targetRef": "skill.primary",
                "priority": "10",
                "enabled": true,
                "policyJson": "{}"
            },
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
            .oneshot(auth_headers(request))
            .await
            .expect("composition slot create request should succeed");
        assert_eq!(response.status(), StatusCode::CREATED);

        let request = Request::builder()
            .method("DELETE")
            .uri("/app/v3/api/ai/agents/agent.slot-delete/composition_slots/slot.skill.primary?requestedAt=2026-06-01T00:05:00Z")
            .body(Body::empty())
            .expect("request should be built");
        let response = app
            .oneshot(auth_headers(request))
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
            AllowAllPolicyProvider::allow("policy.memory"),
        );
        let app = build_combined_router(state);

        let create_body = json!({
            "agentId": "agent.beta",
            "organizationId": "0",
            "ownerUserId": "100",
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
            .uri("/backend/v3/api/ai/agents?tenant_id=100001")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(create_body.to_string()))
            .expect("request should be built");
        let create_response = app
            .clone()
            .oneshot(auth_headers(create_request))
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
            .uri("/backend/v3/api/ai/agents/agent.beta/status?tenant_id=100001")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(update_status_body.to_string()))
            .expect("request should be built");
        let status_response = app
            .oneshot(auth_headers(status_request))
            .await
            .expect("status request should succeed");

        assert_eq!(status_response.status(), StatusCode::CREATED);
        let body_bytes = to_bytes(status_response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let body_json: Value =
            serde_json::from_slice(&body_bytes).expect("response body should be valid json");
        // 信封形状：{ code: 0, data: { item: { status, ... } }, traceId }
        assert_eq!(body_json["code"], 0);
        assert_eq!(body_json["data"]["item"]["status"], "active");
    }

    // --- P1-4 tenant_id 安全防护单元测试 ---

    fn subject_headers_with(tenant: Option<&str>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("x-subject-id", HeaderValue::from_static("u-1"));
        if let Some(tenant) = tenant {
            headers.insert(
                "x-subject-tenant-id",
                HeaderValue::from_str(tenant).unwrap(),
            );
        }
        headers
    }

    #[test]
    fn from_gateway_subject_headers_rejects_missing_tenant_header() {
        // 缺失 tenant header 必须在网关边界被拒绝，避免空 tenant_id 进入应用层
        let headers = subject_headers_with(None);
        let result = AgentRequestContext::from_gateway_subject_headers(&headers);
        let err = result.expect_err("missing tenant header should be rejected");
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
        assert!(err.message.contains("tenant"));
    }

    #[test]
    fn from_gateway_subject_headers_accepts_sdkwork_tenant_header() {
        // x-sdkwork-tenant-id 是 x-subject-tenant-id 的等价替代头
        let mut headers = HeaderMap::new();
        headers.insert("x-subject-id", HeaderValue::from_static("u-1"));
        headers.insert("x-sdkwork-tenant-id", HeaderValue::from_static("100001"));
        let context = AgentRequestContext::from_gateway_subject_headers(&headers)
            .expect("sdkwork tenant header should be accepted");
        assert_eq!(context.tenant_id, "100001");
    }

    #[test]
    fn from_gateway_subject_headers_rejects_tenant_zero() {
        // tenant_id=0 是保留值，即使 header 存在也必须拒绝
        let headers = subject_headers_with(Some("0"));
        // from_gateway_subject_headers 仅做 header 存在性校验，tenant_id=0
        // 在后续 parse_tenant_id 时被拒绝。这里验证 header 层不阻拦解析，
        // 由 validation.rs::parse_tenant_id_rejects_zero 覆盖数值校验。
        let context = AgentRequestContext::from_gateway_subject_headers(&headers)
            .expect("header presence is the gateway concern");
        assert_eq!(context.tenant_id, "0");
        // 数值校验在 tenant_id_u64 / parse_tenant_id 处生效
        let err = parse_tenant_id(context.tenant_id.as_str())
            .expect_err("tenant_id 0 must be rejected by parse_tenant_id");
        match err {
            KernelError::Validation { message } => {
                assert!(message.contains("greater than 0"));
            }
            _ => panic!("expected validation error for tenant_id 0"),
        }
    }

    #[test]
    fn reconcile_resource_tenant_rejects_missing_header() {
        // 严重越权场景：缺失 subject tenant header 时不得直接信任 resource tenant
        let err = reconcile_resource_tenant_with_subject_header("100001", None)
            .expect_err("missing header must be rejected");
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
        assert!(err.message.contains("subject tenant header is required"));
    }

    #[test]
    fn reconcile_resource_tenant_rejects_mismatch() {
        let err =
            reconcile_resource_tenant_with_subject_header("100001", Some("100002".to_string()))
                .expect_err("tenant mismatch must be rejected");
        assert_eq!(err.status(), StatusCode::FORBIDDEN);
        assert!(err.message.contains("does not match"));
    }

    #[test]
    fn reconcile_resource_tenant_rejects_resource_zero() {
        // resource tenant_id=0 也必须被拒绝（parse_tenant_id 拦截）
        let err = reconcile_resource_tenant_with_subject_header("0", Some("100001".to_string()))
            .expect_err("resource tenant 0 must be rejected");
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
        assert!(err.message.contains("greater than 0"));
    }

    #[test]
    fn reconcile_resource_tenant_accepts_match() {
        let result =
            reconcile_resource_tenant_with_subject_header("100001", Some("100001".to_string()))
                .expect("matching tenants should be accepted");
        assert_eq!(result, "100001");
    }
}
