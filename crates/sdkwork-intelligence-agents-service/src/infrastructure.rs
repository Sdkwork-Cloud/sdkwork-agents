use crate::domain::{
    AgentBusinessRecord, AgentCompositionSlotRecord, AgentInteractionRecord,
    AgentMessageRecord, AgentProviderBindingRecord, AgentSessionRecord,
};
use crate::id::{AgentBusinessIdGenerator, AgentIdGenerator};
use crate::ports::{
    AgentAuditSink, AgentListQuery, AgentRepository, InteractionListQuery, MessageListQuery,
    SessionListQuery,
};
use sdkwork_agent_kernel::{
    KernelError, KernelEvent, KernelResult, PolicyDecision, PolicyProvider, PolicyRequest,
    ProviderHealth, ProviderManifest,
};
use sdkwork_utils_rust::{is_blank, trim};
use std::sync::{LazyLock, Mutex, RwLock};

// ---------------------------------------------------------------------------
// Production environment safety checks
// ---------------------------------------------------------------------------

/// Environment variable name for development authentication bypass.
/// When set to "true", AllowAllPolicyProvider permits all requests without
/// IAM validation. This MUST NOT be enabled in production environments.
pub const ENV_DEV_AUTH_BYPASS: &str = "SDKWORK_AGENTS_DEV_AUTH_BYPASS";

/// Environment variable name indicating production deployment.
/// When set to "production", "prod", or "live", security bypasses are forbidden.
pub const ENV_DEPLOYMENT_ENV: &str = "SDKWORK_DEPLOYMENT_ENV";

/// Production environment identifiers that forbid security bypasses.
const PRODUCTION_ENV_IDENTIFIERS: &[&str] = &["production", "prod", "live", "staging"];

/// Validates that development authentication bypass is not enabled in production.
///
/// # Panics
/// Panics if `SDKWORK_AGENTS_DEV_AUTH_BYPASS=true` and the deployment environment
/// is identified as production. This is a fail-closed safety check that must run
/// during application bootstrap before any policy provider is instantiated.
///
/// # Security Standard Compliance
/// This check implements SECURITY_SPEC §5.1 requirement that rate limiting and
/// authentication must be enforced in production, preventing accidental deployment
/// of development-only security bypasses.
pub fn validate_production_security_config() {
    let dev_bypass = std::env::var(ENV_DEV_AUTH_BYPASS)
        .unwrap_or_default()
        .to_lowercase();
    
    if dev_bypass != "true" {
        return; // No bypass enabled, safe to proceed
    }
    
    // Check deployment environment
    let deployment_env = std::env::var(ENV_DEPLOYMENT_ENV)
        .unwrap_or_default()
        .to_lowercase();
    
    // Also check common alternative environment variable names
    let alt_env = std::env::var("ENVIRONMENT")
        .unwrap_or_default()
        .to_lowercase();
    
    let is_production = PRODUCTION_ENV_IDENTIFIERS
        .iter()
        .any(|id| deployment_env == *id || alt_env == *id);
    
    if is_production {
        panic!(
            "SECURITY VIOLATION: {} is enabled in production environment '{}'. \
            This configuration bypasses IAM authentication and is forbidden in production. \
            Remove this environment variable immediately or set it to 'false'. \
            See SECURITY_SPEC §5.1 for authentication requirements.",
            ENV_DEV_AUTH_BYPASS,
            if deployment_env.is_empty() { &alt_env } else { &deployment_env }
        );
    }
    
    // Log warning for development environments
    tracing::warn!(
        env_var = ENV_DEV_AUTH_BYPASS,
        value = %dev_bypass,
        "Development authentication bypass is enabled. This MUST NOT be used in production."
    );
}

/// Returns true if the current deployment is a production environment.
pub fn is_production_environment() -> bool {
    let deployment_env = std::env::var(ENV_DEPLOYMENT_ENV)
        .unwrap_or_default()
        .to_lowercase();
    let alt_env = std::env::var("ENVIRONMENT")
        .unwrap_or_default()
        .to_lowercase();
    
    PRODUCTION_ENV_IDENTIFIERS
        .iter()
        .any(|id| deployment_env == *id || alt_env == *id)
}

