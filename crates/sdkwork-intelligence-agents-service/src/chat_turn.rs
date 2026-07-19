//! Durable idempotency ledger for one commercial chat turn.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentChatTurnStatus {
    Requested,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl AgentChatTurnStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Requested => 0,
            Self::Running => 1,
            Self::Completed => 2,
            Self::Failed => 3,
            Self::Cancelled => 4,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Requested),
            1 => Some(Self::Running),
            2 => Some(Self::Completed),
            3 => Some(Self::Failed),
            4 => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentChatTurnRecord {
    pub id: u64,
    pub turn_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub client_request_id: Option<String>,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub request_message_id: String,
    pub response_message_id: Option<String>,
    pub status: AgentChatTurnStatus,
    pub requested_model_id: Option<String>,
    pub provider_binding_id: Option<String>,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub finish_reason: Option<String>,
    pub error_code: Option<String>,
    pub error_detail: Option<String>,
    pub trace_id: Option<String>,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub cancel_requested_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub retention_until: Option<String>,
}

impl AgentChatTurnRecord {
    pub fn mark_running(&mut self, occurred_at: impl Into<String>) {
        let occurred_at = occurred_at.into();
        self.status = AgentChatTurnStatus::Running;
        self.started_at = Some(occurred_at.clone());
        self.updated_at = occurred_at;
        self.version = self.version.saturating_add(1);
    }

    pub fn mark_failed(
        &mut self,
        error_code: impl Into<String>,
        error_detail: impl Into<String>,
        occurred_at: impl Into<String>,
    ) {
        let occurred_at = occurred_at.into();
        self.status = AgentChatTurnStatus::Failed;
        self.error_code = Some(error_code.into());
        self.error_detail = Some(error_detail.into());
        self.completed_at = Some(occurred_at.clone());
        self.updated_at = occurred_at;
        self.version = self.version.saturating_add(1);
    }

    pub fn mark_completed(&mut self, occurred_at: impl Into<String>) {
        let occurred_at = occurred_at.into();
        self.status = AgentChatTurnStatus::Completed;
        self.completed_at = Some(occurred_at.clone());
        self.updated_at = occurred_at;
        self.version = self.version.saturating_add(1);
    }

    pub fn mark_cancelled(&mut self, occurred_at: impl Into<String>) {
        let occurred_at = occurred_at.into();
        self.status = AgentChatTurnStatus::Cancelled;
        self.cancel_requested_at = Some(occurred_at.clone());
        self.cancelled_at = Some(occurred_at.clone());
        self.completed_at = Some(occurred_at.clone());
        self.updated_at = occurred_at;
        self.version = self.version.saturating_add(1);
    }
}
