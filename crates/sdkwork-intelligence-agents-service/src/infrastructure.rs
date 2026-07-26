use crate::agent_turn::{AgentTurnRecord, AgentTurnStatus};
use crate::domain::{
    AgentBusinessRecord, AgentCompositionSlotKind, AgentCompositionSlotRecord,
    AgentInteractionRecord, AgentItemDriveRefRecord, AgentItemFeedbackRecord,
    AgentProviderBindingRecord, AgentResourceType, AgentResourceUserStateRecord,
    AgentSessionCheckpointRecord, AgentSessionItemRecord, AgentSessionItemStatus,
    AgentSessionRecord, AgentSessionRuntimeBindingRecord, AgentSessionRuntimeBindingStatus,
    AgentTaskRecord,
};
use crate::id::{AgentBusinessIdGenerator, AgentIdGenerator};
use crate::in_memory_pagination::{count_iterator, paginate_items, paginate_iterator};
use crate::ports::{
    AgentAuditSink, AgentListQuery, AgentRepository, AuditEventListQuery, CompositionSlotListQuery,
    InteractionListQuery, ItemFeedbackListQuery, McpMarketplaceListQuery,
    ProjectCompositionSlotListQuery, ProjectListQuery, ProviderBindingListQuery,
    ResourceUserStateListQuery, SessionCheckpointListQuery, SessionItemListQuery, SessionListQuery,
    SessionRuntimeBindingListQuery, TaskListQuery, TurnListQuery, TurnRequestWriteOutcome,
    WorkspaceListQuery,
};
use crate::project::{AgentProjectCompositionSlotRecord, AgentProjectRecord, AgentProjectStatus};
use crate::validation::parse_rfc3339_datetime;
use crate::workspace::{AgentWorkspaceRecord, AgentWorkspaceStatus};
use sdkwork_agent_kernel::{
    KernelError, KernelEvent, KernelResult, PolicyDecision, PolicyProvider, PolicyRequest,
    ProviderHealth, ProviderManifest,
};
use sdkwork_utils_rust::{is_blank, trim};
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{LazyLock, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

// ---------------------------------------------------------------------------
// Production environment safety checks
// ---------------------------------------------------------------------------

/// Environment variable name for development authentication bypass.
/// When set to "true", AllowAllPolicyProvider permits all requests without
/// IAM validation. This MUST NOT be enabled in production environments.
pub const ENV_DEV_AUTH_BYPASS: &str = "SDKWORK_AGENTS_DEV_AUTH_BYPASS";

/// Environment variable name indicating production deployment.
/// Prefer `sdkwork_agents_contract::agents_deployment_environment_name()` for gating.
pub const ENV_DEPLOYMENT_ENV: &str = "SDKWORK_DEPLOYMENT_ENV";

/// Validates that development authentication bypass is not enabled in production.
///
/// Delegates to `sdkwork-agents-contract::ensure_dev_auth_bypass_allowed` so
/// callers can fail bootstrap explicitly instead of crashing the process.
pub fn validate_production_security_config() -> Result<(), String> {
    sdkwork_agents_contract::ensure_dev_auth_bypass_allowed()?;

    if agents_dev_auth_bypass_enabled_from_env() {
        tracing::warn!(
            env_var = ENV_DEV_AUTH_BYPASS,
            deployment = %sdkwork_agents_contract::agents_deployment_environment_name(),
            "Development authentication bypass is enabled. This MUST NOT be used in production."
        );
    }
    Ok(())
}

fn agents_dev_auth_bypass_enabled_from_env() -> bool {
    std::env::var(ENV_DEV_AUTH_BYPASS)
        .ok()
        .and_then(|value| sdkwork_utils_rust::parse_bool(value.trim()))
        .unwrap_or(false)
}

/// Returns true if the current deployment is a production environment.
pub fn is_production_environment() -> bool {
    sdkwork_agents_contract::agents_is_production_like_environment()
}

// ---------------------------------------------------------------------------
// Metrics types for observability (O-01)
// ---------------------------------------------------------------------------

use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::Instant;

/// Prometheus snapshot for agents managed-store HTTP and business gauges.
#[derive(Debug, Clone, Default)]
pub struct AgentServiceMetrics {
    pub http_requests_total: u64,
    pub http_errors_total: u64,
    pub http_requests_per_second: f64,
    pub service_worker_rejections_total: u64,
    pub provider_worker_rejections_total: u64,
    /// Total number of agents across all tenants
    pub total_agents: u64,
    /// Number of active (non-deleted) agents
    pub active_agents: u64,
    /// Number of soft-deleted agents
    pub deleted_agents: u64,
    /// Total number of provider bindings
    pub total_provider_bindings: u64,
    /// Number of active provider bindings
    pub active_provider_bindings: u64,
    /// Total number of composition slots
    pub total_composition_slots: u64,
    /// Number of audit events recorded
    pub audit_events_count: u64,
    /// Request count by operation (operation -> count)
    pub request_counts: std::collections::HashMap<String, u64>,
    /// Error count by operation (operation -> count)
    pub error_counts: std::collections::HashMap<String, u64>,
}

#[derive(Debug)]
struct ScrapeState {
    instant: Instant,
    request_total: u64,
}

/// Process-wide agents metrics registry for Prometheus scraping.
pub struct AgentMetricsRegistry {
    http_requests_total: AtomicU64,
    http_errors_total: AtomicU64,
    service_worker_rejections_total: AtomicU64,
    provider_worker_rejections_total: AtomicU64,
    scrape_state: Mutex<ScrapeState>,
}

impl AgentMetricsRegistry {
    pub fn global() -> &'static Self {
        static REGISTRY: LazyLock<AgentMetricsRegistry> = LazyLock::new(|| AgentMetricsRegistry {
            http_requests_total: AtomicU64::new(0),
            http_errors_total: AtomicU64::new(0),
            service_worker_rejections_total: AtomicU64::new(0),
            provider_worker_rejections_total: AtomicU64::new(0),
            scrape_state: Mutex::new(ScrapeState {
                instant: Instant::now(),
                request_total: 0,
            }),
        });
        &REGISTRY
    }

    pub fn record_http_request(&self, status: u16) {
        self.http_requests_total
            .fetch_add(1, AtomicOrdering::Relaxed);
        if status >= 400 {
            self.http_errors_total.fetch_add(1, AtomicOrdering::Relaxed);
        }
    }

    pub fn record_service_worker_rejection(&self) {
        self.service_worker_rejections_total
            .fetch_add(1, AtomicOrdering::Relaxed);
    }

    pub fn record_provider_worker_rejection(&self) {
        self.provider_worker_rejections_total
            .fetch_add(1, AtomicOrdering::Relaxed);
    }

    pub fn snapshot(&self) -> AgentServiceMetrics {
        let http_requests_total = self.http_requests_total.load(AtomicOrdering::Relaxed);
        let http_errors_total = self.http_errors_total.load(AtomicOrdering::Relaxed);
        let service_worker_rejections_total = self
            .service_worker_rejections_total
            .load(AtomicOrdering::Relaxed);
        let provider_worker_rejections_total = self
            .provider_worker_rejections_total
            .load(AtomicOrdering::Relaxed);
        let http_requests_per_second = {
            let mut state = self.scrape_state.recovering_lock();
            let now = Instant::now();
            let elapsed = now.duration_since(state.instant).as_secs_f64();
            let delta = http_requests_total.saturating_sub(state.request_total);
            state.instant = now;
            state.request_total = http_requests_total;
            if elapsed > 0.0 {
                delta as f64 / elapsed
            } else {
                0.0
            }
        };

        AgentServiceMetrics {
            http_requests_total,
            http_errors_total,
            http_requests_per_second,
            service_worker_rejections_total,
            provider_worker_rejections_total,
            ..AgentServiceMetrics::default()
        }
    }
}

impl AgentServiceMetrics {
    /// Export metrics in Prometheus text exposition format
    pub fn to_prometheus_text(&self) -> String {
        let mut output = String::with_capacity(1024);

        output.push_str("# HELP sdkwork_agents_requests_total Total managed-store HTTP requests\n");
        output.push_str("# TYPE sdkwork_agents_requests_total counter\n");
        output.push_str(&format!(
            "sdkwork_agents_requests_total {}\n",
            self.http_requests_total
        ));

        output.push_str(
            "# HELP sdkwork_agents_errors_total Total managed-store HTTP error responses\n",
        );
        output.push_str("# TYPE sdkwork_agents_errors_total counter\n");
        output.push_str(&format!(
            "sdkwork_agents_errors_total {}\n",
            self.http_errors_total
        ));

        output.push_str("# HELP sdkwork_agents_requests_per_second Managed-store HTTP requests per second (scrape-window rate)\n");
        output.push_str("# TYPE sdkwork_agents_requests_per_second gauge\n");
        output.push_str(&format!(
            "sdkwork_agents_requests_per_second {}\n",
            self.http_requests_per_second
        ));

        output.push_str("# HELP sdkwork_agents_service_worker_rejections_total Requests rejected before entering the blocking service pool\n");
        output.push_str("# TYPE sdkwork_agents_service_worker_rejections_total counter\n");
        output.push_str(&format!(
            "sdkwork_agents_service_worker_rejections_total {}\n",
            self.service_worker_rejections_total
        ));

        output.push_str("# HELP sdkwork_agents_provider_worker_rejections_total Provider executions rejected when bounded capacity is exhausted\n");
        output.push_str("# TYPE sdkwork_agents_provider_worker_rejections_total counter\n");
        output.push_str(&format!(
            "sdkwork_agents_provider_worker_rejections_total {}\n",
            self.provider_worker_rejections_total
        ));

        // Help and type declarations
        output.push_str("# HELP sdkwork_agents_total Total number of agents\n");
        output.push_str("# TYPE sdkwork_agents_total gauge\n");
        output.push_str(&format!("sdkwork_agents_total {}\n", self.total_agents));

        output.push_str("# HELP sdkwork_agents_active Number of active (non-deleted) agents\n");
        output.push_str("# TYPE sdkwork_agents_active gauge\n");
        output.push_str(&format!("sdkwork_agents_active {}\n", self.active_agents));

        output.push_str("# HELP sdkwork_agents_deleted Number of soft-deleted agents\n");
        output.push_str("# TYPE sdkwork_agents_deleted gauge\n");
        output.push_str(&format!("sdkwork_agents_deleted {}\n", self.deleted_agents));

        output.push_str(
            "# HELP sdkwork_agents_provider_bindings_total Total number of provider bindings\n",
        );
        output.push_str("# TYPE sdkwork_agents_provider_bindings_total gauge\n");
        output.push_str(&format!(
            "sdkwork_agents_provider_bindings_total {}\n",
            self.total_provider_bindings
        ));

        output.push_str(
            "# HELP sdkwork_agents_provider_bindings_active Number of active provider bindings\n",
        );
        output.push_str("# TYPE sdkwork_agents_provider_bindings_active gauge\n");
        output.push_str(&format!(
            "sdkwork_agents_provider_bindings_active {}\n",
            self.active_provider_bindings
        ));

        output.push_str(
            "# HELP sdkwork_agents_composition_slots_total Total number of composition slots\n",
        );
        output.push_str("# TYPE sdkwork_agents_composition_slots_total gauge\n");
        output.push_str(&format!(
            "sdkwork_agents_composition_slots_total {}\n",
            self.total_composition_slots
        ));

        output.push_str("# HELP sdkwork_agents_audit_events_total Total number of audit events\n");
        output.push_str("# TYPE sdkwork_agents_audit_events_total counter\n");
        output.push_str(&format!(
            "sdkwork_agents_audit_events_total {}\n",
            self.audit_events_count
        ));

        // Request counts by operation
        output.push_str(
            "# HELP sdkwork_agents_requests_by_operation_total Total requests by operation\n",
        );
        output.push_str("# TYPE sdkwork_agents_requests_by_operation_total counter\n");
        for (operation, count) in &self.request_counts {
            output.push_str(&format!(
                "sdkwork_agents_requests_by_operation_total{{operation=\"{}\"}} {}\n",
                operation, count
            ));
        }

        // Error counts by operation
        output.push_str(
            "# HELP sdkwork_agents_errors_by_operation_total Total errors by operation\n",
        );
        output.push_str("# TYPE sdkwork_agents_errors_by_operation_total counter\n");
        for (operation, count) in &self.error_counts {
            output.push_str(&format!(
                "sdkwork_agents_errors_by_operation_total{{operation=\"{}\"}} {}\n",
                operation, count
            ));
        }

        output
    }
}

/// In-memory agent repository with interior mutability via `RwLock`.
///
/// All trait methods use `&self`; the `RwLock` handles concurrent access.
/// This makes the repository compatible with the stateless `AgentsService`
/// and eliminates the global `Mutex<AgentsService>` bottleneck.
type AgentPrimaryKey = (u64, String);
type AgentIndexKey = (u64, Reverse<String>, Reverse<u64>, String);
type ProjectPrimaryKey = (u64, u64, String);
type ProjectIndexKey = (u64, u64, Reverse<String>, Reverse<u64>, String);
type WorkspacePrimaryKey = (u64, u64, String);
type WorkspaceIndexKey = (
    u64,
    u64,
    u64,
    Reverse<bool>,
    Reverse<String>,
    Reverse<u64>,
    String,
);
type ProjectCompositionSlotPrimaryKey = (u64, u64, String, String);
type ProviderBindingPrimaryKey = (u64, String, String);
type ProviderBindingIndexKey = (u64, String, Reverse<bool>, Reverse<String>, String);
type CompositionSlotPrimaryKey = (u64, String, String);
type CompositionSlotIndexKey = (u64, String, i32, String);
type SessionPrimaryKey = (u64, u64, String);
type SessionRuntimeBindingPrimaryKey = (u64, u64, String, String);
type SessionCheckpointPrimaryKey = (u64, u64, String, String);
type ResourceUserStatePrimaryKey = (u64, u64, u64, i16, String);
type SessionIndexKey = (u64, u64, Reverse<String>, String);
type SessionItemPrimaryKey = (u64, u64, String, String);
type ItemFeedbackPrimaryKey = (u64, u64, String, u64);
type ItemDriveRefPrimaryKey = (u64, u64, String, String, String);
type SessionItemIndexKey = (u64, u64, String, u64, String);
type TurnPrimaryKey = (u64, u64, String);
type TurnIdempotencyKey = (u64, u64, u64, String);
type InteractionPrimaryKey = (u64, u64, String, String);
type InteractionIndexKey = (u64, u64, String, Reverse<String>, String);
type TaskPrimaryKey = (u64, String);
type TaskIndexKey = (u64, Reverse<String>, String);

trait RecoveringRwLock<T> {
    fn recovering_read(&self) -> RwLockReadGuard<'_, T>;

    fn recovering_write(&self) -> RwLockWriteGuard<'_, T>;
}

impl<T> RecoveringRwLock<T> for RwLock<T> {
    fn recovering_read(&self) -> RwLockReadGuard<'_, T> {
        match self.read() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::error!(
                    "in-memory agents repository RwLock was poisoned; recovering guard"
                );
                poisoned.into_inner()
            }
        }
    }

    fn recovering_write(&self) -> RwLockWriteGuard<'_, T> {
        match self.write() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::error!(
                    "in-memory agents repository RwLock was poisoned; recovering mutable guard"
                );
                poisoned.into_inner()
            }
        }
    }
}

