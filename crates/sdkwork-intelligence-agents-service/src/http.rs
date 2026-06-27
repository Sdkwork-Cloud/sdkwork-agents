use crate::application::{
    AgentCompositionSlotCreateCommand, AgentCompositionSlotDeleteCommand,
    AgentCompositionSlotGetCommand, AgentCompositionSlotListCommand,
    AgentCompositionSlotUpdateCommand, AgentsService,
};
use crate::domain::{
    AgentCompositionSlotKind, AgentCompositionSlotRecord, AgentCompositionTargetModule,
    AgentProviderBindingRecord,
};
use crate::dto::{
    ActivateAgentProviderBindingRequestDto, AgentCompositionSlotCreateRequestDto,
    AgentCompositionSlotRecordDto,
    AgentCompositionSlotUpdateRequestDto,
    AgentManagementProfileDto, AgentPreviewResponseRequestDto,
    AgentPromptOptimizationRequestDto, AgentProviderBindingRecordDto,
    AgentProviderBindingRequestDto, AgentRecordDto,
    AgentRuntimeExecutionRecordDto, CreateAgentRequestDto, DeleteAgentRequestDto,
    GetAgentRequestDto, ListAgentsRequestDto, RestoreAgentRequestDto, UpdateAgentRequestDto,
    UpdateAgentStatusRequestDto,
};
use crate::ports::{AgentAuditSink, AgentRepository};
use crate::validation::{
    parse_expected_version, parse_optional_rfc3339_datetime, parse_rfc3339_datetime,
    parse_tenant_id, validate_requested_at, validate_standard_id,
};
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{Extension, Path, Query, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{middleware, Json, Router};
use sdkwork_agent_kernel::{
    AgentManifest, KernelError, KernelErrorKind, KernelResult, PolicyDecision, PolicyProvider,
    PolicyRequest, PolicySubject, ProviderHealth,
};
use sdkwork_code_kernel::CodeTaskIntent;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
#[cfg(feature = "postgres-sync")]
use std::sync::Mutex as ServiceMutex;
use time::OffsetDateTime;
#[cfg(not(feature = "postgres-sync"))]
use tokio::sync::Mutex as ServiceMutex;

const HEADER_SUBJECT_ID: &str = "x-subject-id";
const HEADER_SUBJECT_TENANT_ID: &str = "x-subject-tenant-id";
const HEADER_SUBJECT_ROLES: &str = "x-subject-roles";
const HEADER_SDKWORK_USER_ID: &str = "x-sdkwork-user-id";
const HEADER_SDKWORK_ACTOR_ID: &str = "x-sdkwork-actor-id";
const HEADER_SDKWORK_TENANT_ID: &str = "x-sdkwork-tenant-id";
const HEADER_SDKWORK_PERMISSION_SCOPE: &str = "x-sdkwork-permission-scope";
const MAX_PAGE_SIZE: usize = 200;
const DEFAULT_PAGE_SIZE: usize = 20;
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
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRequestContext {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub subject_id: String,
    pub roles: Vec<String>,
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

    /// Build trusted context from gateway-injected subject headers before resource tenant reconciliation.
    pub(crate) fn from_gateway_subject_headers(headers: &HeaderMap) -> Result<Self, ApiProblem> {
        let subject_id = required_header_any(
            headers,
            &[
                HEADER_SUBJECT_ID,
                HEADER_SDKWORK_USER_ID,
                HEADER_SDKWORK_ACTOR_ID,
            ])?;
        // tenant_id is the mandatory multi-tenant isolation key. The gateway
        // must always inject either `x-subject-tenant-id` or
        // `x-sdkwork-tenant-id`. Falling back to an empty string (the previous
        // behavior) allowed a request with a missing tenant header to enter the
        // application layer as tenant_id="", which downstream code then parsed
        // as 0 or used to bypass tenant-scoped filters. Reject at the edge.
        let tenant_id = required_header_any(
            headers,
            &[HEADER_SUBJECT_TENANT_ID, HEADER_SDKWORK_TENANT_ID])?;
        let mut roles = Vec::new();
        if let Some(roles_header) = optional_header_any(
            headers,
            &[HEADER_SUBJECT_ROLES, HEADER_SDKWORK_PERMISSION_SCOPE]) {
            for role in roles_header
                .split([',', ' '])
                .map(str::trim)
                .filter(|role| !role.is_empty())
            {
                roles.push(role.to_string());
            }
        }
        Ok(Self {
            tenant_id,
            organization_id: None,
            owner_user_id: subject_id.clone(),
            subject_id,
            roles,
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

#[derive(Debug, Clone)]
struct RequestScope {
    tenant_id: String,
    organization_id: String,
    owner_user_id: String,
    subject: PolicySubject,
}

impl RequestScope {
    fn from_context(context: AgentRequestContext) -> Self {
        let subject = context.subject();
        Self {
            tenant_id: context.tenant_id.clone(),
            organization_id: context.organization_id.unwrap_or_else(|| "0".to_string()),
            owner_user_id: context.owner_user_id.clone(),
            subject,
        }
    }

    fn from_trusted_extension(
        mut context: AgentRequestContext,
        resource_tenant_id: String,
        organization_id: Option<String>,
        owner_user_id: Option<String>) -> Result<Self, ApiProblem> {
        let header_tenant = if context.tenant_id.is_empty() {
            None
        } else {
            Some(context.tenant_id.clone())
        };
        let tenant_id = reconcile_resource_tenant_with_subject_header(
            resource_tenant_id.as_str(),
            header_tenant)?;
        context.tenant_id = tenant_id;
        if let Some(organization_id) = organization_id {
            context.organization_id = Some(organization_id);
        }
        if let Some(owner_user_id) = owner_user_id {
            context.owner_user_id = owner_user_id;
        }
        Ok(Self::from_context(context))
    }

    fn tenant_id_u64(&self) -> Result<u64, ApiProblem> {
        parse_tenant_id(self.tenant_id.as_str()).map_err(ApiProblem::from_kernel_error)
    }
}

struct DynAgentRepository(Box<dyn AgentRepository + Send>);
struct DynAgentAuditSink(Box<dyn AgentAuditSink + Send>);
struct DynPolicyProvider(Box<dyn PolicyProvider + Send + Sync>);

impl DynAgentRepository {
    fn new<R>(repository: R) -> Self
    where
        R: AgentRepository + Send + 'static,
    {
        Self(Box::new(repository))
    }
}

impl DynAgentAuditSink {
    fn new<A>(audit_sink: A) -> Self
    where
        A: AgentAuditSink + Send + 'static,
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
    fn next_id(&mut self) -> KernelResult<u64> {
        self.0.next_id()
    }

    fn insert(&mut self, record: crate::domain::AgentBusinessRecord) -> KernelResult<()> {
        self.0.insert(record)
    }

    fn update(&mut self, record: crate::domain::AgentBusinessRecord) -> KernelResult<()> {
        self.0.update(record)
    }

    fn get(&self, tenant_id: u64, agent_id: &str) -> Option<crate::domain::AgentBusinessRecord> {
        self.0.get(tenant_id, agent_id)
    }

    fn list(
        &self,
        query: &crate::ports::AgentListQuery) -> Vec<crate::domain::AgentBusinessRecord> {
        self.0.list(query)
    }

    fn insert_provider_binding(&mut self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        self.0.insert_provider_binding(record)
    }

    fn update_provider_binding(&mut self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        self.0.update_provider_binding(record)
    }

    fn get_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str) -> Option<AgentProviderBindingRecord> {
        self.0.get_provider_binding(tenant_id, agent_id, binding_id)
    }

    fn list_provider_bindings(
        &self,
        tenant_id: u64,
        agent_id: &str) -> Vec<AgentProviderBindingRecord> {
        self.0.list_provider_bindings(tenant_id, agent_id)
    }

    fn insert_composition_slot(&mut self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        self.0.insert_composition_slot(record)
    }

    fn update_composition_slot(&mut self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        self.0.update_composition_slot(record)
    }

    fn get_composition_slot(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str) -> Option<AgentCompositionSlotRecord> {
        self.0.get_composition_slot(tenant_id, agent_id, slot_id)
    }

    fn list_composition_slots(
        &self,
        tenant_id: u64,
        agent_id: &str) -> Vec<AgentCompositionSlotRecord> {
        self.0.list_composition_slots(tenant_id, agent_id)
    }

}

impl AgentAuditSink for DynAgentAuditSink {
    fn record(&mut self, event: sdkwork_agent_kernel::KernelEvent) -> KernelResult<()> {
        self.0.record(event)
    }

    fn list_events(
        &self,
        tenant_id: u64,
        agent_id: &str) -> KernelResult<Vec<sdkwork_agent_kernel::KernelEvent>> {
        self.0.list_events(tenant_id, agent_id)
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

type HttpService = AgentsService<DynAgentRepository, DynAgentAuditSink, DynPolicyProvider>;

#[derive(Clone)]
pub struct AgentHttpState {
    service: Arc<ServiceMutex<HttpService>>,
}

impl AgentHttpState {
    pub fn new<R, A, P>(repository: R, audit_sink: A, policy_provider: P) -> Self
    where
        R: AgentRepository + Send + 'static,
        A: AgentAuditSink + Send + 'static,
        P: PolicyProvider + Send + Sync + 'static,
    {
        let service = AgentsService::new(
            DynAgentRepository::new(repository),
            DynAgentAuditSink::new(audit_sink),
            DynPolicyProvider::new(policy_provider));
        Self {
            service: Arc::new(ServiceMutex::new(service)),
        }
    }
}

async fn inject_gateway_agent_context(
    mut request: Request<axum::body::Body>,
    next: Next) -> Response {
    match AgentRequestContext::from_gateway_subject_headers(request.headers()) {
        Ok(context) => {
            request.extensions_mut().insert(context);
            next.run(request).await
        }
        Err(problem) => problem.into_response(),
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
async fn trace_request(
    request: Request<axum::body::Body>,
    next: Next) -> Response {
    let method = request.method().clone();
    let path = request
        .uri()
        .path()
        .to_string();
    let started = std::time::Instant::now();
    let response = next.run(request).await;
    let elapsed = started.elapsed();
    tracing::info!(
        method = %method,
        path = %path,
        status = response.status().as_u16(),
        elapsed_ms = elapsed.as_millis() as u64,
        "agents.managed_store.request"
    );
    response
}

fn with_gateway_trusted_context(router: Router<AgentHttpState>) -> Router<AgentHttpState> {
    router
        .layer(middleware::from_fn(trace_request))
        .layer(middleware::from_fn(inject_gateway_agent_context))
}

/// Raw app-api route tree without gateway or web-framework middleware.
pub fn build_app_routes() -> Router<AgentHttpState> {
    
            Router::new()
                .route(
                    "/app/v3/api/ai/agents",
                    get(app_list_agents).post(app_create_agent))
                .route(
                    "/app/v3/api/ai/agents/{agentId}",
                    get(app_get_agent)
                        .patch(app_update_agent)
                        .delete(app_delete_agent))
                .route(
                    "/app/v3/api/ai/agents/{agentId}/restore",
                    post(app_restore_agent))
                .route(
                    "/app/v3/api/ai/agents/{agentId}/provider_bindings",
                    get(app_list_provider_bindings).post(app_add_provider_binding))
                .route(
                    "/app/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
                    post(app_activate_provider_binding))
                .route(
                    "/app/v3/api/ai/agents/{agentId}/preview_responses",
                    post(app_create_preview_response))
                .route(
                    "/app/v3/api/ai/agents/{agentId}/prompt_optimizations",
                    post(app_create_prompt_optimization))
                .route(
                    "/app/v3/api/ai/agents/{agentId}/composition_slots",
                    get(app_list_composition_slots).post(app_create_composition_slot))
                .route(
                    "/app/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
                    get(app_get_composition_slot)
                        .patch(app_update_composition_slot)
                        .delete(app_delete_composition_slot))
}

pub fn build_app_router() -> Router<AgentHttpState> {
    build_app_routes()
}

/// Raw open-api route tree without gateway or web-framework middleware.
pub fn build_open_routes() -> Router<AgentHttpState> {
    
            Router::new()
                .route(
                    "/agent/v3/api/ai/agents",
                    get(backend_list_agents).post(backend_create_agent))
                .route(
                    "/agent/v3/api/ai/agents/{agentId}",
                    get(backend_get_agent)
                        .patch(backend_update_agent)
                        .delete(open_delete_agent))
                .route(
                    "/agent/v3/api/ai/agents/{agentId}/restore",
                    post(backend_restore_agent))
                .route(
                    "/agent/v3/api/ai/agents/{agentId}/provider_bindings",
                    get(backend_list_provider_bindings).post(backend_add_provider_binding))
                .route(
                    "/agent/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
                    post(backend_activate_provider_binding))
                .route(
                    "/agent/v3/api/ai/agents/{agentId}/preview_responses",
                    post(open_create_preview_response))
                .route(
                    "/agent/v3/api/ai/agents/{agentId}/prompt_optimizations",
                    post(open_create_prompt_optimization))
                .route(
                    "/agent/v3/api/ai/agents/{agentId}/composition_slots",
                    get(backend_list_composition_slots).post(backend_create_composition_slot))
                .route(
                    "/agent/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
                    get(backend_get_composition_slot)
                        .patch(backend_update_composition_slot)
                        .delete(backend_delete_composition_slot))
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
                    get(backend_list_agents).post(backend_create_agent))
                .route(
                    "/backend/v3/api/ai/agents/{agentId}",
                    get(backend_get_agent).patch(backend_update_agent))
                .route(
                    "/backend/v3/api/ai/agents/{agentId}/status",
                    post(backend_update_agent_status))
                .route(
                    "/backend/v3/api/ai/agents/{agentId}/restore",
                    post(backend_restore_agent))
                .route(
                    "/backend/v3/api/ai/agents/{agentId}/audit_events",
                    get(backend_list_agent_audit_events))
                .route(
                    "/backend/v3/api/ai/agents/{agentId}/provider_bindings",
                    get(backend_list_provider_bindings).post(backend_add_provider_binding))
                .route(
                    "/backend/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
                    post(backend_activate_provider_binding))
                .route(
                    "/backend/v3/api/ai/agents/{agentId}/composition_slots",
                    get(backend_list_composition_slots).post(backend_create_composition_slot))
                .route(
                    "/backend/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
                    get(backend_get_composition_slot)
                        .patch(backend_update_composition_slot)
                        .delete(backend_delete_composition_slot))
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
        .with_state(state)
}

/// Raw combined route tree for served production mounts.
pub fn build_combined_routes() -> Router<AgentHttpState> {
    build_open_routes()
        .merge(build_app_routes())
        .merge(build_backend_routes())
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

#[derive(Debug, Clone, Deserialize)]
struct TenantQueryParams {
    tenant_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AppDeleteQueryParams {
    expected_version: Option<String>,
    requested_at: String,
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
struct TenantAgentPathParams {
    #[serde(rename = "agentId")]
    agent_id: String,
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
struct CompositionSlotDeleteQueryParams {
    expected_version: Option<String>,
    requested_at: String,
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

#[derive(Debug, Clone, Serialize)]
struct AgentCompositionSlotResponse {
    data: AgentCompositionSlotRecordResponse,
}

#[derive(Debug, Clone, Serialize)]
struct AgentCompositionSlotListDataResponse {
    items: Vec<AgentCompositionSlotRecordResponse>,
}

#[derive(Debug, Clone, Serialize)]
struct AgentCompositionSlotListResponse {
    data: AgentCompositionSlotListDataResponse,
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
struct DeleteAgentBody {
    expected_version: Option<String>,
    requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RestoreAgentBody {
    expected_version: Option<String>,
    requested_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfoResponse {
    page: usize,
    page_size: usize,
    total_items: String,
    total_pages: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentListDataResponse {
    items: Vec<AgentRecordResponse>,
    page_info: PageInfoResponse,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentListResponse {
    data: AgentListDataResponse,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentResponse {
    data: AgentRecordResponse,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentProviderBindingResponse {
    data: AgentProviderBindingRecordResponse,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentProviderBindingListResponse {
    data: AgentProviderBindingListDataResponse,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentProviderBindingListDataResponse {
    items: Vec<AgentProviderBindingRecordResponse>,
    page_info: PageInfoResponse,
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
struct AgentRuntimeExecutionResponse {
    data: AgentRuntimeExecutionRecordResponse,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    suggested_prompts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    agent_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    users: Option<String>,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentAuditEventsListResponse {
    data: AgentAuditEventsData,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentAuditEventsData {
    items: Vec<AgentAuditEventResponse>,
    page_info: PageInfoResponse,
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
    model: Option<String>,
    suggested_prompts: Option<Vec<String>>,
    system_prompt: Option<String>,
    temperature: Option<f64>,
    #[serde(rename = "type")]
    agent_type: Option<String>,
    users: Option<String>,
    welcome_message: Option<String>,
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
            model: value.model,
            suggested_prompts: value.suggested_prompts.unwrap_or_default(),
            system_prompt: value.system_prompt,
            temperature: value.temperature,
            agent_type: value.agent_type,
            users: value.users,
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
    profile: &AgentManagementProfileBody) -> Result<(), ApiProblem> {
    validate_optional_profile_string(profile.author.as_deref(), "managementProfile.author", 128)?;
    validate_optional_profile_string(profile.avatar.as_deref(), "managementProfile.avatar", 512)?;
    validate_optional_profile_string(
        profile.category_id.as_deref(),
        "managementProfile.categoryId",
        64)?;
    validate_optional_profile_string(profile.color.as_deref(), "managementProfile.color", 32)?;
    validate_optional_profile_string(
        profile.icon_name.as_deref(),
        "managementProfile.iconName",
        64)?;
    if let Some(model) = profile.model.as_deref() {
        validate_standard_id(model, "managementProfile.model", Some("model."))
            .map_err(ApiProblem::from_kernel_error)?;
    }
    validate_profile_suggested_prompts(profile.suggested_prompts.as_deref().unwrap_or_default())?;
    validate_optional_profile_string(
        profile.system_prompt.as_deref(),
        "managementProfile.systemPrompt",
        32768)?;
    if let Some(temperature) = profile.temperature {
        if temperature < 0.0 {
            return Err(ApiProblem::validation(
                "managementProfile.temperature must be greater than or equal to 0"));
        }
        if temperature > 2.0 {
            return Err(ApiProblem::validation(
                "managementProfile.temperature must be less than or equal to 2"));
        }
    }
    if let Some(agent_type) = profile.agent_type.as_deref() {
        if !matches!(agent_type, "normal" | "independent") {
            return Err(ApiProblem::validation(
                "managementProfile.type must be one of normal, independent"));
        }
    }
    validate_optional_profile_string(profile.users.as_deref(), "managementProfile.users", 128)?;
    validate_optional_profile_string(
        profile.welcome_message.as_deref(),
        "managementProfile.welcomeMessage",
        4096)?;
    Ok(())
}

fn validate_optional_profile_string(
    value: Option<&str>,
    field_name: &str,
    max_length: usize) -> Result<(), ApiProblem> {
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
            "managementProfile.suggestedPrompts must contain at most 12 items"));
    }
    for value in values {
        let length = value.chars().count();
        if length == 0 {
            return Err(ApiProblem::validation(
                "managementProfile.suggestedPrompts items is required"));
        }
        if length > 256 {
            return Err(ApiProblem::validation(
                "managementProfile.suggestedPrompts items must be at most 256 characters"));
        }
    }
    Ok(())
}



#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProblemDetailResponse {
    r#type: String,
    title: String,
    status: u16,
    detail: String,
    code: String,
    error_category: String,
    retryable: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ApiProblem {
    status: StatusCode,
    response: ProblemDetailResponse,
}

#[derive(Debug, Clone, Copy)]
enum ErrorCategory {
    Validation,
    Permission,
    Business,
    Concurrency,
    Resource,
    Internal,
}

impl ErrorCategory {
    fn as_str(self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::Permission => "permission",
            Self::Business => "business",
            Self::Concurrency => "concurrency",
            Self::Resource => "resource",
            Self::Internal => "internal",
        }
    }
}

impl ApiProblem {
    fn validation(detail: impl Into<String>) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "validation_error",
            ErrorCategory::Validation,
            false,
            detail)
    }

    fn bad_request(detail: impl Into<String>) -> Self {
        Self::validation(detail)
    }

    fn permission(detail: impl Into<String>) -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "permission_required",
            ErrorCategory::Permission,
            false,
            detail)
    }

    fn conflict(detail: impl Into<String>) -> Self {
        Self::new(
            StatusCode::CONFLICT,
            "conflict",
            ErrorCategory::Business,
            false,
            detail)
    }

    fn version_conflict(detail: impl Into<String>) -> Self {
        Self::new(
            StatusCode::CONFLICT,
            "version_conflict",
            ErrorCategory::Concurrency,
            true,
            detail)
    }

    fn not_found(detail: impl Into<String>) -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "not_found",
            ErrorCategory::Resource,
            false,
            detail)
    }

    fn internal(detail: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            ErrorCategory::Internal,
            false,
            detail)
    }

    fn new(
        status: StatusCode,
        code: impl Into<String>,
        error_category: ErrorCategory,
        retryable: bool,
        detail: impl Into<String>) -> Self {
        let code = code.into();
        Self {
            status,
            response: ProblemDetailResponse {
                r#type: format!("https://sdkwork.dev/problems/{code}"),
                title: code.clone(),
                status: status.as_u16(),
                detail: detail.into(),
                code,
                error_category: error_category.as_str().to_string(),
                retryable,
            },
        }
    }

    fn from_kernel_error(error: KernelError) -> Self {
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

    fn from_json_rejection(rejection: JsonRejection) -> Self {
        Self::new(
            rejection.status(),
            "validation_error",
            ErrorCategory::Validation,
            false,
            format!("invalid json request: {}", rejection.body_text()))
    }

    fn from_query_rejection(rejection: QueryRejection) -> Self {
        Self::new(
            rejection.status(),
            "validation_error",
            ErrorCategory::Validation,
            false,
            format!("invalid query request: {}", rejection.body_text()))
    }

    fn from_path_rejection(rejection: PathRejection) -> Self {
        Self::new(
            rejection.status(),
            "validation_error",
            ErrorCategory::Validation,
            false,
            format!("invalid path request: {}", rejection.body_text()))
    }
}

impl IntoResponse for ApiProblem {
    fn into_response(self) -> Response {
        let mut response = (self.status, Json(self.response)).into_response();
        response.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"));
        response
    }
}

async fn app_list_agents(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    query: Result<Query<AppListAgentsQueryParams>, QueryRejection>) -> Result<Json<AgentListResponse>, ApiProblem> {
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

async fn backend_list_agents(
    State(state): State<AgentHttpState>,
    query: Result<Query<ListAgentsQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>) -> Result<Json<AgentListResponse>, ApiProblem> {
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let scope = RequestScope::from_trusted_extension(
        context,
        query.tenant_id.clone(),
        query.organization_id.clone(),
        query.owner_user_id.clone())?;
    execute_list(state, query, scope).await
}

async fn app_create_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<CreateAgentBody>, JsonRejection>) -> Result<(StatusCode, Json<AgentResponse>), ApiProblem> {
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    execute_create(state, RequestScope::from_context(context), body).await
}

async fn backend_create_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    body: Result<Json<CreateAgentBody>, JsonRejection>) -> Result<(StatusCode, Json<AgentResponse>), ApiProblem> {
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    let scope = RequestScope::from_trusted_extension(
        context,
        query.tenant_id,
        body.organization_id.clone(),
        body.owner_user_id.clone())?;
    execute_create(state, scope, body).await
}

async fn app_get_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    agent_id: Result<Path<String>, PathRejection>) -> Result<Json<AgentResponse>, ApiProblem> {
    let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
    execute_get(state, RequestScope::from_context(context), agent_id).await
}

async fn backend_get_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>) -> Result<Json<AgentResponse>, ApiProblem> {
    let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let scope = RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
    execute_get(state, scope, agent_id).await
}

async fn app_update_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<UpdateAgentBody>, JsonRejection>) -> Result<Json<AgentResponse>, ApiProblem> {
    let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    execute_update(state, RequestScope::from_context(context), agent_id, body).await
}

async fn backend_update_agent(
    State(state): State<AgentHttpState>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<UpdateAgentBody>, JsonRejection>) -> Result<Json<AgentResponse>, ApiProblem> {
    let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    let scope = RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
    execute_update(state, scope, agent_id, body).await
}

async fn app_delete_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<DeleteAgentBody>, JsonRejection>) -> Result<Json<AgentResponse>, ApiProblem> {
    let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    execute_delete(state, RequestScope::from_context(context), agent_id, body).await
}

async fn open_delete_agent(
    State(state): State<AgentHttpState>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<DeleteAgentBody>, JsonRejection>) -> Result<Json<AgentResponse>, ApiProblem> {
    let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    let scope = RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
    execute_delete(state, scope, agent_id, body).await
}

async fn app_restore_agent(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    agent_id: Result<Path<String>, PathRejection>,
    body: Result<Json<RestoreAgentBody>, JsonRejection>) -> Result<Json<AgentResponse>, ApiProblem> {
    let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    execute_restore(state, RequestScope::from_context(context), agent_id, body).await
}

async fn backend_update_agent_status(
    State(state): State<AgentHttpState>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<UpdateAgentStatusBody>, JsonRejection>) -> Result<Json<AgentResponse>, ApiProblem> {
    let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    let scope = RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
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

    let record = with_service_mut(&state, move |service| service.change_status(command)).await?;
    Ok(Json(AgentResponse {
        data: map_agent_record(&AgentRecordDto::from_record(&record))?,
    }))
}

async fn backend_restore_agent(
    State(state): State<AgentHttpState>,
    agent_id: Result<Path<String>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<RestoreAgentBody>, JsonRejection>) -> Result<Json<AgentResponse>, ApiProblem> {
    let Path(agent_id) = agent_id.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    let scope = RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
    execute_restore(state, scope, agent_id, body).await
}

async fn backend_list_agent_audit_events(
    State(state): State<AgentHttpState>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<AuditEventsQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>) -> Result<Json<AgentAuditEventsListResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let scope = RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
    let subject = scope.subject.clone();
    let tenant_id = scope.tenant_id_u64()?;
    let events = with_service_mut(&state, move |service| {
        service.list_agent_audit_events(tenant_id, path.agent_id.as_str(), subject)
    })
    .await?;
    let mut events = filter_audit_events(events, &query)?;
    sort_audit_events_by_occurred_at_desc(&mut events)?;

    let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
    let total_items = events.len();
    let total_pages = if total_items == 0 {
        0
    } else {
        total_items.div_ceil(page_size)
    };
    let paged = paginate(events, page, page_size);

    let items: Vec<AgentAuditEventResponse> = paged
        .into_iter()
        .map(|event| AgentAuditEventResponse {
            event_id: event.event_id,
            event_type: event.event_type,
            severity: kernel_event_severity(event.severity).to_string(),
            payload: event.payload,
            occurred_at: event.occurred_at.unwrap_or_default(),
        })
        .collect();

    Ok(Json(AgentAuditEventsListResponse {
        data: AgentAuditEventsData {
            items,
            page_info: PageInfoResponse {
                page,
                page_size,
                total_items: total_items.to_string(),
                total_pages,
            },
        },
    }))
}

async fn app_list_provider_bindings(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<AppListQueryParams>, QueryRejection>) -> Result<Json<AgentProviderBindingListResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    execute_list_provider_bindings(
        state,
        RequestScope::from_context(context),
        query.page,
        query.page_size,
        path.agent_id)
    .await
}

async fn backend_list_provider_bindings(
    State(state): State<AgentHttpState>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<TenantListQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>) -> Result<Json<AgentProviderBindingListResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let scope = RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
    execute_list_provider_bindings(state, scope, query.page, query.page_size, path.agent_id).await
}

async fn app_add_provider_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    body: Result<Json<AgentProviderBindingBody>, JsonRejection>) -> Result<(StatusCode, Json<AgentProviderBindingResponse>), ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    execute_add_provider_binding(
        state,
        RequestScope::from_context(context),
        path.agent_id,
        body)
    .await
}

async fn backend_add_provider_binding(
    State(state): State<AgentHttpState>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentProviderBindingBody>, JsonRejection>) -> Result<(StatusCode, Json<AgentProviderBindingResponse>), ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    let scope = RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
    execute_add_provider_binding(state, scope, path.agent_id, body).await
}

async fn app_activate_provider_binding(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    path: Result<Path<TenantAgentBindingPathParams>, PathRejection>,
    body: Result<Json<ActivateProviderBindingBody>, JsonRejection>) -> Result<Json<AgentProviderBindingResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    execute_activate_provider_binding(state, RequestScope::from_context(context), path, body).await
}

async fn backend_activate_provider_binding(
    State(state): State<AgentHttpState>,
    path: Result<Path<TenantAgentBindingPathParams>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<ActivateProviderBindingBody>, JsonRejection>) -> Result<Json<AgentProviderBindingResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    let scope = RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
    execute_activate_provider_binding(state, scope, path, body).await
}

async fn app_create_preview_response(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    body: Result<Json<AgentPreviewResponseBody>, JsonRejection>) -> Result<Json<AgentRuntimeExecutionResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    execute_create_preview_response(
        state,
        RequestScope::from_context(context),
        path.agent_id,
        body)
    .await
}

