use crate::domain::{
    AgentBusinessRecord, AgentCompositionSlotRecord, AgentInteractionRecord, AgentMessageRecord,
    AgentProviderBindingRecord, AgentSessionRecord, AgentTaskRecord, AgentVisibility,
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

/// Maximum number of prior session messages loaded into LLM chat context.
pub const CHAT_CONTEXT_MESSAGE_LIMIT: usize = 50;
/// Maximum user message payload accepted on chat and task prompt surfaces.
pub const MAX_CHAT_USER_CONTENT_BYTES: usize = 256 * 1024;

/// Build offset-mode pagination metadata from a repository page and total count.
pub fn offset_paginated_result<T>(
    items: Vec<T>,
    pagination: &PaginationParams,
    total_count: u64,
) -> PaginatedResult<T> {
    let has_more = (pagination.offset + items.len()) < total_count as usize;
    PaginatedResult {
        items,
        next_page_token: None,
        total_count: Some(total_count),
        has_more,
    }
}

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
        let page_size = limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
        let offset = offset.unwrap_or(0);
        let page = offset / page_size + 1;
        Self::default().with_page_size(page_size).with_page(page)
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
    /// When set, restricts results to a single visibility level (for example public marketplace).
    pub visibility: Option<AgentVisibility>,
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
            visibility: None,
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

    /// Restrict list results to one visibility level.
    pub fn with_visibility(mut self, visibility: AgentVisibility) -> Self {
        self.visibility = Some(visibility);
        self
    }

    /// Restrict list results to publicly visible agents (marketplace scope).
    pub fn with_public_visibility_only(mut self) -> Self {
        self.visibility = Some(AgentVisibility::Public);
        self
    }
}

/// Query parameters for listing provider bindings under one agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderBindingListQuery {
    pub tenant_id: u64,
    pub agent_id: String,
    pub pagination: PaginationParams,
}

