use crate::domain::{
    AgentBusinessRecord, AgentCompositionSlotRecord, AgentInteractionRecord, AgentMessageRecord,
    AgentProviderBindingRecord, AgentSessionRecord,
};
use crate::validation::optional_non_blank;
use sdkwork_agent_kernel::{KernelError, KernelEvent, KernelResult};

// ---------------------------------------------------------------------------
// Pagination types for list operations
// ---------------------------------------------------------------------------

/// Maximum allowed page size to prevent memory exhaustion and ensure
/// consistent API performance per DATABASE_SPEC §16 pagination requirements.
pub const MAX_PAGE_SIZE: usize = 200;

/// Default page size for list operations when not specified.
pub const DEFAULT_PAGE_SIZE: usize = 20;

/// Pagination parameters for list queries.
/// Implements DATABASE_SPEC §16 requirements for mandatory pagination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginationParams {
    /// Maximum number of items to return per page (1-200)
    pub page_size: usize,
    /// Zero-based row offset for page navigation
    pub offset: usize,
    /// Cursor for fetching the next page (offset-free pagination)
    pub page_token: Option<String>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page_size: DEFAULT_PAGE_SIZE,
            offset: 0,
            page_token: None,
        }
    }
}

impl PaginationParams {
    /// Create pagination params with a specific page size.
    /// Page size is clamped to [1, MAX_PAGE_SIZE].
    pub fn with_page_size(mut self, size: usize) -> Self {
        self.page_size = size.clamp(1, MAX_PAGE_SIZE);
        self
    }

    /// Apply one-based page number using the current page size.
    pub fn with_page(mut self, page: usize) -> Self {
        let page = page.max(1);
        self.offset = (page - 1) * self.page_size;
        self
    }

    /// Create pagination params with a cursor token.
    pub fn with_page_token(mut self, token: impl Into<String>) -> Self {
        self.page_token = Some(token.into());
        self
    }

    /// Create pagination from optional limit and offset.
    /// This provides backward compatibility with legacy limit/offset APIs.
    pub fn from_limit_offset(limit: Option<usize>, offset: Option<usize>) -> Self {
        let mut params = Self::default();
        if let Some(size) = limit {
            params = params.with_page_size(size);
        }
        // Note: offset is not directly supported; cursor-based pagination is preferred
        if offset.is_some() {
            tracing::warn!(
                "Offset-based pagination is deprecated. Use cursor-based pagination with page_token."
            );
        }
        params
    }
}

/// Paginated result set for list operations.
/// Includes pagination metadata for API responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginatedResult<T> {
    /// Items in the current page
    pub items: Vec<T>,
    /// Token to fetch the next page, if more results exist
    pub next_page_token: Option<String>,
    /// Total count of items (optional, may be expensive to compute)
    pub total_count: Option<u64>,
    /// Whether there are more results available
    pub has_more: bool,
}

impl<T> PaginatedResult<T> {
    /// Create a paginated result from items, computing has_more from next_page_token.
    pub fn new(items: Vec<T>, next_page_token: Option<String>, total_count: Option<u64>) -> Self {
        let has_more = next_page_token.is_some();
        Self {
            items,
            next_page_token,
            total_count,
            has_more,
        }
    }

    /// Create an empty paginated result.
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            next_page_token: None,
            total_count: Some(0),
            has_more: false,
        }
    }

    /// Map items to a different type.
    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> PaginatedResult<U> {
        PaginatedResult {
            items: self.items.into_iter().map(f).collect(),
            next_page_token: self.next_page_token,
            total_count: self.total_count,
            has_more: self.has_more,
        }
    }
}

// ---------------------------------------------------------------------------
// Query types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentListQuery {
    pub tenant_id: u64,
    pub organization_id: Option<u64>,
    pub owner_user_id: Option<u64>,
    pub include_deleted: bool,
    pub search_query: Option<String>,
    pub pagination: PaginationParams,
}