async fn app_create_prompt_optimization(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    body: Result<Json<AgentPromptOptimizationBody>, JsonRejection>) -> Result<Json<AgentRuntimeExecutionResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    execute_create_prompt_optimization(
        state,
        RequestScope::from_context(context),
        path.agent_id,
        body)
    .await
}

async fn open_create_preview_response(
    State(state): State<AgentHttpState>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentPreviewResponseBody>, JsonRejection>) -> Result<Json<AgentRuntimeExecutionResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    let scope = RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
    execute_create_preview_response(state, scope, path.agent_id, body).await
}

async fn open_create_prompt_optimization(
    State(state): State<AgentHttpState>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentPromptOptimizationBody>, JsonRejection>) -> Result<Json<AgentRuntimeExecutionResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    let scope = RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
    execute_create_prompt_optimization(state, scope, path.agent_id, body).await
}

async fn app_list_composition_slots(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>) -> Result<Json<AgentCompositionSlotListResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    execute_list_composition_slots(state, RequestScope::from_context(context), path.agent_id).await
}

async fn app_create_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    body: Result<Json<AgentCompositionSlotCreateRequestDto>, JsonRejection>) -> Result<(StatusCode, Json<AgentCompositionSlotResponse>), ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    execute_create_composition_slot(
        state,
        RequestScope::from_context(context),
        path.agent_id,
        body)
    .await
}

