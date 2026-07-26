//! Workspace aggregate for grouping user-owned agent projects.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentWorkspaceStatus {
    Active,
    Archived,
    Deleted,
}

impl AgentWorkspaceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
            Self::Deleted => "deleted",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Active => 0,
            Self::Archived => 1,
            Self::Deleted => 2,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Active),
            1 => Some(Self::Archived),
            2 => Some(Self::Deleted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentWorkspaceRecord {
    pub id: u64,
    pub workspace_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub status: AgentWorkspaceStatus,
    pub created_by: u64,
    pub updated_by: u64,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
    pub archived_by: Option<u64>,
    pub deleted_at: Option<String>,
    pub deleted_by: Option<u64>,
    pub retention_until: Option<String>,
}

impl AgentWorkspaceRecord {
    pub fn mark_updated(&mut self, user_id: u64, updated_at: impl Into<String>) {
        self.updated_by = user_id;
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }

    pub fn archive(&mut self, user_id: u64, archived_at: impl Into<String>) {
        let occurred_at = archived_at.into();
        self.status = AgentWorkspaceStatus::Archived;
        self.archived_at = Some(occurred_at.clone());
        self.archived_by = Some(user_id);
        self.mark_updated(user_id, occurred_at);
    }

    pub fn soft_delete(&mut self, user_id: u64, deleted_at: impl Into<String>) {
        let occurred_at = deleted_at.into();
        self.status = AgentWorkspaceStatus::Deleted;
        self.deleted_at = Some(occurred_at.clone());
        self.deleted_by = Some(user_id);
        self.mark_updated(user_id, occurred_at);
    }
}

pub fn default_workspace_id(owner_user_id: u64) -> String {
    format!("workspace.default.{owner_user_id}")
}