trait RecoveringMutex<T> {
    fn recovering_lock(&self) -> MutexGuard<'_, T>;
}

impl<T> RecoveringMutex<T> for Mutex<T> {
    fn recovering_lock(&self) -> MutexGuard<'_, T> {
        match self.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::error!("in-memory agents mutex was poisoned; recovering guard");
                poisoned.into_inner()
            }
        }
    }
}

#[derive(Debug)]
pub struct InMemoryAgentRepository {
    id_generator: AgentBusinessIdGenerator,
    agents: RwLock<HashMap<AgentPrimaryKey, AgentBusinessRecord>>,
    agent_list_index: RwLock<BTreeMap<AgentIndexKey, AgentPrimaryKey>>,
    projects: RwLock<HashMap<ProjectPrimaryKey, AgentProjectRecord>>,
    project_index: RwLock<BTreeMap<ProjectIndexKey, ProjectPrimaryKey>>,
    workspaces: RwLock<HashMap<WorkspacePrimaryKey, AgentWorkspaceRecord>>,
    workspace_index: RwLock<BTreeMap<WorkspaceIndexKey, WorkspacePrimaryKey>>,
    project_composition_slots:
        RwLock<HashMap<ProjectCompositionSlotPrimaryKey, AgentProjectCompositionSlotRecord>>,
    provider_bindings: RwLock<HashMap<ProviderBindingPrimaryKey, AgentProviderBindingRecord>>,
    provider_binding_index: RwLock<BTreeMap<ProviderBindingIndexKey, ProviderBindingPrimaryKey>>,
    composition_slots: RwLock<HashMap<CompositionSlotPrimaryKey, AgentCompositionSlotRecord>>,
    composition_slot_index: RwLock<BTreeMap<CompositionSlotIndexKey, CompositionSlotPrimaryKey>>,
    sessions: RwLock<HashMap<SessionPrimaryKey, AgentSessionRecord>>,
    session_index: RwLock<BTreeMap<SessionIndexKey, SessionPrimaryKey>>,
    session_runtime_bindings:
        RwLock<HashMap<SessionRuntimeBindingPrimaryKey, AgentSessionRuntimeBindingRecord>>,
    session_checkpoints: RwLock<HashMap<SessionCheckpointPrimaryKey, AgentSessionCheckpointRecord>>,
    resource_user_states:
        RwLock<HashMap<ResourceUserStatePrimaryKey, AgentResourceUserStateRecord>>,
    items: RwLock<HashMap<SessionItemPrimaryKey, AgentSessionItemRecord>>,
    item_feedback: RwLock<HashMap<ItemFeedbackPrimaryKey, AgentItemFeedbackRecord>>,
    item_drive_refs: RwLock<HashMap<ItemDriveRefPrimaryKey, AgentItemDriveRefRecord>>,
    session_item_index: RwLock<BTreeMap<SessionItemIndexKey, SessionItemPrimaryKey>>,
    turns: RwLock<HashMap<TurnPrimaryKey, AgentTurnRecord>>,
    turn_idempotency: RwLock<HashMap<TurnIdempotencyKey, TurnPrimaryKey>>,
    interactions: RwLock<HashMap<InteractionPrimaryKey, AgentInteractionRecord>>,
    interaction_index: RwLock<BTreeMap<InteractionIndexKey, InteractionPrimaryKey>>,
    tasks: RwLock<HashMap<TaskPrimaryKey, AgentTaskRecord>>,
    task_index: RwLock<BTreeMap<TaskIndexKey, TaskPrimaryKey>>,
}

impl InMemoryAgentRepository {
    pub fn try_new() -> KernelResult<Self> {
        Ok(Self::with_id_generator(
            AgentBusinessIdGenerator::new_default()?,
        ))
    }

    pub fn new() -> Self {
        Self::try_new().expect("default agents in-memory repository constructor should succeed")
    }

    fn with_id_generator(id_generator: AgentBusinessIdGenerator) -> Self {
        Self {
            id_generator,
            agents: RwLock::new(HashMap::new()),
            agent_list_index: RwLock::new(BTreeMap::new()),
            projects: RwLock::new(HashMap::new()),
            project_index: RwLock::new(BTreeMap::new()),
            workspaces: RwLock::new(HashMap::new()),
            workspace_index: RwLock::new(BTreeMap::new()),
            project_composition_slots: RwLock::new(HashMap::new()),
            provider_bindings: RwLock::new(HashMap::new()),
            provider_binding_index: RwLock::new(BTreeMap::new()),
            composition_slots: RwLock::new(HashMap::new()),
            composition_slot_index: RwLock::new(BTreeMap::new()),
            sessions: RwLock::new(HashMap::new()),
            session_index: RwLock::new(BTreeMap::new()),
            session_runtime_bindings: RwLock::new(HashMap::new()),
            session_checkpoints: RwLock::new(HashMap::new()),
            resource_user_states: RwLock::new(HashMap::new()),
            items: RwLock::new(HashMap::new()),
            item_feedback: RwLock::new(HashMap::new()),
            item_drive_refs: RwLock::new(HashMap::new()),
            session_item_index: RwLock::new(BTreeMap::new()),
            turns: RwLock::new(HashMap::new()),
            turn_idempotency: RwLock::new(HashMap::new()),
            interactions: RwLock::new(HashMap::new()),
            interaction_index: RwLock::new(BTreeMap::new()),
            tasks: RwLock::new(HashMap::new()),
            task_index: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn records(&self) -> Vec<AgentBusinessRecord> {
        self.agents.recovering_read().values().cloned().collect()
    }
}

fn agent_primary_key(record: &AgentBusinessRecord) -> AgentPrimaryKey {
    (record.tenant_id, record.agent_id.clone())
}

fn agent_index_key(record: &AgentBusinessRecord) -> AgentIndexKey {
    (
        record.tenant_id,
        Reverse(record.updated_at.clone()),
        Reverse(record.id),
        record.agent_id.clone(),
    )
}

fn project_primary_key(record: &AgentProjectRecord) -> ProjectPrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.project_id.clone(),
    )
}

fn project_index_key(record: &AgentProjectRecord) -> ProjectIndexKey {
    (
        record.tenant_id,
        record.organization_id,
        Reverse(record.updated_at.clone()),
        Reverse(record.id),
        record.project_id.clone(),
    )
}

fn workspace_primary_key(record: &AgentWorkspaceRecord) -> WorkspacePrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.workspace_id.clone(),
    )
}

fn workspace_index_key(record: &AgentWorkspaceRecord) -> WorkspaceIndexKey {
    (
        record.tenant_id,
        record.organization_id,
        record.owner_user_id,
        Reverse(record.is_default),
        Reverse(record.updated_at.clone()),
        Reverse(record.id),
        record.workspace_id.clone(),
    )
}

fn provider_binding_primary_key(record: &AgentProviderBindingRecord) -> ProviderBindingPrimaryKey {
    (
        record.tenant_id,
        record.agent_id.clone(),
        record.binding_id.clone(),
    )
}

fn provider_binding_index_key(record: &AgentProviderBindingRecord) -> ProviderBindingIndexKey {
    (
        record.tenant_id,
        record.agent_id.clone(),
        Reverse(record.active),
        Reverse(record.updated_at.clone()),
        record.binding_id.clone(),
    )
}

fn composition_slot_primary_key(record: &AgentCompositionSlotRecord) -> CompositionSlotPrimaryKey {
    (
        record.tenant_id,
        record.agent_id.clone(),
        record.slot_id.clone(),
    )
}

fn composition_slot_index_key(record: &AgentCompositionSlotRecord) -> CompositionSlotIndexKey {
    (
        record.tenant_id,
        record.agent_id.clone(),
        record.priority,
        record.slot_id.clone(),
    )
}

fn session_primary_key(record: &AgentSessionRecord) -> SessionPrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.session_id.clone(),
    )
}

fn session_index_key(record: &AgentSessionRecord) -> SessionIndexKey {
    (
        record.tenant_id,
        record.organization_id,
        Reverse(record.updated_at.clone()),
        record.session_id.clone(),
    )
}

fn resource_user_state_primary_key(
    record: &AgentResourceUserStateRecord,
) -> ResourceUserStatePrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.user_id,
        record.resource_type.as_db_code(),
        record.resource_id.clone(),
    )
}

fn resource_user_state_matches_agent(
    record: &AgentResourceUserStateRecord,
    query: &ResourceUserStateListQuery,
    sessions: &HashMap<SessionPrimaryKey, AgentSessionRecord>,
) -> bool {
    let Some(agent_id) = query.agent_id.as_deref() else {
        return true;
    };
    if record.resource_type != AgentResourceType::Session {
        return false;
    }
    sessions
        .get(&(
            record.tenant_id,
            record.organization_id,
            record.resource_id.clone(),
        ))
        .is_some_and(|session| {
            session.organization_id == record.organization_id
                && session.owner_user_id == record.user_id
                && session.agent_id == agent_id
                && session.deleted_at.is_none()
        })
}

fn session_item_primary_key(record: &AgentSessionItemRecord) -> SessionItemPrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.session_id.clone(),
        record.item_id.clone(),
    )
}

fn item_feedback_primary_key(record: &AgentItemFeedbackRecord) -> ItemFeedbackPrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.item_id.clone(),
        record.user_id,
    )
}

fn session_item_index_key(record: &AgentSessionItemRecord) -> SessionItemIndexKey {
    (
        record.tenant_id,
        record.organization_id,
        record.session_id.clone(),
        record.sequence,
        record.item_id.clone(),
    )
}

fn interaction_primary_key(record: &AgentInteractionRecord) -> InteractionPrimaryKey {
    (
        record.tenant_id,
        record.organization_id,
        record.session_id.clone(),
        record.interaction_id.clone(),
    )
}

fn interaction_index_key(record: &AgentInteractionRecord) -> InteractionIndexKey {
    (
        record.tenant_id,
        record.organization_id,
        record.session_id.clone(),
        Reverse(record.created_at.clone()),
        record.interaction_id.clone(),
    )
}

fn task_primary_key(record: &AgentTaskRecord) -> TaskPrimaryKey {
    (record.tenant_id, record.task_id.clone())
}

fn task_index_key(record: &AgentTaskRecord) -> TaskIndexKey {
    (
        record.tenant_id,
        Reverse(record.updated_at.clone()),
        record.task_id.clone(),
    )
}

fn active_agent_ids_for_tenant(
    agents: &HashMap<AgentPrimaryKey, AgentBusinessRecord>,
    agent_list_index: &BTreeMap<AgentIndexKey, AgentPrimaryKey>,
    tenant_id: u64,
) -> HashSet<String> {
    agent_list_index
        .iter()
        .filter(|((indexed_tenant_id, _, _, _), _)| *indexed_tenant_id == tenant_id)
        .filter_map(|(_, primary_key)| agents.get(primary_key))
        .filter(|record| !record.is_deleted())
        .map(|record| record.agent_id.clone())
        .collect()
}

impl Default for InMemoryAgentRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRepository for InMemoryAgentRepository {
    fn check_readiness(&self) -> KernelResult<()> {
        Ok(())
    }

    fn next_id(&self) -> KernelResult<u64> {
        self.id_generator.next_id()
    }

    fn insert(&self, record: AgentBusinessRecord) -> KernelResult<()> {
        let primary_key = agent_primary_key(&record);
        let mut agents = self.agents.recovering_write();
        if agents
            .values()
            .any(|existing| existing.tenant_id == record.tenant_id && existing.code == record.code)
        {
            return Err(KernelError::conflict("agent code already exists"));
        }
        if agents.contains_key(&primary_key) {
            return Err(KernelError::conflict("agent already exists"));
        }
        let index_key = agent_index_key(&record);
        agents.insert(primary_key.clone(), record);
        self.agent_list_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update(&self, record: AgentBusinessRecord) -> KernelResult<()> {
        let primary_key = agent_primary_key(&record);
        let mut agents = self.agents.recovering_write();
        let existing = agents
            .get(&primary_key)
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        let expected_version = existing.version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "agent version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        if agents.iter().any(|(key, existing)| {
            *key != primary_key
                && existing.tenant_id == record.tenant_id
                && existing.code == record.code
        }) {
            return Err(KernelError::conflict("agent code already exists"));
        }
        let previous_index_key = agent_index_key(existing);
        let next_index_key = agent_index_key(&record);
        agents.insert(primary_key.clone(), record);
        let mut index = self.agent_list_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get(&self, tenant_id: u64, agent_id: &str) -> KernelResult<Option<AgentBusinessRecord>> {
        Ok(self
            .agents
            .recovering_read()
            .get(&(tenant_id, agent_id.to_string()))
            .cloned())
    }