async fn app_get_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>) -> Result<Json<AgentCompositionSlotResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    execute_get_composition_slot(
        state,
        RequestScope::from_context(context),
        path.agent_id,
        path.slot_id)
    .await
}

async fn app_update_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
    body: Result<Json<AgentCompositionSlotUpdateRequestDto>, JsonRejection>) -> Result<Json<AgentCompositionSlotResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    execute_update_composition_slot(
        state,
        RequestScope::from_context(context),
        path.agent_id,
        path.slot_id,
        body)
    .await
}

async fn app_delete_composition_slot(
    State(state): State<AgentHttpState>,
    Extension(context): Extension<AgentRequestContext>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
    query: Result<Query<CompositionSlotDeleteQueryParams>, QueryRejection>) -> Result<Json<AgentCompositionSlotResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    execute_delete_composition_slot(
        state,
        RequestScope::from_context(context),
        path.agent_id,
        path.slot_id,
        query)
    .await
}

async fn backend_list_composition_slots(
    State(state): State<AgentHttpState>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>) -> Result<Json<AgentCompositionSlotListResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let scope = RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
    execute_list_composition_slots(state, scope, path.agent_id).await
}

async fn backend_create_composition_slot(
    State(state): State<AgentHttpState>,
    path: Result<Path<TenantAgentPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentCompositionSlotCreateRequestDto>, JsonRejection>) -> Result<(StatusCode, Json<AgentCompositionSlotResponse>), ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    execute_create_composition_slot(
        state,
        RequestScope::from_context(context),
        path.agent_id,
        body)
    .await
}

