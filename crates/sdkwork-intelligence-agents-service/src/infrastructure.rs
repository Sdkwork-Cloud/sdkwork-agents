use crate::domain::{
    AgentBusinessRecord, AgentCompositionSlotKind, AgentCompositionSlotRecord,
    AgentInteractionRecord, AgentMessageRecord, AgentProviderBindingRecord, AgentSessionRecord,
    AgentTaskRecord,
};
use crate::id::{AgentBusinessIdGenerator, AgentIdGenerator};
use crate::in_memory_pagination::{count_iterator, paginate_iterator, paginate_messages};
use crate::ports::{
    AgentAuditSink, AgentListQuery, AgentRepository, AuditEventListQuery, CompositionSlotListQuery,
    InteractionListQuery, McpMarketplaceListQuery, MessageListQuery, ProviderBindingListQuery,
    SessionListQuery, TaskListQuery,
};
use crate::validation::parse_rfc3339_datetime;
use sdkwork_agent_kernel::{
    KernelError, KernelEvent, KernelResult, PolicyDecision, PolicyProvider, PolicyRequest,
    ProviderHealth, ProviderManifest,
};
use sdkwork_utils_rust::{is_blank, trim};
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{LazyLock, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use time::OffsetDateTime;

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
    scrape_state: Mutex<ScrapeState>,
}