    fn list(&self, query: &AgentListQuery) -> KernelResult<Vec<AgentBusinessRecord>> {
        let agents = self.agents.recovering_read();
        let index = self.agent_list_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, _, _, _), _)| *tenant_id == query.tenant_id)
            .filter_map(|(_, primary_key)| agents.get(primary_key))
            .filter(|record| agent_matches_list_query(record, query))
            .cloned();
        Ok(paginate_iterator(iter, &query.pagination))
    }

    fn count_agents(&self, query: &AgentListQuery) -> KernelResult<u64> {
        let agents = self.agents.recovering_read();
        let index = self.agent_list_index.recovering_read();
        Ok(count_iterator(
            index
                .iter()
                .filter(|((tenant_id, _, _, _), _)| *tenant_id == query.tenant_id)
                .filter_map(|(_, primary_key)| agents.get(primary_key))
                .filter(|record| agent_matches_list_query(record, query)),
        ))
    }

    fn insert_workspace(&self, record: AgentWorkspaceRecord) -> KernelResult<()> {
        let primary_key = workspace_primary_key(&record);
        let mut workspaces = self.workspaces.recovering_write();
        if workspaces.contains_key(&primary_key) {
            return Err(KernelError::conflict("workspace already exists"));
        }
        if record.is_default
            && workspaces.values().any(|existing| {
                existing.tenant_id == record.tenant_id
                    && existing.organization_id == record.organization_id
                    && existing.owner_user_id == record.owner_user_id
                    && existing.is_default
                    && existing.status != AgentWorkspaceStatus::Deleted
            })
        {
            return Err(KernelError::conflict("default workspace already exists"));
        }
        let index_key = workspace_index_key(&record);
        workspaces.insert(primary_key.clone(), record);
        self.workspace_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update_workspace(&self, record: AgentWorkspaceRecord) -> KernelResult<()> {
        let primary_key = workspace_primary_key(&record);
        let mut workspaces = self.workspaces.recovering_write();
        let existing = workspaces
            .get(&primary_key)
            .ok_or_else(|| KernelError::validation("workspace not found"))?;
        if existing.status == AgentWorkspaceStatus::Deleted {
            return Err(KernelError::validation("workspace not found"));
        }
        if record.version != existing.version.saturating_add(1) {
            return Err(KernelError::conflict("workspace version mismatch"));
        }
        if record.owner_user_id != existing.owner_user_id
            || record.is_default != existing.is_default
            || record.created_by != existing.created_by
            || record.created_at != existing.created_at
        {
            return Err(KernelError::validation(
                "workspace immutable identity cannot be changed",
            ));
        }
        let previous_index_key = workspace_index_key(existing);
        let next_index_key = workspace_index_key(&record);
        workspaces.insert(primary_key.clone(), record);
        let mut index = self.workspace_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_workspace(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
    ) -> KernelResult<Option<AgentWorkspaceRecord>> {
        Ok(self
            .workspaces
            .recovering_read()
            .get(&(tenant_id, organization_id, workspace_id.to_string()))
            .filter(|record| record.status != AgentWorkspaceStatus::Deleted)
            .cloned())
    }

    fn get_default_workspace(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
    ) -> KernelResult<Option<AgentWorkspaceRecord>> {
        Ok(self
            .workspaces
            .recovering_read()
            .values()
            .find(|record| {
                record.tenant_id == tenant_id
                    && record.organization_id == organization_id
                    && record.owner_user_id == owner_user_id
                    && record.is_default
                    && record.status == AgentWorkspaceStatus::Active
            })
            .cloned())
    }

    fn list_workspaces(
        &self,
        query: &WorkspaceListQuery,
    ) -> KernelResult<Vec<AgentWorkspaceRecord>> {
        let workspaces = self.workspaces.recovering_read();
        let index = self.workspace_index.recovering_read();
        let records = index
            .iter()
            .filter(
                |((tenant_id, organization_id, owner_user_id, _, _, _, _), _)| {
                    *tenant_id == query.tenant_id
                        && *organization_id == query.organization_id
                        && *owner_user_id == query.owner_user_id
                },
            )
            .filter_map(|(_, key)| workspaces.get(key))
            .filter(|record| workspace_matches_list_query(record, query))
            .cloned();
        Ok(paginate_iterator(records, &query.pagination))
    }

    fn count_workspaces(&self, query: &WorkspaceListQuery) -> KernelResult<u64> {
        Ok(count_iterator(
            self.workspaces
                .recovering_read()
                .values()
                .filter(|record| workspace_matches_list_query(record, query)),
        ))
    }

    fn insert_project(&self, record: AgentProjectRecord) -> KernelResult<()> {
        let primary_key = project_primary_key(&record);
        let mut projects = self.projects.recovering_write();
        if projects.contains_key(&primary_key) {
            return Err(KernelError::conflict("project already exists"));
        }
        if let (Some(source_kind), Some(source_ref)) = (
            record.import_source_kind.as_deref(),
            record.import_source_ref.as_deref(),
        ) {
            let import_source_exists = projects.values().any(|existing| {
                existing.tenant_id == record.tenant_id
                    && existing.organization_id == record.organization_id
                    && existing.owner_user_id == record.owner_user_id
                    && existing.status != AgentProjectStatus::Deleted
                    && existing.import_source_kind.as_deref() == Some(source_kind)
                    && existing.import_source_ref.as_deref() == Some(source_ref)
            });
            if import_source_exists {
                return Err(KernelError::conflict(
                    "project import source already exists",
                ));
            }
        }
        let index_key = project_index_key(&record);
        projects.insert(primary_key.clone(), record);
        self.project_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update_project(&self, record: AgentProjectRecord) -> KernelResult<()> {
        let primary_key = project_primary_key(&record);
        let mut projects = self.projects.recovering_write();
        let existing = projects
            .get(&primary_key)
            .ok_or_else(|| KernelError::validation("project not found"))?;
        let expected_version = existing.version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "project version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        if let (Some(source_kind), Some(source_ref)) = (
            record.import_source_kind.as_deref(),
            record.import_source_ref.as_deref(),
        ) {
            let import_source_exists = projects.iter().any(|(key, candidate)| {
                key != &primary_key
                    && candidate.tenant_id == record.tenant_id
                    && candidate.organization_id == record.organization_id
                    && candidate.owner_user_id == record.owner_user_id
                    && candidate.status != AgentProjectStatus::Deleted
                    && candidate.import_source_kind.as_deref() == Some(source_kind)
                    && candidate.import_source_ref.as_deref() == Some(source_ref)
            });
            if import_source_exists {
                return Err(KernelError::conflict(
                    "project import source already exists",
                ));
            }
        }
        let previous_index_key = project_index_key(existing);
        let next_index_key = project_index_key(&record);
        projects.insert(primary_key.clone(), record);
        let mut index = self.project_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_project(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
    ) -> KernelResult<Option<AgentProjectRecord>> {
        Ok(self
            .projects
            .recovering_read()
            .get(&(tenant_id, organization_id, project_id.to_string()))
            .filter(|record| record.status != AgentProjectStatus::Deleted)
            .cloned())
    }

    fn get_project_by_import_source(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        source_kind: &str,
        source_ref: &str,
    ) -> KernelResult<Option<AgentProjectRecord>> {
        Ok(self
            .projects
            .recovering_read()
            .values()
            .find(|record| {
                record.tenant_id == tenant_id
                    && record.organization_id == organization_id
                    && record.owner_user_id == owner_user_id
                    && record.status != AgentProjectStatus::Deleted
                    && record.import_source_kind.as_deref() == Some(source_kind)
                    && record.import_source_ref.as_deref() == Some(source_ref)
            })
            .cloned())
    }

    fn list_projects(&self, query: &ProjectListQuery) -> KernelResult<Vec<AgentProjectRecord>> {
        let projects = self.projects.recovering_read();
        let index = self.project_index.recovering_read();
        let records = index
            .iter()
            .filter(|((tenant_id, organization_id, _, _, _), _)| {
                *tenant_id == query.tenant_id && *organization_id == query.organization_id
            })
            .filter_map(|(_, key)| projects.get(key))
            .filter(|record| project_matches_list_query(record, query))
            .cloned();
        Ok(paginate_iterator(records, &query.pagination))
    }

    fn count_projects(&self, query: &ProjectListQuery) -> KernelResult<u64> {
        let projects = self.projects.recovering_read();
        Ok(count_iterator(projects.values().filter(|record| {
            project_matches_list_query(record, query)
        })))
    }

    fn insert_project_composition_slot(
        &self,
        record: AgentProjectCompositionSlotRecord,
    ) -> KernelResult<()> {
        let key = (
            record.tenant_id,
            record.organization_id,
            record.project_id.clone(),
            record.slot_id.clone(),
        );
        let mut slots = self.project_composition_slots.recovering_write();
        if slots.contains_key(&key) {
            return Err(KernelError::conflict(
                "project composition slot already exists",
            ));
        }
        slots.insert(key, record);
        Ok(())
    }

    fn update_project_composition_slot(
        &self,
        record: AgentProjectCompositionSlotRecord,
    ) -> KernelResult<()> {
        let key = (
            record.tenant_id,
            record.organization_id,
            record.project_id.clone(),
            record.slot_id.clone(),
        );
        let mut slots = self.project_composition_slots.recovering_write();
        let existing = slots
            .get(&key)
            .ok_or_else(|| KernelError::validation("project composition slot not found"))?;
        if record.version != existing.version.saturating_add(1) {
            return Err(KernelError::conflict(
                "project composition slot version mismatch",
            ));
        }
        slots.insert(key, record);
        Ok(())
    }

    fn get_project_composition_slot(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentProjectCompositionSlotRecord>> {
        Ok(self
            .project_composition_slots
            .recovering_read()
            .get(&(
                tenant_id,
                organization_id,
                project_id.to_string(),
                slot_id.to_string(),
            ))
            .cloned())
    }

    fn list_project_composition_slots(
        &self,
        query: &ProjectCompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentProjectCompositionSlotRecord>> {
        let mut slots = self
            .project_composition_slots
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.project_id == query.project_id
                    && record.deleted_at.is_none()
                    && query
                        .slot_kind
                        .as_ref()
                        .is_none_or(|kind| &record.slot_kind == kind)
                    && query
                        .enabled
                        .is_none_or(|enabled| record.enabled == enabled)
            })
            .cloned()
            .collect::<Vec<_>>();
        slots.sort_by(|left, right| {
            left.priority
                .cmp(&right.priority)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(paginate_iterator(slots.into_iter(), &query.pagination))
    }

    fn count_project_composition_slots(
        &self,
        query: &ProjectCompositionSlotListQuery,
    ) -> KernelResult<u64> {
        Ok(self
            .project_composition_slots
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.project_id == query.project_id
                    && record.deleted_at.is_none()
                    && query
                        .slot_kind
                        .as_ref()
                        .is_none_or(|kind| &record.slot_kind == kind)
                    && query
                        .enabled
                        .is_none_or(|enabled| record.enabled == enabled)
            })
            .count() as u64)
    }

    fn insert_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        let primary_key = provider_binding_primary_key(&record);
        let mut bindings = self.provider_bindings.recovering_write();
        if bindings.contains_key(&primary_key) {
            return Err(KernelError::conflict(
                "agent provider binding already exists",
            ));
        }
        if record.active
            && bindings.values().any(|existing| {
                existing.tenant_id == record.tenant_id
                    && existing.agent_id == record.agent_id
                    && existing.active
            })
        {
            return Err(KernelError::conflict(
                "active provider binding already exists",
            ));
        }
        let index_key = provider_binding_index_key(&record);
        bindings.insert(primary_key.clone(), record);
        self.provider_binding_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        let primary_key = provider_binding_primary_key(&record);
        let mut bindings = self.provider_bindings.recovering_write();
        let existing = bindings
            .get(&primary_key)
            .ok_or_else(|| KernelError::validation("agent provider binding not found"))?;
        let expected_version = existing.version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "provider binding version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        if record.active
            && bindings.iter().any(|(key, existing)| {
                *key != primary_key
                    && existing.tenant_id == record.tenant_id
                    && existing.agent_id == record.agent_id
                    && existing.active
            })
        {
            return Err(KernelError::conflict(
                "active provider binding already exists",
            ));
        }
        let previous_index_key = provider_binding_index_key(existing);
        let next_index_key = provider_binding_index_key(&record);
        bindings.insert(primary_key.clone(), record);
        let mut index = self.provider_binding_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRecord>> {
        Ok(self
            .provider_bindings
            .recovering_read()
            .get(&(tenant_id, agent_id.to_string(), binding_id.to_string()))
            .cloned())
    }

    fn get_active_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRecord>> {
        Ok(self
            .provider_bindings
            .recovering_read()
            .values()
            .find(|binding| {
                binding.tenant_id == tenant_id && binding.agent_id == agent_id && binding.active
            })
            .cloned())
    }

    fn list_provider_bindings(
        &self,
        query: &ProviderBindingListQuery,
    ) -> KernelResult<Vec<AgentProviderBindingRecord>> {
        let bindings = self.provider_bindings.recovering_read();
        let index = self.provider_binding_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, agent_id, _, _, _), _)| {
                *tenant_id == query.tenant_id && agent_id == &query.agent_id
            })
            .filter_map(|(_, primary_key)| bindings.get(primary_key))
            .cloned();
        Ok(paginate_iterator(iter, &query.pagination))
    }

    fn count_provider_bindings(&self, query: &ProviderBindingListQuery) -> KernelResult<u64> {
        let bindings = self.provider_bindings.recovering_read();
        let index = self.provider_binding_index.recovering_read();
        Ok(count_iterator(
            index
                .iter()
                .filter(|((tenant_id, agent_id, _, _, _), _)| {
                    *tenant_id == query.tenant_id && agent_id == &query.agent_id
                })
                .filter_map(|(_, primary_key)| bindings.get(primary_key)),
        ))
    }

    fn insert_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        let primary_key = composition_slot_primary_key(&record);
        let mut slots = self.composition_slots.recovering_write();
        if slots.contains_key(&primary_key) {
            return Err(KernelError::conflict("composition slot already exists"));
        }
        let index_key = composition_slot_index_key(&record);
        slots.insert(primary_key.clone(), record);
        self.composition_slot_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        let primary_key = composition_slot_primary_key(&record);
        let mut slots = self.composition_slots.recovering_write();
        let existing = slots
            .get(&primary_key)
            .ok_or_else(|| KernelError::validation("composition slot not found"))?;
        let expected_version = existing.version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "composition slot version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        let previous_index_key = composition_slot_index_key(existing);
        let next_index_key = composition_slot_index_key(&record);
        slots.insert(primary_key.clone(), record);
        let mut index = self.composition_slot_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_composition_slot(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentCompositionSlotRecord>> {
        Ok(self
            .composition_slots
            .recovering_read()
            .get(&(tenant_id, agent_id.to_string(), slot_id.to_string()))
            .cloned())
    }

    fn list_composition_slots(
        &self,
        query: &CompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>> {
        let slots = self.composition_slots.recovering_read();
        let index = self.composition_slot_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, agent_id, _, _), _)| {
                *tenant_id == query.tenant_id && agent_id == &query.agent_id
            })
            .filter_map(|(_, primary_key)| slots.get(primary_key))
            .filter(|record| !record.is_deleted())
            .cloned();
        Ok(paginate_iterator(iter, &query.pagination))
    }

    fn count_composition_slots(&self, query: &CompositionSlotListQuery) -> KernelResult<u64> {
        let slots = self.composition_slots.recovering_read();
        let index = self.composition_slot_index.recovering_read();
        Ok(count_iterator(
            index
                .iter()
                .filter(|((tenant_id, agent_id, _, _), _)| {
                    *tenant_id == query.tenant_id && agent_id == &query.agent_id
                })
                .filter_map(|(_, primary_key)| slots.get(primary_key))
                .filter(|record| !record.is_deleted()),
        ))
    }

    fn list_mcp_marketplace_slots(
        &self,
        query: &McpMarketplaceListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>> {
        let agents = self.agents.recovering_read();
        let agent_index = self.agent_list_index.recovering_read();
        let active_agent_ids = active_agent_ids_for_tenant(&agents, &agent_index, query.tenant_id);
        let slots = self.composition_slots.recovering_read();
        let index = self.composition_slot_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, _, _, _), _)| *tenant_id == query.tenant_id)
            .filter_map(|(_, primary_key)| slots.get(primary_key))
            .filter(|record| {
                record.slot_kind == AgentCompositionSlotKind::Mcp
                    && !record.is_deleted()
                    && active_agent_ids.contains(&record.agent_id)
                    && query
                        .q
                        .as_deref()
                        .map(|q| {
                            crate::mcp_marketplace::composition_slot_matches_mcp_search(record, q)
                        })
                        .unwrap_or(true)
            })
            .cloned();
        Ok(paginate_iterator(iter, &query.pagination))
    }

    fn count_mcp_marketplace_slots(&self, query: &McpMarketplaceListQuery) -> KernelResult<u64> {
        let agents = self.agents.recovering_read();
        let agent_index = self.agent_list_index.recovering_read();
        let active_agent_ids = active_agent_ids_for_tenant(&agents, &agent_index, query.tenant_id);
        let slots = self.composition_slots.recovering_read();
        let index = self.composition_slot_index.recovering_read();
        Ok(count_iterator(
            index
                .iter()
                .filter(|((tenant_id, _, _, _), _)| *tenant_id == query.tenant_id)
                .filter_map(|(_, primary_key)| slots.get(primary_key))
                .filter(|record| {
                    record.slot_kind == AgentCompositionSlotKind::Mcp
                        && !record.is_deleted()
                        && active_agent_ids.contains(&record.agent_id)
                        && query
                            .q
                            .as_deref()
                            .map(|q| {
                                crate::mcp_marketplace::composition_slot_matches_mcp_search(
                                    record, q,
                                )
                            })
                            .unwrap_or(true)
                }),
        ))
    }

    fn insert_session(&self, record: AgentSessionRecord) -> KernelResult<()> {
        let primary_key = session_primary_key(&record);
        let mut sessions = self.sessions.recovering_write();
        if sessions.contains_key(&primary_key) {
            return Err(KernelError::conflict("session already exists"));
        }
        let index_key = session_index_key(&record);
        sessions.insert(primary_key.clone(), record);
        self.session_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update_session(&self, record: AgentSessionRecord) -> KernelResult<()> {
        let primary_key = session_primary_key(&record);
        let mut sessions = self.sessions.recovering_write();
        let existing = sessions
            .get(&primary_key)
            .ok_or_else(|| KernelError::validation("session not found"))?;
        let expected_version = existing.version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "session version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        let previous_index_key = session_index_key(existing);
        let next_index_key = session_index_key(&record);
        sessions.insert(primary_key.clone(), record);
        let mut index = self.session_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRecord>> {
        Ok(self
            .sessions
            .recovering_read()
            .get(&(tenant_id, organization_id, session_id.to_string()))
            .filter(|record| record.deleted_at.is_none())
            .cloned())
    }

    fn list_sessions(&self, query: &SessionListQuery) -> KernelResult<Vec<AgentSessionRecord>> {
        let sessions = self.sessions.recovering_read();
        let index = self.session_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, _, _, _), _)| *tenant_id == query.tenant_id)
            .filter_map(|(_, primary_key)| sessions.get(primary_key))
            .filter(|record| session_matches_list_query(record, query))
            .cloned();
        Ok(paginate_iterator(iter, &query.pagination))
    }

    fn count_sessions(&self, query: &SessionListQuery) -> KernelResult<u64> {
        let sessions = self.sessions.recovering_read();
        let index = self.session_index.recovering_read();
        Ok(count_iterator(
            index
                .iter()
                .filter(|((tenant_id, _, _, _), _)| *tenant_id == query.tenant_id)
                .filter_map(|(_, primary_key)| sessions.get(primary_key))
                .filter(|record| session_matches_list_query(record, query)),
        ))
    }

    fn insert_session_runtime_binding(
        &self,
        record: AgentSessionRuntimeBindingRecord,
    ) -> KernelResult<()> {
        let key = (
            record.tenant_id,
            record.organization_id,
            record.session_id.clone(),
            record.runtime_binding_id.clone(),
        );
        let mut bindings = self.session_runtime_bindings.recovering_write();
        if bindings.contains_key(&key) {
            return Err(KernelError::conflict(
                "session runtime binding already exists",
            ));
        }
        if record.is_current
            && bindings.values().any(|candidate| {
                candidate.tenant_id == record.tenant_id
                    && candidate.organization_id == record.organization_id
                    && candidate.session_id == record.session_id
                    && candidate.is_current
            })
        {
            return Err(KernelError::conflict(
                "session already has a current runtime binding",
            ));
        }
        bindings.insert(key, record);
        Ok(())
    }

    fn update_session_runtime_binding(
        &self,
        record: AgentSessionRuntimeBindingRecord,
    ) -> KernelResult<()> {
        let key = (
            record.tenant_id,
            record.organization_id,
            record.session_id.clone(),
            record.runtime_binding_id.clone(),
        );
        let mut bindings = self.session_runtime_bindings.recovering_write();
        let existing = bindings
            .get(&key)
            .ok_or_else(|| KernelError::validation("session runtime binding not found"))?;
        if record.version != existing.version.saturating_add(1) {
            return Err(KernelError::conflict(
                "session runtime binding version mismatch",
            ));
        }
        if record.is_current
            && bindings.iter().any(|(candidate_key, candidate)| {
                candidate_key != &key
                    && candidate.tenant_id == record.tenant_id
                    && candidate.organization_id == record.organization_id
                    && candidate.session_id == record.session_id
                    && candidate.is_current
            })
        {
            return Err(KernelError::conflict(
                "session already has a current runtime binding",
            ));
        }
        bindings.insert(key, record);
        Ok(())
    }

    fn get_session_runtime_binding(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRecord>> {
        Ok(self
            .session_runtime_bindings
            .recovering_read()
            .get(&(
                tenant_id,
                organization_id,
                session_id.to_string(),
                runtime_binding_id.to_string(),
            ))
            .cloned())
    }

    fn get_current_session_runtime_binding(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRecord>> {
        Ok(self
            .session_runtime_bindings
            .recovering_read()
            .values()
            .find(|record| {
                record.tenant_id == tenant_id
                    && record.organization_id == organization_id
                    && record.session_id == session_id
                    && record.is_current
                    && record.status == AgentSessionRuntimeBindingStatus::Active
            })
            .cloned())
    }

    fn list_session_runtime_bindings(
        &self,
        query: &SessionRuntimeBindingListQuery,
    ) -> KernelResult<Vec<AgentSessionRuntimeBindingRecord>> {
        let mut records = self
            .session_runtime_bindings
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.session_id == query.session_id
                    && (!query.current_only || record.is_current)
                    && query
                        .status
                        .as_deref()
                        .map(|status| record.status.as_str() == status)
                        .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            right
                .is_current
                .cmp(&left.is_current)
                .then_with(|| right.updated_at.cmp(&left.updated_at))
                .then_with(|| right.id.cmp(&left.id))
        });
        Ok(paginate_iterator(records.into_iter(), &query.pagination))
    }

    fn count_session_runtime_bindings(
        &self,
        query: &SessionRuntimeBindingListQuery,
    ) -> KernelResult<u64> {
        Ok(count_iterator(
            self.session_runtime_bindings
                .recovering_read()
                .values()
                .filter(|record| {
                    record.tenant_id == query.tenant_id
                        && record.organization_id == query.organization_id
                        && record.session_id == query.session_id
                        && (!query.current_only || record.is_current)
                        && query
                            .status
                            .as_deref()
                            .map(|status| record.status.as_str() == status)
                            .unwrap_or(true)
                }),
        ))
    }

    fn activate_session_runtime_binding_atomic(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
        expected_version: u64,
        updated_at: String,
    ) -> KernelResult<AgentSessionRuntimeBindingRecord> {
        let target_key = (
            tenant_id,
            organization_id,
            session_id.to_string(),
            runtime_binding_id.to_string(),
        );
        let mut bindings = self.session_runtime_bindings.recovering_write();
        let target = bindings
            .get(&target_key)
            .cloned()
            .ok_or_else(|| KernelError::validation("session runtime binding not found"))?;
        if target.version != expected_version {
            return Err(KernelError::conflict(
                "session runtime binding version mismatch",
            ));
        }
        if target.is_current && target.status == AgentSessionRuntimeBindingStatus::Active {
            return Ok(target);
        }
        for candidate in bindings.values_mut().filter(|candidate| {
            candidate.tenant_id == tenant_id
                && candidate.organization_id == organization_id
                && candidate.session_id == session_id
                && candidate.runtime_binding_id != runtime_binding_id
                && candidate.is_current
        }) {
            candidate.deactivate(
                AgentSessionRuntimeBindingStatus::Deactivated,
                updated_at.clone(),
            );
        }
        let target = bindings
            .get_mut(&target_key)
            .ok_or_else(|| KernelError::validation("session runtime binding not found"))?;
        target.activate(updated_at);
        Ok(target.clone())
    }

    fn insert_session_checkpoint(&self, record: AgentSessionCheckpointRecord) -> KernelResult<()> {
        let key = (
            record.tenant_id,
            record.organization_id,
            record.session_id.clone(),
            record.checkpoint_id.clone(),
        );
        let mut checkpoints = self.session_checkpoints.recovering_write();
        if checkpoints.contains_key(&key) {
            return Err(KernelError::conflict("session checkpoint already exists"));
        }
        checkpoints.insert(key, record);
        Ok(())
    }

    fn update_session_checkpoint(&self, record: AgentSessionCheckpointRecord) -> KernelResult<()> {
        let key = (
            record.tenant_id,
            record.organization_id,
            record.session_id.clone(),
            record.checkpoint_id.clone(),
        );
        let mut checkpoints = self.session_checkpoints.recovering_write();
        let existing = checkpoints
            .get(&key)
            .ok_or_else(|| KernelError::validation("session checkpoint not found"))?;
        if record.version != existing.version.saturating_add(1) {
            return Err(KernelError::conflict("session checkpoint version mismatch"));
        }
        checkpoints.insert(key, record);
        Ok(())
    }

    fn get_session_checkpoint(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        checkpoint_id: &str,
    ) -> KernelResult<Option<AgentSessionCheckpointRecord>> {
        Ok(self
            .session_checkpoints
            .recovering_read()
            .get(&(
                tenant_id,
                organization_id,
                session_id.to_string(),
                checkpoint_id.to_string(),
            ))
            .cloned())
    }

    fn list_session_checkpoints(
        &self,
        query: &SessionCheckpointListQuery,
    ) -> KernelResult<Vec<AgentSessionCheckpointRecord>> {
        let mut records = self
            .session_checkpoints
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.session_id == query.session_id
                    && query
                        .status
                        .as_deref()
                        .map(|status| record.status.as_str() == status)
                        .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| right.id.cmp(&left.id))
        });
        Ok(paginate_iterator(records.into_iter(), &query.pagination))
    }

    fn count_session_checkpoints(&self, query: &SessionCheckpointListQuery) -> KernelResult<u64> {
        Ok(count_iterator(
            self.session_checkpoints
                .recovering_read()
                .values()
                .filter(|record| {
                    record.tenant_id == query.tenant_id
                        && record.organization_id == query.organization_id
                        && record.session_id == query.session_id
                        && query
                            .status
                            .as_deref()
                            .map(|status| record.status.as_str() == status)
                            .unwrap_or(true)
                }),
        ))
    }

    fn upsert_resource_user_state(
        &self,
        record: AgentResourceUserStateRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentResourceUserStateRecord> {
        let key = resource_user_state_primary_key(&record);
        let mut states = self.resource_user_states.recovering_write();
        match states.get(&key) {
            Some(existing) => {
                if expected_version != Some(existing.version) {
                    return Err(KernelError::conflict(
                        "resource user state version mismatch",
                    ));
                }
                if record.version != existing.version.saturating_add(1) {
                    return Err(KernelError::conflict(
                        "resource user state version mismatch",
                    ));
                }
            }
            None => {
                if expected_version.is_some() || record.version != 0 {
                    return Err(KernelError::conflict(
                        "resource user state version mismatch",
                    ));
                }
            }
        }
        states.insert(key, record.clone());
        Ok(record)
    }

    fn get_resource_user_state(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        resource_type: AgentResourceType,
        resource_id: &str,
    ) -> KernelResult<Option<AgentResourceUserStateRecord>> {
        Ok(self
            .resource_user_states
            .recovering_read()
            .get(&(
                tenant_id,
                organization_id,
                user_id,
                resource_type.as_db_code(),
                resource_id.to_owned(),
            ))
            .cloned())
    }

    fn list_resource_user_states(
        &self,
        query: &ResourceUserStateListQuery,
    ) -> KernelResult<Vec<AgentResourceUserStateRecord>> {
        let sessions = self.sessions.recovering_read();
        let mut records = self
            .resource_user_states
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.user_id == query.user_id
                    && record.resource_type == query.resource_type
                    && (!query.pinned_only || record.pinned_at.is_some())
                    && (query.include_hidden || record.hidden_at.is_none())
            })
            .filter(|record| resource_user_state_matches_agent(record, query, &sessions))
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            right
                .pinned_at
                .cmp(&left.pinned_at)
                .then_with(|| right.last_opened_at.cmp(&left.last_opened_at))
                .then_with(|| right.id.cmp(&left.id))
        });
        Ok(records
            .into_iter()
            .skip(query.pagination.offset)
            .take(query.pagination.page_size)
            .collect())
    }

    fn count_resource_user_states(&self, query: &ResourceUserStateListQuery) -> KernelResult<u64> {
        let sessions = self.sessions.recovering_read();
        Ok(self
            .resource_user_states
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.user_id == query.user_id
                    && record.resource_type == query.resource_type
                    && (!query.pinned_only || record.pinned_at.is_some())
                    && (query.include_hidden || record.hidden_at.is_none())
            })
            .filter(|record| resource_user_state_matches_agent(record, query, &sessions))
            .count() as u64)
    }

    fn append_session_item(
        &self,
        mut record: AgentSessionItemRecord,
    ) -> KernelResult<(AgentSessionRecord, AgentSessionItemRecord)> {
        if record.sequence != 0
            || record.turn_id.is_some()
            || record.status != AgentSessionItemStatus::Completed
        {
            return Err(KernelError::validation(
                "standalone session item must be an unsequenced completed item without a turn",
            ));
        }
        let session_primary_key = (
            record.tenant_id,
            record.organization_id,
            record.session_id.clone(),
        );
        let mut sessions = self.sessions.recovering_write();
        let mut session_index = self.session_index.recovering_write();
        let mut items = self.items.recovering_write();
        let mut item_index = self.session_item_index.recovering_write();
        let existing_session = sessions
            .get(&session_primary_key)
            .cloned()
            .ok_or_else(|| KernelError::validation("active session not found"))?;
        if !existing_session.status.is_active()
            || existing_session.deleted_at.is_some()
            || existing_session.owner_user_id != record.created_by
        {
            return Err(KernelError::validation("active session not found"));
        }
        record.sequence = existing_session.last_item_sequence.saturating_add(1);
        let primary_key = session_item_primary_key(&record);
        if items.contains_key(&primary_key) {
            return Err(KernelError::conflict("session item already exists"));
        }
        let index_key = session_item_index_key(&record);
        if item_index.contains_key(&index_key) {
            return Err(KernelError::conflict("session item sequence conflict"));
        }
        let mut updated_session = existing_session.clone();
        updated_session.updated_by = record.created_by;
        updated_session.record_item(
            record.input_tokens,
            record.output_tokens,
            record.updated_at.clone(),
        );
        let previous_session_index_key = session_index_key(&existing_session);
        let next_session_index_key = session_index_key(&updated_session);

        items.insert(primary_key.clone(), record.clone());
        item_index.insert(index_key, primary_key);
        sessions.insert(session_primary_key.clone(), updated_session.clone());
        session_index.remove(&previous_session_index_key);
        session_index.insert(next_session_index_key, session_primary_key);
        Ok((updated_session, record))
    }

    fn update_session_item(&self, record: AgentSessionItemRecord) -> KernelResult<()> {
        let primary_key = session_item_primary_key(&record);
        let mut items = self.items.recovering_write();
        let Some(existing) = items.get(&primary_key) else {
            return Err(KernelError::validation("session item not found"));
        };
        let previous_index_key = session_item_index_key(existing);
        let next_index_key = session_item_index_key(&record);
        items.insert(primary_key.clone(), record);
        let mut index = self.session_item_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_session_item(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        item_id: &str,
    ) -> KernelResult<Option<AgentSessionItemRecord>> {
        Ok(self
            .items
            .recovering_read()
            .get(&(
                tenant_id,
                organization_id,
                session_id.to_string(),
                item_id.to_string(),
            ))
            .cloned())
    }

    fn list_session_items(
        &self,
        query: &SessionItemListQuery,
    ) -> KernelResult<Vec<AgentSessionItemRecord>> {
        let items = self.items.recovering_read();
        let index = self.session_item_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, organization_id, session_id, _, _), _)| {
                *tenant_id == query.tenant_id
                    && *organization_id == query.organization_id
                    && session_id == &query.session_id
            })
            .filter_map(|(_, primary_key)| items.get(primary_key))
            .filter(|record| message_matches_list_query(record, query))
            .cloned();
        Ok(paginate_items(iter, &query.pagination, query.sort))
    }

    fn count_session_items(&self, query: &SessionItemListQuery) -> KernelResult<u64> {
        let items = self.items.recovering_read();
        let index = self.session_item_index.recovering_read();
        Ok(count_iterator(
            index
                .iter()
                .filter(|((tenant_id, organization_id, session_id, _, _), _)| {
                    *tenant_id == query.tenant_id
                        && *organization_id == query.organization_id
                        && session_id == &query.session_id
                })
                .filter_map(|(_, primary_key)| items.get(primary_key))
                .filter(|record| message_matches_list_query(record, query)),
        ))
    }

    fn upsert_item_feedback(
        &self,
        record: AgentItemFeedbackRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentItemFeedbackRecord> {
        let key = item_feedback_primary_key(&record);
        let mut feedback = self.item_feedback.recovering_write();
        match feedback.get(&key) {
            Some(existing) => {
                let reviving_without_version = existing.deleted_at.is_some()
                    && record.deleted_at.is_none()
                    && expected_version.is_none();
                if !reviving_without_version && expected_version != Some(existing.version) {
                    return Err(KernelError::conflict("item feedback version mismatch"));
                }
                if record.version != existing.version.saturating_add(1) {
                    return Err(KernelError::conflict("item feedback version mismatch"));
                }
            }
            None => {
                if expected_version.is_some() || record.version != 0 {
                    return Err(KernelError::conflict("item feedback version mismatch"));
                }
            }
        }
        feedback.insert(key, record.clone());
        Ok(record)
    }

    fn get_item_feedback(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
        user_id: u64,
        include_deleted: bool,
    ) -> KernelResult<Option<AgentItemFeedbackRecord>> {
        Ok(self
            .item_feedback
            .recovering_read()
            .get(&(tenant_id, organization_id, item_id.to_string(), user_id))
            .filter(|record| include_deleted || record.deleted_at.is_none())
            .cloned())
    }

    fn list_item_feedback(
        &self,
        query: &ItemFeedbackListQuery,
    ) -> KernelResult<Vec<AgentItemFeedbackRecord>> {
        let items = self.items.recovering_read();
        let mut records = self
            .item_feedback
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.user_id == query.user_id
                    && record.deleted_at.is_none()
            })
            .filter_map(|record| {
                items
                    .get(&(
                        query.tenant_id,
                        query.organization_id,
                        query.session_id.clone(),
                        record.item_id.clone(),
                    ))
                    .map(|message| (message.sequence, record.clone()))
            })
            .collect::<Vec<_>>();
        records.sort_by_key(|(sequence, record)| (*sequence, record.id));
        Ok(records
            .into_iter()
            .skip(query.pagination.offset)
            .take(query.pagination.page_size)
            .map(|(_, record)| record)
            .collect())
    }

    fn count_item_feedback(&self, query: &ItemFeedbackListQuery) -> KernelResult<u64> {
        let items = self.items.recovering_read();
        Ok(self
            .item_feedback
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == query.tenant_id
                    && record.organization_id == query.organization_id
                    && record.user_id == query.user_id
                    && record.deleted_at.is_none()
                    && items.contains_key(&(
                        query.tenant_id,
                        query.organization_id,
                        query.session_id.clone(),
                        record.item_id.clone(),
                    ))
            })
            .count() as u64)
    }

    fn get_turn_by_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentTurnRecord>> {
        let index = self.turn_idempotency.recovering_read();
        let turns = self.turns.recovering_read();
        Ok(index
            .get(&(
                tenant_id,
                organization_id,
                owner_user_id,
                idempotency_key.to_string(),
            ))
            .and_then(|primary_key| turns.get(primary_key))
            .cloned())
    }

    fn get_turn(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<Option<AgentTurnRecord>> {
        Ok(self
            .turns
            .recovering_read()
            .get(&(tenant_id, organization_id, turn_id.to_string()))
            .cloned())
    }

    fn list_turns(&self, query: &TurnListQuery) -> KernelResult<Vec<AgentTurnRecord>> {
        let mut records = self
            .turns
            .recovering_read()
            .values()
            .filter(|turn| {
                turn.tenant_id == query.tenant_id
                    && turn.organization_id == query.organization_id
                    && turn.session_id == query.session_id
                    && query
                        .status
                        .as_deref()
                        .map(|status| turn.status.as_str() == status)
                        .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| right.id.cmp(&left.id))
        });
        Ok(paginate_iterator(records.into_iter(), &query.pagination))
    }

    fn count_turns(&self, query: &TurnListQuery) -> KernelResult<u64> {
        Ok(count_iterator(
            self.turns.recovering_read().values().filter(|turn| {
                turn.tenant_id == query.tenant_id
                    && turn.organization_id == query.organization_id
                    && turn.session_id == query.session_id
                    && query
                        .status
                        .as_deref()
                        .map(|status| turn.status.as_str() == status)
                        .unwrap_or(true)
            }),
        ))
    }

    fn list_reconcilable_turns(
        &self,
        stale_before: &str,
        limit: usize,
    ) -> KernelResult<Vec<AgentTurnRecord>> {
        let mut turns = self
            .turns
            .recovering_read()
            .values()
            .filter(|turn| {
                matches!(
                    turn.status,
                    crate::agent_turn::AgentTurnStatus::Requested
                        | crate::agent_turn::AgentTurnStatus::Running
                ) && turn.updated_at.as_str() < stale_before
            })
            .cloned()
            .collect::<Vec<_>>();
        turns.sort_by(|left, right| {
            left.updated_at
                .cmp(&right.updated_at)
                .then_with(|| left.id.cmp(&right.id))
        });
        turns.truncate(limit.min(200));
        Ok(turns)
    }

    fn insert_turn_request(
        &self,
        turn: AgentTurnRecord,
        mut request_item: AgentSessionItemRecord,
        drive_refs: Vec<AgentItemDriveRefRecord>,
    ) -> KernelResult<TurnRequestWriteOutcome> {
        if turn.status != AgentTurnStatus::Requested
            || turn.version != 0
            || turn.response_item_id.is_some()
            || turn.request_item_id != request_item.item_id
            || turn.tenant_id != request_item.tenant_id
            || turn.organization_id != request_item.organization_id
            || turn.session_id != request_item.session_id
            || request_item.turn_id.as_deref() != Some(turn.turn_id.as_str())
        {
            return Err(KernelError::validation(
                "turn request and session item scope mismatch",
            ));
        }

        let primary_key = (turn.tenant_id, turn.organization_id, turn.turn_id.clone());
        let idempotency_key = (
            turn.tenant_id,
            turn.organization_id,
            turn.owner_user_id,
            turn.idempotency_key.clone(),
        );
        let session_primary_key = (
            turn.tenant_id,
            turn.organization_id,
            turn.session_id.clone(),
        );
        let mut pending_drive_refs = Vec::with_capacity(drive_refs.len());
        for record in drive_refs {
            if record.tenant_id != request_item.tenant_id
                || record.organization_id != request_item.organization_id
                || record.item_id != request_item.item_id
            {
                return Err(KernelError::validation(
                    "session item Drive reference scope mismatch",
                ));
            }
            let key = (
                record.tenant_id,
                record.organization_id,
                record.item_id.clone(),
                record.drive_node_id.clone(),
                record.resource_role.as_str().to_string(),
            );
            if pending_drive_refs
                .iter()
                .any(|(candidate, _)| candidate == &key)
            {
                return Err(KernelError::conflict(
                    "duplicate session item Drive reference",
                ));
            }
            pending_drive_refs.push((key, record));
        }

        let mut turns = self.turns.recovering_write();
        let mut turn_idempotency = self.turn_idempotency.recovering_write();
        let mut sessions = self.sessions.recovering_write();
        let mut session_index = self.session_index.recovering_write();
        let mut items = self.items.recovering_write();
        let mut session_item_index = self.session_item_index.recovering_write();
        let mut item_drive_refs = self.item_drive_refs.recovering_write();

        if let Some(existing) = turns.get(&primary_key) {
            if existing.idempotency_key == turn.idempotency_key {
                return Ok(TurnRequestWriteOutcome::Existing(Box::new(
                    existing.clone(),
                )));
            }
            return Err(KernelError::conflict("turn idempotency conflict"));
        }
        if let Some(existing_primary_key) = turn_idempotency.get(&idempotency_key) {
            let existing =
                turns
                    .get(existing_primary_key)
                    .cloned()
                    .ok_or_else(|| KernelError::Internal {
                        message: "turn idempotency index references a missing turn".to_string(),
                    })?;
            return Ok(TurnRequestWriteOutcome::Existing(Box::new(existing)));
        }
        let existing_session = sessions
            .get(&session_primary_key)
            .cloned()
            .ok_or_else(|| KernelError::validation("active session not found"))?;
        if !existing_session.status.is_active() || existing_session.deleted_at.is_some() {
            return Err(KernelError::validation("active session not found"));
        }
        let request_primary_key = session_item_primary_key(&request_item);
        if items.contains_key(&request_primary_key) {
            return Err(KernelError::conflict("request item already exists"));
        }
        if pending_drive_refs
            .iter()
            .any(|(key, _)| item_drive_refs.contains_key(key))
        {
            return Err(KernelError::conflict(
                "duplicate session item Drive reference",
            ));
        }

        request_item.sequence = existing_session.last_item_sequence.saturating_add(1);
        let request_index_key = session_item_index_key(&request_item);
        if session_item_index.contains_key(&request_index_key) {
            return Err(KernelError::conflict("session item sequence conflict"));
        }
        let mut updated_session = existing_session.clone();
        updated_session.updated_by = request_item.created_by;
        updated_session.record_item(
            request_item.input_tokens,
            request_item.output_tokens,
            request_item.updated_at.clone(),
        );

        let previous_session_index_key = session_index_key(&existing_session);
        let next_session_index_key = session_index_key(&updated_session);
        turns.insert(primary_key.clone(), turn);
        turn_idempotency.insert(idempotency_key, primary_key);
        items.insert(request_primary_key.clone(), request_item.clone());
        session_item_index.insert(request_index_key, request_primary_key);
        sessions.insert(session_primary_key.clone(), updated_session.clone());
        session_index.remove(&previous_session_index_key);
        session_index.insert(next_session_index_key, session_primary_key);
        for (key, record) in pending_drive_refs {
            item_drive_refs.insert(key, record);
        }

        Ok(TurnRequestWriteOutcome::Inserted {
            session: Box::new(updated_session),
            request_item: Box::new(request_item),
        })
    }

    fn update_turn_state(
        &self,
        turn: AgentTurnRecord,
        expected_version: u64,
    ) -> KernelResult<AgentTurnRecord> {
        let primary_key = (turn.tenant_id, turn.organization_id, turn.turn_id.clone());
        let mut turns = self.turns.recovering_write();
        let existing = turns
            .get(&primary_key)
            .ok_or_else(|| KernelError::validation("turn not found"))?;
        if existing.version != expected_version
            || turn.version != expected_version.saturating_add(1)
        {
            return Err(KernelError::conflict("turn version mismatch"));
        }
        if existing.idempotency_key != turn.idempotency_key
            || existing.payload_hash != turn.payload_hash
            || existing.session_id != turn.session_id
            || existing.agent_id != turn.agent_id
            || existing.owner_user_id != turn.owner_user_id
        {
            return Err(KernelError::validation("turn immutable scope mismatch"));
        }
        turns.insert(primary_key, turn.clone());
        Ok(turn)
    }

    fn complete_turn(
        &self,
        turn: AgentTurnRecord,
        expected_turn_version: u64,
        expected_fencing_token: u64,
        expected_lease_token: Option<String>,
        mut response_item: AgentSessionItemRecord,
    ) -> KernelResult<(AgentSessionRecord, AgentSessionItemRecord)> {
        if turn.status != AgentTurnStatus::Completed
            || turn.version != expected_turn_version.saturating_add(1)
            || turn.response_item_id.as_deref() != Some(response_item.item_id.as_str())
            || turn.tenant_id != response_item.tenant_id
            || turn.organization_id != response_item.organization_id
            || turn.session_id != response_item.session_id
            || response_item.turn_id.as_deref() != Some(turn.turn_id.as_str())
            || response_item.parent_item_id.as_deref() != Some(turn.request_item_id.as_str())
        {
            return Err(KernelError::validation(
                "completed turn and response item scope mismatch",
            ));
        }
        let turn_primary_key = (turn.tenant_id, turn.organization_id, turn.turn_id.clone());
        let session_primary_key = (
            turn.tenant_id,
            turn.organization_id,
            turn.session_id.clone(),
        );
        let mut turns = self.turns.recovering_write();
        let mut sessions = self.sessions.recovering_write();
        let mut session_index = self.session_index.recovering_write();
        let mut items = self.items.recovering_write();
        let mut session_item_index = self.session_item_index.recovering_write();

        let existing_turn = turns
            .get(&turn_primary_key)
            .cloned()
            .ok_or_else(|| KernelError::validation("turn not found"))?;
        if existing_turn.version != expected_turn_version
            || existing_turn.fencing_token != expected_fencing_token
            || existing_turn.lease_token != expected_lease_token
            || existing_turn.payload_hash != turn.payload_hash
            || existing_turn.idempotency_key != turn.idempotency_key
            || existing_turn.session_id != turn.session_id
            || existing_turn.agent_id != turn.agent_id
            || existing_turn.owner_user_id != turn.owner_user_id
            || existing_turn.status != AgentTurnStatus::Running
        {
            return Err(KernelError::conflict("turn completion conflict"));
        }
        let existing_session = sessions
            .get(&session_primary_key)
            .cloned()
            .ok_or_else(|| KernelError::validation("session not found"))?;
        if existing_session.deleted_at.is_some() {
            return Err(KernelError::validation("session not found"));
        }
        let response_primary_key = session_item_primary_key(&response_item);
        if items.contains_key(&response_primary_key) {
            return Err(KernelError::conflict("response item already exists"));
        }
        response_item.sequence = existing_session.last_item_sequence.saturating_add(1);
        let response_index_key = session_item_index_key(&response_item);
        if session_item_index.contains_key(&response_index_key) {
            return Err(KernelError::conflict("session item sequence conflict"));
        }
        let mut updated_session = existing_session.clone();
        updated_session.updated_by = response_item.created_by;
        updated_session.record_item(
            response_item.input_tokens,
            response_item.output_tokens,
            response_item.updated_at.clone(),
        );

        let previous_session_index_key = session_index_key(&existing_session);
        let next_session_index_key = session_index_key(&updated_session);
        items.insert(response_primary_key.clone(), response_item.clone());
        session_item_index.insert(response_index_key, response_primary_key);
        sessions.insert(session_primary_key.clone(), updated_session.clone());
        session_index.remove(&previous_session_index_key);
        session_index.insert(next_session_index_key, session_primary_key);
        turns.insert(turn_primary_key, turn);

        Ok((updated_session, response_item))
    }

    fn list_item_drive_refs(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
    ) -> KernelResult<Vec<AgentItemDriveRefRecord>> {
        let mut records = self
            .item_drive_refs
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == tenant_id
                    && record.organization_id == organization_id
                    && record.item_id == item_id
                    && record.deleted_at.is_none()
                    && record.status == 0
            })
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by_key(|record| (record.sort_order, record.id));
        Ok(records)
    }

    fn list_item_drive_refs_batch(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_ids: &[String],
    ) -> KernelResult<Vec<AgentItemDriveRefRecord>> {
        let item_ids = item_ids.iter().map(String::as_str).collect::<HashSet<_>>();
        let mut records = self
            .item_drive_refs
            .recovering_read()
            .values()
            .filter(|record| {
                record.tenant_id == tenant_id
                    && record.organization_id == organization_id
                    && item_ids.contains(record.item_id.as_str())
                    && record.deleted_at.is_none()
                    && record.status == 0
            })
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            left.item_id
                .cmp(&right.item_id)
                .then(left.sort_order.cmp(&right.sort_order))
                .then(left.id.cmp(&right.id))
        });
        Ok(records)
    }

    fn insert_interaction(&self, record: AgentInteractionRecord) -> KernelResult<()> {
        let primary_key = interaction_primary_key(&record);
        let mut interactions = self.interactions.recovering_write();
        if interactions.contains_key(&primary_key) {
            return Err(KernelError::conflict(format!(
                "interaction {} already exists for session {}",
                record.interaction_id, record.session_id
            )));
        }
        let index_key = interaction_index_key(&record);
        interactions.insert(primary_key.clone(), record);
        self.interaction_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update_interaction(&self, record: AgentInteractionRecord) -> KernelResult<()> {
        let primary_key = interaction_primary_key(&record);
        let mut interactions = self.interactions.recovering_write();
        let existing = interactions.get(&primary_key).ok_or_else(|| {
            KernelError::validation(format!(
                "interaction {} not found for session {}",
                record.interaction_id, record.session_id
            ))
        })?;
        let expected_version = existing.version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "interaction version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        let previous_index_key = interaction_index_key(existing);
        let next_index_key = interaction_index_key(&record);
        interactions.insert(primary_key.clone(), record);
        let mut index = self.interaction_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_interaction(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        interaction_id: &str,
    ) -> KernelResult<Option<AgentInteractionRecord>> {
        Ok(self
            .interactions
            .recovering_read()
            .get(&(
                tenant_id,
                organization_id,
                session_id.to_string(),
                interaction_id.to_string(),
            ))
            .cloned())
    }

    fn list_interactions(
        &self,
        query: &InteractionListQuery,
    ) -> KernelResult<Vec<AgentInteractionRecord>> {
        let interactions = self.interactions.recovering_read();
        let index = self.interaction_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, organization_id, session_id, _, _), _)| {
                *tenant_id == query.tenant_id
                    && *organization_id == query.organization_id
                    && session_id == &query.session_id
            })
            .filter_map(|(_, primary_key)| interactions.get(primary_key))
            .filter(|record| interaction_matches_list_query(record, query))
            .cloned();
        Ok(paginate_iterator(iter, &query.pagination))
    }

    fn count_interactions(&self, query: &InteractionListQuery) -> KernelResult<u64> {
        let interactions = self.interactions.recovering_read();
        let index = self.interaction_index.recovering_read();
        Ok(count_iterator(
            index
                .iter()
                .filter(|((tenant_id, organization_id, session_id, _, _), _)| {
                    *tenant_id == query.tenant_id
                        && *organization_id == query.organization_id
                        && session_id == &query.session_id
                })
                .filter_map(|(_, primary_key)| interactions.get(primary_key))
                .filter(|record| interaction_matches_list_query(record, query)),
        ))
    }

    fn insert_task(&self, record: AgentTaskRecord) -> KernelResult<()> {
        let primary_key = task_primary_key(&record);
        let mut tasks = self.tasks.recovering_write();
        if tasks.contains_key(&primary_key) {
            return Err(KernelError::conflict("task already exists"));
        }
        let index_key = task_index_key(&record);
        tasks.insert(primary_key.clone(), record);
        self.task_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update_task(&self, record: AgentTaskRecord) -> KernelResult<()> {
        let primary_key = task_primary_key(&record);
        let mut tasks = self.tasks.recovering_write();
        let existing = tasks
            .get(&primary_key)
            .ok_or_else(|| KernelError::validation("task not found"))?;
        let expected_version = existing.version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "task version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        let previous_index_key = task_index_key(existing);
        let next_index_key = task_index_key(&record);
        tasks.insert(primary_key.clone(), record);
        let mut index = self.task_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_task(&self, tenant_id: u64, task_id: &str) -> KernelResult<Option<AgentTaskRecord>> {
        Ok(self
            .tasks
            .recovering_read()
            .get(&(tenant_id, task_id.to_string()))
            .cloned())
    }

    fn list_tasks(&self, query: &TaskListQuery) -> KernelResult<Vec<AgentTaskRecord>> {
        let tasks = self.tasks.recovering_read();
        let index = self.task_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, _, _), _)| *tenant_id == query.tenant_id)
            .filter_map(|(_, primary_key)| tasks.get(primary_key))
            .filter(|record| task_matches_list_query(record, query))
            .cloned();
        Ok(paginate_iterator(iter, &query.pagination))
    }

    fn count_tasks(&self, query: &TaskListQuery) -> KernelResult<u64> {
        let tasks = self.tasks.recovering_read();
        let index = self.task_index.recovering_read();
        Ok(count_iterator(
            index
                .iter()
                .filter(|((tenant_id, _, _), _)| *tenant_id == query.tenant_id)
                .filter_map(|(_, primary_key)| tasks.get(primary_key))
                .filter(|record| task_matches_list_query(record, query)),
        ))
    }
}

