use crate::agent_turn::AgentTurnRecord;
use crate::domain::{
    AgentBusinessRecord, AgentCompositionSlotKind, AgentCompositionSlotRecord,
    AgentInteractionRecord, AgentItemDriveRefRecord, AgentItemFeedbackRecord,
    AgentProviderBindingRecord, AgentResourceType, AgentResourceUserStateRecord,
    AgentSessionCheckpointRecord, AgentSessionItemRecord, AgentSessionRecord,
    AgentSessionRuntimeBindingRecord, AgentTaskRecord, AgentVisibility,
};
use crate::project::{AgentProjectCompositionSlotRecord, AgentProjectRecord, AgentProjectStatus};
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

/// Maximum number of prior session items loaded into one turn context.
pub const TURN_CONTEXT_ITEM_LIMIT: usize = 50;
/// Maximum user-input payload accepted on turn and task prompt surfaces.
pub const MAX_TURN_INPUT_CONTENT_BYTES: usize = 256 * 1024;

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

/// Query parameters for listing MCP marketplace catalog entries.
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
pub enum SessionItemListSort {
    #[default]
    SequenceAsc,
    /// Public list order for newest items first, with normal offset pagination.
    SequenceDesc,
    /// Recent context window for turn execution (descending sequence, bounded limit).
    RecentContextDesc,
}

/// Query parameters for listing agent sessions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionListQuery {
    pub tenant_id: u64,
    pub organization_id: Option<u64>,
    pub agent_id: Option<String>,
    pub project_id: Option<String>,
    pub owner_user_id: Option<u64>,
    pub status: Option<String>,
    pub include_archived: bool,
    pub pagination: PaginationParams,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: Option<u64>,
    pub status: Option<AgentProjectStatus>,
    pub search_query: Option<String>,
    pub include_deleted: bool,
    pub pagination: PaginationParams,
}