async fn backend_get_composition_slot(
    State(state): State<AgentHttpState>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
    query: Result<Query<TenantQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>) -> Result<Json<AgentCompositionSlotResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    let scope = RequestScope::from_trusted_extension(context, query.tenant_id.clone(), None, None)?;
    execute_get_composition_slot(state, scope, path.agent_id, path.slot_id).await
}

async fn backend_update_composition_slot(
    State(state): State<AgentHttpState>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
    Extension(context): Extension<AgentRequestContext>,
    body: Result<Json<AgentCompositionSlotUpdateRequestDto>, JsonRejection>) -> Result<Json<AgentCompositionSlotResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Json(body) = body.map_err(ApiProblem::from_json_rejection)?;
    execute_update_composition_slot(
        state,
        RequestScope::from_context(context),
        path.agent_id,
        path.slot_id,
        body)
    .await
}

async fn backend_delete_composition_slot(
    State(state): State<AgentHttpState>,
    path: Result<Path<TenantAgentSlotPathParams>, PathRejection>,
    query: Result<Query<CompositionSlotDeleteQueryParams>, QueryRejection>,
    Extension(context): Extension<AgentRequestContext>) -> Result<Json<AgentCompositionSlotResponse>, ApiProblem> {
    let Path(path) = path.map_err(ApiProblem::from_path_rejection)?;
    let Query(query) = query.map_err(ApiProblem::from_query_rejection)?;
    execute_delete_composition_slot(
        state,
        RequestScope::from_context(context),
        path.agent_id,
        path.slot_id,
        query)
    .await
}
async fn execute_list_composition_slots(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String) -> Result<Json<AgentCompositionSlotListResponse>, ApiProblem> {
    let tenant_id = scope.tenant_id_u64()?;
    let command = AgentCompositionSlotListCommand {
        tenant_id,
        agent_id,
        requested_by: scope.subject,
    };
    let records = with_service_mut(&state, move |service| service.list_composition_slots(command))
        .await?;
    let items = records
        .iter()
        .map(|record| map_composition_slot_record(&AgentCompositionSlotRecordDto::from_record(record)))
        .collect();
    Ok(Json(AgentCompositionSlotListResponse {
        data: AgentCompositionSlotListDataResponse { items },
    }))
}