// ---------------------------------------------------------------------------
// Policy provider infrastructure
// ---------------------------------------------------------------------------

/// Policy mode that determines the outcome of every policy evaluation.
/// `Allow` permits all requests with the given reason; `Deny` rejects all
/// requests with the given reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyMode {
    Allow(String),
    Deny(String),
}

/// A simple static policy provider that returns the same decision for every
/// request based on its configured [`PolicyMode`].
///
/// **Never use this provider in production.** It does not evaluate subject
/// roles or resource attributes. Production deployments use
/// [`IamGatedPolicyProvider`]. Use [`DenyAllPolicyProvider`] as the
/// fail-closed default whenever no policy provider is explicitly configured.
/// This type is retained only for local development and integration-test
/// scenarios.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowAllPolicyProvider {
    pub provider_id: String,
    pub mode: PolicyMode,
}

impl AllowAllPolicyProvider {
    /// Create a dev-only provider that allows every request. The `provider_id`
    /// should identify the policy source (e.g. `"policy.agents.dev"` for
    /// development).
    ///
    /// # Security Check
    /// This method validates that the application is not running with
    /// development auth bypass in a production-like profile.
    pub fn try_allow(provider_id: impl Into<String>) -> Result<Self, String> {
        validate_production_security_config()?;
        Ok(Self {
            provider_id: provider_id.into(),
            mode: PolicyMode::Allow("static.allow".to_string()),
        })
    }