impl ProjectListQuery {
    pub fn for_organization(tenant_id: u64, organization_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id,
            owner_user_id: None,
            status: None,
            search_query: None,
            include_deleted: false,
            pagination: PaginationParams::default(),
        }
    }

    pub fn for_owner(mut self, owner_user_id: u64) -> Self {
        self.owner_user_id = Some(owner_user_id);
        self
    }

    pub fn with_status(mut self, status: AgentProjectStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_search(mut self, search_query: impl Into<String>) -> Self {
        self.search_query = optional_non_blank(search_query.into());
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCompositionSlotListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub slot_kind: Option<AgentCompositionSlotKind>,
    pub enabled: Option<bool>,
    pub pagination: PaginationParams,
}

impl ProjectCompositionSlotListQuery {
    pub fn for_project(
        tenant_id: u64,
        organization_id: u64,
        project_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            project_id: project_id.into(),
            slot_kind: None,
            enabled: None,
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

impl SessionListQuery {
    pub fn for_tenant(tenant_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id: None,
            agent_id: None,
            project_id: None,
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

    pub fn for_organization(mut self, organization_id: u64) -> Self {
        self.organization_id = Some(organization_id);
        self
    }

    pub fn for_project(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceUserStateListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub resource_type: AgentResourceType,
    pub agent_id: Option<String>,
    pub pinned_only: bool,
    pub include_hidden: bool,
    pub pagination: PaginationParams,
}

impl ResourceUserStateListQuery {
    pub fn for_user_sessions(tenant_id: u64, organization_id: u64, user_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id,
            user_id,
            resource_type: AgentResourceType::Session,
            agent_id: None,
            pinned_only: false,
            include_hidden: false,
            pagination: PaginationParams::default(),
        }
    }

    pub fn pinned_only(mut self) -> Self {
        self.pinned_only = true;
        self
    }

    pub fn for_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

/// Query parameters for listing agent items within a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionItemListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub sort: SessionItemListSort,
    pub pagination: PaginationParams,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemFeedbackListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub session_id: String,
    pub pagination: PaginationParams,
}

impl ItemFeedbackListQuery {
    pub fn for_user_session(
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            user_id,
            session_id: session_id.into(),
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

impl SessionItemListQuery {
    pub fn for_session(
        tenant_id: u64,
        organization_id: u64,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            session_id: session_id.into(),
            kind: None,
            status: None,
            sort: SessionItemListSort::default(),
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = Some(kind.into());
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

    pub fn with_sort(mut self, sort: SessionItemListSort) -> Self {
        self.sort = sort;
        self
    }

    /// Load the most recent items for turn execution context.
    pub fn for_recent_turn_context(
        tenant_id: u64,
        organization_id: u64,
        session_id: impl Into<String>,
        limit: usize,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            session_id: session_id.into(),
            kind: None,
            status: None,
            sort: SessionItemListSort::RecentContextDesc,
            pagination: PaginationParams::default().with_page_size(limit),
        }
    }
}

/// Query parameters for listing live interactions within a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub pagination: PaginationParams,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRuntimeBindingListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub status: Option<String>,
    pub current_only: bool,
    pub pagination: PaginationParams,
}

impl SessionRuntimeBindingListQuery {
    pub fn for_session(
        tenant_id: u64,
        organization_id: u64,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            session_id: session_id.into(),
            status: None,
            current_only: false,
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn current_only(mut self) -> Self {
        self.current_only = true;
        self
    }

    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCheckpointListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub status: Option<String>,
    pub pagination: PaginationParams,
}

impl SessionCheckpointListQuery {
    pub fn for_session(
        tenant_id: u64,
        organization_id: u64,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
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

impl InteractionListQuery {
    pub fn for_session(
        tenant_id: u64,
        organization_id: u64,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            session_id: session_id.into(),
            kind: None,
            status: None,
            pagination: PaginationParams::default(),
        }
    }

    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = Some(kind.into());
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub status: Option<String>,
    pub pagination: PaginationParams,
}

impl TurnListQuery {
    pub fn for_session(
        tenant_id: u64,
        organization_id: u64,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnRequestWriteOutcome {
    Inserted {
        session: Box<AgentSessionRecord>,
        request_item: Box<AgentSessionItemRecord>,
    },
    Existing(Box<AgentTurnRecord>),
}

/// Thread-safe agent repository port.
///
/// All methods use `&self` — implementations MUST provide interior mutability
/// (e.g. `Mutex`, `RwLock`, or atomic database transactions). This eliminates
/// the need for a global `Mutex<AgentsService>` and enables true concurrent
/// request processing.
pub trait AgentRepository: Send + Sync {
    /// Verify that the repository's required backing store can serve requests.
    fn check_readiness(&self) -> KernelResult<()>;

    fn next_id(&self) -> KernelResult<u64>;

    fn insert(&self, record: AgentBusinessRecord) -> KernelResult<()>;

    fn update(&self, record: AgentBusinessRecord) -> KernelResult<()>;

    fn get(&self, tenant_id: u64, agent_id: &str) -> KernelResult<Option<AgentBusinessRecord>>;

    /// List one page of agents. Pagination is enforced at the repository layer.
    fn list(&self, query: &AgentListQuery) -> KernelResult<Vec<AgentBusinessRecord>>;

    /// Count agents matching list filters (excluding pagination bounds).
    fn count_agents(&self, query: &AgentListQuery) -> KernelResult<u64>;

    fn insert_project(&self, record: AgentProjectRecord) -> KernelResult<()>;

    fn update_project(&self, record: AgentProjectRecord) -> KernelResult<()>;

    fn get_project(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
    ) -> KernelResult<Option<AgentProjectRecord>>;

    fn list_projects(&self, query: &ProjectListQuery) -> KernelResult<Vec<AgentProjectRecord>>;

    fn count_projects(&self, query: &ProjectListQuery) -> KernelResult<u64>;

    fn insert_project_composition_slot(
        &self,
        record: AgentProjectCompositionSlotRecord,
    ) -> KernelResult<()>;

    fn update_project_composition_slot(
        &self,
        record: AgentProjectCompositionSlotRecord,
    ) -> KernelResult<()>;

    fn get_project_composition_slot(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentProjectCompositionSlotRecord>>;

    fn list_project_composition_slots(
        &self,
        query: &ProjectCompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentProjectCompositionSlotRecord>>;

    fn count_project_composition_slots(
        &self,
        query: &ProjectCompositionSlotListQuery,
    ) -> KernelResult<u64>;

    /// List agents with pagination metadata.
    fn list_paginated(
        &self,
        query: &AgentListQuery,
    ) -> KernelResult<PaginatedResult<AgentBusinessRecord>> {
        let total_count = self.count_agents(query)?;
        let items = self.list(query)?;
        Ok(offset_paginated_result(
            items,
            &query.pagination,
            total_count,
        ))
    }

    fn insert_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()>;

    fn update_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()>;

    fn get_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRecord>>;

    /// Load the single active provider binding for an agent.
    ///
    /// Implementations SHOULD override this with a dedicated indexed query
    /// (`WHERE active = TRUE LIMIT 1`) to avoid paginated full scans on hot
    /// paths such as turn execution and task execution. The default
    /// implementation falls back to a paginated scan for backward compatibility.
    fn get_active_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRecord>>;

    fn list_provider_bindings(
        &self,
        query: &ProviderBindingListQuery,
    ) -> KernelResult<Vec<AgentProviderBindingRecord>>;

    fn count_provider_bindings(&self, query: &ProviderBindingListQuery) -> KernelResult<u64>;

    /// Atomically deactivate all active bindings for an agent and activate one target binding.
    fn activate_provider_binding_atomic(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
        updated_at: String,
    ) -> KernelResult<AgentProviderBindingRecord> {
        let mut record = self
            .get_provider_binding(tenant_id, agent_id, binding_id)?
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
            )?;
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
            .get_provider_binding(tenant_id, agent_id, binding_id)?
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
    ) -> KernelResult<Option<AgentCompositionSlotRecord>>;

    fn list_composition_slots(
        &self,
        query: &CompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>>;

    fn count_composition_slots(&self, query: &CompositionSlotListQuery) -> KernelResult<u64>;

    fn list_mcp_marketplace_slots(
        &self,
        query: &McpMarketplaceListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>>;

    fn count_mcp_marketplace_slots(&self, query: &McpMarketplaceListQuery) -> KernelResult<u64>;

    // -----------------------------------------------------------------------
    // Session persistence
    // -----------------------------------------------------------------------

    fn insert_session(&self, record: AgentSessionRecord) -> KernelResult<()>;

    fn update_session(&self, record: AgentSessionRecord) -> KernelResult<()>;

    fn get_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRecord>>;

    fn list_sessions(&self, query: &SessionListQuery) -> KernelResult<Vec<AgentSessionRecord>>;

    fn count_sessions(&self, query: &SessionListQuery) -> KernelResult<u64>;

    fn insert_session_runtime_binding(
        &self,
        record: AgentSessionRuntimeBindingRecord,
    ) -> KernelResult<()>;

    fn update_session_runtime_binding(
        &self,
        record: AgentSessionRuntimeBindingRecord,
    ) -> KernelResult<()>;

    fn get_session_runtime_binding(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRecord>>;

    fn get_current_session_runtime_binding(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRecord>>;

    fn list_session_runtime_bindings(
        &self,
        query: &SessionRuntimeBindingListQuery,
    ) -> KernelResult<Vec<AgentSessionRuntimeBindingRecord>>;

    fn count_session_runtime_bindings(
        &self,
        query: &SessionRuntimeBindingListQuery,
    ) -> KernelResult<u64>;

    /// Switches the current runtime binding under one repository transaction.
    fn activate_session_runtime_binding_atomic(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
        expected_version: u64,
        updated_at: String,
    ) -> KernelResult<AgentSessionRuntimeBindingRecord>;

    fn insert_session_checkpoint(&self, record: AgentSessionCheckpointRecord) -> KernelResult<()>;

    fn update_session_checkpoint(&self, record: AgentSessionCheckpointRecord) -> KernelResult<()>;

    fn get_session_checkpoint(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        checkpoint_id: &str,
    ) -> KernelResult<Option<AgentSessionCheckpointRecord>>;

    fn list_session_checkpoints(
        &self,
        query: &SessionCheckpointListQuery,
    ) -> KernelResult<Vec<AgentSessionCheckpointRecord>>;

    fn count_session_checkpoints(&self, query: &SessionCheckpointListQuery) -> KernelResult<u64>;

    fn upsert_resource_user_state(
        &self,
        record: AgentResourceUserStateRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentResourceUserStateRecord>;

    fn get_resource_user_state(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        resource_type: AgentResourceType,
        resource_id: &str,
    ) -> KernelResult<Option<AgentResourceUserStateRecord>>;

    fn list_resource_user_states(
        &self,
        query: &ResourceUserStateListQuery,
    ) -> KernelResult<Vec<AgentResourceUserStateRecord>>;

    fn count_resource_user_states(&self, query: &ResourceUserStateListQuery) -> KernelResult<u64>;

    // -----------------------------------------------------------------------
    // Session-item persistence
    // -----------------------------------------------------------------------

    /// Atomically append one item and update its owning session counters.
    fn append_session_item(
        &self,
        record: AgentSessionItemRecord,
    ) -> KernelResult<(AgentSessionRecord, AgentSessionItemRecord)>;

    fn update_session_item(&self, record: AgentSessionItemRecord) -> KernelResult<()>;

    fn get_session_item(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        item_id: &str,
    ) -> KernelResult<Option<AgentSessionItemRecord>>;

    fn list_session_items(
        &self,
        query: &SessionItemListQuery,
    ) -> KernelResult<Vec<AgentSessionItemRecord>>;

    fn count_session_items(&self, query: &SessionItemListQuery) -> KernelResult<u64>;

    fn upsert_item_feedback(
        &self,
        record: AgentItemFeedbackRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentItemFeedbackRecord>;

    fn get_item_feedback(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
        user_id: u64,
        include_deleted: bool,
    ) -> KernelResult<Option<AgentItemFeedbackRecord>>;

    fn list_item_feedback(
        &self,
        query: &ItemFeedbackListQuery,
    ) -> KernelResult<Vec<AgentItemFeedbackRecord>>;

    fn count_item_feedback(&self, query: &ItemFeedbackListQuery) -> KernelResult<u64>;

    fn get_turn_by_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentTurnRecord>>;
    fn get_turn(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<Option<AgentTurnRecord>>;

    fn list_turns(&self, query: &TurnListQuery) -> KernelResult<Vec<AgentTurnRecord>>;

    fn count_turns(&self, query: &TurnListQuery) -> KernelResult<u64>;
    fn list_reconcilable_turns(
        &self,
        stale_before: &str,
        limit: usize,
    ) -> KernelResult<Vec<AgentTurnRecord>>;

    /// Atomically persist a requested turn, its user-input item, any Drive
    /// references, and the corresponding session counter update.
    fn insert_turn_request(
        &self,
        turn: AgentTurnRecord,
        request_item: AgentSessionItemRecord,
        drive_refs: Vec<AgentItemDriveRefRecord>,
    ) -> KernelResult<TurnRequestWriteOutcome>;

    fn update_turn_state(
        &self,
        turn: AgentTurnRecord,
        expected_version: u64,
    ) -> KernelResult<AgentTurnRecord>;

    /// Atomically persist a turn response item, transition the turn to its
    /// completed state, and update the corresponding session counters.
    fn complete_turn(
        &self,
        turn: AgentTurnRecord,
        expected_turn_version: u64,
        expected_fencing_token: u64,
        expected_lease_token: Option<String>,
        response_item: AgentSessionItemRecord,
    ) -> KernelResult<(AgentSessionRecord, AgentSessionItemRecord)>;

    fn list_item_drive_refs(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
    ) -> KernelResult<Vec<AgentItemDriveRefRecord>>;

    fn list_item_drive_refs_batch(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_ids: &[String],
    ) -> KernelResult<Vec<AgentItemDriveRefRecord>> {
        let mut records = Vec::new();
        for item_id in item_ids {
            records.extend(self.list_item_drive_refs(tenant_id, organization_id, item_id)?);
        }
        Ok(records)
    }

    // -----------------------------------------------------------------------
    // Interaction persistence
    // -----------------------------------------------------------------------

    fn insert_interaction(&self, record: AgentInteractionRecord) -> KernelResult<()>;

    fn update_interaction(&self, record: AgentInteractionRecord) -> KernelResult<()>;

    fn get_interaction(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        interaction_id: &str,
    ) -> KernelResult<Option<AgentInteractionRecord>>;

    fn list_interactions(
        &self,
        query: &InteractionListQuery,
    ) -> KernelResult<Vec<AgentInteractionRecord>>;

    fn count_interactions(&self, query: &InteractionListQuery) -> KernelResult<u64>;

    // -----------------------------------------------------------------------
    // Task persistence
    // -----------------------------------------------------------------------

    fn insert_task(&self, record: AgentTaskRecord) -> KernelResult<()>;

    fn update_task(&self, record: AgentTaskRecord) -> KernelResult<()>;

    fn get_task(&self, tenant_id: u64, task_id: &str) -> KernelResult<Option<AgentTaskRecord>>;

    fn list_tasks(&self, query: &TaskListQuery) -> KernelResult<Vec<AgentTaskRecord>>;

    fn count_tasks(&self, query: &TaskListQuery) -> KernelResult<u64>;
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