impl AgentListQuery {
    pub fn for_tenant(tenant_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id: None,
            owner_user_id: None,
            include_deleted: false,
            search_query: None,
            pagination: PaginationParams::default(),
        }
    }

    pub fn for_organization(mut self, organization_id: u64) -> Self {
        self.organization_id = Some(organization_id);
        self
    }

    pub fn for_owner(mut self, owner_user_id: u64) -> Self {
        self.owner_user_id = Some(owner_user_id);
        self
    }

    pub fn with_deleted(mut self) -> Self {
        self.include_deleted = true;
        self
    }

    pub fn with_search(mut self, query: impl Into<String>) -> Self {
        let query = query.into();
        self.search_query = optional_non_blank(query);
        self
    }

    /// Set pagination parameters for the query.
    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }

    /// Set page size for the query.
    pub fn with_page_size(mut self, size: usize) -> Self {
        self.pagination = self.pagination.with_page_size(size);
        self
    }

    /// Set page token for cursor-based pagination.
    pub fn with_page_token(mut self, token: impl Into<String>) -> Self {
        self.pagination = self.pagination.with_page_token(token);
        self
    }
}

/// Query parameters for listing agent sessions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionListQuery {
    pub tenant_id: u64,
    pub agent_id: Option<String>,
    pub owner_user_id: Option<u64>,
    pub status: Option<String>,
    pub include_archived: bool,
    pub pagination: PaginationParams,
}

impl SessionListQuery {
    pub fn for_tenant(tenant_id: u64) -> Self {
        Self {
            tenant_id,
            agent_id: None,
            owner_user_id: None,
            status: None,
            include_archived: false,
            pagination: PaginationParams::default(),
        }
    }