impl AgentMetricsRegistry {
    pub fn global() -> &'static Self {
        static REGISTRY: LazyLock<AgentMetricsRegistry> = LazyLock::new(|| AgentMetricsRegistry {
            http_requests_total: AtomicU64::new(0),
            http_errors_total: AtomicU64::new(0),
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

    pub fn snapshot(&self) -> AgentServiceMetrics {
        let http_requests_total = self.http_requests_total.load(AtomicOrdering::Relaxed);
        let http_errors_total = self.http_errors_total.load(AtomicOrdering::Relaxed);
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
type ProviderBindingPrimaryKey = (u64, String, String);
type ProviderBindingIndexKey = (u64, String, Reverse<bool>, Reverse<String>, String);
type CompositionSlotPrimaryKey = (u64, String, String);
type CompositionSlotIndexKey = (u64, String, i32, String);
type SessionPrimaryKey = (u64, String);
type SessionIndexKey = (u64, Reverse<String>, String);
type MessagePrimaryKey = (u64, String, String);
type MessageIndexKey = (u64, String, u64, String);
type InteractionPrimaryKey = (u64, String, String);
type InteractionIndexKey = (u64, String, Reverse<String>, String);
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
    provider_bindings: RwLock<HashMap<ProviderBindingPrimaryKey, AgentProviderBindingRecord>>,
    provider_binding_index: RwLock<BTreeMap<ProviderBindingIndexKey, ProviderBindingPrimaryKey>>,
    composition_slots: RwLock<HashMap<CompositionSlotPrimaryKey, AgentCompositionSlotRecord>>,
    composition_slot_index: RwLock<BTreeMap<CompositionSlotIndexKey, CompositionSlotPrimaryKey>>,
    sessions: RwLock<HashMap<SessionPrimaryKey, AgentSessionRecord>>,
    session_index: RwLock<BTreeMap<SessionIndexKey, SessionPrimaryKey>>,
    messages: RwLock<HashMap<MessagePrimaryKey, AgentMessageRecord>>,
    message_index: RwLock<BTreeMap<MessageIndexKey, MessagePrimaryKey>>,
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
            provider_bindings: RwLock::new(HashMap::new()),
            provider_binding_index: RwLock::new(BTreeMap::new()),
            composition_slots: RwLock::new(HashMap::new()),
            composition_slot_index: RwLock::new(BTreeMap::new()),
            sessions: RwLock::new(HashMap::new()),
            session_index: RwLock::new(BTreeMap::new()),
            messages: RwLock::new(HashMap::new()),
            message_index: RwLock::new(BTreeMap::new()),
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
    (record.tenant_id, record.session_id.clone())
}

fn session_index_key(record: &AgentSessionRecord) -> SessionIndexKey {
    (
        record.tenant_id,
        Reverse(record.updated_at.clone()),
        record.session_id.clone(),
    )
}

fn message_primary_key(record: &AgentMessageRecord) -> MessagePrimaryKey {
    (
        record.tenant_id,
        record.session_id.clone(),
        record.message_id.clone(),
    )
}

fn message_index_key(record: &AgentMessageRecord) -> MessageIndexKey {
    (
        record.tenant_id,
        record.session_id.clone(),
        record.sequence,
        record.message_id.clone(),
    )
}

fn interaction_primary_key(record: &AgentInteractionRecord) -> InteractionPrimaryKey {
    (
        record.tenant_id,
        record.session_id.clone(),
        record.interaction_id.clone(),
    )
}

fn interaction_index_key(record: &AgentInteractionRecord) -> InteractionIndexKey {
    (
        record.tenant_id,
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

    fn get(&self, tenant_id: u64, agent_id: &str) -> Option<AgentBusinessRecord> {
        self.agents
            .recovering_read()
            .get(&(tenant_id, agent_id.to_string()))
            .cloned()
    }

    fn list(&self, query: &AgentListQuery) -> Vec<AgentBusinessRecord> {
        let agents = self.agents.recovering_read();
        let index = self.agent_list_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, _, _, _), _)| *tenant_id == query.tenant_id)
            .filter_map(|(_, primary_key)| agents.get(primary_key))
            .filter(|record| agent_matches_list_query(record, query))
            .cloned();
        paginate_iterator(iter, &query.pagination)
    }

    fn count_agents(&self, query: &AgentListQuery) -> u64 {
        let agents = self.agents.recovering_read();
        let index = self.agent_list_index.recovering_read();
        count_iterator(
            index
                .iter()
                .filter(|((tenant_id, _, _, _), _)| *tenant_id == query.tenant_id)
                .filter_map(|(_, primary_key)| agents.get(primary_key))
                .filter(|record| agent_matches_list_query(record, query)),
        )
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
    ) -> Option<AgentProviderBindingRecord> {
        self.provider_bindings
            .recovering_read()
            .get(&(tenant_id, agent_id.to_string(), binding_id.to_string()))
            .cloned()
    }

    fn list_provider_bindings(
        &self,
        query: &ProviderBindingListQuery,
    ) -> Vec<AgentProviderBindingRecord> {
        let bindings = self.provider_bindings.recovering_read();
        let index = self.provider_binding_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, agent_id, _, _, _), _)| {
                *tenant_id == query.tenant_id && agent_id == &query.agent_id
            })
            .filter_map(|(_, primary_key)| bindings.get(primary_key))
            .cloned();
        paginate_iterator(iter, &query.pagination)
    }

    fn count_provider_bindings(&self, query: &ProviderBindingListQuery) -> u64 {
        let bindings = self.provider_bindings.recovering_read();
        let index = self.provider_binding_index.recovering_read();
        count_iterator(
            index
                .iter()
                .filter(|((tenant_id, agent_id, _, _, _), _)| {
                    *tenant_id == query.tenant_id && agent_id == &query.agent_id
                })
                .filter_map(|(_, primary_key)| bindings.get(primary_key)),
        )
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
    ) -> Option<AgentCompositionSlotRecord> {
        self.composition_slots
            .recovering_read()
            .get(&(tenant_id, agent_id.to_string(), slot_id.to_string()))
            .cloned()
    }

    fn list_composition_slots(
        &self,
        query: &CompositionSlotListQuery,
    ) -> Vec<AgentCompositionSlotRecord> {
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
        paginate_iterator(iter, &query.pagination)
    }

    fn count_composition_slots(&self, query: &CompositionSlotListQuery) -> u64 {
        let slots = self.composition_slots.recovering_read();
        let index = self.composition_slot_index.recovering_read();
        count_iterator(
            index
                .iter()
                .filter(|((tenant_id, agent_id, _, _), _)| {
                    *tenant_id == query.tenant_id && agent_id == &query.agent_id
                })
                .filter_map(|(_, primary_key)| slots.get(primary_key))
                .filter(|record| !record.is_deleted()),
        )
    }

    fn list_mcp_marketplace_slots(
        &self,
        query: &McpMarketplaceListQuery,
    ) -> Vec<AgentCompositionSlotRecord> {
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
        paginate_iterator(iter, &query.pagination)
    }

    fn count_mcp_marketplace_slots(&self, query: &McpMarketplaceListQuery) -> u64 {
        let agents = self.agents.recovering_read();
        let agent_index = self.agent_list_index.recovering_read();
        let active_agent_ids = active_agent_ids_for_tenant(&agents, &agent_index, query.tenant_id);
        let slots = self.composition_slots.recovering_read();
        let index = self.composition_slot_index.recovering_read();
        count_iterator(
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
        )
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

    fn get_session(&self, tenant_id: u64, session_id: &str) -> Option<AgentSessionRecord> {
        self.sessions
            .recovering_read()
            .get(&(tenant_id, session_id.to_string()))
            .cloned()
    }

    fn list_sessions(&self, query: &SessionListQuery) -> Vec<AgentSessionRecord> {
        let sessions = self.sessions.recovering_read();
        let index = self.session_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, _, _), _)| *tenant_id == query.tenant_id)
            .filter_map(|(_, primary_key)| sessions.get(primary_key))
            .filter(|record| session_matches_list_query(record, query))
            .cloned();
        paginate_iterator(iter, &query.pagination)
    }

    fn count_sessions(&self, query: &SessionListQuery) -> u64 {
        let sessions = self.sessions.recovering_read();
        let index = self.session_index.recovering_read();
        count_iterator(
            index
                .iter()
                .filter(|((tenant_id, _, _), _)| *tenant_id == query.tenant_id)
                .filter_map(|(_, primary_key)| sessions.get(primary_key))
                .filter(|record| session_matches_list_query(record, query)),
        )
    }

    fn insert_message(&self, record: AgentMessageRecord) -> KernelResult<()> {
        let primary_key = message_primary_key(&record);
        let mut messages = self.messages.recovering_write();
        if messages.contains_key(&primary_key) {
            return Err(KernelError::conflict("message already exists"));
        }
        let index_key = message_index_key(&record);
        messages.insert(primary_key.clone(), record);
        self.message_index
            .recovering_write()
            .insert(index_key, primary_key);
        Ok(())
    }

    fn update_message(&self, record: AgentMessageRecord) -> KernelResult<()> {
        let primary_key = message_primary_key(&record);
        let mut messages = self.messages.recovering_write();
        if !messages.contains_key(&primary_key) {
            return Err(KernelError::validation("message not found"));
        }
        let existing = messages.get(&primary_key).expect("message exists");
        let previous_index_key = message_index_key(existing);
        let next_index_key = message_index_key(&record);
        messages.insert(primary_key.clone(), record);
        let mut index = self.message_index.recovering_write();
        index.remove(&previous_index_key);
        index.insert(next_index_key, primary_key);
        Ok(())
    }

    fn get_message(
        &self,
        tenant_id: u64,
        session_id: &str,
        message_id: &str,
    ) -> Option<AgentMessageRecord> {
        self.messages
            .recovering_read()
            .get(&(tenant_id, session_id.to_string(), message_id.to_string()))
            .cloned()
    }

    fn list_messages(&self, query: &MessageListQuery) -> Vec<AgentMessageRecord> {
        let messages = self.messages.recovering_read();
        let index = self.message_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, session_id, _, _), _)| {
                *tenant_id == query.tenant_id && session_id == &query.session_id
            })
            .filter_map(|(_, primary_key)| messages.get(primary_key))
            .filter(|record| message_matches_list_query(record, query))
            .cloned();
        paginate_messages(iter, &query.pagination, query.sort)
    }

    fn count_messages(&self, query: &MessageListQuery) -> u64 {
        let messages = self.messages.recovering_read();
        let index = self.message_index.recovering_read();
        count_iterator(
            index
                .iter()
                .filter(|((tenant_id, session_id, _, _), _)| {
                    *tenant_id == query.tenant_id && session_id == &query.session_id
                })
                .filter_map(|(_, primary_key)| messages.get(primary_key))
                .filter(|record| message_matches_list_query(record, query)),
        )
    }

    fn next_message_sequence(&self, tenant_id: u64, session_id: &str) -> KernelResult<u64> {
        let index = self.message_index.recovering_read();
        let max_sequence = index
            .iter()
            .filter(|((indexed_tenant_id, indexed_session_id, _, _), _)| {
                *indexed_tenant_id == tenant_id && indexed_session_id == session_id
            })
            .map(|((_, _, sequence, _), _)| *sequence)
            .max()
            .unwrap_or(0);
        Ok(max_sequence.saturating_add(1))
    }

    fn insert_chat_turn(
        &self,
        session: AgentSessionRecord,
        mut user_message: AgentMessageRecord,
        mut assistant_message: AgentMessageRecord,
    ) -> KernelResult<(AgentSessionRecord, AgentMessageRecord, AgentMessageRecord)> {
        let tenant_id = user_message.tenant_id;
        let session_id = user_message.session_id.clone();
        let user_sequence = self.next_message_sequence(tenant_id, session_id.as_str())?;
        user_message.sequence = user_sequence;
        assistant_message.sequence = user_sequence.saturating_add(1);
        self.insert_message(user_message.clone())?;
        self.insert_message(assistant_message.clone())?;
        self.update_session(session.clone())?;
        Ok((session, user_message, assistant_message))
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
        session_id: &str,
        interaction_id: &str,
    ) -> Option<AgentInteractionRecord> {
        self.interactions
            .recovering_read()
            .get(&(
                tenant_id,
                session_id.to_string(),
                interaction_id.to_string(),
            ))
            .cloned()
    }

    fn list_interactions(&self, query: &InteractionListQuery) -> Vec<AgentInteractionRecord> {
        let interactions = self.interactions.recovering_read();
        let index = self.interaction_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, session_id, _, _), _)| {
                *tenant_id == query.tenant_id && session_id == &query.session_id
            })
            .filter_map(|(_, primary_key)| interactions.get(primary_key))
            .filter(|record| interaction_matches_list_query(record, query))
            .cloned();
        paginate_iterator(iter, &query.pagination)
    }

    fn count_interactions(&self, query: &InteractionListQuery) -> u64 {
        let interactions = self.interactions.recovering_read();
        let index = self.interaction_index.recovering_read();
        count_iterator(
            index
                .iter()
                .filter(|((tenant_id, session_id, _, _), _)| {
                    *tenant_id == query.tenant_id && session_id == &query.session_id
                })
                .filter_map(|(_, primary_key)| interactions.get(primary_key))
                .filter(|record| interaction_matches_list_query(record, query)),
        )
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

    fn get_task(&self, tenant_id: u64, task_id: &str) -> Option<AgentTaskRecord> {
        self.tasks
            .recovering_read()
            .get(&(tenant_id, task_id.to_string()))
            .cloned()
    }

    fn list_tasks(&self, query: &TaskListQuery) -> Vec<AgentTaskRecord> {
        let tasks = self.tasks.recovering_read();
        let index = self.task_index.recovering_read();
        let iter = index
            .iter()
            .filter(|((tenant_id, _, _), _)| *tenant_id == query.tenant_id)
            .filter_map(|(_, primary_key)| tasks.get(primary_key))
            .filter(|record| task_matches_list_query(record, query))
            .cloned();
        paginate_iterator(iter, &query.pagination)
    }

    fn count_tasks(&self, query: &TaskListQuery) -> u64 {
        let tasks = self.tasks.recovering_read();
        let index = self.task_index.recovering_read();
        count_iterator(
            index
                .iter()
                .filter(|((tenant_id, _, _), _)| *tenant_id == query.tenant_id)
                .filter_map(|(_, primary_key)| tasks.get(primary_key))
                .filter(|record| task_matches_list_query(record, query)),
        )
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
/// roles or resource attributes. Use [`IamGatedPolicyProvider`] for
/// production deployments, or [`DenyAllPolicyProvider`] as a fail-closed
/// placeholder when IAM integration is not yet wired. This type is retained
/// only for local development and integration-test scenarios.
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

/// Role codes that grant wildcard AI permissions per the IAM module manifest
/// `roleGrantExtensions` (org_admin and org_operations both map to `ai.*`).
const IAM_ADMIN_ROLE_CODES: &[&str] = &["org_admin", "org_operations"];

/// Policy actions (from `AgentsService::authorize`) that are read-only and
/// therefore require only `ai.agents.read`. Any action not in this set is
/// treated as a manage operation requiring `ai.agents.manage`.
const READ_ONLY_POLICY_ACTIONS: &[&str] = &[
    "retrieve",
    "list",
    "audit.read",
    "provider_binding.list",
    "composition_slot.list",
    "composition_slot.retrieve",
    "task.list",
    "task.retrieve",
    "interaction.list",
    "interaction.retrieve",
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

    /// Determine the required IAM permission for the given policy action.
    fn required_permission_for_action(action: Option<&str>) -> &'static str {
        match action {
            Some(action) if READ_ONLY_POLICY_ACTIONS.contains(&action) => {
                IAM_PERMISSION_AGENTS_READ
            }
            _ => IAM_PERMISSION_AGENTS_MANAGE,
        }
    }

    /// Return `true` if the subject's role/scope entry satisfies the required
    /// permission. Supports wildcards `ai.*` and `*`, and known admin role
    /// codes that grant `ai.*`. Also honors the implication that
    /// `ai.agents.manage` grants `ai.agents.read` (manage implies read).
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
        // Manage permission implies read permission within the same resource.
        if entry == IAM_PERMISSION_AGENTS_MANAGE
            && required_permission == IAM_PERMISSION_AGENTS_READ
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

/// In-memory audit sink that stores events in a `Vec`. Uses `Mutex` for
/// interior mutability so it can implement `AgentAuditSink` with `&self`.
/// Events are lost when the process exits. Use
/// [`crate::persistence::PostgresAgentAuditSink`] for production deployments
/// that require persistent audit trails.
#[derive(Debug, Clone, Default)]
pub struct InMemoryAgentAuditSink {
    events: std::sync::Arc<Mutex<Vec<KernelEvent>>>,
}

impl InMemoryAgentAuditSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<KernelEvent> {
        self.events.recovering_lock().clone()
    }
}

impl AgentAuditSink for InMemoryAgentAuditSink {
    fn record(&self, event: KernelEvent) -> KernelResult<()> {
        self.events.recovering_lock().push(event);
        Ok(())
    }

    fn list_events(
        &self,
        query: &AuditEventListQuery,
    ) -> KernelResult<crate::ports::PaginatedResult<KernelEvent>> {
        use crate::ports::offset_paginated_result;
        let events = self.events.recovering_lock();
        let mut matched: Vec<KernelEvent> = events
            .iter()
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
            .filter(|event| {
                let occurred_at = event.occurred_at.as_deref().unwrap_or_default();
                query
                    .from
                    .as_ref()
                    .map(|from| occurred_at >= from.as_str())
                    .unwrap_or(true)
            })
            .filter(|event| {
                let occurred_at = event.occurred_at.as_deref().unwrap_or_default();
                query
                    .to
                    .as_ref()
                    .map(|to| occurred_at <= to.as_str())
                    .unwrap_or(true)
            })
            .cloned()
            .collect();
        matched.sort_by(|left, right| {
            audit_event_occurred_at(right)
                .cmp(&audit_event_occurred_at(left))
                .then_with(|| right.event_id.cmp(&left.event_id))
        });
        let total_count = matched.len() as u64;
        let page = paginate_iterator(matched.into_iter(), &query.pagination);
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
    query.include_archived || record.status.as_str() != "archived"
}

fn message_matches_list_query(record: &AgentMessageRecord, query: &MessageListQuery) -> bool {
    if record.tenant_id != query.tenant_id || record.session_id != query.session_id {
        return false;
    }
    if let Some(role) = query.role.as_ref() {
        if record.role.as_str() != role {
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
    if record.tenant_id != query.tenant_id || record.session_id != query.session_id {
        return false;
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

fn audit_event_occurred_at(event: &KernelEvent) -> OffsetDateTime {
    let occurred_at_raw = event
        .occurred_at
        .as_deref()
        .unwrap_or("1970-01-01T00:00:00Z");
    parse_rfc3339_datetime(occurred_at_raw, "audit event occurred_at")
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
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

        let provider = provider_result.expect(
            "AllowAllPolicyProvider::allow must fail closed instead of panicking",
        );
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
    fn iam_gated_provider_allows_read_action_with_manage_permission() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("retrieve", &["ai.agents.manage"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Allow);
    }

    #[test]
    fn iam_gated_provider_denies_manage_action_with_only_read_permission() {
        let provider = IamGatedPolicyProvider::default();
        let request = policy_request_with_action_and_roles("create", &["ai.agents.read"]);
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
        let request = policy_request_with_action_and_roles("update", &["ai.agents.manage"]);
        let decision = provider.evaluate(request).expect("evaluate should succeed");
        assert_eq!(decision.decision, PolicyDecisionValue::Allow);
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