async fn execute_create_composition_slot(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: AgentCompositionSlotCreateRequestDto) -> Result<(StatusCode, Json<AgentCompositionSlotResponse>), ApiProblem> {
    let tenant_id =
        parse_tenant_id(body.data.tenant_id.as_str()).map_err(ApiProblem::from_kernel_error)?;
    let organization_id = parse_tenant_id(body.data.organization_id.as_str())
        .map_err(ApiProblem::from_kernel_error)?;
    validate_requested_at(body.requested_at.as_str()).map_err(ApiProblem::from_kernel_error)?;
    let slot_kind = AgentCompositionSlotKind::from_str(body.data.slot_kind.as_str())
        .ok_or_else(|| ApiProblem::bad_request("invalid slotKind"))?;
    let target_module = AgentCompositionTargetModule::from_str(body.data.target_module.as_str())
        .ok_or_else(|| ApiProblem::bad_request("invalid targetModule"))?;
    let priority = body
        .data
        .priority
        .as_deref()
        .map(|s| s.parse::<i32>().map_err(|_| KernelError::validation("invalid priority")))
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
    let record = with_service_mut(&state, move |service| service.create_composition_slot(command))
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(AgentCompositionSlotResponse {
            data: map_composition_slot_record(&AgentCompositionSlotRecordDto::from_record(&record)),
        })))
}