    pub fn for_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    pub fn for_owner(mut self, owner_user_id: u64) -> Self {
        self.owner_user_id = Some(owner_user_id);
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn include_archived(mut self) -> Self {
        self.include_archived = true;
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Query parameters for listing agent messages within a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageListQuery {
    pub tenant_id: u64,
    pub session_id: String,
    pub role: Option<String>,
    pub status: Option<String>,
    pub pagination: PaginationParams,
}

impl MessageListQuery {
    pub fn for_session(tenant_id: u64, session_id: impl Into<String>) -> Self {
        Self {
            tenant_id,
            session_id: session_id.into(),
            role: None,
            status: None,
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Query parameters for listing live interactions within a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionListQuery {
    pub tenant_id: u64,
    pub session_id: String,
    pub status: Option<String>,
    pub pagination: PaginationParams,
}

impl InteractionListQuery {
    pub fn for_session(tenant_id: u64, session_id: impl Into<String>) -> Self {
        Self {
            tenant_id,
            session_id: session_id.into(),
            status: None,
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Thread-safe agent repository port.
///
/// All methods use `&self` — implementations MUST provide interior mutability
/// (e.g. `Mutex`, `RwLock`, or atomic database transactions). This eliminates
/// the need for a global `Mutex<AgentsService>` and enables true concurrent
/// request processing.
pub trait AgentRepository: Send + Sync {
    fn next_id(&self) -> KernelResult<u64>;

    fn insert(&self, record: AgentBusinessRecord) -> KernelResult<()>;

    fn update(&self, record: AgentBusinessRecord) -> KernelResult<()>;

    fn get(&self, tenant_id: u64, agent_id: &str) -> Option<AgentBusinessRecord>;

    /// List agents without pagination (backward compatibility).
    /// For production use, prefer `list_paginated`.
    fn list(&self, query: &AgentListQuery) -> Vec<AgentBusinessRecord>;

    /// List agents with pagination support.
    /// Implements DATABASE_SPEC §16 mandatory pagination requirements.
    /// Default implementation delegates to `list` and applies in-memory pagination.
    fn list_paginated(&self, query: &AgentListQuery) -> PaginatedResult<AgentBusinessRecord> {
        let all_items = self.list(query);
        let page_size = query.pagination.page_size;
        
        // In-memory pagination for default implementation
        // Production implementations should override this with database-level pagination
        let total_count = all_items.len() as u64;
        let items: Vec<AgentBusinessRecord> = all_items
            .into_iter()
            .take(page_size)
            .collect();
        
        let has_more = (items.len() as u64) < total_count;
        let next_page_token = if has_more {
            // Generate cursor based on last item's updated_at and id
            items.last().map(|last| {
                format!("{}_{}", last.updated_at, last.id)
            })
        } else {
            None
        };
        
        PaginatedResult::new(items, next_page_token, Some(total_count))
    }

    fn insert_provider_binding(&self, _record: AgentProviderBindingRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.provider_binding".to_string(),
        })
    }

    fn update_provider_binding(&self, _record: AgentProviderBindingRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.provider_binding".to_string(),
        })
    }

    fn get_provider_binding(
        &self,
        _tenant_id: u64,
        _agent_id: &str,
        _binding_id: &str,
    ) -> Option<AgentProviderBindingRecord> {
        None
    }

    fn list_provider_bindings(
        &self,
        _tenant_id: u64,
        _agent_id: &str,
    ) -> Vec<AgentProviderBindingRecord> {
        Vec::new()
    }

    fn insert_composition_slot(&self, _record: AgentCompositionSlotRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.composition_slot".to_string(),
        })
    }

    fn update_composition_slot(&self, _record: AgentCompositionSlotRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.composition_slot".to_string(),
        })
    }

    fn get_composition_slot(
        &self,
        _tenant_id: u64,
        _agent_id: &str,
        _slot_id: &str,
    ) -> Option<AgentCompositionSlotRecord> {
        None
    }

    fn list_composition_slots(
        &self,
        _tenant_id: u64,
        _agent_id: &str,
    ) -> Vec<AgentCompositionSlotRecord> {
        Vec::new()
    }

    // -----------------------------------------------------------------------
    // Session persistence — default stubs return empty/error for backward
    // compatibility with adapters that have not yet implemented sessions.
    // -----------------------------------------------------------------------

    fn insert_session(&self, _record: AgentSessionRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.session".to_string(),
        })
    }

    fn update_session(&self, _record: AgentSessionRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.session".to_string(),
        })
    }

    fn get_session(
        &self,
        _tenant_id: u64,
        _session_id: &str,
    ) -> Option<AgentSessionRecord> {
        None
    }

    fn list_sessions(&self, _query: &SessionListQuery) -> Vec<AgentSessionRecord> {
        Vec::new()
    }

    // -----------------------------------------------------------------------
    // Message persistence — default stubs return empty/error for backward
    // compatibility with adapters that have not yet implemented messages.
    // -----------------------------------------------------------------------

    fn insert_message(&self, _record: AgentMessageRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.message".to_string(),
        })
    }

    fn update_message(&self, _record: AgentMessageRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.message".to_string(),
        })
    }

    fn get_message(
        &self,
        _tenant_id: u64,
        _session_id: &str,
        _message_id: &str,
    ) -> Option<AgentMessageRecord> {
        None
    }

    fn list_messages(&self, _query: &MessageListQuery) -> Vec<AgentMessageRecord> {
        Vec::new()
    }

    /// Next message sequence number for a session. Implementations should
    /// return `message_count + 1` for the session, or `1` if no messages exist.
    fn next_message_sequence(&self, _tenant_id: u64, _session_id: &str) -> KernelResult<u64> {
        Ok(1)
    }

    // -----------------------------------------------------------------------
    // Interaction persistence — default stubs return empty/error for backward
    // compatibility with adapters that have not yet implemented interactions.
    // -----------------------------------------------------------------------

    fn insert_interaction(&self, _record: AgentInteractionRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.interaction".to_string(),
        })
    }

    fn update_interaction(&self, _record: AgentInteractionRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.interaction".to_string(),
        })
    }

    fn get_interaction(
        &self,
        _tenant_id: u64,
        _session_id: &str,
        _interaction_id: &str,
    ) -> Option<AgentInteractionRecord> {
        None
    }

    fn list_interactions(&self, _query: &InteractionListQuery) -> Vec<AgentInteractionRecord> {
        Vec::new()
    }
}

/// Thread-safe audit event sink port.
///
/// All methods use `&self` for the same reason as `AgentRepository`.
pub trait AgentAuditSink: Send + Sync {
    fn record(&self, event: KernelEvent) -> KernelResult<()>;

    fn list_events(&self, _tenant_id: u64, _agent_id: &str) -> KernelResult<Vec<KernelEvent>> {
        Ok(Vec::new())
    }
}
