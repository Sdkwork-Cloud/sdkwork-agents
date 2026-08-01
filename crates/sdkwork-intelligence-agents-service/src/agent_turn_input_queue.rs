//! Durable, session-scoped user inputs waiting to become authoritative Turns.

use crate::agent_turn::AgentTurnMode;
use crate::domain::AgentItemResourceRole;

pub const MAX_TURN_INPUT_QUEUE_ENTRIES_PER_SESSION: usize = 32;
pub const MAX_TURN_INPUT_QUEUE_CONTENT_BYTES_PER_SESSION: usize = 4 * 1024 * 1024;
pub const MAX_TURN_INPUT_QUEUE_DRIVE_REFS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTurnInputQueueStatus {
    Queued,
    Executing,
    Failed,
}

impl AgentTurnInputQueueStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Executing => "executing",
            Self::Failed => "failed",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Queued => 0,
            Self::Executing => 1,
            Self::Failed => 2,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Queued),
            1 => Some(Self::Executing),
            2 => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTurnInputQueueDriveRef {
    pub resource_role: AgentItemResourceRole,
    pub drive_space_id: String,
    pub drive_node_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTurnInputQueueEntry {
    pub id: u64,
    pub queue_entry_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub content: String,
    pub display_text: String,
    pub content_type: String,
    pub attachment_names: Vec<String>,
    pub drive_refs: Vec<AgentTurnInputQueueDriveRef>,
    pub turn_mode: AgentTurnMode,
    pub runtime_binding_id: Option<String>,
    pub requested_model_id: Option<String>,
    pub access_mode_id: Option<String>,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub client_request_id: String,
    pub position: u64,
    pub status: AgentTurnInputQueueStatus,
    pub claim_owner: Option<String>,
    pub claim_token_hash: Option<String>,
    pub claim_expires_at: Option<String>,
    pub fencing_token: u64,
    pub error_code: Option<String>,
    pub error_detail: Option<String>,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub claimed_at: Option<String>,
    pub failed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnInputQueueListQuery {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub pagination: crate::ports::PaginationParams,
}

impl TurnInputQueueListQuery {
    pub fn for_session(
        tenant_id: u64,
        organization_id: u64,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            session_id: session_id.into(),
            pagination: crate::ports::PaginationParams::default(),
        }
    }

    pub fn with_pagination(mut self, pagination: crate::ports::PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnInputQueueClaimRequest {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub owner_user_id: u64,
    pub claim_owner: String,
    pub claim_token_hash: String,
    pub claim_expires_at: String,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnInputQueueFailureRequest {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub owner_user_id: u64,
    pub queue_entry_id: String,
    pub expected_version: u64,
    pub expected_fencing_token: u64,
    pub claim_token_hash: String,
    pub error_code: String,
    pub error_detail: Option<String>,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnInputQueueClaimOutcome {
    Claimed(AgentTurnInputQueueEntry),
    Busy(AgentTurnInputQueueEntry),
    Blocked(AgentTurnInputQueueEntry),
    ActiveTurn,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnInputQueueReorderEntry {
    pub queue_entry_id: String,
    pub expected_version: u64,
}