async fn execute_get_composition_slot(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    slot_id: String) -> Result<Json<AgentCompositionSlotResponse>, ApiProblem> {
    let tenant_id = scope.tenant_id_u64()?;
    let command = AgentCompositionSlotGetCommand {
        tenant_id,
        agent_id,
        slot_id,
        requested_by: scope.subject,
    };
    let record = with_service_mut(&state, move |service| service.get_composition_slot(command))
        .await?;
    Ok(Json(AgentCompositionSlotResponse {
        data: map_composition_slot_record(&AgentCompositionSlotRecordDto::from_record(&record)),
    }))
}

async fn execute_update_composition_slot(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    slot_id: String,
    body: AgentCompositionSlotUpdateRequestDto) -> Result<Json<AgentCompositionSlotResponse>, ApiProblem> {
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
            AgentCompositionSlotKind::from_str(value)
                .ok_or_else(|| KernelError::validation("invalid slotKind"))
        })
        .transpose()
        .map_err(ApiProblem::from_kernel_error)?;
    let target_module = body
        .data
        .target_module
        .as_deref()
        .map(|value| {
            AgentCompositionTargetModule::from_str(value)
                .ok_or_else(|| KernelError::validation("invalid targetModule"))
        })
        .transpose()
        .map_err(ApiProblem::from_kernel_error)?;
    let priority = body
        .data
        .priority
        .as_deref()
        .map(|s| s.parse::<i32>().map_err(|_| KernelError::validation("invalid priority")))
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
    let record = with_service_mut(&state, move |service| service.update_composition_slot(command))
        .await?;
    Ok(Json(AgentCompositionSlotResponse {
        data: map_composition_slot_record(&AgentCompositionSlotRecordDto::from_record(&record)),
    }))
}

async fn execute_delete_composition_slot(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    slot_id: String,
    query: CompositionSlotDeleteQueryParams) -> Result<Json<AgentCompositionSlotResponse>, ApiProblem> {
    validate_requested_at(query.requested_at.as_str()).map_err(ApiProblem::from_kernel_error)?;
    let tenant_id = scope.tenant_id_u64()?;
    let expected_version = query
        .expected_version
        .as_deref()
        .map(parse_expected_version)
        .transpose()
        .map_err(ApiProblem::from_kernel_error)?;
    let command = AgentCompositionSlotDeleteCommand {
        tenant_id,
        agent_id,
        slot_id,
        expected_version,
        requested_by: scope.subject,
        requested_at: query.requested_at,
    };
    let record = with_service_mut(&state, move |service| service.delete_composition_slot(command))
        .await?;
    Ok(Json(AgentCompositionSlotResponse {
        data: map_composition_slot_record(&AgentCompositionSlotRecordDto::from_record(&record)),
    }))
}

fn map_composition_slot_record(
    record: &AgentCompositionSlotRecordDto) -> AgentCompositionSlotRecordResponse {
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
#[cfg(not(feature = "postgres-sync"))]
async fn with_service_mut<T>(
    state: &AgentHttpState,
    action: impl FnOnce(&mut HttpService) -> KernelResult<T>,
) -> Result<T, ApiProblem> {
    let mut service = state.service.lock().await;
    action(&mut *service).map_err(ApiProblem::from_kernel_error)
}

#[cfg(feature = "postgres-sync")]
async fn with_service_mut<T, F>(state: &AgentHttpState, action: F) -> Result<T, ApiProblem>
where
    F: FnOnce(&mut HttpService) -> KernelResult<T> + Send + 'static,
    T: Send + 'static,
{
    let service = Arc::clone(&state.service);
    let result = tokio::task::spawn_blocking(move || {
        let mut guard = service.lock().map_err(|_| KernelError::Internal {
            message: "agents managed store service lock poisoned".to_string(),
        })?;
        action(&mut *guard)
    })
    .await
    .map_err(|_| {
        ApiProblem::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            ErrorCategory::Internal,
            false,
            "agents managed store service worker failed",
        )
    })?;
    result.map_err(ApiProblem::from_kernel_error)
}
async fn execute_list(
    state: AgentHttpState,
    query: ListAgentsQueryParams,
    scope: RequestScope,
) -> Result<Json<AgentListResponse>, ApiProblem> {
    let include_deleted = query.include_deleted.unwrap_or(false);
    let request_dto = ListAgentsRequestDto {
        tenant_id: scope.tenant_id,
        organization_id: query.organization_id,
        owner_user_id: query.owner_user_id,
        include_deleted,
        search_query: query.q,
    };
    let command = request_dto
        .into_command(scope.subject)
        .map_err(ApiProblem::from_kernel_error)?;

    let mut records = with_service_mut(&state, move |service| service.list_agents(command)).await?;
    if matches!(
        query.scope.as_deref(),
        Some("market" | "public" | "published")
    ) {
        records.retain(|record| record.visibility.as_str() == "public");
    }
    let (page, page_size) = normalized_pagination(query.page, query.page_size)?;
    let total_items = records.len();
    let paged = paginate(records, page, page_size);

    let items: Vec<AgentRecordResponse> = paged
        .iter()
        .map(|record| map_agent_record(&AgentRecordDto::from_record(record)))
        .collect::<Result<Vec<_>, _>>()?;

    let total_pages = if total_items == 0 {
        0
    } else {
        total_items.div_ceil(page_size)
    };

    Ok(Json(AgentListResponse {
        data: AgentListDataResponse {
            items,
            page_info: PageInfoResponse {
                page,
                page_size,
                total_items: total_items.to_string(),
                total_pages,
            },
        },
    }))
}