// ---------------------------------------------------------------------------
// Metrics types for observability (O-01)
// ---------------------------------------------------------------------------

use std::cmp::Ordering;
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
        self.http_requests_total.fetch_add(1, AtomicOrdering::Relaxed);
        if status >= 400 {
            self.http_errors_total.fetch_add(1, AtomicOrdering::Relaxed);
        }
    }

    pub fn snapshot(&self) -> AgentServiceMetrics {
        let http_requests_total = self.http_requests_total.load(AtomicOrdering::Relaxed);
        let http_errors_total = self.http_errors_total.load(AtomicOrdering::Relaxed);
        let http_requests_per_second = {
            let mut state = self
                .scrape_state
                .lock()
                .expect("agents metrics scrape state poisoned");
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

        output.push_str("# HELP sdkwork_agents_errors_total Total managed-store HTTP error responses\n");
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
        
        output.push_str("# HELP sdkwork_agents_provider_bindings_total Total number of provider bindings\n");
        output.push_str("# TYPE sdkwork_agents_provider_bindings_total gauge\n");
        output.push_str(&format!("sdkwork_agents_provider_bindings_total {}\n", self.total_provider_bindings));
        
        output.push_str("# HELP sdkwork_agents_provider_bindings_active Number of active provider bindings\n");
        output.push_str("# TYPE sdkwork_agents_provider_bindings_active gauge\n");
        output.push_str(&format!("sdkwork_agents_provider_bindings_active {}\n", self.active_provider_bindings));
        
        output.push_str("# HELP sdkwork_agents_composition_slots_total Total number of composition slots\n");
        output.push_str("# TYPE sdkwork_agents_composition_slots_total gauge\n");
        output.push_str(&format!("sdkwork_agents_composition_slots_total {}\n", self.total_composition_slots));
        
        output.push_str("# HELP sdkwork_agents_audit_events_total Total number of audit events\n");
        output.push_str("# TYPE sdkwork_agents_audit_events_total counter\n");
        output.push_str(&format!("sdkwork_agents_audit_events_total {}\n", self.audit_events_count));
        
        // Request counts by operation
        output.push_str("# HELP sdkwork_agents_requests_by_operation_total Total requests by operation\n");
        output.push_str("# TYPE sdkwork_agents_requests_by_operation_total counter\n");
        for (operation, count) in &self.request_counts {
            output.push_str(&format!(
                "sdkwork_agents_requests_by_operation_total{{operation=\"{}\"}} {}\n",
                operation, count
            ));
        }
        
        // Error counts by operation
        output.push_str("# HELP sdkwork_agents_errors_by_operation_total Total errors by operation\n");
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
/// All trait methods use `&self` — the `RwLock` handles concurrent access.
/// This makes the repository compatible with the stateless `AgentsService`
/// and eliminates the global `Mutex<AgentsService>` bottleneck.
#[derive(Debug)]
pub struct InMemoryAgentRepository {
    id_generator: AgentBusinessIdGenerator,
    records: RwLock<Vec<AgentBusinessRecord>>,
    provider_bindings: RwLock<Vec<AgentProviderBindingRecord>>,
    composition_slots: RwLock<Vec<AgentCompositionSlotRecord>>,
    sessions: RwLock<Vec<AgentSessionRecord>>,
    messages: RwLock<Vec<AgentMessageRecord>>,
    interactions: RwLock<Vec<AgentInteractionRecord>>,
}

impl InMemoryAgentRepository {
    pub fn new() -> Self {
        Self {
            id_generator: AgentBusinessIdGenerator::new_default()
                .expect("default agents managed store snowflake node id is valid"),
            records: RwLock::new(Vec::new()),
            provider_bindings: RwLock::new(Vec::new()),
            composition_slots: RwLock::new(Vec::new()),
            sessions: RwLock::new(Vec::new()),
            messages: RwLock::new(Vec::new()),
            interactions: RwLock::new(Vec::new()),
        }
    }

    pub fn records(&self) -> Vec<AgentBusinessRecord> {
        self.records
            .read()
            .expect("in-memory repository rwlock poisoned")
            .clone()
    }
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
        let mut records = self
            .records
            .write()
            .expect("in-memory repository rwlock poisoned");
        if records.iter().any(|existing| {
            existing.tenant_id == record.tenant_id && existing.agent_id == record.agent_id
        }) {
            return Err(KernelError::conflict("agent already exists"));
        }
        if records
            .iter()
            .any(|existing| existing.tenant_id == record.tenant_id && existing.code == record.code)
        {
            return Err(KernelError::conflict("agent code already exists"));
        }
        records.push(record);
        Ok(())
    }

    fn update(&self, record: AgentBusinessRecord) -> KernelResult<()> {
        let mut records = self
            .records
            .write()
            .expect("in-memory repository rwlock poisoned");
        let index = records
            .iter()
            .position(|existing| {
                existing.tenant_id == record.tenant_id && existing.agent_id == record.agent_id
            })
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        let expected_version = records[index].version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "agent version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        if records.iter().enumerate().any(|(current, existing)| {
            current != index
                && existing.tenant_id == record.tenant_id
                && existing.code == record.code
        }) {
            return Err(KernelError::conflict("agent code already exists"));
        }
        records[index] = record;
        Ok(())
    }

    fn get(&self, tenant_id: u64, agent_id: &str) -> Option<AgentBusinessRecord> {
        let records = self
            .records
            .read()
            .expect("in-memory repository rwlock poisoned");
        records
            .iter()
            .find(|record| record.tenant_id == tenant_id && record.agent_id == agent_id)
            .cloned()
    }

    fn list(&self, query: &AgentListQuery) -> Vec<AgentBusinessRecord> {
        let records = self
            .records
            .read()
            .expect("in-memory repository rwlock poisoned");
        records
            .iter()
            .filter(|record| record.tenant_id == query.tenant_id)
            .filter(|record| {
                if let Some(organization_id) = query.organization_id {
                    record.organization_id == organization_id
                } else {
                    true
                }
            })
            .filter(|record| {
                if let Some(owner_user_id) = query.owner_user_id {
                    record.owner_user_id == owner_user_id
                } else {
                    true
                }
            })
            .filter(|record| query.include_deleted || !record.is_deleted())
            .filter(|record| {
                let Some(search_query) = query.search_query.as_ref() else {
                    return true;
                };
                if is_blank(Some(search_query.as_str())) {
                    return true;
                }
                let normalized_query = trim(search_query).to_lowercase();

                let description = record.description.as_deref().unwrap_or("");
                record
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
                        .contains(normalized_query.as_str())
            })
            .cloned()
            .collect()
    }

    fn insert_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        let mut bindings = self
            .provider_bindings
            .write()
            .expect("in-memory repository rwlock poisoned");
        if bindings.iter().any(|existing| {
            existing.tenant_id == record.tenant_id
                && existing.agent_id == record.agent_id
                && existing.binding_id == record.binding_id
        }) {
            return Err(KernelError::conflict(
                "agent provider binding already exists",
            ));
        }
        if record.active
            && bindings.iter().any(|existing| {
                existing.tenant_id == record.tenant_id
                    && existing.agent_id == record.agent_id
                    && existing.active
            })
        {
            return Err(KernelError::conflict(
                "active provider binding already exists",
            ));
        }
        bindings.push(record);
        Ok(())
    }

    fn update_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        let mut bindings = self
            .provider_bindings
            .write()
            .expect("in-memory repository rwlock poisoned");
        let index = bindings
            .iter()
            .position(|existing| {
                existing.tenant_id == record.tenant_id
                    && existing.agent_id == record.agent_id
                    && existing.binding_id == record.binding_id
            })
            .ok_or_else(|| KernelError::validation("agent provider binding not found"))?;

        let expected_version = bindings[index].version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "provider binding version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        if record.active
            && bindings.iter().enumerate().any(|(current, existing)| {
                current != index
                    && existing.tenant_id == record.tenant_id
                    && existing.agent_id == record.agent_id
                    && existing.active
            })
        {
            return Err(KernelError::conflict(
                "active provider binding already exists",
            ));
        }
        bindings[index] = record;
        Ok(())
    }

    fn get_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
    ) -> Option<AgentProviderBindingRecord> {
        let bindings = self
            .provider_bindings
            .read()
            .expect("in-memory repository rwlock poisoned");
        bindings
            .iter()
            .find(|record| {
                record.tenant_id == tenant_id
                    && record.agent_id == agent_id
                    && record.binding_id == binding_id
            })
            .cloned()
    }

    fn list_provider_bindings(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> Vec<AgentProviderBindingRecord> {
        let bindings = self
            .provider_bindings
            .read()
            .expect("in-memory repository rwlock poisoned");
        let mut records: Vec<AgentProviderBindingRecord> = bindings
            .iter()
            .filter(|record| record.tenant_id == tenant_id && record.agent_id == agent_id)
            .cloned()
            .collect();
        records.sort_by(compare_provider_bindings_standard_order);
        records
    }

    fn insert_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        let mut slots = self
            .composition_slots
            .write()
            .expect("in-memory repository rwlock poisoned");
        if slots.iter().any(|existing| {
            existing.tenant_id == record.tenant_id
                && existing.agent_id == record.agent_id
                && existing.slot_id == record.slot_id
        }) {
            return Err(KernelError::conflict("composition slot already exists"));
        }
        slots.push(record);
        Ok(())
    }

    fn update_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        let mut slots = self
            .composition_slots
            .write()
            .expect("in-memory repository rwlock poisoned");
        let index = slots
            .iter()
            .position(|existing| {
                existing.tenant_id == record.tenant_id
                    && existing.agent_id == record.agent_id
                    && existing.slot_id == record.slot_id
            })
            .ok_or_else(|| KernelError::validation("composition slot not found"))?;

        let expected_version = slots[index].version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "composition slot version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        slots[index] = record;
        Ok(())
    }

    fn get_composition_slot(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str,
    ) -> Option<AgentCompositionSlotRecord> {
        let slots = self
            .composition_slots
            .read()
            .expect("in-memory repository rwlock poisoned");
        slots
            .iter()
            .find(|record| {
                record.tenant_id == tenant_id
                    && record.agent_id == agent_id
                    && record.slot_id == slot_id
            })
            .cloned()
    }

    fn list_composition_slots(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> Vec<AgentCompositionSlotRecord> {
        let slots = self
            .composition_slots
            .read()
            .expect("in-memory repository rwlock poisoned");
        slots
            .iter()
            .filter(|record| record.tenant_id == tenant_id && record.agent_id == agent_id)
            .cloned()
            .collect()
    }

    fn insert_session(&self, record: AgentSessionRecord) -> KernelResult<()> {
        let mut sessions = self
            .sessions
            .write()
            .expect("in-memory repository rwlock poisoned");
        if sessions.iter().any(|existing| {
            existing.tenant_id == record.tenant_id && existing.session_id == record.session_id
        }) {
            return Err(KernelError::conflict("session already exists"));
        }
        sessions.push(record);
        Ok(())
    }

    fn update_session(&self, record: AgentSessionRecord) -> KernelResult<()> {
        let mut sessions = self
            .sessions
            .write()
            .expect("in-memory repository rwlock poisoned");
        let index = sessions
            .iter()
            .position(|existing| {
                existing.tenant_id == record.tenant_id && existing.session_id == record.session_id
            })
            .ok_or_else(|| KernelError::validation("session not found"))?;
        sessions[index] = record;
        Ok(())
    }

    fn get_session(&self, tenant_id: u64, session_id: &str) -> Option<AgentSessionRecord> {
        let sessions = self
            .sessions
            .read()
            .expect("in-memory repository rwlock poisoned");
        sessions
            .iter()
            .find(|record| record.tenant_id == tenant_id && record.session_id == session_id)
            .cloned()
    }

    fn list_sessions(&self, query: &SessionListQuery) -> Vec<AgentSessionRecord> {
        let sessions = self
            .sessions
            .read()
            .expect("in-memory repository rwlock poisoned");
        sessions
            .iter()
            .filter(|record| record.tenant_id == query.tenant_id)
            .filter(|record| {
                query
                    .agent_id
                    .as_ref()
                    .is_none_or(|agent_id| record.agent_id == *agent_id)
            })
            .filter(|record| {
                query
                    .owner_user_id
                    .is_none_or(|owner_user_id| record.owner_user_id == owner_user_id)
            })
            .filter(|record| {
                query
                    .status
                    .as_ref()
                    .is_none_or(|status| record.status.as_str() == status)
            })
            .filter(|record| query.include_archived || record.status.as_str() != "archived")
            .skip(query.pagination.offset)
            .take(query.pagination.page_size).cloned()
            .collect()
    }

    fn insert_message(&self, record: AgentMessageRecord) -> KernelResult<()> {
        let mut messages = self
            .messages
            .write()
            .expect("in-memory repository rwlock poisoned");
        if messages.iter().any(|existing| {
            existing.tenant_id == record.tenant_id
                && existing.session_id == record.session_id
                && existing.message_id == record.message_id
        }) {
            return Err(KernelError::conflict("message already exists"));
        }
        messages.push(record);
        Ok(())
    }

    fn update_message(&self, record: AgentMessageRecord) -> KernelResult<()> {
        let mut messages = self
            .messages
            .write()
            .expect("in-memory repository rwlock poisoned");
        let index = messages
            .iter()
            .position(|existing| {
                existing.tenant_id == record.tenant_id
                    && existing.session_id == record.session_id
                    && existing.message_id == record.message_id
            })
            .ok_or_else(|| KernelError::validation("message not found"))?;
        messages[index] = record;
        Ok(())
    }

    fn get_message(
        &self,
        tenant_id: u64,
        session_id: &str,
        message_id: &str,
    ) -> Option<AgentMessageRecord> {
        let messages = self
            .messages
            .read()
            .expect("in-memory repository rwlock poisoned");
        messages
            .iter()
            .find(|record| {
                record.tenant_id == tenant_id
                    && record.session_id == session_id
                    && record.message_id == message_id
            })
            .cloned()
    }

    fn list_messages(&self, query: &MessageListQuery) -> Vec<AgentMessageRecord> {
        let messages = self
            .messages
            .read()
            .expect("in-memory repository rwlock poisoned");
        let mut records: Vec<AgentMessageRecord> = messages
            .iter()
            .filter(|record| {
                record.tenant_id == query.tenant_id && record.session_id == query.session_id
            })
            .filter(|record| {
                query
                    .role
                    .as_ref()
                    .is_none_or(|role| record.role.as_str() == role)
            })
            .filter(|record| {
                query
                    .status
                    .as_ref()
                    .is_none_or(|status| record.status.as_str() == status)
            })
            .cloned()
            .collect();
        records.sort_by_key(|record| record.sequence);
        records
            .into_iter()
            .skip(query.pagination.offset)
            .take(query.pagination.page_size)
            .collect()
    }

    fn next_message_sequence(&self, tenant_id: u64, session_id: &str) -> KernelResult<u64> {
        let messages = self
            .messages
            .read()
            .expect("in-memory repository rwlock poisoned");
        let max_sequence = messages
            .iter()
            .filter(|record| record.tenant_id == tenant_id && record.session_id == session_id)
            .map(|record| record.sequence)
            .max()
            .unwrap_or(0);
        Ok(max_sequence.saturating_add(1))
    }

    fn insert_interaction(&self, record: AgentInteractionRecord) -> KernelResult<()> {
        let mut interactions = self
            .interactions
            .write()
            .expect("in-memory repository rwlock poisoned");
        if interactions.iter().any(|existing| {
            existing.tenant_id == record.tenant_id
                && existing.session_id == record.session_id
                && existing.interaction_id == record.interaction_id
        }) {
            return Err(KernelError::conflict(format!(
                "interaction {} already exists for session {}",
                record.interaction_id, record.session_id
            )));
        }
        interactions.push(record);
        Ok(())
    }

    fn update_interaction(&self, record: AgentInteractionRecord) -> KernelResult<()> {
        let mut interactions = self
            .interactions
            .write()
            .expect("in-memory repository rwlock poisoned");
        let existing = interactions.iter_mut().find(|existing| {
            existing.tenant_id == record.tenant_id
                && existing.session_id == record.session_id
                && existing.interaction_id == record.interaction_id
        });
        match existing {
            Some(slot) => {
                *slot = record;
                Ok(())
            }
            None => Err(KernelError::validation(format!(
                "interaction {} not found for session {}",
                record.interaction_id, record.session_id
            ))),
        }
    }

    fn get_interaction(
        &self,
        tenant_id: u64,
        session_id: &str,
        interaction_id: &str,
    ) -> Option<AgentInteractionRecord> {
        let interactions = self
            .interactions
            .read()
            .expect("in-memory repository rwlock poisoned");
        interactions
            .iter()
            .find(|record| {
                record.tenant_id == tenant_id
                    && record.session_id == session_id
                    && record.interaction_id == interaction_id
            })
            .cloned()
    }

    fn list_interactions(&self, query: &InteractionListQuery) -> Vec<AgentInteractionRecord> {
        let interactions = self
            .interactions
            .read()
            .expect("in-memory repository rwlock poisoned");
        let mut records: Vec<AgentInteractionRecord> = interactions
            .iter()
            .filter(|record| {
                record.tenant_id == query.tenant_id && record.session_id == query.session_id
            })
            .filter(|record| {
                query
                    .status
                    .as_ref()
                    .is_none_or(|status| record.status.as_str() == status)
            })
            .cloned()
            .collect();
        records.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        records
            .into_iter()
            .skip(query.pagination.offset)
            .take(query.pagination.page_size)
            .collect()
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
    /// Create a provider that allows every request. The `provider_id` should
    /// identify the policy source (e.g. `"policy.agents.dev"` for development).
    ///
    /// # Security Check
    /// This method automatically validates that the application is not running
    /// in production. If `SDKWORK_AGENTS_DEV_AUTH_BYPASS=true` and the environment
    /// is identified as production, this method will panic to prevent security
    /// misconfiguration.
    pub fn allow(provider_id: impl Into<String>) -> Self {
        validate_production_security_config();
        Self {
            provider_id: provider_id.into(),
            mode: PolicyMode::Allow("static.allow".to_string()),
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
        self.events
            .lock()
            .expect("in-memory audit sink mutex poisoned")
            .clone()
    }
}

impl AgentAuditSink for InMemoryAgentAuditSink {
    fn record(&self, event: KernelEvent) -> KernelResult<()> {
        self.events
            .lock()
            .expect("in-memory audit sink mutex poisoned")
            .push(event);
        Ok(())
    }

    fn list_events(&self, _tenant_id: u64, agent_id: &str) -> KernelResult<Vec<KernelEvent>> {
        let events = self
            .events
            .lock()
            .expect("in-memory audit sink mutex poisoned");
        Ok(events
            .iter()
            .filter(|event| {
                crate::persistence::extract_event_context(event.payload.as_str(), "agent_id")
                    .map(|id| id == agent_id)
                    .unwrap_or(false)
            })
            .cloned()
            .collect())
    }
}

// ---------------------------------------------------------------------------
// Shared comparison helpers for in-memory repository sorting
// ---------------------------------------------------------------------------

/// Sort order for provider bindings: active first, then by updated_at desc,
/// then by binding_id ascending.
pub(crate) fn compare_provider_bindings_standard_order(
    left: &AgentProviderBindingRecord,
    right: &AgentProviderBindingRecord,
) -> Ordering {
    right
        .active
        .cmp(&left.active)
        .then_with(|| right.updated_at.cmp(&left.updated_at))
        .then_with(|| left.binding_id.cmp(&right.binding_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        AgentBusinessStatus, AgentImplementationKind, AgentImplementationType,
        AgentProviderBindingRecord, AgentVisibility,
    };
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
            .list_provider_bindings(100_001, "agent.alpha")
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
}