    /// Compatibility constructor for tests and local-only callers.
    ///
    /// If the environment is misconfigured as production-like with dev auth
    /// bypass enabled, it logs the violation and returns a deny-all static
    /// provider instead of panicking or allowing requests.
    pub fn allow(provider_id: impl Into<String>) -> Self {
        let provider_id = provider_id.into();
        match Self::try_allow(provider_id.clone()) {
            Ok(provider) => provider,
            Err(message) => {
                tracing::error!(
                    env_var = ENV_DEV_AUTH_BYPASS,
                    deployment = %sdkwork_agents_contract::agents_deployment_environment_name(),
                    error = %message,
                    "development auth bypass policy provider refused production-like configuration"
                );
                Self {
                    provider_id,
                    mode: PolicyMode::Deny("static.deny.security_misconfiguration".to_string()),
                }
            }
        }
    }
}

impl PolicyProvider for AllowAllPolicyProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            self.provider_id.clone(),
            "policy",
            "static-policy-provider",
            "0.1.0",
            vec!["policy.evaluate".to_string()],
        )
    }

    fn evaluate(&self, request: PolicyRequest) -> KernelResult<PolicyDecision> {
        let decision_id = format!(
            "decision_{}_{}",
            self.provider_id, request.policy_request_id
        );
        match &self.mode {
            PolicyMode::Allow(reason) => Ok(PolicyDecision::allow(
                decision_id,
                request.policy_request_id,
                self.provider_id.clone(),
            )
            .with_safe_reason(reason.clone())),
            PolicyMode::Deny(reason) => Ok(PolicyDecision::deny(
                decision_id,
                request.policy_request_id,
                self.provider_id.clone(),
                reason,
            )),
        }
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }
}