async fn execute_create(
    state: AgentHttpState,
    scope: RequestScope,
    body: CreateAgentBody,
) -> Result<(StatusCode, Json<AgentResponse>), ApiProblem> {
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

    let record = with_service_mut(&state, move |service| service.create_agent(command)).await?;
    Ok((
        StatusCode::CREATED,
        Json(AgentResponse {
            data: map_agent_record(&AgentRecordDto::from_record(&record))?,
        }),
    ))
}

async fn execute_get(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
) -> Result<Json<AgentResponse>, ApiProblem> {
    let command = GetAgentRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service_mut(&state, move |service| service.get_agent(command)).await?;
    Ok(Json(AgentResponse {
        data: map_agent_record(&AgentRecordDto::from_record(&record))?,
    }))
}

async fn execute_update(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: UpdateAgentBody,
) -> Result<Json<AgentResponse>, ApiProblem> {
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
                    with_service_mut(&state, move |service| service.get_agent(command)).await?;
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

    let record = with_service_mut(&state, move |service| service.update_agent(command)).await?;
    Ok(Json(AgentResponse {
        data: map_agent_record(&AgentRecordDto::from_record(&record))?,
    }))
}

async fn execute_delete(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: DeleteAgentBody,
) -> Result<Json<AgentResponse>, ApiProblem> {
    let command = DeleteAgentRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
        expected_version: body.expected_version,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service_mut(&state, move |service| service.delete_agent(command)).await?;
    Ok(Json(AgentResponse {
        data: map_agent_record(&AgentRecordDto::from_record(&record))?,
    }))
}