impl ProviderBindingListQuery {
    pub fn for_agent(tenant_id: u64, agent_id: impl Into<String>) -> Self {
        Self {
            tenant_id,
            agent_id: agent_id.into(),
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Query parameters for listing composition slots under one agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositionSlotListQuery {
    pub tenant_id: u64,
    pub agent_id: String,
    pub pagination: PaginationParams,
}

impl CompositionSlotListQuery {
    pub fn for_agent(tenant_id: u64, agent_id: impl Into<String>) -> Self {
        Self {
            tenant_id,
            agent_id: agent_id.into(),
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Query parameters for listing persisted audit events for one agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEventListQuery {
    pub tenant_id: u64,
    pub agent_id: String,
    pub action: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub pagination: PaginationParams,
}

impl AuditEventListQuery {
    pub fn for_agent(tenant_id: u64, agent_id: impl Into<String>) -> Self {
        Self {
            tenant_id,
            agent_id: agent_id.into(),
            action: None,
            from: None,
            to: None,
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = optional_non_blank(action.into());
        self
    }

    pub fn with_from(mut self, from: impl Into<String>) -> Self {
        self.from = optional_non_blank(from.into());
        self
    }

    pub fn with_to(mut self, to: impl Into<String>) -> Self {
        self.to = optional_non_blank(to.into());
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Query parameters for listing MCP marketplace projection rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpMarketplaceListQuery {
    pub tenant_id: u64,
    pub q: Option<String>,
    pub pagination: PaginationParams,
}

impl McpMarketplaceListQuery {
    pub fn for_tenant(tenant_id: u64) -> Self {
        Self {
            tenant_id,
            q: None,
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_q(mut self, q: impl Into<String>) -> Self {
        let value = q.into().trim().to_string();
        self.q = if value.is_empty() { None } else { Some(value) };
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Message list sort order. Default API lists use ascending sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MessageListSort {
    #[default]
    SequenceAsc,
    /// Recent context window for chat completion (descending sequence, bounded limit).
    RecentContextDesc,
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
    pub sort: MessageListSort,
    pub pagination: PaginationParams,
}

impl MessageListQuery {
    pub fn for_session(tenant_id: u64, session_id: impl Into<String>) -> Self {
        Self {
            tenant_id,
            session_id: session_id.into(),
            role: None,
            status: None,
            sort: MessageListSort::default(),
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

    /// Load the most recent messages for chat completion context.
    pub fn for_recent_chat_context(
        tenant_id: u64,
        session_id: impl Into<String>,
        limit: usize,
    ) -> Self {
        Self {
            tenant_id,
            session_id: session_id.into(),
            role: None,
            status: None,
            sort: MessageListSort::RecentContextDesc,
            pagination: PaginationParams::default().with_page_size(limit),
        }
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

/// Query parameters for listing agent tasks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskListQuery {
    pub tenant_id: u64,
    pub agent_id: Option<String>,
    pub owner_user_id: Option<u64>,
    pub status: Option<String>,
    pub pagination: PaginationParams,
}

impl TaskListQuery {
    pub fn for_tenant(tenant_id: u64) -> Self {
        Self {
            tenant_id,
            agent_id: None,
            owner_user_id: None,
            status: None,
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

    /// List one page of agents. Pagination is enforced at the repository layer.
    fn list(&self, query: &AgentListQuery) -> Vec<AgentBusinessRecord>;

    /// Count agents matching list filters (excluding pagination bounds).
    fn count_agents(&self, query: &AgentListQuery) -> u64;

    /// List agents with pagination metadata.
    fn list_paginated(&self, query: &AgentListQuery) -> PaginatedResult<AgentBusinessRecord> {
        let total_count = self.count_agents(query);
        let items = self.list(query);
        offset_paginated_result(items, &query.pagination, total_count)
    }

    fn insert_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()>;

    fn update_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()>;

    fn get_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
    ) -> Option<AgentProviderBindingRecord>;

    fn list_provider_bindings(
        &self,
        query: &ProviderBindingListQuery,
    ) -> Vec<AgentProviderBindingRecord>;

    fn count_provider_bindings(&self, query: &ProviderBindingListQuery) -> u64;

    /// Atomically deactivate all active bindings for an agent and activate one target binding.
    fn activate_provider_binding_atomic(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
        updated_at: String,
    ) -> KernelResult<AgentProviderBindingRecord> {
        let mut record = self
            .get_provider_binding(tenant_id, agent_id, binding_id)
            .ok_or_else(|| KernelError::validation("agent provider binding not found"))?;
        if record.active {
            return Ok(record);
        }
        let mut page = 1usize;
        loop {
            let batch = self.list_provider_bindings(
                &ProviderBindingListQuery::for_agent(tenant_id, agent_id).with_pagination(
                    PaginationParams::default()
                        .with_page_size(MAX_PAGE_SIZE)
                        .with_page(page),
                ),
            );
            if batch.is_empty() {
                break;
            }
            let batch_len = batch.len();
            for mut binding in batch {
                if binding.active {
                    binding.active = false;
                    binding.mark_updated(updated_at.clone());
                    self.update_provider_binding(binding)?;
                }
            }
            if batch_len < MAX_PAGE_SIZE {
                break;
            }
            page = page.saturating_add(1);
        }
        record = self
            .get_provider_binding(tenant_id, agent_id, binding_id)
            .ok_or_else(|| KernelError::validation("agent provider binding not found"))?;
        record.active = true;
        record.mark_updated(updated_at);
        self.update_provider_binding(record.clone())?;
        Ok(record)
    }

    fn insert_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()>;

    fn update_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()>;

    fn get_composition_slot(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str,
    ) -> Option<AgentCompositionSlotRecord>;

    fn list_composition_slots(
        &self,
        query: &CompositionSlotListQuery,
    ) -> Vec<AgentCompositionSlotRecord>;

    fn count_composition_slots(&self, query: &CompositionSlotListQuery) -> u64;

    fn list_mcp_marketplace_slots(
        &self,
        query: &McpMarketplaceListQuery,
    ) -> Vec<AgentCompositionSlotRecord>;

    fn count_mcp_marketplace_slots(&self, query: &McpMarketplaceListQuery) -> u64;

    // -----------------------------------------------------------------------
    // Session persistence
    // -----------------------------------------------------------------------

    fn insert_session(&self, record: AgentSessionRecord) -> KernelResult<()>;

    fn update_session(&self, record: AgentSessionRecord) -> KernelResult<()>;

    fn get_session(&self, tenant_id: u64, session_id: &str) -> Option<AgentSessionRecord>;

    fn list_sessions(&self, query: &SessionListQuery) -> Vec<AgentSessionRecord>;

    fn count_sessions(&self, query: &SessionListQuery) -> u64;

    // -----------------------------------------------------------------------
    // Message persistence
    // -----------------------------------------------------------------------

    fn insert_message(&self, record: AgentMessageRecord) -> KernelResult<()>;

    fn update_message(&self, record: AgentMessageRecord) -> KernelResult<()>;

    fn get_message(
        &self,
        tenant_id: u64,
        session_id: &str,
        message_id: &str,
    ) -> Option<AgentMessageRecord>;

    fn list_messages(&self, query: &MessageListQuery) -> Vec<AgentMessageRecord>;

    fn count_messages(&self, query: &MessageListQuery) -> u64;

    /// Next message sequence number for a session. Implementations should
    /// return `message_count + 1` for the session, or `1` if no messages exist.
    fn next_message_sequence(&self, tenant_id: u64, session_id: &str) -> KernelResult<u64>;

    /// Atomically persist one user + assistant chat turn and update session counters.
    fn insert_chat_turn(
        &self,
        session: AgentSessionRecord,
        mut user_message: AgentMessageRecord,
        mut assistant_message: AgentMessageRecord,
    ) -> KernelResult<(AgentSessionRecord, AgentMessageRecord, AgentMessageRecord)> {
        let user_sequence =
            self.next_message_sequence(user_message.tenant_id, user_message.session_id.as_str())?;
        user_message.sequence = user_sequence;
        self.insert_message(user_message.clone())?;

        let assistant_sequence = self.next_message_sequence(
            assistant_message.tenant_id,
            assistant_message.session_id.as_str(),
        )?;
        assistant_message.sequence = assistant_sequence;
        self.insert_message(assistant_message.clone())?;

        self.update_session(session.clone())?;
        Ok((session, user_message, assistant_message))
    }

    // -----------------------------------------------------------------------
    // Interaction persistence
    // -----------------------------------------------------------------------

    fn insert_interaction(&self, record: AgentInteractionRecord) -> KernelResult<()>;

    fn update_interaction(&self, record: AgentInteractionRecord) -> KernelResult<()>;

    fn get_interaction(
        &self,
        tenant_id: u64,
        session_id: &str,
        interaction_id: &str,
    ) -> Option<AgentInteractionRecord>;

    fn list_interactions(&self, query: &InteractionListQuery) -> Vec<AgentInteractionRecord>;

    fn count_interactions(&self, query: &InteractionListQuery) -> u64;

    // -----------------------------------------------------------------------
    // Task persistence
    // -----------------------------------------------------------------------

    fn insert_task(&self, record: AgentTaskRecord) -> KernelResult<()>;

    fn update_task(&self, record: AgentTaskRecord) -> KernelResult<()>;

    fn get_task(&self, tenant_id: u64, task_id: &str) -> Option<AgentTaskRecord>;

    fn list_tasks(&self, query: &TaskListQuery) -> Vec<AgentTaskRecord>;

    fn count_tasks(&self, query: &TaskListQuery) -> u64;
}

/// Thread-safe audit event sink port.
///
/// All methods use `&self` for the same reason as `AgentRepository`.
pub trait AgentAuditSink: Send + Sync {
    fn record(&self, event: KernelEvent) -> KernelResult<()>;

    fn list_events(
        &self,
        query: &AuditEventListQuery,
    ) -> KernelResult<PaginatedResult<KernelEvent>>;
}