/// Fail-closed policy provider that denies every request. Use this as the
/// default when no IAM integration is configured, so that misconfiguration
/// can never accidentally allow unauthorized access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenyAllPolicyProvider {
    pub provider_id: String,
    pub reason: String,
}

impl DenyAllPolicyProvider {
    pub fn new(provider_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            reason: reason.into(),
        }
    }
}

impl Default for DenyAllPolicyProvider {
    fn default() -> Self {
        Self::new(
            "policy.agents.deny-all",
            "no policy provider configured; access denied by default",
        )
    }
}

impl PolicyProvider for DenyAllPolicyProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            self.provider_id.clone(),
            "policy",
            "deny-all-policy-provider",
            "0.1.0",
            vec!["policy.evaluate".to_string()],
        )
    }

    fn evaluate(&self, request: PolicyRequest) -> KernelResult<PolicyDecision> {
        let decision_id = format!(
            "decision_{}_{}",
            self.provider_id, request.policy_request_id
        );
        Ok(PolicyDecision::deny(
            decision_id,
            request.policy_request_id,
            self.provider_id.clone(),
            self.reason.clone(),
        ))
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }
}

/// IAM permission codes that gate agent business operations. These align with
/// the `ai` module manifest in `sdkwork-iam/iam/modules/ai/iam.module.manifest.json`.
pub const IAM_PERMISSION_AGENTS_MANAGE: &str = "ai.agents.manage";
pub const IAM_PERMISSION_AGENTS_READ: &str = "ai.agents.read";
pub const IAM_PERMISSION_AGENTS_USE: &str = "ai.agents.use";

/// Role codes that grant wildcard AI permissions per the IAM module manifest
/// `roleGrantExtensions` (org_admin and org_operations both map to `ai.*`).
const IAM_ADMIN_ROLE_CODES: &[&str] = &["org_admin", "org_operations"];

/// Canonical read-only operation suffixes used by `AgentsService::authorize`.
/// Resource-qualified actions such as `project.list` and `code_engine.list`
/// resolve through the same operation suffix as unqualified actions.
const READ_ONLY_POLICY_OPERATIONS: &[&str] = &["list", "read", "retrieve"];

/// Owner-scoped and runtime mutations available to ordinary app users. This
/// list is intentionally exact so future actions remain manage-only until
/// their ownership checks and permission classification are reviewed.
const SELF_SERVICE_POLICY_ACTIONS: &[&str] = &[
    "checkpoint.create",
    "checkpoint.invalidate",
    "checkpoint.restore",
    "create",
    "interaction.answer",
    "interaction.approve",
    "interaction.claim",
    "interaction.create",
    "item_feedback.update",
    "project.archive",
    "project.composition_slot.create",
    "project.composition_slot.delete",
    "project.composition_slot.update",
    "project.create",
    "project.delete",
    "project.update",
    "runtime.preview_response",
    "runtime.prompt_optimization",
    "session.archive",
    "session.close",
    "session.create",
    "session.delete",
    "session.update",
    "session.user_state.update",
    "session_item.create",
    "session_runtime_binding.activate",
    "session_runtime_binding.create",
    "session_runtime_binding.deactivate",
    "session_runtime_binding.update",
    "task.cancel",
    "task.create",
    "task.execute",
    "turn.cancel",
    "turn.create",
    "workspace.archive",
    "workspace.create",
    "workspace.delete",
    "workspace.ensureDefault",
    "workspace.update",
];

/// IAM-gated policy provider that maps agent business actions to IAM permission
/// scopes and evaluates the request subject's roles/scopes against the
/// required permission.
///
/// This provider implements defense-in-depth at the application service layer.
/// The web framework layer (`IamAuthorizationPolicy` from `sdkwork-iam-web-adapter`)
/// performs HTTP route-level authorization first; this provider performs
/// resource-action-level authorization as a second gate.
///
/// Subject roles are populated from gateway-injected headers
/// (`x-subject-roles` or `x-sdkwork-permission-scope`) and may contain either
/// IAM permission scopes (e.g. `ai.agents.read`) or role codes (e.g.
/// `org_admin`). Both are honored:
///
/// - Permission scopes are matched directly, with `ai.*` and `*` wildcards.
/// - Admin role codes (`org_admin`, `org_operations`) are granted `ai.*`.
///
/// Requests without a subject or with no matching permission are denied
/// (fail-closed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IamGatedPolicyProvider {
    pub provider_id: String,
}

impl IamGatedPolicyProvider {
    pub fn new(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
        }
    }

    fn is_read_only_action(action: &str) -> bool {
        let operation = action.rsplit('.').next().unwrap_or(action);
        READ_ONLY_POLICY_OPERATIONS.contains(&operation)
    }

    fn is_self_service_action(action: &str) -> bool {
        SELF_SERVICE_POLICY_ACTIONS.contains(&action)
    }

    /// Determine the required IAM permission for the given policy action.
    fn required_permission_for_action(action: Option<&str>) -> &'static str {
        match action {
            Some(action) if Self::is_read_only_action(action) => IAM_PERMISSION_AGENTS_READ,
            Some(action) if Self::is_self_service_action(action) => IAM_PERMISSION_AGENTS_USE,
            _ => IAM_PERMISSION_AGENTS_MANAGE,
        }
    }

    /// Return `true` if the subject's role/scope entry satisfies the required
    /// permission. Supports wildcards `ai.*` and `*`, and known admin role
    /// codes that grant `ai.*`. Also honors the implication that
    /// `ai.agents.manage` grants both read and use capabilities.
    fn entry_grants_permission(entry: &str, required_permission: &str) -> bool {
        let entry = entry.trim();
        if entry.is_empty() {
            return false;
        }
        if entry == "*" {
            return true;
        }
        if entry == required_permission {
            return true;
        }
        // Manage permission implies all narrower capabilities for the same resource.
        if entry == IAM_PERMISSION_AGENTS_MANAGE
            && matches!(
                required_permission,
                IAM_PERMISSION_AGENTS_READ | IAM_PERMISSION_AGENTS_USE
            )
        {
            return true;
        }
        // Wildcard within the ai domain (e.g. `ai.*` matches `ai.agents.read`).
        if entry == "ai.*" && required_permission.starts_with("ai.") {
            return true;
        }
        // Admin role codes that the IAM module grants `ai.*` to.
        if IAM_ADMIN_ROLE_CODES.contains(&entry) && required_permission.starts_with("ai.") {
            return true;
        }
        false
    }

    /// Return `true` if the subject has any role/scope entry that satisfies
    /// the required permission.
    fn subject_has_permission(
        subject: Option<&sdkwork_agent_kernel::PolicySubject>,
        required_permission: &str,
    ) -> bool {
        let Some(subject) = subject else {
            return false;
        };
        subject
            .roles
            .iter()
            .any(|entry| Self::entry_grants_permission(entry.as_str(), required_permission))
    }
}