async fn execute_restore(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: RestoreAgentBody,
) -> Result<Json<AgentResponse>, ApiProblem> {
    let command = RestoreAgentRequestDto {
        tenant_id: scope.tenant_id,
        agent_id,
        expected_version: body.expected_version,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service_mut(&state, move |service| service.restore_agent(command)).await?;
    Ok(Json(AgentResponse {
        data: map_agent_record(&AgentRecordDto::from_record(&record))?,
    }))
}


async fn execute_list_provider_bindings(
    state: AgentHttpState,
    scope: RequestScope,
    page: Option<usize>,
    page_size: Option<usize>,
    agent_id: String) -> Result<Json<AgentProviderBindingListResponse>, ApiProblem> {
    let tenant_id = scope.tenant_id_u64()?;

    let records = with_service_mut(&state, move |service| {
        service.list_provider_bindings(tenant_id, agent_id.as_str(), scope.subject)
    })
    .await?;
    let (page, page_size) = normalized_pagination(page, page_size)?;
    let total_items = records.len();
    let paged = paginate(records, page, page_size);

    let items = paged
        .iter()
        .map(|record| {
            map_provider_binding_record(&AgentProviderBindingRecordDto::from_record(record))
        })
        .collect();
    let total_pages = if total_items == 0 {
        0
    } else {
        total_items.div_ceil(page_size)
    };

    Ok(Json(AgentProviderBindingListResponse {
        data: AgentProviderBindingListDataResponse {
            items,
            page_info: PageInfoResponse {
                page,
                page_size,
                total_items: total_items.to_string(),
                total_pages,
            },
        },
    }))
}

async fn execute_add_provider_binding(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: AgentProviderBindingBody) -> Result<(StatusCode, Json<AgentProviderBindingResponse>), ApiProblem> {
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

    let record =
        with_service_mut(&state, move |service| service.add_provider_binding(command)).await?;
    Ok((
        StatusCode::CREATED,
        Json(AgentProviderBindingResponse {
            data: map_provider_binding_record(&AgentProviderBindingRecordDto::from_record(&record)),
        })))
}

async fn execute_activate_provider_binding(
    state: AgentHttpState,
    scope: RequestScope,
    path: TenantAgentBindingPathParams,
    body: ActivateProviderBindingBody) -> Result<Json<AgentProviderBindingResponse>, ApiProblem> {
    let command = ActivateAgentProviderBindingRequestDto {
        tenant_id: scope.tenant_id,
        agent_id: path.agent_id,
        binding_id: path.binding_id,
        requested_at: body.requested_at,
    }
    .into_command(scope.subject)
    .map_err(ApiProblem::from_kernel_error)?;

    let record = with_service_mut(&state, move |service| {
        service.activate_provider_binding(command)
    })
    .await?;
    Ok(Json(AgentProviderBindingResponse {
        data: map_provider_binding_record(&AgentProviderBindingRecordDto::from_record(&record)),
    }))
}

async fn execute_create_preview_response(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: AgentPreviewResponseBody) -> Result<Json<AgentRuntimeExecutionResponse>, ApiProblem> {
    let input_payload_json = json_value_to_string(
        body.input_payload
            .unwrap_or_else(|| json!({ "content": body.content })),
        "inputPayload")?;
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

    let record = with_service_mut(&state, move |service| {
        service.create_preview_response(command)
    })
    .await?;
    Ok(Json(AgentRuntimeExecutionResponse {
        data: map_runtime_execution_record(&AgentRuntimeExecutionRecordDto::from_record(&record))?,
    }))
}

async fn execute_create_prompt_optimization(
    state: AgentHttpState,
    scope: RequestScope,
    agent_id: String,
    body: AgentPromptOptimizationBody) -> Result<Json<AgentRuntimeExecutionResponse>, ApiProblem> {
    let input_payload_json = json_value_to_string(
        body.input_payload
            .unwrap_or_else(|| json!({ "prompt": body.prompt })),
        "inputPayload")?;
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

    let record = with_service_mut(&state, move |service| {
        service.create_prompt_optimization(command)
    })
    .await?;
    Ok(Json(AgentRuntimeExecutionResponse {
        data: map_runtime_execution_record(&AgentRuntimeExecutionRecordDto::from_record(&record))?,
    }))
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
    profile: &AgentManagementProfileDto) -> AgentManagementProfileResponse {
    AgentManagementProfileResponse {
        author: profile.author.clone(),
        avatar: profile.avatar.clone(),
        category_id: profile.category_id.clone(),
        color: profile.color.clone(),
        debug_mode: profile.debug_mode,
        icon_name: profile.icon_name.clone(),
        json_mode: profile.json_mode,
        model: profile.model.clone(),
        suggested_prompts: profile.suggested_prompts.clone(),
        system_prompt: profile.system_prompt.clone(),
        temperature: profile.temperature,
        agent_type: profile.agent_type.clone(),
        users: profile.users.clone(),
        welcome_message: profile.welcome_message.clone(),
    }
}

fn map_provider_binding_record(
    record: &AgentProviderBindingRecordDto) -> AgentProviderBindingRecordResponse {
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
    record: &AgentRuntimeExecutionRecordDto) -> Result<AgentRuntimeExecutionRecordResponse, ApiProblem> {
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

fn filter_audit_events(
    events: Vec<sdkwork_agent_kernel::KernelEvent>,
    query: &AuditEventsQueryParams) -> Result<Vec<sdkwork_agent_kernel::KernelEvent>, ApiProblem> {
    if let Some(action) = query.action.as_ref() {
        if !ALLOWED_AUDIT_ACTIONS.contains(&action.as_str()) {
            return Err(ApiProblem::validation(format!(
                "action must be one of {}",
                ALLOWED_AUDIT_ACTIONS.join(", ")
            )));
        }
    }

    let from = parse_optional_query_datetime("from", query.from.as_deref())?;
    let to = parse_optional_query_datetime("to", query.to.as_deref())?;
    if let (Some(from_value), Some(to_value)) = (from.as_ref(), to.as_ref()) {
        if from_value > to_value {
            return Err(ApiProblem::validation(
                "from must be less than or equal to to"));
        }
    }

    let mut filtered = Vec::new();
    for event in events {
        let action_ok = query
            .action
            .as_ref()
            .map(|action| action == audit_event_action(event.event_type.as_str()))
            .unwrap_or(true);
        if !action_ok {
            continue;
        }

        let occurred_at_raw = event
            .occurred_at
            .as_deref()
            .ok_or_else(|| ApiProblem::internal("audit event occurred_at is missing"))?;
        let occurred_at = parse_rfc3339_datetime(occurred_at_raw, "audit event occurred_at")
            .map_err(|error| ApiProblem::internal(error.safe_message()))?;

        let from_ok = from
            .as_ref()
            .map(|from_value| occurred_at >= *from_value)
            .unwrap_or(true);
        let to_ok = to
            .as_ref()
            .map(|to_value| occurred_at <= *to_value)
            .unwrap_or(true);
        if from_ok && to_ok {
            filtered.push(event);
        }
    }
    Ok(filtered)
}

fn sort_audit_events_by_occurred_at_desc(
    events: &mut [sdkwork_agent_kernel::KernelEvent],
) -> Result<(), ApiProblem> {
    use std::cmp::Ordering;

    events.sort_by(|left, right| {
        let left_at = audit_event_occurred_at(left);
        let right_at = audit_event_occurred_at(right);
        match right_at.cmp(&left_at) {
            Ordering::Equal => match right.event_type.cmp(&left.event_type) {
                Ordering::Equal => right.event_id.cmp(&left.event_id),
                other => other,
            },
            other => other,
        }
    });
    Ok(())
}

fn audit_event_occurred_at(
    event: &sdkwork_agent_kernel::KernelEvent,
) -> OffsetDateTime {
    let occurred_at_raw = event
        .occurred_at
        .as_deref()
        .unwrap_or("1970-01-01T00:00:00Z");
    parse_rfc3339_datetime(occurred_at_raw, "audit event occurred_at")
        .unwrap_or_else(|_| OffsetDateTime::UNIX_EPOCH)
}

fn audit_event_action(event_type: &str) -> &str {
    event_type.rsplit('.').next().unwrap_or(event_type)
}

fn parse_optional_query_datetime(
    field_name: &str,
    value: Option<&str>) -> Result<Option<OffsetDateTime>, ApiProblem> {
    parse_optional_rfc3339_datetime(value, field_name).map_err(ApiProblem::from_kernel_error)
}

fn reconcile_resource_tenant_with_subject_header(
    resource_tenant_id: &str,
    header_tenant_id: Option<String>) -> Result<String, ApiProblem> {
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
            "subject tenant does not match resource tenant"));
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

fn normalized_pagination(
    page: Option<usize>,
    page_size: Option<usize>) -> Result<(usize, usize), ApiProblem> {
    let page = page.unwrap_or(1);
    if page == 0 {
        return Err(ApiProblem::validation(
            "page must be greater than or equal to 1"));
    }

    let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    if page_size == 0 {
        return Err(ApiProblem::validation(
            "page_size must be greater than or equal to 1"));
    }
    if page_size > MAX_PAGE_SIZE {
        return Err(ApiProblem::validation(format!(
            "page_size must be less than or equal to {MAX_PAGE_SIZE}"
        )));
    }

    Ok((page, page_size))
}

fn total_pages(total_items: usize, page_size: usize) -> usize {
    if total_items == 0 {
        0
    } else {
        total_items.div_ceil(page_size)
    }
}

fn paginate<T: Clone>(items: Vec<T>, page: usize, page_size: usize) -> Vec<T> {
    let start = (page - 1).saturating_mul(page_size);
    items.into_iter().skip(start).take(page_size).collect()
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

    fn test_agent_context() -> AgentRequestContext {
        AgentRequestContext::new("100001", "100")
            .with_organization_id("0")
            .with_subject_id("u-1")
            .with_roles(["agent.write", "agent.read"])
    }

    #[tokio::test]
    async fn app_create_and_retrieve_agent_should_work() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            AllowAllPolicyProvider::allow("policy.memory"));
        let app = build_combined_router(state).layer(Extension(test_agent_context()));

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
        assert_eq!(body_json["data"]["agentId"], "agent.alpha");
    }

    #[tokio::test]
    async fn open_api_create_and_retrieve_agent_should_work() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            AllowAllPolicyProvider::allow("policy.memory"));
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
    async fn backend_status_update_should_work() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            AllowAllPolicyProvider::allow("policy.memory"));
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

        assert_eq!(status_response.status(), StatusCode::OK);
        let body_bytes = to_bytes(status_response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let body_json: Value =
            serde_json::from_slice(&body_bytes).expect("response body should be valid json");
        assert_eq!(body_json["data"]["status"], "active");
    }

    // --- P1-4 tenant_id 安全防护单元测试 ---

    fn subject_headers_with(tenant: Option<&str>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("x-subject-id", HeaderValue::from_static("u-1"));
        if let Some(tenant) = tenant {
            headers.insert("x-subject-tenant-id", HeaderValue::from_str(tenant).unwrap());
        }
        headers
    }

    #[test]
    fn from_gateway_subject_headers_rejects_missing_tenant_header() {
        // 缺失 tenant header 必须在网关边界被拒绝，避免空 tenant_id 进入应用层
        let headers = subject_headers_with(None);
        let result = AgentRequestContext::from_gateway_subject_headers(&headers);
        let err = result.expect_err("missing tenant header should be rejected");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert!(err.response.detail.contains("tenant"));
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
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert!(err.response.detail.contains("subject tenant header is required"));
    }

    #[test]
    fn reconcile_resource_tenant_rejects_mismatch() {
        let err = reconcile_resource_tenant_with_subject_header(
            "100001",
            Some("100002".to_string()),
        )
        .expect_err("tenant mismatch must be rejected");
        assert_eq!(err.status, StatusCode::FORBIDDEN);
        assert!(err.response.detail.contains("does not match"));
    }

    #[test]
    fn reconcile_resource_tenant_rejects_resource_zero() {
        // resource tenant_id=0 也必须被拒绝（parse_tenant_id 拦截）
        let err = reconcile_resource_tenant_with_subject_header(
            "0",
            Some("100001".to_string()),
        )
        .expect_err("resource tenant 0 must be rejected");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert!(err.response.detail.contains("greater than 0"));
    }

    #[test]
    fn reconcile_resource_tenant_accepts_match() {
        let result = reconcile_resource_tenant_with_subject_header(
            "100001",
            Some("100001".to_string()),
        )
        .expect("matching tenants should be accepted");
        assert_eq!(result, "100001");
    }
}
