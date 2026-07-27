//! Commercial chat project aggregate.

use crate::domain::{AgentCompositionSlotKind, AgentCompositionTargetModule};

pub(crate) fn normalize_project_name(value: &str) -> String {
    value.trim().to_lowercase()
}

pub(crate) fn project_names_equal(left: &str, right: &str) -> bool {
    normalize_project_name(left) == normalize_project_name(right)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentProjectVisibility {
    Private,
    Organization,
    Shared,
}

impl AgentProjectVisibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Organization => "organization",
            Self::Shared => "shared",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Private => 0,
            Self::Organization => 1,
            Self::Shared => 2,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Private),
            1 => Some(Self::Organization),
            2 => Some(Self::Shared),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentProjectStatus {
    Active,
    Archived,
    Deleted,
}

impl AgentProjectStatus {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentProjectDriveAccessMode {
    Disabled,
    OwnerLibrary,
    ExplicitResources,
}

impl AgentProjectDriveAccessMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::OwnerLibrary => "owner_library",
            Self::ExplicitResources => "explicit_resources",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Disabled => 0,
            Self::OwnerLibrary => 1,
            Self::ExplicitResources => 2,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Disabled),
            1 => Some(Self::OwnerLibrary),
            2 => Some(Self::ExplicitResources),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProjectRecord {
    pub id: u64,
    pub project_id: String,
    pub workspace_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub visibility: AgentProjectVisibility,
    pub status: AgentProjectStatus,
    pub drive_access_mode: AgentProjectDriveAccessMode,
    pub default_agent_id: Option<String>,
    pub default_model_id: Option<String>,
    pub import_source_kind: Option<String>,
    pub import_source_ref: Option<String>,
    pub drive_space_id: Option<String>,
    pub drive_root_entry_id: Option<String>,
    pub drive_logical_path: Option<String>,
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

impl AgentProjectRecord {
    pub fn mark_updated(&mut self, user_id: u64, updated_at: impl Into<String>) {
        self.updated_by = user_id;
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }

    pub fn archive(&mut self, user_id: u64, archived_at: impl Into<String>) {
        let occurred_at = archived_at.into();
        self.status = AgentProjectStatus::Archived;
        self.archived_at = Some(occurred_at.clone());
        self.archived_by = Some(user_id);
        self.mark_updated(user_id, occurred_at);
    }

    pub fn soft_delete(&mut self, user_id: u64, deleted_at: impl Into<String>) {
        let occurred_at = deleted_at.into();
        self.status = AgentProjectStatus::Deleted;
        self.deleted_at = Some(occurred_at.clone());
        self.deleted_by = Some(user_id);
        self.mark_updated(user_id, occurred_at);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProjectCompositionSlotRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub slot_id: String,
    pub slot_kind: AgentCompositionSlotKind,
    pub target_module: AgentCompositionTargetModule,
    pub target_ref: String,
    pub target_version_ref: Option<String>,
    pub priority: i32,
    pub enabled: bool,
    pub policy_json: String,
    pub created_by: u64,
    pub updated_by: u64,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub deleted_by: Option<u64>,
    pub retention_until: Option<String>,
}

impl AgentProjectCompositionSlotRecord {
    pub fn mark_updated(&mut self, user_id: u64, updated_at: impl Into<String>) {
        self.updated_by = user_id;
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }

    pub fn soft_delete(&mut self, user_id: u64, deleted_at: impl Into<String>) {
        let deleted_at = deleted_at.into();
        self.deleted_at = Some(deleted_at.clone());
        self.deleted_by = Some(user_id);
        self.mark_updated(user_id, deleted_at);
    }
}