impl Default for IamGatedPolicyProvider {
    fn default() -> Self {
        Self::new("policy.agents.production.iam-gated")
    }
}

impl PolicyProvider for IamGatedPolicyProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            self.provider_id.clone(),
            "policy",
            "iam-gated-policy-provider",
            "0.1.0",
            vec!["policy.evaluate".to_string()],
        )
    }

    fn evaluate(&self, request: PolicyRequest) -> KernelResult<PolicyDecision> {
        let decision_id = format!(
            "decision_{}_{}",
            self.provider_id, request.policy_request_id
        );
        let action = request.action.as_deref();
        let required_permission = Self::required_permission_for_action(action);
        if Self::subject_has_permission(request.subject.as_ref(), required_permission) {
            Ok(PolicyDecision::allow(
                decision_id,
                request.policy_request_id,
                self.provider_id.clone(),
            )
            .with_safe_reason(format!("iam.permission.satisfied:{required_permission}")))
        } else {
            Ok(PolicyDecision::deny(
                decision_id,
                request.policy_request_id,
                self.provider_id.clone(),
                "iam.permission.missing",
            )
            .with_safe_reason(format!("iam.permission.missing:{required_permission}")))
        }
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }
}

// ---------------------------------------------------------------------------
// In-memory audit sink infrastructure
// ---------------------------------------------------------------------------

/// Maximum number of audit events retained by [`InMemoryAgentAuditSink`].
/// When exceeded, the oldest events (smallest sort key) are evicted. This
/// prevents unbounded memory growth in long-running processes.
const MAX_IN_MEMORY_AUDIT_EVENTS: usize = 10_000;

/// Sort key for audit events: `(occurred_at, event_id)` wrapped in `Reverse`
/// for descending chronological order. The `BTreeMap` maintains this index
/// incrementally, satisfying PAGINATION_SPEC §5.3 (no per-request rebuild).
type AuditEventIndexKey = Reverse<(time::OffsetDateTime, String)>;

fn audit_event_sort_key(event: &KernelEvent) -> KernelResult<AuditEventIndexKey> {
    let occurred_at = event
        .occurred_at
        .as_deref()
        .ok_or_else(|| KernelError::validation("audit event occurred_at is required"))?;
    let occurred_at = parse_rfc3339_datetime(occurred_at, "audit event occurred_at")?;
    Ok(Reverse((occurred_at, event.event_id.clone())))
}

/// In-memory audit sink backed by a `BTreeMap` with bounded capacity.
/// Uses `Mutex` for interior mutability. Events are lost when the process
/// exits. Use [`crate::persistence::SqlAgentAuditSink`] for production
/// deployments that require persistent audit trails.
#[derive(Debug, Clone, Default)]
pub struct InMemoryAgentAuditSink {
    events: std::sync::Arc<Mutex<BTreeMap<AuditEventIndexKey, KernelEvent>>>,
}

impl InMemoryAgentAuditSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<KernelEvent> {
        self.events.recovering_lock().values().cloned().collect()
    }
}

impl AgentAuditSink for InMemoryAgentAuditSink {
    fn record(&self, event: KernelEvent) -> KernelResult<()> {
        let key = audit_event_sort_key(&event)?;
        let mut events = self.events.recovering_lock();
        events.insert(key, event);
        // Reverse ordering places the oldest timestamp at the back of the map.
        while events.len() > MAX_IN_MEMORY_AUDIT_EVENTS {
            if let Some(oldest_key) = events.keys().next_back().cloned() {
                events.remove(&oldest_key);
            } else {
                break;
            }
        }
        Ok(())
    }

    fn list_events(
        &self,
        query: &AuditEventListQuery,
    ) -> KernelResult<crate::ports::PaginatedResult<KernelEvent>> {
        use crate::ports::offset_paginated_result;
        let from = query
            .from
            .as_deref()
            .map(|value| parse_rfc3339_datetime(value, "from"))
            .transpose()?;
        let to = query
            .to
            .as_deref()
            .map(|value| parse_rfc3339_datetime(value, "to"))
            .transpose()?;
        let events = self.events.recovering_lock();
        // BTreeMap<Reverse<...>> iterates in descending order (newest first).
        // Iterate the incrementally maintained index directly — no collect/sort.
        let filtered = events
            .iter()
            .filter(|(Reverse((occurred_at, _)), _)| {
                from.map(|from| *occurred_at >= from).unwrap_or(true)
                    && to.map(|to| *occurred_at <= to).unwrap_or(true)
            })
            .map(|(_, event)| event)
            .filter(|event| {
                crate::persistence::extract_event_context(event.payload.as_str(), "agent_id")
                    .map(|id| id == query.agent_id)
                    .unwrap_or(false)
            })
            .filter(|event| {
                query
                    .action
                    .as_ref()
                    .map(|action| {
                        event
                            .event_type
                            .rsplit('.')
                            .next()
                            .map(|value| value == action)
                            .unwrap_or(false)
                    })
                    .unwrap_or(true)
            })
            .cloned();
        let total_count = count_iterator(filtered.clone());
        let page = paginate_iterator(filtered, &query.pagination);
        Ok(offset_paginated_result(
            page,
            &query.pagination,
            total_count,
        ))
    }
}

// ---------------------------------------------------------------------------
// Shared comparison helpers for in-memory repository sorting
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Shared comparison helpers for in-memory repository sorting
// ---------------------------------------------------------------------------

fn session_matches_list_query(record: &AgentSessionRecord, query: &SessionListQuery) -> bool {
    if record.tenant_id != query.tenant_id {
        return false;
    }
    if record.deleted_at.is_some() {
        return false;
    }
    if let Some(organization_id) = query.organization_id {
        if record.organization_id != organization_id {
            return false;
        }
    }
    if let Some(agent_id) = query.agent_id.as_ref() {
        if record.agent_id != *agent_id {
            return false;
        }
    }
    if let Some(owner_user_id) = query.owner_user_id {
        if record.owner_user_id != owner_user_id {
            return false;
        }
    }
    if let Some(project_id) = query.project_id.as_ref() {
        if record.project_id.as_ref() != Some(project_id) {
            return false;
        }
    }
    if let Some(status) = query.status.as_ref() {
        if record.status.as_str() != status {
            return false;
        }
    }
    query.include_archived || record.status.as_str() != "archived"
}

fn project_matches_list_query(record: &AgentProjectRecord, query: &ProjectListQuery) -> bool {
    if record.tenant_id != query.tenant_id || record.organization_id != query.organization_id {
        return false;
    }
    if let Some(owner_user_id) = query.owner_user_id {
        if record.owner_user_id != owner_user_id {
            return false;
        }
    }
    if let Some(workspace_id) = query.workspace_id.as_deref() {
        if record.workspace_id != workspace_id {
            return false;
        }
    }
    if let Some(status) = query.status {
        if record.status != status {
            return false;
        }
    }
    if !query.include_deleted && record.status == AgentProjectStatus::Deleted {
        return false;
    }
    if let Some(search_query) = query.search_query.as_deref() {
        let needle = trim(search_query).to_lowercase();
        let description = record.description.as_deref().unwrap_or_default();
        if !record.name.to_lowercase().contains(&needle)
            && !description.to_lowercase().contains(&needle)
        {
            return false;
        }
    }
    true
}

fn workspace_matches_list_query(record: &AgentWorkspaceRecord, query: &WorkspaceListQuery) -> bool {
    if record.tenant_id != query.tenant_id
        || record.organization_id != query.organization_id
        || record.owner_user_id != query.owner_user_id
    {
        return false;
    }
    if let Some(status) = query.status {
        if record.status != status {
            return false;
        }
    }
    query.include_deleted || record.status != AgentWorkspaceStatus::Deleted
}

fn message_matches_list_query(
    record: &AgentSessionItemRecord,
    query: &SessionItemListQuery,
) -> bool {
    if record.tenant_id != query.tenant_id
        || record.organization_id != query.organization_id
        || record.session_id != query.session_id
    {
        return false;
    }
    if let Some(kind) = query.kind.as_ref() {
        if record.kind.as_str() != kind {
            return false;
        }
    }
    if let Some(status) = query.status.as_ref() {
        if record.status.as_str() != status {
            return false;
        }
    }
    true
}

fn interaction_matches_list_query(
    record: &AgentInteractionRecord,
    query: &InteractionListQuery,
) -> bool {
    if record.tenant_id != query.tenant_id
        || record.organization_id != query.organization_id
        || record.session_id != query.session_id
    {
        return false;
    }
    if let Some(kind) = query.kind.as_ref() {
        if record.kind.as_str() != kind {
            return false;
        }
    }
    if let Some(status) = query.status.as_ref() {
        if record.status.as_str() != status {
            return false;
        }
    }
    true
}

fn task_matches_list_query(record: &AgentTaskRecord, query: &TaskListQuery) -> bool {
    if record.tenant_id != query.tenant_id {
        return false;
    }
    if let Some(agent_id) = query.agent_id.as_ref() {
        if record.agent_id != *agent_id {
            return false;
        }
    }
    if let Some(owner_user_id) = query.owner_user_id {
        if record.owner_user_id != owner_user_id {
            return false;
        }
    }
    if let Some(status) = query.status.as_ref() {
        if record.status.as_str() != status {
            return false;
        }
    }
    true
}

fn agent_matches_list_query(record: &AgentBusinessRecord, query: &AgentListQuery) -> bool {
    if record.tenant_id != query.tenant_id {
        return false;
    }
    if let Some(organization_id) = query.organization_id {
        if record.organization_id != organization_id {
            return false;
        }
    }
    if let Some(owner_user_id) = query.owner_user_id {
        if record.owner_user_id != owner_user_id {
            return false;
        }
    }
    if !query.include_deleted && record.is_deleted() {
        return false;
    }
    if let Some(visibility) = query.visibility {
        if record.visibility != visibility {
            return false;
        }
    }
    if let Some(search_query) = query.search_query.as_ref() {
        if !is_blank(Some(search_query.as_str())) {
            let normalized_query = trim(search_query).to_lowercase();
            let description = record.description.as_deref().unwrap_or("");
            let matches = record
                .agent_id
                .to_lowercase()
                .contains(normalized_query.as_str())
                || record
                    .code
                    .to_lowercase()
                    .contains(normalized_query.as_str())
                || record
                    .display_name
                    .to_lowercase()
                    .contains(normalized_query.as_str())
                || description
                    .to_lowercase()
                    .contains(normalized_query.as_str());
            if !matches {
                return false;
            }
        }
    }
    true
}

/// Sort order for provider bindings: active first, then by updated_at desc,
/// then by binding_id ascending. Encoded in `provider_binding_index_key`.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        AgentBusinessStatus, AgentImplementationKind, AgentImplementationType,
        AgentProviderBindingRecord, AgentVisibility,
    };
    use crate::ports::{PaginationParams, ProviderBindingListQuery, MAX_PAGE_SIZE};
    use sdkwork_agent_kernel::AgentManifest;
    use sdkwork_agent_kernel::KernelErrorKind;

    fn sample_manifest(agent_id: &str) -> AgentManifest {
        AgentManifest {
            schema_version: "1.0.0".to_string(),
            manifest_type: "agent".to_string(),
            agent_id: agent_id.to_string(),
            name: "sample-agent".to_string(),
            display_name: "Sample Agent".to_string(),
            description: "sample".to_string(),
            version: "0.1.0".to_string(),
            domain: "intelligence".to_string(),
            required_capabilities: vec!["model.chat".to_string()],
            optional_capabilities: vec!["tool.invoke".to_string()],
            required_capability_requirements: vec![],
            optional_capability_requirements: vec![],
            event_families: vec!["agent.lifecycle".to_string()],
            owner_name: "sdkwork".to_string(),
            status: "active".to_string(),
        }
    }

    #[test]
    fn in_memory_repository_rejects_stale_record_version_update() {
        let repository = InMemoryAgentRepository::new();
        let record = AgentBusinessRecord {
            id: 1,
            agent_id: "agent.alpha".to_string(),
            tenant_id: 100_001,
            organization_id: 0,
            owner_user_id: 100,
            code: "alpha".to_string(),
            display_name: "Alpha".to_string(),
            description: None,
            manifest: sample_manifest("agent.alpha"),
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: AgentImplementationType::SdkworkNative,
            status: AgentBusinessStatus::Draft,
            visibility: AgentVisibility::Organization,
            tags: vec!["starter".to_string()],
            version: 1,
            created_at: "2026-06-01T00:00:00Z".to_string(),
            updated_at: "2026-06-01T00:00:00Z".to_string(),
            deleted_at: None,
        };
        repository
            .insert(record.clone())
            .expect("initial insert should succeed");

        let mut stale = record.clone();
        stale.display_name = "Alpha stale".to_string();
        let error = repository
            .update(stale)
            .expect_err("stale version should fail");
        assert_eq!(error.kind(), KernelErrorKind::Conflict);
        assert!(error.message().contains("version mismatch"));
    }

    #[test]
    fn in_memory_repository_records_do_not_panic_after_poisoned_lock() {
        let repository = InMemoryAgentRepository::new();
        let poison_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = repository
                .agents
                .write()
                .expect("test lock should be available");
            panic!("poison in-memory repository lock");
        }));
        assert!(poison_result.is_err());

        let read_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| repository.records()));

        assert!(
            read_result.is_ok(),
            "in-memory repository must not panic after RwLock poisoning"
        );
    }

    fn audit_event(event_id: String, occurred_at: &str) -> KernelEvent {
        KernelEvent::new(
            event_id,
            "agent.business.updated",
            sdkwork_agent_kernel::KernelEventSeverity::Info,
            r#"{"_context":{"agent_id":"agent.audit"}}"#,
        )
        .from_source(sdkwork_agent_kernel::KernelEventSource::Runtime)
        .occurred_at(occurred_at)
    }

    #[test]
    fn in_memory_audit_sink_rejects_invalid_occurred_at() {
        let sink = InMemoryAgentAuditSink::new();
        let error = sink
            .record(audit_event("event.invalid".to_string(), "not-a-timestamp"))
            .expect_err("invalid audit timestamps must fail closed");

        assert_eq!(error.kind(), KernelErrorKind::ValidationError);
        assert!(sink.events().is_empty());
    }

    #[test]
    fn in_memory_audit_sink_evicts_oldest_event_at_capacity() {
        let sink = InMemoryAgentAuditSink::new();
        sink.record(audit_event(
            "event.oldest".to_string(),
            "2026-05-31T23:59:59Z",
        ))
        .expect("oldest event should be accepted");

        for index in 0..MAX_IN_MEMORY_AUDIT_EVENTS {
            sink.record(audit_event(
                format!("event.current.{index:05}"),
                "2026-06-01T00:00:00Z",
            ))
            .expect("current event should be accepted");
        }

        let events = sink.events();
        assert_eq!(events.len(), MAX_IN_MEMORY_AUDIT_EVENTS);
        assert!(events.iter().all(|event| event.event_id != "event.oldest"));
        assert!(events
            .iter()
            .any(|event| event.event_id == "event.current.09999"));
    }

    #[test]
    fn allow_all_policy_provider_fails_closed_without_panic_in_production_like_bypass() {
        let _guard = sdkwork_agents_contract::env_test_lock();
        let previous_bypass = std::env::var("SDKWORK_AGENTS_DEV_AUTH_BYPASS").ok();
        let previous_deploy = std::env::var("SDKWORK_DEPLOYMENT_ENV").ok();
        let previous_environment = std::env::var("ENVIRONMENT").ok();
        let previous_agents_env = std::env::var("SDKWORK_AGENTS_ENVIRONMENT").ok();
        let previous_profile = std::env::var("SDKWORK_AGENTS_CONFIG_PROFILE").ok();

        std::env::remove_var("ENVIRONMENT");
        std::env::remove_var("SDKWORK_AGENTS_ENVIRONMENT");
        std::env::remove_var("SDKWORK_AGENTS_CONFIG_PROFILE");
        std::env::set_var("SDKWORK_AGENTS_DEV_AUTH_BYPASS", "true");
        std::env::set_var("SDKWORK_DEPLOYMENT_ENV", "production");

        let provider_result = std::panic::catch_unwind(|| {
            AllowAllPolicyProvider::allow("policy.agents.misconfigured")
        });

        restore_optional_env("SDKWORK_AGENTS_DEV_AUTH_BYPASS", previous_bypass);
        restore_optional_env("SDKWORK_DEPLOYMENT_ENV", previous_deploy);
        restore_optional_env("ENVIRONMENT", previous_environment);
        restore_optional_env("SDKWORK_AGENTS_ENVIRONMENT", previous_agents_env);
        restore_optional_env("SDKWORK_AGENTS_CONFIG_PROFILE", previous_profile);

        let provider = provider_result
            .expect("AllowAllPolicyProvider::allow must fail closed instead of panicking");
        assert!(
            matches!(provider.mode, PolicyMode::Deny(_)),
            "misconfigured AllowAllPolicyProvider must deny instead of allowing"
        );
    }

    #[test]
    fn in_memory_repository_rejects_stale_provider_binding_update() {
        let repository = InMemoryAgentRepository::new();
        let record = AgentProviderBindingRecord {
            id: 101,
            tenant_id: 100_001,
            agent_id: "agent.alpha".to_string(),
            binding_id: "binding.rig.default".to_string(),
            provider_id: "provider.model.rig-rust".to_string(),
            implementation_kind: AgentImplementationKind::TypedLocalProvider,
            configuration_profile_id: "profile.rig.local".to_string(),
            capabilities: vec!["model.chat".to_string()],
            active: true,
            version: 1,
            created_at: "2026-06-01T00:00:00Z".to_string(),
            updated_at: "2026-06-01T00:00:00Z".to_string(),
        };
        repository
            .insert_provider_binding(record.clone())
            .expect("initial binding insert should succeed");

        let mut stale = record.clone();
        stale.provider_id = "provider.model.rig-alt".to_string();
        let error = repository
            .update_provider_binding(stale)
            .expect_err("stale binding version should fail");

        assert_eq!(error.kind(), KernelErrorKind::Conflict);
        assert!(error
            .message()
            .contains("provider binding version mismatch"));
    }

    #[test]
    fn in_memory_repository_rejects_second_active_provider_binding() {
        let repository = InMemoryAgentRepository::new();
        repository
            .insert_provider_binding(AgentProviderBindingRecord {
                id: 102,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.default".to_string(),
                provider_id: "provider.model.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.local".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 1,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                updated_at: "2026-06-01T00:00:00Z".to_string(),
            })
            .expect("first active binding insert should succeed");

        let error = repository
            .insert_provider_binding(AgentProviderBindingRecord {
                id: 103,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.alt".to_string(),
                provider_id: "provider.model.rig-alt".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.alt".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 1,
                created_at: "2026-06-01T00:01:00Z".to_string(),
                updated_at: "2026-06-01T00:01:00Z".to_string(),
            })
            .expect_err("second active binding should fail");

        assert_eq!(error.kind(), KernelErrorKind::Conflict);
        assert!(error
            .message()
            .contains("active provider binding already exists"));
    }

    #[test]
    fn in_memory_repository_rejects_update_that_creates_second_active_provider_binding() {
        let repository = InMemoryAgentRepository::new();
        repository
            .insert_provider_binding(AgentProviderBindingRecord {
                id: 104,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.default".to_string(),
                provider_id: "provider.model.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.local".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 1,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                updated_at: "2026-06-01T00:00:00Z".to_string(),
            })
            .expect("first active binding insert should succeed");
        repository
            .insert_provider_binding(AgentProviderBindingRecord {
                id: 105,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.alt".to_string(),
                provider_id: "provider.model.rig-alt".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.alt".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: false,
                version: 1,
                created_at: "2026-06-01T00:01:00Z".to_string(),
                updated_at: "2026-06-01T00:01:00Z".to_string(),
            })
            .expect("inactive binding insert should succeed");

        let error = repository
            .update_provider_binding(AgentProviderBindingRecord {
                id: 105,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.alt".to_string(),
                provider_id: "provider.model.rig-alt".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.alt".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 2,
                created_at: "2026-06-01T00:01:00Z".to_string(),
                updated_at: "2026-06-01T00:02:00Z".to_string(),
            })
            .expect_err("update cannot create a second active binding");

        assert_eq!(error.kind(), KernelErrorKind::Conflict);
        assert!(error
            .message()
            .contains("active provider binding already exists"));
    }

    #[test]
    fn in_memory_repository_lists_provider_bindings_in_standard_order() {
        let repository = InMemoryAgentRepository::new();
        for record in [
            AgentProviderBindingRecord {
                id: 106,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.beta".to_string(),
                provider_id: "provider.model.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.beta".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: false,
                version: 1,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                updated_at: "2026-06-01T00:02:00Z".to_string(),
            },
            AgentProviderBindingRecord {
                id: 107,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.default".to_string(),
                provider_id: "provider.model.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.default".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 1,
                created_at: "2026-06-01T00:01:00Z".to_string(),
                updated_at: "2026-06-01T00:01:00Z".to_string(),
            },
            AgentProviderBindingRecord {
                id: 108,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.alpha".to_string(),
                provider_id: "provider.model.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.alpha".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: false,
                version: 1,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                updated_at: "2026-06-01T00:02:00Z".to_string(),
            },
        ] {
            repository
                .insert_provider_binding(record)
                .expect("binding insert should succeed");
        }

        let binding_ids: Vec<String> = repository
            .list_provider_bindings(
                &ProviderBindingListQuery::for_agent(100_001, "agent.alpha")
                    .with_pagination(PaginationParams::default().with_page_size(MAX_PAGE_SIZE)),
            )
            .expect("binding list should succeed")
            .into_iter()
            .map(|record| record.binding_id)
            .collect();

        assert_eq!(
            binding_ids,
            vec![
                "binding.rig.default".to_string(),
                "binding.rig.alpha".to_string(),
                "binding.rig.beta".to_string()
            ]
        );
    }

    // --- IamGatedPolicyProvider tests ---

    use sdkwork_agent_kernel::{PolicyDecisionValue, PolicySubject};

    fn policy_request_with_action_and_roles(action: &str, roles: &[&str]) -> PolicyRequest {
        let mut subject = PolicySubject::new("user.test", "100001");
        for role in roles {
            subject = subject.with_role(*role);
        }
        PolicyRequest::new(
            "req.test",
            "agent.business.manage",
            "agent.business.tenant.100001",
        )
        .with_subject(subject)
        .with_action(action)
    }

    #[test]
    fn iam_gated_provider_allows_read_action_with_read_permission() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("list", &["ai.agents.read"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Allow);
    }

    #[test]
    fn iam_gated_provider_allows_resource_qualified_read_actions_with_read_permission() {
        let provider = IamGatedPolicyProvider::default();
        for action in [
            "code_engine.list",
            "project.list",
            "project.retrieve",
            "session.user_state.list",
            "audit.read",
        ] {
            let request = policy_request_with_action_and_roles(action, &["ai.agents.read"]);
            let decision = provider.evaluate(request).expect("evaluate should succeed");
            assert_eq!(
                decision.decision,
                PolicyDecisionValue::Allow,
                "{action} must require only ai.agents.read"
            );
        }
    }

    #[test]
    fn iam_gated_provider_allows_read_action_with_manage_permission() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("retrieve", &["ai.agents.manage"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Allow);
    }

    #[test]
    fn iam_gated_provider_allows_self_service_actions_with_use_permission() {
        let provider = IamGatedPolicyProvider::default();
        for action in [
            "create",
            "project.create",
            "project.update",
            "project.archive",
            "project.delete",
            "project.composition_slot.create",
            "project.composition_slot.update",
            "project.composition_slot.delete",
            "session.create",
            "session.update",
            "session.delete",
            "session.close",
            "session.archive",
            "session.user_state.update",
            "session_item.create",
            "item_feedback.update",
            "turn.create",
            "turn.cancel",
            "task.create",
            "task.cancel",
            "task.execute",
            "interaction.create",
            "interaction.claim",
            "interaction.approve",
            "interaction.answer",
            "checkpoint.create",
            "checkpoint.restore",
            "checkpoint.invalidate",
            "session_runtime_binding.create",
            "session_runtime_binding.update",
            "session_runtime_binding.activate",
            "session_runtime_binding.deactivate",
            "runtime.preview_response",
            "runtime.prompt_optimization",
            "workspace.ensureDefault",
        ] {
            let request = policy_request_with_action_and_roles(action, &["ai.agents.use"]);
            let decision = provider.evaluate(request).expect("evaluate should succeed");
            assert_eq!(
                decision.decision,
                PolicyDecisionValue::Allow,
                "{action} must require ai.agents.use"
            );
        }
    }

    #[test]
    fn iam_gated_provider_keeps_management_actions_behind_manage_permission() {
        let provider = IamGatedPolicyProvider::default();
        for action in [
            "update",
            "delete",
            "change_status",
            "provider_binding.add",
            "provider_binding.activate",
            "composition_slot.create",
            "session.unclassified_mutation",
        ] {
            let request = policy_request_with_action_and_roles(action, &["ai.agents.use"]);
            let decision = provider.evaluate(request).expect("evaluate should succeed");
            assert_eq!(
                decision.decision,
                PolicyDecisionValue::Deny,
                "{action} must remain restricted to ai.agents.manage"
            );
            assert!(decision
                .safe_reason
                .as_deref()
                .unwrap_or_default()
                .contains("iam.permission.missing:ai.agents.manage"));
        }
    }

    #[test]
    fn iam_gated_provider_denies_manage_action_with_only_read_permission() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("update", &["ai.agents.read"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
        assert!(decision
            .safe_reason
            .as_deref()
            .unwrap_or_default()
            .contains("iam.permission.missing:ai.agents.manage"));
    }

    #[test]
    fn iam_gated_provider_allows_manage_action_with_manage_permission() {
        let provider = IamGatedPolicyProvider::default();
        for action in ["update", "project.create", "session.create"] {
            let request = policy_request_with_action_and_roles(action, &["ai.agents.manage"]);
            let decision = provider.evaluate(request).expect("evaluate should succeed");
            assert_eq!(decision.decision, PolicyDecisionValue::Allow);
        }
    }

    #[test]
    fn iam_gated_provider_allows_all_actions_with_ai_wildcard() {
        let provider = IamGatedPolicyProvider::default();
        for action in [
            "list",
            "retrieve",
            "create",
            "update",
            "delete",
            "change_status",
        ] {
            let request = policy_request_with_action_and_roles(action, &["ai.*"]);
            let decision = provider.evaluate(request).expect("evaluate should succeed");
            assert_eq!(
                decision.decision,
                PolicyDecisionValue::Allow,
                "action {action} should be allowed with ai.* wildcard"
            );
        }
    }

    #[test]
    fn iam_gated_provider_allows_all_actions_with_global_wildcard() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("delete", &["*"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Allow);
    }

    #[test]
    fn iam_gated_provider_allows_all_actions_with_org_admin_role() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("delete", &["org_admin"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Allow);
    }

    #[test]
    fn iam_gated_provider_allows_all_actions_with_org_operations_role() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("create", &["org_operations"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Allow);
    }

    #[test]
    fn iam_gated_provider_denies_when_subject_has_no_permissions() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("list", &[]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
    }

    #[test]
    fn iam_gated_provider_denies_when_subject_is_missing() {
        let provider = IamGatedPolicyProvider::default();
        let request = PolicyRequest::new(
            "req.test",
            "agent.business.manage",
            "agent.business.tenant.100001",
        )
        .with_action("list");
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
    }

    #[test]
    fn iam_gated_provider_denies_unrecognized_permission_scope() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("list", &["iam.users.read"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
    }

    #[test]
    fn iam_gated_provider_treats_unknown_action_as_manage() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("unknown_action", &["ai.agents.read"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
    }

    // --- Self-service action tests ---

    #[test]
    fn iam_gated_provider_denies_self_service_action_without_use_permission() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("project.create", &["ai.agents.read"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
        assert!(decision
            .safe_reason
            .as_deref()
            .unwrap_or_default()
            .contains("iam.permission.missing:ai.agents.use"));
    }

    #[test]
    fn iam_gated_provider_allows_create_with_use_permission() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("create", &["ai.agents.use"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Allow);
    }

    #[test]
    fn iam_gated_provider_denies_self_service_action_without_subject() {
        let provider = IamGatedPolicyProvider::default();
        let request = PolicyRequest::new(
            "req.test",
            "agent.business.manage",
            "agent.business.tenant.100001",
        )
        .with_action("project.create");
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
        assert!(decision
            .safe_reason
            .as_deref()
            .unwrap_or_default()
            .contains("iam.permission.missing:ai.agents.use"));
    }

    #[test]
    fn iam_gated_provider_treats_missing_action_as_manage() {
        let provider = IamGatedPolicyProvider::default();
        let request = PolicyRequest::new(
            "req.test",
            "agent.business.manage",
            "agent.business.tenant.100001",
        )
        .with_subject(PolicySubject::new("user.test", "100001").with_role("ai.agents.use"));
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
        assert!(decision
            .safe_reason
            .as_deref()
            .unwrap_or_default()
            .contains("iam.permission.missing:ai.agents.manage"));
    }

    #[test]
    fn iam_gated_provider_denies_change_status_without_manage_permission() {
        let provider = IamGatedPolicyProvider::default();
        // change_status (activation) must still require ai.agents.manage,
        // preserving the review workflow while draft creation requires use permission.
        let request = policy_request_with_action_and_roles("change_status", &["ai.agents.read"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
        assert!(decision
            .safe_reason
            .as_deref()
            .unwrap_or_default()
            .contains("iam.permission.missing:ai.agents.manage"));
    }

    // --- DenyAllPolicyProvider tests ---

    #[test]
    fn deny_all_provider_denies_every_request() {
        let provider = DenyAllPolicyProvider::default();
        let request =
            policy_request_with_action_and_roles("list", &["ai.agents.manage", "ai.*", "*"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Deny);
    }

    #[test]
    fn deny_all_provider_default_has_descriptive_reason() {
        let provider = DenyAllPolicyProvider::default();
        assert!(!provider.reason.is_empty());
        assert!(!provider.provider_id.is_empty());
    }

    fn restore_optional_env(key: &str, value: Option<String>) {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
