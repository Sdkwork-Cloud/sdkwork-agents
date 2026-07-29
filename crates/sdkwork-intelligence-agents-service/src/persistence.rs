use crate::agent_turn::{AgentTurnMode, AgentTurnRecord, AgentTurnStatus};
use crate::domain::{
    AgentBusinessRecord, AgentBusinessStatus, AgentCompositionSlotKind, AgentCompositionSlotRecord,
    AgentCompositionTargetModule, AgentImplementationKind, AgentImplementationType,
    AgentInteractionKind, AgentInteractionRecord, AgentInteractionStatus, AgentItemDriveRefRecord,
    AgentItemFeedbackRating, AgentItemFeedbackRecord, AgentItemResourceRole,
    AgentProviderBindingRecord, AgentResourceType, AgentResourceUserStateRecord,
    AgentSessionCheckpointRecord, AgentSessionCheckpointStatus, AgentSessionEntrySurface,
    AgentSessionItemKind, AgentSessionItemRecord, AgentSessionItemStatus, AgentSessionKind,
    AgentSessionRecord, AgentSessionRuntimeBindingRecord, AgentSessionRuntimeBindingStatus,
    AgentSessionStatus, AgentSessionTitleSource, AgentTaskRecord, AgentTaskStatus, AgentVisibility,
};
use crate::ports::{
    AgentAuditSink, AgentListQuery, AgentRepository, AuditEventListQuery, CompositionSlotListQuery,
    InteractionListQuery, ItemFeedbackListQuery, McpMarketplaceListQuery,
    ProjectCompositionSlotListQuery, ProjectListQuery, ProviderBindingListQuery,
    ResourceUserStateListQuery, SessionActivitySummaryListQuery, SessionCheckpointListQuery,
    SessionItemListQuery, SessionItemListSort, SessionListQuery, SessionRuntimeBindingListQuery,
    TaskListQuery, TurnListQuery, TurnRequestWriteOutcome, WorkspaceListQuery,
};
#[cfg(feature = "postgres-sync")]
use crate::postgres_sync_pool::{BlockingPostgresPool, PgRow};
use crate::project::{
    project_names_equal, AgentProjectCompositionSlotRecord, AgentProjectDriveAccessMode,
    AgentProjectRecord, AgentProjectStatus, AgentProjectVisibility,
};
use crate::session_activity::{
    encode_session_activity_cursor, SessionActivityCursor, SessionActivitySource,
    SessionActivitySummaryParts, SessionActivitySummaryRecord,
};
use crate::validation::{validate_capabilities, validate_standard_id};
use crate::workspace::{AgentWorkspaceRecord, AgentWorkspaceStatus};
#[cfg(feature = "postgres-sync")]
use crate::{pg_execute, pg_query, pg_query_optional};
use sdkwork_agent_kernel::{
    AgentManifest, KernelError, KernelErrorKind, KernelEvent, KernelEventSeverity,
    KernelEventSource, KernelResult,
};
use sdkwork_code_kernel::CodeTaskIntent;
use sdkwork_utils_rust::{is_blank, sha256_hash, trim};
use serde::{Deserialize, Serialize};
#[cfg(feature = "postgres-sync")]
use sqlx::Row;
#[cfg(feature = "postgres-sync")]
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[cfg(feature = "postgres-sync")]
use std::future::Future;

/// Maximum number of retries for a retryable PostgreSQL transaction failure.
#[cfg(feature = "postgres-sync")]
const TRANSACTION_MAX_RETRIES: usize = 3;

/// Initial backoff (milliseconds) before the first transaction retry.
/// Backoff doubles on each retry: 10 ms, 20 ms, 40 ms.
#[cfg(feature = "postgres-sync")]
const TRANSACTION_INITIAL_BACKOFF_MS: u64 = 10;

/// Returns true for PostgreSQL transaction failures that are safe to retry
/// after the failed transaction has been rolled back.
#[cfg(feature = "postgres-sync")]
fn is_retryable_postgres_transaction_error(error: &sqlx::Error) -> bool {
    matches!(
        error,
        sqlx::Error::Database(db)
            if db
                .code()
                .is_some_and(|code| code == "40001" || code == "40P01")
    )
}

/// Executes `operation` and retries serialization/deadlock failures using
/// bounded exponential backoff with jitter.
///
/// The closure must create a fresh transaction on each call — any state
/// mutated inside the transaction body must be cloned from captured
/// references so that a retry starts from a clean snapshot.
#[cfg(feature = "postgres-sync")]
async fn retry_postgres_transaction<T, F, Fut>(operation: F) -> Result<T, sqlx::Error>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, sqlx::Error>>,
{
    let mut backoff_ms = TRANSACTION_INITIAL_BACKOFF_MS;
    let mut last_error: Option<sqlx::Error> = None;
    for attempt in 0..=TRANSACTION_MAX_RETRIES {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                if !is_retryable_postgres_transaction_error(&error) {
                    return Err(error);
                }
                let jitter_bound = (backoff_ms / 2).max(1);
                let jitter_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|duration| u64::from(duration.subsec_nanos()) % jitter_bound)
                    .unwrap_or(0);
                let delay_ms = backoff_ms.saturating_add(jitter_ms);
                tracing::warn!(
                    target: "sdkwork.agents.persistence.transaction_retry",
                    attempt,
                    max_retries = TRANSACTION_MAX_RETRIES,
                    delay_ms,
                    "retryable postgres transaction failure detected"
                );
                last_error = Some(error);
                if attempt < TRANSACTION_MAX_RETRIES {
                    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                    backoff_ms = backoff_ms.saturating_mul(2);
                }
            }
        }
    }
    Err(last_error.expect("transaction retry loop exhausted without an error"))
}

#[cfg(feature = "postgres-sync")]
use crate::id::{AgentBusinessIdGenerator, AgentIdGenerator};
mod sql;

pub use sql::{
    SQL_ACTIVATE_AGENT_PROVIDER_BINDING, SQL_COUNT_AGENT, SQL_COUNT_AGENT_COMPOSITION_SLOTS,
    SQL_COUNT_AGENT_PROVIDER_BINDINGS, SQL_COUNT_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID,
    SQL_COUNT_MCP_MARKETPLACE_SLOTS, SQL_DEACTIVATE_ACTIVE_AGENT_PROVIDER_BINDINGS,
    SQL_INSERT_AGENT, SQL_INSERT_AGENT_COMPOSITION_SLOT, SQL_INSERT_AGENT_PROVIDER_BINDING,
    SQL_INSERT_AUDIT_EVENT, SQL_LIST_AGENT, SQL_LIST_AGENT_COMPOSITION_SLOTS,
    SQL_LIST_AGENT_PROVIDER_BINDINGS, SQL_LIST_MCP_MARKETPLACE_SLOTS,
    SQL_SELECT_ACTIVE_AGENT_PROVIDER_BINDING, SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID,
    SQL_SELECT_AGENT_COMPOSITION_SLOT, SQL_SELECT_AGENT_PROVIDER_BINDING, SQL_UPDATE_AGENT,
    SQL_UPDATE_AGENT_COMPOSITION_SLOT, SQL_UPDATE_AGENT_PROVIDER_BINDING,
};
#[cfg(feature = "postgres-sync")]
pub use sql::{
    SQL_ACTIVATE_AGENT_SESSION_RUNTIME_BINDING, SQL_COMPLETE_AGENT_TURN_STATE,
    SQL_COUNT_AGENT_INTERACTIONS, SQL_COUNT_AGENT_ITEM_FEEDBACK, SQL_COUNT_AGENT_PROJECTS,
    SQL_COUNT_AGENT_PROJECT_COMPOSITION_SLOTS, SQL_COUNT_AGENT_RESOURCE_USER_STATES,
    SQL_COUNT_AGENT_SESSIONS, SQL_COUNT_AGENT_SESSION_CHECKPOINTS, SQL_COUNT_AGENT_SESSION_ITEMS,
    SQL_COUNT_AGENT_SESSION_RUNTIME_BINDINGS, SQL_COUNT_AGENT_TASKS, SQL_COUNT_AGENT_TURNS,
    SQL_COUNT_AGENT_WORKSPACES, SQL_DEACTIVATE_CURRENT_AGENT_SESSION_RUNTIME_BINDINGS,
    SQL_INSERT_AGENT_INTERACTION, SQL_INSERT_AGENT_ITEM_DRIVE_REF, SQL_INSERT_AGENT_PROJECT,
    SQL_INSERT_AGENT_PROJECT_COMPOSITION_SLOT, SQL_INSERT_AGENT_SESSION,
    SQL_INSERT_AGENT_SESSION_CHECKPOINT, SQL_INSERT_AGENT_SESSION_ITEM,
    SQL_INSERT_AGENT_SESSION_RUNTIME_BINDING, SQL_INSERT_AGENT_TASK, SQL_INSERT_AGENT_TURN,
    SQL_INSERT_AGENT_WORKSPACE, SQL_LIST_AGENT_INTERACTIONS, SQL_LIST_AGENT_ITEM_DRIVE_REFS,
    SQL_LIST_AGENT_ITEM_DRIVE_REFS_BATCH, SQL_LIST_AGENT_ITEM_FEEDBACK, SQL_LIST_AGENT_PROJECTS,
    SQL_LIST_AGENT_PROJECT_COMPOSITION_SLOTS, SQL_LIST_AGENT_RESOURCE_USER_STATES,
    SQL_LIST_AGENT_SESSIONS, SQL_LIST_AGENT_SESSION_ACTIVITY_HEADS,
    SQL_LIST_AGENT_SESSION_CHECKPOINTS, SQL_LIST_AGENT_SESSION_ITEMS,
    SQL_LIST_AGENT_SESSION_ITEMS_DESC, SQL_LIST_AGENT_SESSION_ITEMS_RECENT_CONTEXT,
    SQL_LIST_AGENT_SESSION_RUNTIME_BINDINGS, SQL_LIST_AGENT_TASKS, SQL_LIST_AGENT_TURNS,
    SQL_LIST_AGENT_WORKSPACES, SQL_LIST_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID,
    SQL_LIST_RECONCILABLE_AGENT_TURNS, SQL_LOCK_AGENT_PROJECT_WORKSPACE_NAME,
    SQL_LOCK_AGENT_SESSION_RUNTIME_BINDING, SQL_RECORD_AGENT_SESSION_ITEM,
    SQL_SELECT_AGENT_INTERACTION, SQL_SELECT_AGENT_ITEM_FEEDBACK, SQL_SELECT_AGENT_PROJECT,
    SQL_SELECT_AGENT_PROJECT_BY_IMPORT_SOURCE, SQL_SELECT_AGENT_PROJECT_BY_WORKSPACE_NAME,
    SQL_SELECT_AGENT_PROJECT_COMPOSITION_SLOT, SQL_SELECT_AGENT_RESOURCE_USER_STATE,
    SQL_SELECT_AGENT_SESSION, SQL_SELECT_AGENT_SESSION_BY_CREATE_IDEMPOTENCY,
    SQL_SELECT_AGENT_SESSION_CHECKPOINT, SQL_SELECT_AGENT_SESSION_ITEM,
    SQL_SELECT_AGENT_SESSION_RUNTIME_BINDING, SQL_SELECT_AGENT_TASK, SQL_SELECT_AGENT_TURN,
    SQL_SELECT_AGENT_TURN_BY_IDEMPOTENCY, SQL_SELECT_AGENT_WORKSPACE,
    SQL_SELECT_CURRENT_AGENT_SESSION_RUNTIME_BINDING, SQL_SELECT_DEFAULT_AGENT_WORKSPACE,
    SQL_UPDATE_AGENT_INTERACTION, SQL_UPDATE_AGENT_PROJECT,
    SQL_UPDATE_AGENT_PROJECT_COMPOSITION_SLOT, SQL_UPDATE_AGENT_SESSION,
    SQL_UPDATE_AGENT_SESSION_CHECKPOINT, SQL_UPDATE_AGENT_SESSION_ITEM,
    SQL_UPDATE_AGENT_SESSION_RUNTIME_BINDING, SQL_UPDATE_AGENT_TASK, SQL_UPDATE_AGENT_TURN_STATE,
    SQL_UPDATE_AGENT_WORKSPACE, SQL_UPSERT_AGENT_ITEM_FEEDBACK,
    SQL_UPSERT_AGENT_RESOURCE_USER_STATE,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentBusinessRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub agent_id: String,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub manifest_json: String,
    pub default_code_task_intent_json: Option<String>,
    pub implementation_provider_id: Option<String>,
    pub implementation_kind: Option<String>,
    pub implementation_type: String,
    pub status: i16,
    pub visibility: i16,
    pub tags_json: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub version: u64,
}

impl AgentBusinessRow {
    pub fn from_record(record: &AgentBusinessRecord) -> KernelResult<Self> {
        validate_agent_business_storage_contract(record)?;
        Ok(Self {
            id: record.id,
            uuid: build_agent_business_uuid(record.tenant_id, &record.agent_id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            owner_user_id: record.owner_user_id,
            agent_id: record.agent_id.clone(),
            code: record.code.clone(),
            display_name: record.display_name.clone(),
            description: record.description.clone(),
            manifest_json: manifest_to_json(&record.manifest)?,
            default_code_task_intent_json: intent_to_json(
                record.default_code_task_intent.as_ref(),
            )?,
            implementation_provider_id: record.implementation_provider_id.clone(),
            implementation_kind: record
                .implementation_kind
                .map(|kind| kind.as_str().to_string()),
            implementation_type: record.implementation_type.as_str().to_string(),
            status: record.status.as_db_code(),
            visibility: record.visibility.as_db_code(),
            tags_json: tags_to_json(&record.tags)?,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            deleted_at: record.deleted_at.clone(),
            version: record.version,
        })
    }

    pub fn into_record(self) -> KernelResult<AgentBusinessRecord> {
        let record = AgentBusinessRecord {
            id: self.id,
            agent_id: self.agent_id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            owner_user_id: self.owner_user_id,
            code: self.code,
            display_name: self.display_name,
            description: self.description,
            manifest: manifest_from_json(&self.manifest_json)?,
            default_code_task_intent: intent_from_json(
                self.default_code_task_intent_json.as_deref(),
            )?,
            implementation_provider_id: self.implementation_provider_id,
            implementation_kind: self
                .implementation_kind
                .as_deref()
                .map(parse_implementation_kind)
                .transpose()?,
            implementation_type: parse_implementation_type(&self.implementation_type)?,
            status: AgentBusinessStatus::from_db_code(self.status).ok_or_else(|| {
                KernelError::validation(format!("invalid db status code: {}", self.status))
            })?,
            visibility: AgentVisibility::from_db_code(self.visibility).ok_or_else(|| {
                KernelError::validation(format!("invalid db visibility code: {}", self.visibility))
            })?,
            tags: tags_from_json(&self.tags_json)?,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
            version: self.version,
        };
        validate_agent_business_storage_contract(&record)?;
        Ok(record)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentWorkspaceRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub workspace_id: String,
    pub owner_user_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub status: i16,
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

impl AgentWorkspaceRow {
    pub fn from_record(record: &AgentWorkspaceRecord) -> Self {
        Self {
            id: record.id,
            uuid: build_workspace_uuid(
                record.tenant_id,
                record.organization_id,
                &record.workspace_id,
            ),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            workspace_id: record.workspace_id.clone(),
            owner_user_id: record.owner_user_id,
            name: record.name.clone(),
            description: record.description.clone(),
            is_default: record.is_default,
            status: record.status.as_db_code(),
            created_by: record.created_by,
            updated_by: record.updated_by,
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            archived_at: record.archived_at.clone(),
            archived_by: record.archived_by,
            deleted_at: record.deleted_at.clone(),
            deleted_by: record.deleted_by,
            retention_until: record.retention_until.clone(),
        }
    }

    pub fn into_record(self) -> KernelResult<AgentWorkspaceRecord> {
        Ok(AgentWorkspaceRecord {
            id: self.id,
            workspace_id: self.workspace_id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            owner_user_id: self.owner_user_id,
            name: self.name,
            description: self.description,
            is_default: self.is_default,
            status: AgentWorkspaceStatus::from_db_code(self.status)
                .ok_or_else(|| KernelError::validation("invalid workspace status"))?,
            created_by: self.created_by,
            updated_by: self.updated_by,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            archived_at: self.archived_at,
            archived_by: self.archived_by,
            deleted_at: self.deleted_at,
            deleted_by: self.deleted_by,
            retention_until: self.retention_until,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProjectRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub workspace_id: String,
    pub owner_user_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub visibility: i16,
    pub status: i16,
    pub drive_access_mode: i16,
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

impl AgentProjectRow {
    pub fn from_record(record: &AgentProjectRecord) -> Self {
        Self {
            id: record.id,
            uuid: build_project_uuid(record.tenant_id, record.organization_id, &record.project_id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            project_id: record.project_id.clone(),
            workspace_id: record.workspace_id.clone(),
            owner_user_id: record.owner_user_id,
            name: record.name.clone(),
            description: record.description.clone(),
            visibility: record.visibility.as_db_code(),
            status: record.status.as_db_code(),
            drive_access_mode: record.drive_access_mode.as_db_code(),
            default_agent_id: record.default_agent_id.clone(),
            default_model_id: record.default_model_id.clone(),
            import_source_kind: record.import_source_kind.clone(),
            import_source_ref: record.import_source_ref.clone(),
            drive_space_id: record.drive_space_id.clone(),
            drive_root_entry_id: record.drive_root_entry_id.clone(),
            drive_logical_path: record.drive_logical_path.clone(),
            created_by: record.created_by,
            updated_by: record.updated_by,
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            archived_at: record.archived_at.clone(),
            archived_by: record.archived_by,
            deleted_at: record.deleted_at.clone(),
            deleted_by: record.deleted_by,
            retention_until: record.retention_until.clone(),
        }
    }

    pub fn into_record(self) -> KernelResult<AgentProjectRecord> {
        Ok(AgentProjectRecord {
            id: self.id,
            project_id: self.project_id,
            workspace_id: self.workspace_id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            owner_user_id: self.owner_user_id,
            name: self.name,
            description: self.description,
            visibility: AgentProjectVisibility::from_db_code(self.visibility)
                .ok_or_else(|| KernelError::validation("invalid project visibility"))?,
            status: AgentProjectStatus::from_db_code(self.status)
                .ok_or_else(|| KernelError::validation("invalid project status"))?,
            drive_access_mode: AgentProjectDriveAccessMode::from_db_code(self.drive_access_mode)
                .ok_or_else(|| KernelError::validation("invalid project drive access mode"))?,
            default_agent_id: self.default_agent_id,
            default_model_id: self.default_model_id,
            import_source_kind: self.import_source_kind,
            import_source_ref: self.import_source_ref,
            drive_space_id: self.drive_space_id,
            drive_root_entry_id: self.drive_root_entry_id,
            drive_logical_path: self.drive_logical_path,
            created_by: self.created_by,
            updated_by: self.updated_by,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            archived_at: self.archived_at,
            archived_by: self.archived_by,
            deleted_at: self.deleted_at,
            deleted_by: self.deleted_by,
            retention_until: self.retention_until,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProjectCompositionSlotRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub slot_id: String,
    pub slot_kind: String,
    pub target_module: String,
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

impl AgentProjectCompositionSlotRow {
    pub fn from_record(record: &AgentProjectCompositionSlotRecord) -> Self {
        Self {
            id: record.id,
            uuid: build_project_composition_slot_uuid(
                record.tenant_id,
                record.organization_id,
                &record.project_id,
                &record.slot_id,
            ),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            project_id: record.project_id.clone(),
            slot_id: record.slot_id.clone(),
            slot_kind: record.slot_kind.as_str().to_string(),
            target_module: record.target_module.as_str().to_string(),
            target_ref: record.target_ref.clone(),
            target_version_ref: record.target_version_ref.clone(),
            priority: record.priority,
            enabled: record.enabled,
            policy_json: record.policy_json.clone(),
            created_by: record.created_by,
            updated_by: record.updated_by,
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            deleted_at: record.deleted_at.clone(),
            deleted_by: record.deleted_by,
            retention_until: record.retention_until.clone(),
        }
    }

    pub fn into_record(self) -> KernelResult<AgentProjectCompositionSlotRecord> {
        Ok(AgentProjectCompositionSlotRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            project_id: self.project_id,
            slot_id: self.slot_id,
            slot_kind: AgentCompositionSlotKind::try_from_str(&self.slot_kind).ok_or_else(
                || {
                    KernelError::validation(format!(
                        "invalid project slot_kind: {}",
                        self.slot_kind
                    ))
                },
            )?,
            target_module: AgentCompositionTargetModule::try_from_str(&self.target_module)
                .ok_or_else(|| {
                    KernelError::validation(format!(
                        "invalid project target_module: {}",
                        self.target_module
                    ))
                })?,
            target_ref: self.target_ref,
            target_version_ref: self.target_version_ref,
            priority: self.priority,
            enabled: self.enabled,
            policy_json: self.policy_json,
            created_by: self.created_by,
            updated_by: self.updated_by,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
            deleted_by: self.deleted_by,
            retention_until: self.retention_until,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderBindingRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub agent_id: String,
    pub binding_id: String,
    pub provider_id: String,
    pub implementation_kind: String,
    pub configuration_profile_id: String,
    pub capabilities_json: String,
    pub active: bool,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
}

impl AgentProviderBindingRow {
    pub fn from_record(record: &AgentProviderBindingRecord) -> KernelResult<Self> {
        validate_provider_binding_storage_contract(record)?;
        Ok(Self {
            id: record.id,
            uuid: build_agent_provider_binding_uuid(
                record.tenant_id,
                &record.agent_id,
                &record.binding_id,
            ),
            tenant_id: record.tenant_id,
            agent_id: record.agent_id.clone(),
            binding_id: record.binding_id.clone(),
            provider_id: record.provider_id.clone(),
            implementation_kind: record.implementation_kind.as_str().to_string(),
            configuration_profile_id: record.configuration_profile_id.clone(),
            capabilities_json: string_list_to_json(&record.capabilities, "capabilities")?,
            active: record.active,
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        })
    }

    pub fn into_record(self) -> KernelResult<AgentProviderBindingRecord> {
        let capabilities = string_list_from_json(&self.capabilities_json, "capabilities")?;
        let record = AgentProviderBindingRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            agent_id: self.agent_id,
            binding_id: self.binding_id,
            provider_id: self.provider_id,
            implementation_kind: parse_implementation_kind(&self.implementation_kind)?,
            configuration_profile_id: self.configuration_profile_id,
            capabilities,
            active: self.active,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
        };
        validate_provider_binding_storage_contract(&record)?;
        Ok(record)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub slot_id: String,
    pub slot_kind: String,
    pub target_module: String,
    pub target_ref: String,
    pub target_version_ref: Option<String>,
    pub priority: i32,
    pub enabled: bool,
    pub policy_json: String,
    pub status: i16,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

impl AgentCompositionSlotRow {
    pub fn from_record(record: &AgentCompositionSlotRecord) -> KernelResult<Self> {
        Ok(Self {
            id: record.id,
            uuid: build_composition_slot_uuid(record.tenant_id, &record.agent_id, &record.slot_id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            agent_id: record.agent_id.clone(),
            slot_id: record.slot_id.clone(),
            slot_kind: record.slot_kind.as_str().to_string(),
            target_module: record.target_module.as_str().to_string(),
            target_ref: record.target_ref.clone(),
            target_version_ref: record.target_version_ref.clone(),
            priority: record.priority,
            enabled: record.enabled,
            policy_json: record.policy_json.clone(),
            status: record.status.as_db_code(),
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            deleted_at: record.deleted_at.clone(),
        })
    }

    pub fn into_record(self) -> KernelResult<AgentCompositionSlotRecord> {
        Ok(AgentCompositionSlotRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            agent_id: self.agent_id,
            slot_id: self.slot_id,
            slot_kind: AgentCompositionSlotKind::try_from_str(self.slot_kind.as_str()).ok_or_else(
                || KernelError::validation(format!("invalid slot_kind: {}", self.slot_kind)),
            )?,
            target_module: AgentCompositionTargetModule::try_from_str(self.target_module.as_str())
                .ok_or_else(|| {
                    KernelError::validation(format!(
                        "invalid target_module: {}",
                        self.target_module
                    ))
                })?,
            target_ref: self.target_ref,
            target_version_ref: self.target_version_ref,
            priority: self.priority,
            enabled: self.enabled,
            policy_json: self.policy_json,
            status: AgentBusinessStatus::from_db_code(self.status).ok_or_else(|| {
                KernelError::validation(format!("invalid composition slot status: {}", self.status))
            })?,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        })
    }
}

// ============================================================================
// AgentSessionRow — persistence row for ai_agent_session
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSessionRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub session_id: String,
    pub owner_user_id: u64,
    pub project_id: Option<String>,
    pub session_kind: i16,
    pub entry_surface: i16,
    pub source_module: Option<String>,
    pub source_context_kind: Option<String>,
    pub source_context_id: Option<String>,
    pub parent_session_id: Option<String>,
    pub forked_from_turn_id: Option<String>,
    pub title: Option<String>,
    pub title_source: i16,
    pub status: i16,
    pub item_count: u64,
    pub last_item_sequence: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub idempotency_key: Option<String>,
    pub payload_hash: Option<String>,
    pub created_by: u64,
    pub updated_by: u64,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub last_item_at: Option<String>,
    pub closed_at: Option<String>,
    pub archived_at: Option<String>,
    pub archived_by: Option<u64>,
    pub deleted_at: Option<String>,
    pub deleted_by: Option<u64>,
    pub retention_until: Option<String>,
}

impl AgentSessionRow {
    pub fn from_record(record: &AgentSessionRecord) -> KernelResult<Self> {
        Ok(Self {
            id: record.id,
            uuid: build_session_uuid(record.tenant_id, &record.session_id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            agent_id: record.agent_id.clone(),
            session_id: record.session_id.clone(),
            owner_user_id: record.owner_user_id,
            project_id: record.project_id.clone(),
            session_kind: record.session_kind.as_db_code(),
            entry_surface: record.entry_surface.as_db_code(),
            source_module: record.source_module.clone(),
            source_context_kind: record.source_context_kind.clone(),
            source_context_id: record.source_context_id.clone(),
            parent_session_id: record.parent_session_id.clone(),
            forked_from_turn_id: record.forked_from_turn_id.clone(),
            title: record.title.clone(),
            title_source: record.title_source.as_db_code(),
            status: record.status.as_db_code(),
            item_count: record.item_count,
            last_item_sequence: record.last_item_sequence,
            total_input_tokens: record.total_input_tokens,
            total_output_tokens: record.total_output_tokens,
            idempotency_key: record.idempotency_key.clone(),
            payload_hash: record.payload_hash.clone(),
            created_by: record.created_by,
            updated_by: record.updated_by,
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            last_item_at: record.last_item_at.clone(),
            closed_at: record.closed_at.clone(),
            archived_at: record.archived_at.clone(),
            archived_by: record.archived_by,
            deleted_at: record.deleted_at.clone(),
            deleted_by: record.deleted_by,
            retention_until: record.retention_until.clone(),
        })
    }

    pub fn into_record(self) -> KernelResult<AgentSessionRecord> {
        let status = AgentSessionStatus::from_db_code(self.status).ok_or_else(|| {
            KernelError::validation(format!("invalid session status db code: {}", self.status))
        })?;
        let session_kind = AgentSessionKind::from_db_code(self.session_kind).ok_or_else(|| {
            KernelError::validation(format!(
                "invalid session kind db code: {}",
                self.session_kind
            ))
        })?;
        let entry_surface =
            AgentSessionEntrySurface::from_db_code(self.entry_surface).ok_or_else(|| {
                KernelError::validation(format!(
                    "invalid session entry surface db code: {}",
                    self.entry_surface
                ))
            })?;
        let title_source =
            AgentSessionTitleSource::from_db_code(self.title_source).ok_or_else(|| {
                KernelError::validation(format!(
                    "invalid session title source db code: {}",
                    self.title_source
                ))
            })?;
        Ok(AgentSessionRecord {
            id: self.id,
            session_id: self.session_id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            agent_id: self.agent_id,
            owner_user_id: self.owner_user_id,
            project_id: self.project_id,
            session_kind,
            entry_surface,
            source_module: self.source_module,
            source_context_kind: self.source_context_kind,
            source_context_id: self.source_context_id,
            parent_session_id: self.parent_session_id,
            forked_from_turn_id: self.forked_from_turn_id,
            title: self.title,
            title_source,
            status,
            item_count: self.item_count,
            last_item_sequence: self.last_item_sequence,
            total_input_tokens: self.total_input_tokens,
            total_output_tokens: self.total_output_tokens,
            idempotency_key: self.idempotency_key,
            payload_hash: self.payload_hash,
            created_by: self.created_by,
            updated_by: self.updated_by,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            last_item_at: self.last_item_at,
            closed_at: self.closed_at,
            archived_at: self.archived_at,
            archived_by: self.archived_by,
            deleted_at: self.deleted_at,
            deleted_by: self.deleted_by,
            retention_until: self.retention_until,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSessionActivityHeadRow {
    pub session: AgentSessionRow,
    pub activity_at: String,
    pub activity_source: SessionActivitySource,
    pub latest_turn: Option<AgentTurnRow>,
    pub pending_interaction: Option<AgentInteractionRow>,
    pub current_runtime_binding: Option<AgentSessionRuntimeBindingRow>,
    pub latest_runtime_binding: Option<AgentSessionRuntimeBindingRow>,
    pub user_state: Option<AgentResourceUserStateRow>,
    pub latest_interaction_id: Option<String>,
    pub latest_interaction_version: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSessionRuntimeBindingRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub session_id: String,
    pub runtime_binding_id: String,
    pub runtime_location_id: Option<String>,
    pub host_mode: String,
    pub transport_kind: String,
    pub provider_binding_id: String,
    pub model_id: String,
    pub provider_id: String,
    pub provider_session_id: Option<String>,
    pub provider_session_tree_id: Option<String>,
    pub provider_parent_session_id: Option<String>,
    pub provider_forked_from_session_id: Option<String>,
    pub status: i16,
    pub is_current: bool,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub activated_at: Option<String>,
    pub deactivated_at: Option<String>,
}

impl AgentSessionRuntimeBindingRow {
    pub fn from_record(record: &AgentSessionRuntimeBindingRecord) -> Self {
        Self {
            id: record.id,
            uuid: build_session_runtime_binding_uuid(
                record.tenant_id,
                record.organization_id,
                &record.session_id,
                &record.runtime_binding_id,
            ),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            owner_user_id: record.owner_user_id,
            session_id: record.session_id.clone(),
            runtime_binding_id: record.runtime_binding_id.clone(),
            runtime_location_id: record.runtime_location_id.clone(),
            host_mode: record.host_mode.clone(),
            transport_kind: record.transport_kind.clone(),
            provider_binding_id: record.provider_binding_id.clone(),
            model_id: record.model_id.clone(),
            provider_id: record.provider_id.clone(),
            provider_session_id: record.provider_session_id.clone(),
            provider_session_tree_id: record.provider_session_tree_id.clone(),
            provider_parent_session_id: record.provider_parent_session_id.clone(),
            provider_forked_from_session_id: record.provider_forked_from_session_id.clone(),
            status: record.status.as_db_code(),
            is_current: record.is_current,
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            activated_at: record.activated_at.clone(),
            deactivated_at: record.deactivated_at.clone(),
        }
    }

    pub fn into_record(self) -> KernelResult<AgentSessionRuntimeBindingRecord> {
        Ok(AgentSessionRuntimeBindingRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            owner_user_id: self.owner_user_id,
            session_id: self.session_id,
            runtime_binding_id: self.runtime_binding_id,
            runtime_location_id: self.runtime_location_id,
            host_mode: self.host_mode,
            transport_kind: self.transport_kind,
            provider_binding_id: self.provider_binding_id,
            model_id: self.model_id,
            provider_id: self.provider_id,
            provider_session_id: self.provider_session_id,
            provider_session_tree_id: self.provider_session_tree_id,
            provider_parent_session_id: self.provider_parent_session_id,
            provider_forked_from_session_id: self.provider_forked_from_session_id,
            status: AgentSessionRuntimeBindingStatus::from_db_code(self.status)
                .ok_or_else(|| KernelError::validation("invalid session runtime binding status"))?,
            is_current: self.is_current,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            activated_at: self.activated_at,
            deactivated_at: self.deactivated_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSessionCheckpointRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub checkpoint_id: String,
    pub turn_id: Option<String>,
    pub runtime_binding_id: Option<String>,
    pub checkpoint_kind: String,
    pub provider_checkpoint_ref: Option<String>,
    pub drive_space_id: Option<String>,
    pub drive_node_id: Option<String>,
    pub resumable: bool,
    pub status: i16,
    pub created_by: u64,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub restored_at: Option<String>,
    pub invalidated_at: Option<String>,
    pub retention_until: Option<String>,
}

impl AgentSessionCheckpointRow {
    pub fn from_record(record: &AgentSessionCheckpointRecord) -> Self {
        Self {
            id: record.id,
            uuid: build_session_checkpoint_uuid(
                record.tenant_id,
                record.organization_id,
                &record.session_id,
                &record.checkpoint_id,
            ),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            session_id: record.session_id.clone(),
            checkpoint_id: record.checkpoint_id.clone(),
            turn_id: record.turn_id.clone(),
            runtime_binding_id: record.runtime_binding_id.clone(),
            checkpoint_kind: record.checkpoint_kind.clone(),
            provider_checkpoint_ref: record.provider_checkpoint_ref.clone(),
            drive_space_id: record.drive_space_id.clone(),
            drive_node_id: record.drive_node_id.clone(),
            resumable: record.resumable,
            status: record.status.as_db_code(),
            created_by: record.created_by,
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            restored_at: record.restored_at.clone(),
            invalidated_at: record.invalidated_at.clone(),
            retention_until: record.retention_until.clone(),
        }
    }

    pub fn into_record(self) -> KernelResult<AgentSessionCheckpointRecord> {
        Ok(AgentSessionCheckpointRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            session_id: self.session_id,
            checkpoint_id: self.checkpoint_id,
            turn_id: self.turn_id,
            runtime_binding_id: self.runtime_binding_id,
            checkpoint_kind: self.checkpoint_kind,
            provider_checkpoint_ref: self.provider_checkpoint_ref,
            drive_space_id: self.drive_space_id,
            drive_node_id: self.drive_node_id,
            resumable: self.resumable,
            status: AgentSessionCheckpointStatus::from_db_code(self.status)
                .ok_or_else(|| KernelError::validation("invalid session checkpoint status"))?,
            created_by: self.created_by,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            restored_at: self.restored_at,
            invalidated_at: self.invalidated_at,
            retention_until: self.retention_until,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentResourceUserStateRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub resource_type: i16,
    pub resource_id: String,
    pub pinned_at: Option<String>,
    pub hidden_at: Option<String>,
    pub last_opened_at: Option<String>,
    pub last_read_item_sequence: Option<u64>,
    pub custom_title: Option<String>,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
}

impl AgentResourceUserStateRow {
    pub fn from_record(record: &AgentResourceUserStateRecord) -> Self {
        Self {
            id: record.id,
            uuid: format!("agents-user-state-{}", record.id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            user_id: record.user_id,
            resource_type: record.resource_type.as_db_code(),
            resource_id: record.resource_id.clone(),
            pinned_at: record.pinned_at.clone(),
            hidden_at: record.hidden_at.clone(),
            last_opened_at: record.last_opened_at.clone(),
            last_read_item_sequence: record.last_read_item_sequence,
            custom_title: record.custom_title.clone(),
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        }
    }

    pub fn into_record(self) -> KernelResult<AgentResourceUserStateRecord> {
        Ok(AgentResourceUserStateRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            user_id: self.user_id,
            resource_type: AgentResourceType::from_db_code(self.resource_type)
                .ok_or_else(|| KernelError::validation("invalid resource user state type"))?,
            resource_id: self.resource_id,
            pinned_at: self.pinned_at,
            hidden_at: self.hidden_at,
            last_opened_at: self.last_opened_at,
            last_read_item_sequence: self.last_read_item_sequence,
            custom_title: self.custom_title,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentItemFeedbackRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub item_id: String,
    pub user_id: u64,
    pub rating: i16,
    pub reason_code: Option<String>,
    pub comment: Option<String>,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

impl AgentItemFeedbackRow {
    pub fn from_record(record: &AgentItemFeedbackRecord) -> Self {
        Self {
            id: record.id,
            uuid: format!("agents-item-feedback-{}", record.id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            item_id: record.item_id.clone(),
            user_id: record.user_id,
            rating: record.rating.as_db_code(),
            reason_code: record.reason_code.clone(),
            comment: record.comment.clone(),
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            deleted_at: record.deleted_at.clone(),
        }
    }

    pub fn into_record(self) -> KernelResult<AgentItemFeedbackRecord> {
        Ok(AgentItemFeedbackRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            item_id: self.item_id,
            user_id: self.user_id,
            rating: AgentItemFeedbackRating::from_db_code(self.rating)
                .ok_or_else(|| KernelError::validation("invalid item feedback rating"))?,
            reason_code: self.reason_code,
            comment: self.comment,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentItemDriveRefRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub item_id: String,
    pub resource_role: String,
    pub drive_space_id: String,
    pub drive_node_id: String,
    pub media_resource_id: Option<String>,
    pub object_blob_id: Option<String>,
    pub resource_hash: Option<String>,
    pub alt_text: Option<String>,
    pub sort_order: u32,
    pub status: i16,
    pub created_by: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub retention_until: Option<String>,
}

impl AgentItemDriveRefRow {
    pub fn from_record(record: &AgentItemDriveRefRecord) -> Self {
        Self {
            id: record.id,
            uuid: format!("agents-item-drive-ref-{}", record.id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            item_id: record.item_id.clone(),
            resource_role: record.resource_role.as_str().to_string(),
            drive_space_id: record.drive_space_id.clone(),
            drive_node_id: record.drive_node_id.clone(),
            media_resource_id: record.media_resource_id.clone(),
            object_blob_id: record.object_blob_id.clone(),
            resource_hash: record.resource_hash.clone(),
            alt_text: record.alt_text.clone(),
            sort_order: record.sort_order,
            status: record.status,
            created_by: record.created_by,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            deleted_at: record.deleted_at.clone(),
            retention_until: record.retention_until.clone(),
        }
    }

    pub fn into_record(self) -> KernelResult<AgentItemDriveRefRecord> {
        let resource_role = match self.resource_role.as_str() {
            "attachment" => AgentItemResourceRole::Attachment,
            "image" => AgentItemResourceRole::Image,
            "audio" => AgentItemResourceRole::Audio,
            "generated_output" => AgentItemResourceRole::GeneratedOutput,
            "artifact" => AgentItemResourceRole::Artifact,
            _ => {
                return Err(KernelError::validation(
                    "invalid session-item resource role",
                ))
            }
        };
        Ok(AgentItemDriveRefRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            item_id: self.item_id,
            resource_role,
            drive_space_id: self.drive_space_id,
            drive_node_id: self.drive_node_id,
            media_resource_id: self.media_resource_id,
            object_blob_id: self.object_blob_id,
            resource_hash: self.resource_hash,
            alt_text: self.alt_text,
            sort_order: self.sort_order,
            status: self.status,
            created_by: self.created_by,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
            retention_until: self.retention_until,
        })
    }
}

// ============================================================================
// AgentSessionItemRow - persistence row for ai_agent_session_item
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSessionItemRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub item_id: String,
    pub kind: i16,
    pub content: Option<String>,
    pub content_type: String,
    pub status: i16,
    pub sequence: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub tool_name: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_arguments_json: Option<String>,
    pub tool_result_json: Option<String>,
    pub parent_item_id: Option<String>,
    pub turn_id: Option<String>,
    pub created_by: u64,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub redacted_at: Option<String>,
    pub redacted_by: Option<u64>,
    pub retention_until: Option<String>,
}

impl AgentSessionItemRow {
    pub fn from_record(record: &AgentSessionItemRecord) -> KernelResult<Self> {
        Ok(Self {
            id: record.id,
            uuid: build_session_item_uuid(record.tenant_id, &record.session_id, &record.item_id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            session_id: record.session_id.clone(),
            item_id: record.item_id.clone(),
            kind: record.kind.as_db_code(),
            content: record.content.clone(),
            content_type: record.content_type.clone(),
            status: record.status.as_db_code(),
            sequence: record.sequence,
            input_tokens: record.input_tokens,
            output_tokens: record.output_tokens,
            model_id: record.model_id.clone(),
            provider_id: record.provider_id.clone(),
            tool_name: record.tool_name.clone(),
            tool_call_id: record.tool_call_id.clone(),
            tool_arguments_json: record.tool_arguments_json.clone(),
            tool_result_json: record.tool_result_json.clone(),
            parent_item_id: record.parent_item_id.clone(),
            turn_id: record.turn_id.clone(),
            created_by: record.created_by,
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            completed_at: record.completed_at.clone(),
            redacted_at: record.redacted_at.clone(),
            redacted_by: record.redacted_by,
            retention_until: record.retention_until.clone(),
        })
    }

    pub fn into_record(self) -> KernelResult<AgentSessionItemRecord> {
        let kind = AgentSessionItemKind::from_db_code(self.kind).ok_or_else(|| {
            KernelError::validation(format!("invalid session item kind db code: {}", self.kind))
        })?;
        let status = AgentSessionItemStatus::from_db_code(self.status).ok_or_else(|| {
            KernelError::validation(format!(
                "invalid session item status db code: {}",
                self.status
            ))
        })?;
        Ok(AgentSessionItemRecord {
            id: self.id,
            item_id: self.item_id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            session_id: self.session_id,
            kind,
            content: self.content,
            content_type: self.content_type,
            status,
            sequence: self.sequence,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            model_id: self.model_id,
            provider_id: self.provider_id,
            tool_name: self.tool_name,
            tool_call_id: self.tool_call_id,
            tool_arguments_json: self.tool_arguments_json,
            tool_result_json: self.tool_result_json,
            parent_item_id: self.parent_item_id,
            turn_id: self.turn_id,
            created_by: self.created_by,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            completed_at: self.completed_at,
            redacted_at: self.redacted_at,
            redacted_by: self.redacted_by,
            retention_until: self.retention_until,
        })
    }
}

// ============================================================================
// AgentInteractionRow — persistence row for ai_agent_interaction
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentTurnRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub turn_id: String,
    pub session_id: String,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub runtime_binding_id: Option<String>,
    pub client_request_id: Option<String>,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub request_item_id: String,
    pub response_item_id: Option<String>,
    pub turn_mode: i16,
    pub status: i16,
    pub requested_model_id: Option<String>,
    pub provider_binding_id: Option<String>,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub finish_reason: Option<String>,
    pub error_code: Option<String>,
    pub error_detail: Option<String>,
    pub trace_id: Option<String>,
    pub attempt_count: u32,
    pub max_attempts: u32,
    pub next_retry_at: Option<String>,
    pub available_at: String,
    pub lease_owner: Option<String>,
    pub lease_token: Option<String>,
    pub lease_expires_at: Option<String>,
    pub fencing_token: u64,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub cancel_requested_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub retention_until: Option<String>,
}

impl AgentTurnRow {
    pub fn from_record(record: &AgentTurnRecord) -> Self {
        Self {
            id: record.id,
            uuid: build_turn_uuid(record.tenant_id, record.organization_id, &record.turn_id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            turn_id: record.turn_id.clone(),
            session_id: record.session_id.clone(),
            agent_id: record.agent_id.clone(),
            owner_user_id: record.owner_user_id,
            runtime_binding_id: record.runtime_binding_id.clone(),
            client_request_id: record.client_request_id.clone(),
            idempotency_key: record.idempotency_key.clone(),
            payload_hash: record.payload_hash.clone(),
            request_item_id: record.request_item_id.clone(),
            response_item_id: record.response_item_id.clone(),
            turn_mode: record.turn_mode.as_db_code(),
            status: record.status.as_db_code(),
            requested_model_id: record.requested_model_id.clone(),
            provider_binding_id: record.provider_binding_id.clone(),
            model_id: record.model_id.clone(),
            provider_id: record.provider_id.clone(),
            input_tokens: record.input_tokens,
            output_tokens: record.output_tokens,
            cached_tokens: record.cached_tokens,
            finish_reason: record.finish_reason.clone(),
            error_code: record.error_code.clone(),
            error_detail: record.error_detail.clone(),
            trace_id: record.trace_id.clone(),
            attempt_count: record.attempt_count,
            max_attempts: record.max_attempts,
            next_retry_at: record.next_retry_at.clone(),
            available_at: record.available_at.clone(),
            lease_owner: record.lease_owner.clone(),
            lease_token: record.lease_token.clone(),
            lease_expires_at: record.lease_expires_at.clone(),
            fencing_token: record.fencing_token,
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            started_at: record.started_at.clone(),
            completed_at: record.completed_at.clone(),
            cancel_requested_at: record.cancel_requested_at.clone(),
            cancelled_at: record.cancelled_at.clone(),
            retention_until: record.retention_until.clone(),
        }
    }

    pub fn into_record(self) -> KernelResult<AgentTurnRecord> {
        Ok(AgentTurnRecord {
            id: self.id,
            turn_id: self.turn_id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            session_id: self.session_id,
            agent_id: self.agent_id,
            owner_user_id: self.owner_user_id,
            runtime_binding_id: self.runtime_binding_id,
            client_request_id: self.client_request_id,
            idempotency_key: self.idempotency_key,
            payload_hash: self.payload_hash,
            request_item_id: self.request_item_id,
            response_item_id: self.response_item_id,
            turn_mode: AgentTurnMode::from_db_code(self.turn_mode)
                .ok_or_else(|| KernelError::validation("invalid turn mode"))?,
            status: AgentTurnStatus::from_db_code(self.status)
                .ok_or_else(|| KernelError::validation("invalid turn status"))?,
            requested_model_id: self.requested_model_id,
            provider_binding_id: self.provider_binding_id,
            model_id: self.model_id,
            provider_id: self.provider_id,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            cached_tokens: self.cached_tokens,
            finish_reason: self.finish_reason,
            error_code: self.error_code,
            error_detail: self.error_detail,
            trace_id: self.trace_id,
            attempt_count: self.attempt_count,
            max_attempts: self.max_attempts,
            next_retry_at: self.next_retry_at,
            available_at: self.available_at,
            lease_owner: self.lease_owner,
            lease_token: self.lease_token,
            lease_expires_at: self.lease_expires_at,
            fencing_token: self.fencing_token,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            started_at: self.started_at,
            completed_at: self.completed_at,
            cancel_requested_at: self.cancel_requested_at,
            cancelled_at: self.cancelled_at,
            retention_until: self.retention_until,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentInteractionRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub turn_id: Option<String>,
    pub runtime_binding_id: Option<String>,
    pub interaction_id: String,
    pub provider_interaction_id: Option<String>,
    pub kind: i16,
    pub status: i16,
    pub prompt: String,
    pub options_json: String,
    pub resolution_json: Option<String>,
    pub claim_owner: Option<String>,
    pub claim_token_hash: Option<String>,
    pub claim_expires_at: Option<String>,
    pub fencing_token: u64,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub resolved_at: Option<String>,
    pub retention_until: Option<String>,
}

impl AgentInteractionRow {
    pub fn from_record(record: &AgentInteractionRecord) -> KernelResult<Self> {
        Ok(Self {
            id: record.id,
            uuid: build_interaction_uuid(
                record.tenant_id,
                &record.session_id,
                &record.interaction_id,
            ),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            session_id: record.session_id.clone(),
            turn_id: record.turn_id.clone(),
            runtime_binding_id: record.runtime_binding_id.clone(),
            interaction_id: record.interaction_id.clone(),
            provider_interaction_id: record.provider_interaction_id.clone(),
            kind: record.kind.as_db_code(),
            status: record.status.as_db_code(),
            prompt: record.prompt.clone(),
            options_json: record.options_json.clone(),
            resolution_json: record.resolution_json.clone(),
            claim_owner: record.claim_owner.clone(),
            claim_token_hash: record.claim_token_hash.clone(),
            claim_expires_at: record.claim_expires_at.clone(),
            fencing_token: record.fencing_token,
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            resolved_at: record.resolved_at.clone(),
            retention_until: record.retention_until.clone(),
        })
    }

    pub fn into_record(self) -> KernelResult<AgentInteractionRecord> {
        let kind = AgentInteractionKind::from_db_code(self.kind).ok_or_else(|| {
            KernelError::validation(format!("invalid interaction kind db code: {}", self.kind))
        })?;
        let status = AgentInteractionStatus::from_db_code(self.status).ok_or_else(|| {
            KernelError::validation(format!(
                "invalid interaction status db code: {}",
                self.status
            ))
        })?;
        Ok(AgentInteractionRecord {
            id: self.id,
            interaction_id: self.interaction_id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            session_id: self.session_id,
            turn_id: self.turn_id,
            runtime_binding_id: self.runtime_binding_id,
            kind,
            status,
            provider_interaction_id: self.provider_interaction_id,
            prompt: self.prompt,
            options_json: self.options_json,
            resolution_json: self.resolution_json,
            claim_owner: self.claim_owner,
            claim_token_hash: self.claim_token_hash,
            claim_expires_at: self.claim_expires_at,
            fencing_token: self.fencing_token,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            resolved_at: self.resolved_at,
            retention_until: self.retention_until,
        })
    }
}

// ============================================================================
// AgentTaskRow — persistence row for ai_agent_task
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTaskRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub task_id: String,
    pub owner_user_id: u64,
    pub title: Option<String>,
    pub prompt: String,
    pub status: i16,
    pub external_ref: Option<String>,
    pub metadata_json: String,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub cancelled_at: Option<String>,
}

impl AgentTaskRow {
    pub fn from_record(record: &AgentTaskRecord) -> KernelResult<Self> {
        Ok(Self {
            id: record.id,
            uuid: build_task_uuid(record.tenant_id, &record.task_id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            agent_id: record.agent_id.clone(),
            task_id: record.task_id.clone(),
            owner_user_id: record.owner_user_id,
            title: record.title.clone(),
            prompt: record.prompt.clone(),
            status: record.status.as_db_code(),
            external_ref: record.external_ref.clone(),
            metadata_json: record.metadata_json.clone(),
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            started_at: record.started_at.clone(),
            completed_at: record.completed_at.clone(),
            cancelled_at: record.cancelled_at.clone(),
        })
    }

    pub fn into_record(self) -> KernelResult<AgentTaskRecord> {
        let status = AgentTaskStatus::from_db_code(self.status).ok_or_else(|| {
            KernelError::validation(format!("invalid task status db code: {}", self.status))
        })?;
        Ok(AgentTaskRecord {
            id: self.id,
            task_id: self.task_id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            agent_id: self.agent_id,
            owner_user_id: self.owner_user_id,
            title: self.title,
            prompt: self.prompt,
            status,
            external_ref: self.external_ref,
            metadata_json: self.metadata_json,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            started_at: self.started_at,
            completed_at: self.completed_at,
            cancelled_at: self.cancelled_at,
        })
    }
}

fn build_session_uuid(tenant_id: u64, session_id: &str) -> String {
    build_storage_uuid("session", tenant_id, &[session_id])
}

fn build_session_runtime_binding_uuid(
    tenant_id: u64,
    organization_id: u64,
    session_id: &str,
    runtime_binding_id: &str,
) -> String {
    build_storage_uuid(
        "session-runtime-binding",
        tenant_id,
        &[&organization_id.to_string(), session_id, runtime_binding_id],
    )
}

fn build_session_checkpoint_uuid(
    tenant_id: u64,
    organization_id: u64,
    session_id: &str,
    checkpoint_id: &str,
) -> String {
    build_storage_uuid(
        "session-checkpoint",
        tenant_id,
        &[&organization_id.to_string(), session_id, checkpoint_id],
    )
}

fn build_session_item_uuid(tenant_id: u64, session_id: &str, item_id: &str) -> String {
    build_storage_uuid("session-item", tenant_id, &[session_id, item_id])
}

fn build_interaction_uuid(tenant_id: u64, session_id: &str, interaction_id: &str) -> String {
    build_storage_uuid("interaction", tenant_id, &[session_id, interaction_id])
}

fn build_task_uuid(tenant_id: u64, task_id: &str) -> String {
    build_storage_uuid("task", tenant_id, &[task_id])
}

fn build_project_uuid(tenant_id: u64, organization_id: u64, project_id: &str) -> String {
    let organization_id = organization_id.to_string();
    build_storage_uuid("project", tenant_id, &[&organization_id, project_id])
}

fn build_workspace_uuid(tenant_id: u64, organization_id: u64, workspace_id: &str) -> String {
    let organization_id = organization_id.to_string();
    build_storage_uuid("workspace", tenant_id, &[&organization_id, workspace_id])
}

fn build_project_composition_slot_uuid(
    tenant_id: u64,
    organization_id: u64,
    project_id: &str,
    slot_id: &str,
) -> String {
    let organization_id = organization_id.to_string();
    build_storage_uuid(
        "project-composition-slot",
        tenant_id,
        &[&organization_id, project_id, slot_id],
    )
}

fn build_turn_uuid(tenant_id: u64, organization_id: u64, turn_id: &str) -> String {
    let organization_id = organization_id.to_string();
    build_storage_uuid("chat-turn", tenant_id, &[&organization_id, turn_id])
}

fn build_composition_slot_uuid(tenant_id: u64, agent_id: &str, slot_id: &str) -> String {
    build_storage_uuid("composition-slot", tenant_id, &[agent_id, slot_id])
}

fn parse_implementation_kind(input: &str) -> KernelResult<AgentImplementationKind> {
    AgentImplementationKind::from_code(input)
        .ok_or_else(|| KernelError::validation(format!("invalid implementation kind: {input}")))
}

fn parse_implementation_type(input: &str) -> KernelResult<AgentImplementationType> {
    AgentImplementationType::from_code(input).ok_or_else(|| {
        KernelError::validation(format!(
            "implementationType must be one of sdkwork-native, rig-rust, openai-agents, langchain, langgraph, crewai, autogen, semantic-kernel, custom: {input}"
        ))
    })
}

fn validate_agent_business_storage_contract(record: &AgentBusinessRecord) -> KernelResult<()> {
    if let Some(provider_id) = record.implementation_provider_id.as_deref() {
        validate_standard_id(provider_id, "implementationProviderId", Some("provider."))?;
    }
    Ok(())
}

fn validate_provider_binding_storage_contract(
    record: &AgentProviderBindingRecord,
) -> KernelResult<()> {
    validate_standard_id(record.binding_id.as_str(), "bindingId", Some("binding."))?;
    validate_standard_id(record.provider_id.as_str(), "providerId", Some("provider."))?;
    validate_standard_id(
        record.configuration_profile_id.as_str(),
        "configurationProfileId",
        Some("profile."),
    )?;
    validate_capabilities(record.capabilities.as_slice(), "capabilities")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentAuditEventRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub agent_internal_id: Option<u64>,
    pub agent_id: Option<String>,
    pub action: String,
    pub actor_type: i16,
    pub actor_id: u64,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub payload_json: String,
    pub created_at: String,
}

impl AgentAuditEventRow {
    /// Build an audit row from a KernelEvent, extracting agent/tenant metadata
    /// from the event's structured context. The context is populated by
    /// `AgentsService::emit_audit_event` (and the related `emit_*_audit_event`
    /// helpers) via the `KernelEventExt::with_context` extension, which embeds
    /// a `_context` JSON object inside the event payload. The following keys
    /// are consulted: `agent_id`, `tenant_id`, `organization_id`,
    /// `agent_internal_id` and `subject_id`.
    ///
    /// Authenticated user and service subjects must map to a positive SQL
    /// `BIGINT`. Internal `system.agents.*` subjects use the reserved system
    /// actor representation.
    pub fn from_kernel_event(event: &KernelEvent, id: u64) -> KernelResult<Self> {
        let occurred_at = event
            .occurred_at
            .clone()
            .ok_or_else(|| KernelError::validation("audit event occurred_at is required"))?;

        let tenant_id = extract_event_context(event.payload.as_str(), "tenant_id")
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0);
        let organization_id = extract_event_context(event.payload.as_str(), "organization_id")
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0);
        let context_agent_internal_id =
            extract_event_context(event.payload.as_str(), "agent_internal_id")
                .and_then(|value| value.parse::<u64>().ok());
        let context_agent_id = extract_event_context(event.payload.as_str(), "agent_id");
        let aggregate_type = extract_event_context(event.payload.as_str(), "aggregate_type")
            .filter(|value| !is_blank(Some(value.as_str())))
            .ok_or_else(|| KernelError::validation("audit aggregate_type context is required"))?;
        let aggregate_id = extract_event_context(event.payload.as_str(), "aggregate_id")
            .filter(|value| !is_blank(Some(value.as_str())))
            .ok_or_else(|| KernelError::validation("audit aggregate_id context is required"))?;
        if aggregate_type == "agent" && context_agent_id.is_none() {
            return Err(KernelError::validation(
                "agent audit context requires agent_id",
            ));
        }
        // The audit table's agent scope is exclusive to agent aggregates. Other
        // aggregates retain agent context in payload_json without populating the
        // constrained agent_id and agent_internal_id columns.
        let (agent_internal_id, agent_id) = if aggregate_type == "agent" {
            (context_agent_internal_id, context_agent_id)
        } else {
            (None, None)
        };
        let subject_id = extract_event_context(event.payload.as_str(), "subject_id")
            .or_else(|| event.correlation_id.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let (actor_type, actor_id) = audit_actor_from_subject_id(subject_id.as_str())?;

        Ok(Self {
            id,
            uuid: build_storage_uuid("audit-event", tenant_id, &[event.event_id.as_str()]),
            tenant_id,
            organization_id,
            aggregate_type,
            aggregate_id,
            agent_internal_id,
            agent_id,
            action: extract_event_context(event.payload.as_str(), "audit_action").unwrap_or_else(
                || {
                    event
                        .event_type
                        .rsplit('.')
                        .next()
                        .unwrap_or("unknown")
                        .to_string()
                },
            ),
            actor_type,
            actor_id,
            request_id: None,
            trace_id: event
                .trace_context
                .as_ref()
                .map(|trace| trace.trace_id.clone()),
            payload_json: serde_json::to_string(&AuditPayloadSnapshot {
                event_id: event.event_id.clone(),
                event_type: event.event_type.clone(),
                severity: severity_as_str(event.severity).to_string(),
                source: source_as_str(event.source).to_string(),
                payload: event.payload.clone(),
            })
            .map_err(|error| {
                KernelError::validation(format!("invalid audit payload json: {error}"))
            })?,
            created_at: occurred_at,
        })
    }

    pub fn into_kernel_event(self) -> KernelResult<KernelEvent> {
        let payload: AuditPayloadSnapshot = serde_json::from_str(self.payload_json.as_str())
            .map_err(|error| {
                KernelError::validation(format!("invalid audit payload json: {error}"))
            })?;
        Ok(KernelEvent::new(
            payload.event_id,
            payload.event_type,
            severity_from_str(payload.severity.as_str())?,
            payload.payload,
        )
        .from_source(source_from_str(payload.source.as_str())?)
        .occurred_at(self.created_at))
    }

    #[cfg(feature = "postgres-sync")]
    fn from_pg_row(row: &PgRow) -> KernelResult<Self> {
        Ok(Self {
            id: int64_to_u64(row.try_get::<i64, _>("id").map_err(map_sqlx_error)?, "id")?,
            uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
            tenant_id: int64_to_u64(
                row.try_get::<i64, _>("tenant_id").map_err(map_sqlx_error)?,
                "tenant_id",
            )?,
            organization_id: int64_to_u64(
                row.try_get::<i64, _>("organization_id")
                    .map_err(map_sqlx_error)?,
                "organization_id",
            )?,
            aggregate_type: row.try_get("aggregate_type").map_err(map_sqlx_error)?,
            aggregate_id: row.try_get("aggregate_id").map_err(map_sqlx_error)?,
            agent_internal_id: row
                .try_get::<Option<i64>, _>("agent_internal_id")
                .map_err(map_sqlx_error)?
                .map(|value| int64_to_u64(value, "agent_internal_id"))
                .transpose()?,
            agent_id: row.try_get("agent_id").map_err(map_sqlx_error)?,
            action: row.try_get("action").map_err(map_sqlx_error)?,
            actor_type: row.try_get("actor_type").map_err(map_sqlx_error)?,
            actor_id: int64_to_u64(
                row.try_get::<i64, _>("actor_id").map_err(map_sqlx_error)?,
                "actor_id",
            )?,
            request_id: row.try_get("request_id").map_err(map_sqlx_error)?,
            trace_id: row.try_get("trace_id").map_err(map_sqlx_error)?,
            payload_json: row.try_get("payload_json").map_err(map_sqlx_error)?,
            created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentTurnRequestRowsOutcome {
    Inserted {
        session: Box<AgentSessionRow>,
        request_item: Box<AgentSessionItemRow>,
    },
    Existing(Box<AgentTurnRow>),
}

/// Thread-safe PostgreSQL adapter trait.
///
/// All methods use `&self` — implementations MUST use interior mutability
/// (e.g. an `Arc<Mutex<...>>` wrapped pool or a connection pool that
/// internally manages transactional state). This aligns with the stateless
/// `AgentRepository` trait and eliminates the global Mutex bottleneck.
pub trait AgentRepositoryAdapter: Send + Sync {
    fn check_readiness(&self) -> KernelResult<()>;
    fn next_id(&self) -> KernelResult<u64>;
    fn insert_row(&self, row: AgentBusinessRow) -> KernelResult<()>;
    fn update_row(&self, row: AgentBusinessRow) -> KernelResult<()>;
    fn get_row(&self, tenant_id: u64, agent_id: &str) -> KernelResult<Option<AgentBusinessRow>>;
    fn list_rows(&self, query: &AgentListQuery) -> KernelResult<Vec<AgentBusinessRow>>;
    fn count_rows(&self, query: &AgentListQuery) -> KernelResult<u64>;
    fn insert_workspace_row(&self, row: AgentWorkspaceRow) -> KernelResult<()>;
    fn update_workspace_row(&self, row: AgentWorkspaceRow) -> KernelResult<()>;
    fn get_workspace_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
    ) -> KernelResult<Option<AgentWorkspaceRow>>;
    fn get_default_workspace_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
    ) -> KernelResult<Option<AgentWorkspaceRow>>;
    fn list_workspace_rows(
        &self,
        query: &WorkspaceListQuery,
    ) -> KernelResult<Vec<AgentWorkspaceRow>>;
    fn count_workspace_rows(&self, query: &WorkspaceListQuery) -> KernelResult<u64>;
    fn insert_project_row(&self, row: AgentProjectRow) -> KernelResult<()>;
    fn update_project_row(&self, row: AgentProjectRow) -> KernelResult<()>;
    fn get_project_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
    ) -> KernelResult<Option<AgentProjectRow>>;
    fn get_project_row_by_workspace_name(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
        name: &str,
    ) -> KernelResult<Option<AgentProjectRow>>;
    fn get_project_row_by_import_source(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        source_kind: &str,
        source_ref: &str,
    ) -> KernelResult<Option<AgentProjectRow>>;
    fn list_project_rows(&self, query: &ProjectListQuery) -> KernelResult<Vec<AgentProjectRow>>;
    fn count_project_rows(&self, query: &ProjectListQuery) -> KernelResult<u64>;
    fn insert_project_composition_slot_row(
        &self,
        row: AgentProjectCompositionSlotRow,
    ) -> KernelResult<()>;
    fn update_project_composition_slot_row(
        &self,
        row: AgentProjectCompositionSlotRow,
    ) -> KernelResult<()>;
    fn get_project_composition_slot_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentProjectCompositionSlotRow>>;
    fn list_project_composition_slot_rows(
        &self,
        query: &ProjectCompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentProjectCompositionSlotRow>>;
    fn count_project_composition_slot_rows(
        &self,
        query: &ProjectCompositionSlotListQuery,
    ) -> KernelResult<u64>;
    fn insert_provider_binding_row(&self, row: AgentProviderBindingRow) -> KernelResult<()>;
    fn update_provider_binding_row(&self, row: AgentProviderBindingRow) -> KernelResult<()>;
    fn activate_provider_binding_atomic(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
        updated_at: String,
    ) -> KernelResult<AgentProviderBindingRow>;
    fn get_provider_binding_row(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRow>>;
    /// Load the single active provider binding row for an agent via an indexed
    /// `WHERE active = TRUE LIMIT 1` lookup. Returns `None` when no active
    /// binding exists for the (tenant, agent) pair.
    fn get_active_provider_binding_row(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRow>>;
    fn list_provider_binding_rows(
        &self,
        query: &ProviderBindingListQuery,
    ) -> KernelResult<Vec<AgentProviderBindingRow>>;
    fn count_provider_binding_rows(&self, query: &ProviderBindingListQuery) -> KernelResult<u64>;
    fn insert_composition_slot_row(&self, row: AgentCompositionSlotRow) -> KernelResult<()>;
    fn update_composition_slot_row(&self, row: AgentCompositionSlotRow) -> KernelResult<()>;
    fn get_composition_slot_row(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentCompositionSlotRow>>;
    fn list_composition_slot_rows(
        &self,
        query: &CompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRow>>;
    fn count_composition_slot_rows(&self, query: &CompositionSlotListQuery) -> KernelResult<u64>;
    fn list_mcp_marketplace_slot_rows(
        &self,
        query: &McpMarketplaceListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRow>>;
    fn count_mcp_marketplace_slot_rows(&self, query: &McpMarketplaceListQuery)
        -> KernelResult<u64>;

    // Session operations
    fn insert_session_row(&self, row: AgentSessionRow) -> KernelResult<()>;
    fn update_session_row(&self, row: AgentSessionRow) -> KernelResult<()>;
    fn get_session_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRow>>;
    fn get_session_row_by_creation_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentSessionRow>>;
    fn list_session_rows(&self, query: &SessionListQuery) -> KernelResult<Vec<AgentSessionRow>>;
    fn list_session_activity_head_rows(
        &self,
        query: &SessionActivitySummaryListQuery,
    ) -> KernelResult<Vec<AgentSessionActivityHeadRow>>;
    fn count_session_rows(&self, query: &SessionListQuery) -> KernelResult<u64>;
    fn insert_session_runtime_binding_row(
        &self,
        row: AgentSessionRuntimeBindingRow,
    ) -> KernelResult<()>;
    fn update_session_runtime_binding_row(
        &self,
        row: AgentSessionRuntimeBindingRow,
    ) -> KernelResult<()>;
    fn get_session_runtime_binding_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRow>>;
    fn get_current_session_runtime_binding_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRow>>;
    fn list_session_runtime_binding_rows(
        &self,
        query: &SessionRuntimeBindingListQuery,
    ) -> KernelResult<Vec<AgentSessionRuntimeBindingRow>>;
    fn count_session_runtime_binding_rows(
        &self,
        query: &SessionRuntimeBindingListQuery,
    ) -> KernelResult<u64>;
    fn activate_session_runtime_binding_row_atomic(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
        expected_version: u64,
        updated_at: String,
    ) -> KernelResult<AgentSessionRuntimeBindingRow>;
    fn insert_session_checkpoint_row(&self, row: AgentSessionCheckpointRow) -> KernelResult<()>;
    fn update_session_checkpoint_row(&self, row: AgentSessionCheckpointRow) -> KernelResult<()>;
    fn get_session_checkpoint_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        checkpoint_id: &str,
    ) -> KernelResult<Option<AgentSessionCheckpointRow>>;
    fn list_session_checkpoint_rows(
        &self,
        query: &SessionCheckpointListQuery,
    ) -> KernelResult<Vec<AgentSessionCheckpointRow>>;
    fn count_session_checkpoint_rows(
        &self,
        query: &SessionCheckpointListQuery,
    ) -> KernelResult<u64>;
    fn upsert_resource_user_state_row(
        &self,
        row: AgentResourceUserStateRow,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentResourceUserStateRow>;
    fn get_resource_user_state_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        resource_type: AgentResourceType,
        resource_id: &str,
    ) -> KernelResult<Option<AgentResourceUserStateRow>>;
    fn list_resource_user_state_rows(
        &self,
        query: &ResourceUserStateListQuery,
    ) -> KernelResult<Vec<AgentResourceUserStateRow>>;
    fn count_resource_user_state_rows(
        &self,
        query: &ResourceUserStateListQuery,
    ) -> KernelResult<u64>;

    // Session-item operations
    fn append_session_item_row(
        &self,
        row: AgentSessionItemRow,
    ) -> KernelResult<(AgentSessionRow, AgentSessionItemRow)>;
    fn update_session_item_row(&self, row: AgentSessionItemRow) -> KernelResult<()>;
    fn get_session_item_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        item_id: &str,
    ) -> KernelResult<Option<AgentSessionItemRow>>;
    fn list_session_item_rows(
        &self,
        query: &SessionItemListQuery,
    ) -> KernelResult<Vec<AgentSessionItemRow>>;
    fn count_session_item_rows(&self, query: &SessionItemListQuery) -> KernelResult<u64>;
    fn upsert_item_feedback_row(
        &self,
        row: AgentItemFeedbackRow,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentItemFeedbackRow>;
    fn get_item_feedback_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
        user_id: u64,
        include_deleted: bool,
    ) -> KernelResult<Option<AgentItemFeedbackRow>>;
    fn list_item_feedback_rows(
        &self,
        query: &ItemFeedbackListQuery,
    ) -> KernelResult<Vec<AgentItemFeedbackRow>>;
    fn count_item_feedback_rows(&self, query: &ItemFeedbackListQuery) -> KernelResult<u64>;
    fn get_turn_row_by_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentTurnRow>>;
    fn get_turn_row(
        &self,
        _tenant_id: u64,
        _organization_id: u64,
        _turn_id: &str,
    ) -> KernelResult<Option<AgentTurnRow>> {
        Err(KernelError::Internal {
            message: "get_turn_row requires an adapter override".to_string(),
        })
    }

    fn list_turn_rows(&self, _query: &TurnListQuery) -> KernelResult<Vec<AgentTurnRow>> {
        Err(KernelError::Internal {
            message: "list_turn_rows requires an adapter override".to_string(),
        })
    }
    fn count_turn_rows(&self, _query: &TurnListQuery) -> KernelResult<u64> {
        Err(KernelError::Internal {
            message: "count_turn_rows requires an adapter override".to_string(),
        })
    }
    fn list_reconcilable_turn_rows(
        &self,
        _stale_before: &str,
        _limit: usize,
    ) -> KernelResult<Vec<AgentTurnRow>> {
        Err(KernelError::Internal {
            message: "list_reconcilable_turn_rows requires an adapter override".to_string(),
        })
    }
    fn insert_turn_request_rows(
        &self,
        _turn: AgentTurnRow,
        _request_item: AgentSessionItemRow,
        _drive_refs: Vec<AgentItemDriveRefRow>,
    ) -> KernelResult<AgentTurnRequestRowsOutcome> {
        Err(KernelError::Internal {
            message: "insert_turn_request_rows requires a transactional adapter override"
                .to_string(),
        })
    }
    fn update_turn_state_row(
        &self,
        _turn: AgentTurnRow,
        _expected_version: u64,
    ) -> KernelResult<AgentTurnRow> {
        Err(KernelError::Internal {
            message: "update_turn_state_row requires an adapter override".to_string(),
        })
    }
    fn complete_turn_rows(
        &self,
        _turn: AgentTurnRow,
        _expected_turn_version: u64,
        _expected_fencing_token: u64,
        _expected_lease_token: Option<String>,
        _response_item: AgentSessionItemRow,
    ) -> KernelResult<(AgentSessionRow, AgentSessionItemRow)> {
        Err(KernelError::Internal {
            message: "complete_turn_rows requires a transactional adapter override".to_string(),
        })
    }
    fn list_item_drive_ref_rows(
        &self,
        _tenant_id: u64,
        _organization_id: u64,
        _item_id: &str,
    ) -> KernelResult<Vec<AgentItemDriveRefRow>> {
        Err(KernelError::Internal {
            message: "list_item_drive_ref_rows requires an adapter override".to_string(),
        })
    }
    fn list_item_drive_ref_rows_batch(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_ids: &[String],
    ) -> KernelResult<Vec<AgentItemDriveRefRow>> {
        let mut rows = Vec::new();
        for item_id in item_ids {
            rows.extend(self.list_item_drive_ref_rows(tenant_id, organization_id, item_id)?);
        }
        Ok(rows)
    }

    // Interaction operations
    fn insert_interaction_row(&self, row: AgentInteractionRow) -> KernelResult<()>;
    fn update_interaction_row(&self, row: AgentInteractionRow) -> KernelResult<()>;
    fn get_interaction_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        interaction_id: &str,
    ) -> KernelResult<Option<AgentInteractionRow>>;
    fn list_interaction_rows(
        &self,
        query: &InteractionListQuery,
    ) -> KernelResult<Vec<AgentInteractionRow>>;
    fn count_interaction_rows(&self, query: &InteractionListQuery) -> KernelResult<u64>;

    // Task operations
    fn insert_task_row(&self, row: AgentTaskRow) -> KernelResult<()>;
    fn update_task_row(&self, row: AgentTaskRow) -> KernelResult<()>;
    fn get_task_row(&self, tenant_id: u64, task_id: &str) -> KernelResult<Option<AgentTaskRow>>;
    fn list_task_rows(&self, query: &TaskListQuery) -> KernelResult<Vec<AgentTaskRow>>;
    fn count_task_rows(&self, query: &TaskListQuery) -> KernelResult<u64>;
}

pub struct SqlAgentRepository<A>
where
    A: AgentRepositoryAdapter,
{
    adapter: A,
}

impl<A> SqlAgentRepository<A>
where
    A: AgentRepositoryAdapter,
{
    pub fn new(adapter: A) -> Self {
        Self { adapter }
    }
}

impl<A> AgentRepository for SqlAgentRepository<A>
where
    A: AgentRepositoryAdapter,
{
    fn check_readiness(&self) -> KernelResult<()> {
        self.adapter.check_readiness()
    }

    fn next_id(&self) -> KernelResult<u64> {
        self.adapter.next_id()
    }

    fn insert(&self, record: AgentBusinessRecord) -> KernelResult<()> {
        self.adapter
            .insert_row(AgentBusinessRow::from_record(&record)?)
    }

    fn update(&self, record: AgentBusinessRecord) -> KernelResult<()> {
        self.adapter
            .update_row(AgentBusinessRow::from_record(&record)?)
    }

    fn get(&self, tenant_id: u64, agent_id: &str) -> KernelResult<Option<AgentBusinessRecord>> {
        self.adapter
            .get_row(tenant_id, agent_id)?
            .map(AgentBusinessRow::into_record)
            .transpose()
    }

    fn list(&self, query: &AgentListQuery) -> KernelResult<Vec<AgentBusinessRecord>> {
        self.adapter
            .list_rows(query)?
            .into_iter()
            .map(AgentBusinessRow::into_record)
            .collect()
    }

    fn count_agents(&self, query: &AgentListQuery) -> KernelResult<u64> {
        self.adapter.count_rows(query)
    }

    fn insert_workspace(&self, record: AgentWorkspaceRecord) -> KernelResult<()> {
        self.adapter
            .insert_workspace_row(AgentWorkspaceRow::from_record(&record))
    }

    fn update_workspace(&self, record: AgentWorkspaceRecord) -> KernelResult<()> {
        self.adapter
            .update_workspace_row(AgentWorkspaceRow::from_record(&record))
    }

    fn get_workspace(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
    ) -> KernelResult<Option<AgentWorkspaceRecord>> {
        self.adapter
            .get_workspace_row(tenant_id, organization_id, workspace_id)?
            .map(AgentWorkspaceRow::into_record)
            .transpose()
    }

    fn get_default_workspace(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
    ) -> KernelResult<Option<AgentWorkspaceRecord>> {
        self.adapter
            .get_default_workspace_row(tenant_id, organization_id, owner_user_id)?
            .map(AgentWorkspaceRow::into_record)
            .transpose()
    }

    fn list_workspaces(
        &self,
        query: &WorkspaceListQuery,
    ) -> KernelResult<Vec<AgentWorkspaceRecord>> {
        self.adapter
            .list_workspace_rows(query)?
            .into_iter()
            .map(AgentWorkspaceRow::into_record)
            .collect()
    }

    fn count_workspaces(&self, query: &WorkspaceListQuery) -> KernelResult<u64> {
        self.adapter.count_workspace_rows(query)
    }

    fn insert_project(&self, record: AgentProjectRecord) -> KernelResult<()> {
        self.adapter
            .insert_project_row(AgentProjectRow::from_record(&record))
    }

    fn update_project(&self, record: AgentProjectRecord) -> KernelResult<()> {
        self.adapter
            .update_project_row(AgentProjectRow::from_record(&record))
    }

    fn get_project(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
    ) -> KernelResult<Option<AgentProjectRecord>> {
        self.adapter
            .get_project_row(tenant_id, organization_id, project_id)?
            .map(AgentProjectRow::into_record)
            .transpose()
    }

    fn get_project_by_workspace_name(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
        name: &str,
    ) -> KernelResult<Option<AgentProjectRecord>> {
        self.adapter
            .get_project_row_by_workspace_name(tenant_id, organization_id, workspace_id, name)?
            .map(AgentProjectRow::into_record)
            .transpose()
    }

    fn get_project_by_import_source(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        source_kind: &str,
        source_ref: &str,
    ) -> KernelResult<Option<AgentProjectRecord>> {
        self.adapter
            .get_project_row_by_import_source(
                tenant_id,
                organization_id,
                owner_user_id,
                source_kind,
                source_ref,
            )?
            .map(AgentProjectRow::into_record)
            .transpose()
    }

    fn list_projects(&self, query: &ProjectListQuery) -> KernelResult<Vec<AgentProjectRecord>> {
        self.adapter
            .list_project_rows(query)?
            .into_iter()
            .map(AgentProjectRow::into_record)
            .collect()
    }

    fn count_projects(&self, query: &ProjectListQuery) -> KernelResult<u64> {
        self.adapter.count_project_rows(query)
    }

    fn insert_project_composition_slot(
        &self,
        record: AgentProjectCompositionSlotRecord,
    ) -> KernelResult<()> {
        self.adapter.insert_project_composition_slot_row(
            AgentProjectCompositionSlotRow::from_record(&record),
        )
    }

    fn update_project_composition_slot(
        &self,
        record: AgentProjectCompositionSlotRecord,
    ) -> KernelResult<()> {
        self.adapter.update_project_composition_slot_row(
            AgentProjectCompositionSlotRow::from_record(&record),
        )
    }

    fn get_project_composition_slot(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentProjectCompositionSlotRecord>> {
        self.adapter
            .get_project_composition_slot_row(tenant_id, organization_id, project_id, slot_id)?
            .map(AgentProjectCompositionSlotRow::into_record)
            .transpose()
    }

    fn list_project_composition_slots(
        &self,
        query: &ProjectCompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentProjectCompositionSlotRecord>> {
        self.adapter
            .list_project_composition_slot_rows(query)?
            .into_iter()
            .map(AgentProjectCompositionSlotRow::into_record)
            .collect()
    }

    fn count_project_composition_slots(
        &self,
        query: &ProjectCompositionSlotListQuery,
    ) -> KernelResult<u64> {
        self.adapter.count_project_composition_slot_rows(query)
    }

    fn insert_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        self.adapter
            .insert_provider_binding_row(AgentProviderBindingRow::from_record(&record)?)
    }

    fn update_provider_binding(&self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        self.adapter
            .update_provider_binding_row(AgentProviderBindingRow::from_record(&record)?)
    }

    fn get_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRecord>> {
        self.adapter
            .get_provider_binding_row(tenant_id, agent_id, binding_id)?
            .map(AgentProviderBindingRow::into_record)
            .transpose()
    }

    fn get_active_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRecord>> {
        self.adapter
            .get_active_provider_binding_row(tenant_id, agent_id)?
            .map(AgentProviderBindingRow::into_record)
            .transpose()
    }

    fn list_provider_bindings(
        &self,
        query: &ProviderBindingListQuery,
    ) -> KernelResult<Vec<AgentProviderBindingRecord>> {
        self.adapter
            .list_provider_binding_rows(query)?
            .into_iter()
            .map(AgentProviderBindingRow::into_record)
            .collect()
    }

    fn count_provider_bindings(&self, query: &ProviderBindingListQuery) -> KernelResult<u64> {
        self.adapter.count_provider_binding_rows(query)
    }

    fn activate_provider_binding_atomic(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
        updated_at: String,
    ) -> KernelResult<AgentProviderBindingRecord> {
        self.adapter
            .activate_provider_binding_atomic(tenant_id, agent_id, binding_id, updated_at)
            .and_then(|row| row.into_record())
    }

    fn insert_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        self.adapter
            .insert_composition_slot_row(AgentCompositionSlotRow::from_record(&record)?)
    }

    fn update_composition_slot(&self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        self.adapter
            .update_composition_slot_row(AgentCompositionSlotRow::from_record(&record)?)
    }

    fn get_composition_slot(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentCompositionSlotRecord>> {
        self.adapter
            .get_composition_slot_row(tenant_id, agent_id, slot_id)?
            .map(AgentCompositionSlotRow::into_record)
            .transpose()
    }

    fn list_composition_slots(
        &self,
        query: &CompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>> {
        self.adapter
            .list_composition_slot_rows(query)?
            .into_iter()
            .map(AgentCompositionSlotRow::into_record)
            .collect()
    }

    fn count_composition_slots(&self, query: &CompositionSlotListQuery) -> KernelResult<u64> {
        self.adapter.count_composition_slot_rows(query)
    }

    fn list_mcp_marketplace_slots(
        &self,
        query: &McpMarketplaceListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>> {
        self.adapter
            .list_mcp_marketplace_slot_rows(query)?
            .into_iter()
            .map(AgentCompositionSlotRow::into_record)
            .collect()
    }

    fn count_mcp_marketplace_slots(&self, query: &McpMarketplaceListQuery) -> KernelResult<u64> {
        self.adapter.count_mcp_marketplace_slot_rows(query)
    }

    // -----------------------------------------------------------------------
    // Session persistence
    // -----------------------------------------------------------------------

    fn insert_session(&self, record: AgentSessionRecord) -> KernelResult<()> {
        self.adapter
            .insert_session_row(AgentSessionRow::from_record(&record)?)
    }

    fn update_session(&self, record: AgentSessionRecord) -> KernelResult<()> {
        self.adapter
            .update_session_row(AgentSessionRow::from_record(&record)?)
    }

    fn get_session(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRecord>> {
        self.adapter
            .get_session_row(tenant_id, organization_id, session_id)?
            .map(AgentSessionRow::into_record)
            .transpose()
    }

    fn get_session_by_creation_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentSessionRecord>> {
        self.adapter
            .get_session_row_by_creation_idempotency(
                tenant_id,
                organization_id,
                owner_user_id,
                idempotency_key,
            )?
            .map(AgentSessionRow::into_record)
            .transpose()
    }

    fn list_sessions(&self, query: &SessionListQuery) -> KernelResult<Vec<AgentSessionRecord>> {
        self.adapter
            .list_session_rows(query)?
            .into_iter()
            .map(AgentSessionRow::into_record)
            .collect()
    }

    fn count_sessions(&self, query: &SessionListQuery) -> KernelResult<u64> {
        self.adapter.count_session_rows(query)
    }

    fn list_session_activity_summaries(
        &self,
        query: &SessionActivitySummaryListQuery,
    ) -> KernelResult<crate::ports::PaginatedResult<SessionActivitySummaryRecord>> {
        let mut heads = self.adapter.list_session_activity_head_rows(query)?;
        let has_more = heads.len() > query.page_size;
        if has_more {
            heads.pop();
        }
        let mut items = Vec::with_capacity(heads.len());
        for head in heads {
            let session = head.session.into_record()?;
            let latest_turn = head
                .latest_turn
                .map(AgentTurnRow::into_record)
                .transpose()?;
            let pending_interaction = head
                .pending_interaction
                .map(AgentInteractionRow::into_record)
                .transpose()?;
            let current_runtime_binding = head
                .current_runtime_binding
                .map(AgentSessionRuntimeBindingRow::into_record)
                .transpose()?;
            let latest_runtime_binding = head
                .latest_runtime_binding
                .map(AgentSessionRuntimeBindingRow::into_record)
                .transpose()?;
            let user_state = head
                .user_state
                .map(AgentResourceUserStateRow::into_record)
                .transpose()?;
            items.push(SessionActivitySummaryRecord::from_parts(
                SessionActivitySummaryParts {
                    session,
                    latest_turn,
                    pending_interaction,
                    current_runtime_binding,
                    latest_runtime_binding,
                    user_state,
                    latest_interaction_component: head
                        .latest_interaction_id
                        .zip(head.latest_interaction_version),
                    activity_at: head.activity_at,
                    activity_source: head.activity_source,
                },
            ));
        }
        let next_page_token = if has_more {
            items
                .last()
                .map(|summary| SessionActivityCursor {
                    activity_at: summary.freshness.activity_at.clone(),
                    session_internal_id: summary.session.id,
                    scope_fingerprint: query.scope_fingerprint(),
                })
                .map(|cursor| encode_session_activity_cursor(&cursor))
                .transpose()?
        } else if items.is_empty() {
            query
                .cursor
                .as_ref()
                .map(encode_session_activity_cursor)
                .transpose()?
        } else {
            None
        };
        Ok(crate::ports::PaginatedResult {
            items,
            next_page_token,
            total_count: None,
            has_more,
        })
    }

    fn insert_session_runtime_binding(
        &self,
        record: AgentSessionRuntimeBindingRecord,
    ) -> KernelResult<()> {
        self.adapter
            .insert_session_runtime_binding_row(AgentSessionRuntimeBindingRow::from_record(&record))
    }

    fn update_session_runtime_binding(
        &self,
        record: AgentSessionRuntimeBindingRecord,
    ) -> KernelResult<()> {
        self.adapter
            .update_session_runtime_binding_row(AgentSessionRuntimeBindingRow::from_record(&record))
    }

    fn get_session_runtime_binding(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRecord>> {
        self.adapter
            .get_session_runtime_binding_row(
                tenant_id,
                organization_id,
                session_id,
                runtime_binding_id,
            )?
            .map(AgentSessionRuntimeBindingRow::into_record)
            .transpose()
    }

    fn get_current_session_runtime_binding(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRecord>> {
        self.adapter
            .get_current_session_runtime_binding_row(tenant_id, organization_id, session_id)?
            .map(AgentSessionRuntimeBindingRow::into_record)
            .transpose()
    }

    fn list_session_runtime_bindings(
        &self,
        query: &SessionRuntimeBindingListQuery,
    ) -> KernelResult<Vec<AgentSessionRuntimeBindingRecord>> {
        self.adapter
            .list_session_runtime_binding_rows(query)?
            .into_iter()
            .map(AgentSessionRuntimeBindingRow::into_record)
            .collect()
    }

    fn count_session_runtime_bindings(
        &self,
        query: &SessionRuntimeBindingListQuery,
    ) -> KernelResult<u64> {
        self.adapter.count_session_runtime_binding_rows(query)
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
        self.adapter
            .activate_session_runtime_binding_row_atomic(
                tenant_id,
                organization_id,
                session_id,
                runtime_binding_id,
                expected_version,
                updated_at,
            )?
            .into_record()
    }

    fn insert_session_checkpoint(&self, record: AgentSessionCheckpointRecord) -> KernelResult<()> {
        self.adapter
            .insert_session_checkpoint_row(AgentSessionCheckpointRow::from_record(&record))
    }

    fn update_session_checkpoint(&self, record: AgentSessionCheckpointRecord) -> KernelResult<()> {
        self.adapter
            .update_session_checkpoint_row(AgentSessionCheckpointRow::from_record(&record))
    }

    fn get_session_checkpoint(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        checkpoint_id: &str,
    ) -> KernelResult<Option<AgentSessionCheckpointRecord>> {
        self.adapter
            .get_session_checkpoint_row(tenant_id, organization_id, session_id, checkpoint_id)?
            .map(AgentSessionCheckpointRow::into_record)
            .transpose()
    }

    fn list_session_checkpoints(
        &self,
        query: &SessionCheckpointListQuery,
    ) -> KernelResult<Vec<AgentSessionCheckpointRecord>> {
        self.adapter
            .list_session_checkpoint_rows(query)?
            .into_iter()
            .map(AgentSessionCheckpointRow::into_record)
            .collect()
    }

    fn count_session_checkpoints(&self, query: &SessionCheckpointListQuery) -> KernelResult<u64> {
        self.adapter.count_session_checkpoint_rows(query)
    }

    fn upsert_resource_user_state(
        &self,
        record: AgentResourceUserStateRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentResourceUserStateRecord> {
        self.adapter
            .upsert_resource_user_state_row(
                AgentResourceUserStateRow::from_record(&record),
                expected_version,
            )?
            .into_record()
    }

    fn get_resource_user_state(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        resource_type: AgentResourceType,
        resource_id: &str,
    ) -> KernelResult<Option<AgentResourceUserStateRecord>> {
        self.adapter
            .get_resource_user_state_row(
                tenant_id,
                organization_id,
                user_id,
                resource_type,
                resource_id,
            )?
            .map(AgentResourceUserStateRow::into_record)
            .transpose()
    }

    fn list_resource_user_states(
        &self,
        query: &ResourceUserStateListQuery,
    ) -> KernelResult<Vec<AgentResourceUserStateRecord>> {
        self.adapter
            .list_resource_user_state_rows(query)?
            .into_iter()
            .map(AgentResourceUserStateRow::into_record)
            .collect()
    }

    fn count_resource_user_states(&self, query: &ResourceUserStateListQuery) -> KernelResult<u64> {
        self.adapter.count_resource_user_state_rows(query)
    }

    // -----------------------------------------------------------------------
    // Session-item persistence
    // -----------------------------------------------------------------------

    fn append_session_item(
        &self,
        record: AgentSessionItemRecord,
    ) -> KernelResult<(AgentSessionRecord, AgentSessionItemRecord)> {
        let (session_row, item_row) = self
            .adapter
            .append_session_item_row(AgentSessionItemRow::from_record(&record)?)?;
        Ok((session_row.into_record()?, item_row.into_record()?))
    }

    fn update_session_item(&self, record: AgentSessionItemRecord) -> KernelResult<()> {
        self.adapter
            .update_session_item_row(AgentSessionItemRow::from_record(&record)?)
    }

    fn get_session_item(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        item_id: &str,
    ) -> KernelResult<Option<AgentSessionItemRecord>> {
        self.adapter
            .get_session_item_row(tenant_id, organization_id, session_id, item_id)?
            .map(AgentSessionItemRow::into_record)
            .transpose()
    }

    fn list_session_items(
        &self,
        query: &SessionItemListQuery,
    ) -> KernelResult<Vec<AgentSessionItemRecord>> {
        self.adapter
            .list_session_item_rows(query)?
            .into_iter()
            .map(AgentSessionItemRow::into_record)
            .collect()
    }

    fn count_session_items(&self, query: &SessionItemListQuery) -> KernelResult<u64> {
        self.adapter.count_session_item_rows(query)
    }

    fn upsert_item_feedback(
        &self,
        record: AgentItemFeedbackRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentItemFeedbackRecord> {
        self.adapter
            .upsert_item_feedback_row(AgentItemFeedbackRow::from_record(&record), expected_version)?
            .into_record()
    }

    fn get_item_feedback(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
        user_id: u64,
        include_deleted: bool,
    ) -> KernelResult<Option<AgentItemFeedbackRecord>> {
        self.adapter
            .get_item_feedback_row(
                tenant_id,
                organization_id,
                item_id,
                user_id,
                include_deleted,
            )?
            .map(AgentItemFeedbackRow::into_record)
            .transpose()
    }

    fn list_item_feedback(
        &self,
        query: &ItemFeedbackListQuery,
    ) -> KernelResult<Vec<AgentItemFeedbackRecord>> {
        self.adapter
            .list_item_feedback_rows(query)?
            .into_iter()
            .map(AgentItemFeedbackRow::into_record)
            .collect()
    }

    fn count_item_feedback(&self, query: &ItemFeedbackListQuery) -> KernelResult<u64> {
        self.adapter.count_item_feedback_rows(query)
    }

    fn get_turn_by_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentTurnRecord>> {
        self.adapter
            .get_turn_row_by_idempotency(
                tenant_id,
                organization_id,
                owner_user_id,
                idempotency_key,
            )?
            .map(AgentTurnRow::into_record)
            .transpose()
    }

    fn get_turn(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<Option<AgentTurnRecord>> {
        self.adapter
            .get_turn_row(tenant_id, organization_id, turn_id)?
            .map(AgentTurnRow::into_record)
            .transpose()
    }

    fn list_turns(&self, query: &TurnListQuery) -> KernelResult<Vec<AgentTurnRecord>> {
        self.adapter
            .list_turn_rows(query)?
            .into_iter()
            .map(AgentTurnRow::into_record)
            .collect()
    }

    fn count_turns(&self, query: &TurnListQuery) -> KernelResult<u64> {
        self.adapter.count_turn_rows(query)
    }

    fn list_reconcilable_turns(
        &self,
        stale_before: &str,
        limit: usize,
    ) -> KernelResult<Vec<AgentTurnRecord>> {
        self.adapter
            .list_reconcilable_turn_rows(stale_before, limit)?
            .into_iter()
            .map(AgentTurnRow::into_record)
            .collect()
    }

    fn insert_turn_request(
        &self,
        turn: AgentTurnRecord,
        request_item: AgentSessionItemRecord,
        drive_refs: Vec<AgentItemDriveRefRecord>,
    ) -> KernelResult<TurnRequestWriteOutcome> {
        let turn_row = AgentTurnRow::from_record(&turn);
        let request_item_row = AgentSessionItemRow::from_record(&request_item)?;
        let drive_ref_rows = drive_refs
            .iter()
            .map(AgentItemDriveRefRow::from_record)
            .collect();
        match self
            .adapter
            .insert_turn_request_rows(turn_row, request_item_row, drive_ref_rows)?
        {
            AgentTurnRequestRowsOutcome::Inserted {
                session,
                request_item,
            } => Ok(TurnRequestWriteOutcome::Inserted {
                session: Box::new((*session).into_record()?),
                request_item: Box::new((*request_item).into_record()?),
            }),
            AgentTurnRequestRowsOutcome::Existing(turn) => Ok(TurnRequestWriteOutcome::Existing(
                Box::new((*turn).into_record()?),
            )),
        }
    }

    fn update_turn_state(
        &self,
        turn: AgentTurnRecord,
        expected_version: u64,
    ) -> KernelResult<AgentTurnRecord> {
        self.adapter
            .update_turn_state_row(AgentTurnRow::from_record(&turn), expected_version)?
            .into_record()
    }

    fn complete_turn(
        &self,
        turn: AgentTurnRecord,
        expected_turn_version: u64,
        expected_fencing_token: u64,
        expected_lease_token: Option<String>,
        response_item: AgentSessionItemRecord,
    ) -> KernelResult<(AgentSessionRecord, AgentSessionItemRecord)> {
        let turn_row = AgentTurnRow::from_record(&turn);
        let response_item_row = AgentSessionItemRow::from_record(&response_item)?;
        let (session_row, response_item_row) = self.adapter.complete_turn_rows(
            turn_row,
            expected_turn_version,
            expected_fencing_token,
            expected_lease_token,
            response_item_row,
        )?;
        Ok((session_row.into_record()?, response_item_row.into_record()?))
    }

    fn list_item_drive_refs(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
    ) -> KernelResult<Vec<AgentItemDriveRefRecord>> {
        self.adapter
            .list_item_drive_ref_rows(tenant_id, organization_id, item_id)?
            .into_iter()
            .map(AgentItemDriveRefRow::into_record)
            .collect()
    }

    fn list_item_drive_refs_batch(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_ids: &[String],
    ) -> KernelResult<Vec<AgentItemDriveRefRecord>> {
        self.adapter
            .list_item_drive_ref_rows_batch(tenant_id, organization_id, item_ids)?
            .into_iter()
            .map(AgentItemDriveRefRow::into_record)
            .collect()
    }

    // -----------------------------------------------------------------------
    // Interaction persistence
    // -----------------------------------------------------------------------

    fn insert_interaction(&self, record: AgentInteractionRecord) -> KernelResult<()> {
        self.adapter
            .insert_interaction_row(AgentInteractionRow::from_record(&record)?)
    }

    fn update_interaction(&self, record: AgentInteractionRecord) -> KernelResult<()> {
        self.adapter
            .update_interaction_row(AgentInteractionRow::from_record(&record)?)
    }

    fn get_interaction(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        interaction_id: &str,
    ) -> KernelResult<Option<AgentInteractionRecord>> {
        self.adapter
            .get_interaction_row(tenant_id, organization_id, session_id, interaction_id)?
            .map(AgentInteractionRow::into_record)
            .transpose()
    }

    fn list_interactions(
        &self,
        query: &InteractionListQuery,
    ) -> KernelResult<Vec<AgentInteractionRecord>> {
        self.adapter
            .list_interaction_rows(query)?
            .into_iter()
            .map(AgentInteractionRow::into_record)
            .collect()
    }

    fn count_interactions(&self, query: &InteractionListQuery) -> KernelResult<u64> {
        self.adapter.count_interaction_rows(query)
    }

    // -----------------------------------------------------------------------
    // Task persistence
    // -----------------------------------------------------------------------

    fn insert_task(&self, record: AgentTaskRecord) -> KernelResult<()> {
        self.adapter
            .insert_task_row(AgentTaskRow::from_record(&record)?)
    }

    fn update_task(&self, record: AgentTaskRecord) -> KernelResult<()> {
        self.adapter
            .update_task_row(AgentTaskRow::from_record(&record)?)
    }

    fn get_task(&self, tenant_id: u64, task_id: &str) -> KernelResult<Option<AgentTaskRecord>> {
        self.adapter
            .get_task_row(tenant_id, task_id)?
            .map(AgentTaskRow::into_record)
            .transpose()
    }

    fn list_tasks(&self, query: &TaskListQuery) -> KernelResult<Vec<AgentTaskRecord>> {
        self.adapter
            .list_task_rows(query)?
            .into_iter()
            .map(AgentTaskRow::into_record)
            .collect()
    }

    fn count_tasks(&self, query: &TaskListQuery) -> KernelResult<u64> {
        self.adapter.count_task_rows(query)
    }
}

pub trait AgentAuditAdapter: Send + Sync {
    fn next_id(&self) -> KernelResult<u64>;
    fn insert_audit_row(&self, row: AgentAuditEventRow) -> KernelResult<()>;
    fn list_audit_rows(&self, query: &AuditEventListQuery)
        -> KernelResult<Vec<AgentAuditEventRow>>;
    fn count_audit_rows(&self, query: &AuditEventListQuery) -> KernelResult<u64>;
}

#[cfg(feature = "postgres-sync")]
pub const AGENTS_DATABASE_SERVICE: &str = "AGENTS";

#[cfg(feature = "postgres-sync")]
pub struct SyncPostgresAdapter {
    pool: BlockingPostgresPool,
    id_generator: AgentBusinessIdGenerator,
}

#[cfg(feature = "postgres-sync")]
impl SyncPostgresAdapter {
    pub fn connect(connection_uri: &str) -> KernelResult<Self> {
        Ok(Self {
            pool: BlockingPostgresPool::connect(connection_uri)?,
            id_generator: AgentBusinessIdGenerator::new_default()?,
        })
    }

    /// Connects through the canonical `sdkwork-database-config` service profile.
    pub fn connect_from_sdkwork_env(service_name: &str) -> KernelResult<Self> {
        Ok(Self {
            pool: BlockingPostgresPool::connect_from_sdkwork_env(service_name)?,
            id_generator: AgentBusinessIdGenerator::new_default()?,
        })
    }

    /// Connects to the canonical Agents PostgreSQL database.
    pub fn connect_from_agents_database_env() -> KernelResult<Self> {
        Self::connect_from_sdkwork_env(AGENTS_DATABASE_SERVICE)
    }

    pub fn from_pool(pool: BlockingPostgresPool) -> KernelResult<Self> {
        Ok(Self {
            pool,
            id_generator: AgentBusinessIdGenerator::new_default()?,
        })
    }

    pub fn with_pool_and_id_generator(
        pool: BlockingPostgresPool,
        id_generator: AgentBusinessIdGenerator,
    ) -> Self {
        Self { pool, id_generator }
    }

    /// Borrow the underlying postgres pool. Allows callers (e.g. the
    /// production bootstrap) to share a single physical pool across the
    /// repository adapter and a separate audit-sink adapter that uses a
    /// dedicated snowflake node id.
    pub fn pool(&self) -> &BlockingPostgresPool {
        &self.pool
    }

    fn with_pool<T>(
        &self,
        action: impl FnOnce(&BlockingPostgresPool) -> KernelResult<T>,
    ) -> KernelResult<T> {
        action(&self.pool)
    }
}

#[cfg(feature = "postgres-sync")]
fn transaction_error(error: KernelError) -> sqlx::Error {
    let prefix = match error.kind() {
        KernelErrorKind::ValidationError => "sdkwork-domain-validation:",
        KernelErrorKind::Conflict => "sdkwork-domain-conflict:",
        _ => "sdkwork-domain-internal:",
    };
    sqlx::Error::Protocol(format!("{prefix}{}", error.message()))
}

#[cfg(feature = "postgres-sync")]
async fn record_session_item_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    item: &AgentSessionItemRow,
    require_active_session: bool,
) -> Result<AgentSessionRow, sqlx::Error> {
    let tenant_id = u64_to_i64(item.tenant_id, "item.tenant_id").map_err(transaction_error)?;
    let organization_id =
        u64_to_i64(item.organization_id, "item.organization_id").map_err(transaction_error)?;
    let input_tokens =
        u64_to_i64(item.input_tokens, "item.input_tokens").map_err(transaction_error)?;
    let output_tokens =
        u64_to_i64(item.output_tokens, "item.output_tokens").map_err(transaction_error)?;
    let updated_by = u64_to_i64(item.created_by, "item.created_by").map_err(transaction_error)?;
    let row = sqlx::query(SQL_RECORD_AGENT_SESSION_ITEM)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(&item.session_id)
        .bind(input_tokens)
        .bind(output_tokens)
        .bind(updated_by)
        .bind(&item.updated_at)
        .bind(require_active_session)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| {
            transaction_error(KernelError::validation(if require_active_session {
                "active session not found"
            } else {
                "session not found"
            }))
        })?;
    pg_row_to_agent_session_row(row).map_err(transaction_error)
}

#[cfg(feature = "postgres-sync")]
async fn insert_turn_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    turn: &AgentTurnRow,
) -> Result<bool, sqlx::Error> {
    let id = u64_to_i64(turn.id, "turn.id").map_err(transaction_error)?;
    let tenant_id = u64_to_i64(turn.tenant_id, "turn.tenant_id").map_err(transaction_error)?;
    let organization_id =
        u64_to_i64(turn.organization_id, "turn.organization_id").map_err(transaction_error)?;
    let owner_user_id =
        u64_to_i64(turn.owner_user_id, "turn.owner_user_id").map_err(transaction_error)?;
    let input_tokens =
        u64_to_i64(turn.input_tokens, "turn.input_tokens").map_err(transaction_error)?;
    let output_tokens =
        u64_to_i64(turn.output_tokens, "turn.output_tokens").map_err(transaction_error)?;
    let cached_tokens =
        u64_to_i64(turn.cached_tokens, "turn.cached_tokens").map_err(transaction_error)?;
    let attempt_count = i32::try_from(turn.attempt_count)
        .map_err(|_| transaction_error(KernelError::validation("turn.attempt_count overflow")))?;
    let max_attempts = i32::try_from(turn.max_attempts)
        .map_err(|_| transaction_error(KernelError::validation("turn.max_attempts overflow")))?;
    let fencing_token =
        u64_to_i64(turn.fencing_token, "turn.fencing_token").map_err(transaction_error)?;
    let version = u64_to_i64(turn.version, "turn.version").map_err(transaction_error)?;
    let inserted = sqlx::query(SQL_INSERT_AGENT_TURN)
        .bind(id)
        .bind(&turn.uuid)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(&turn.turn_id)
        .bind(&turn.session_id)
        .bind(&turn.agent_id)
        .bind(owner_user_id)
        .bind(&turn.runtime_binding_id)
        .bind(&turn.client_request_id)
        .bind(&turn.idempotency_key)
        .bind(&turn.payload_hash)
        .bind(&turn.request_item_id)
        .bind(&turn.response_item_id)
        .bind(turn.turn_mode)
        .bind(turn.status)
        .bind(&turn.requested_model_id)
        .bind(&turn.provider_binding_id)
        .bind(&turn.model_id)
        .bind(&turn.provider_id)
        .bind(input_tokens)
        .bind(output_tokens)
        .bind(cached_tokens)
        .bind(&turn.finish_reason)
        .bind(&turn.error_code)
        .bind(&turn.error_detail)
        .bind(&turn.trace_id)
        .bind(attempt_count)
        .bind(max_attempts)
        .bind(&turn.next_retry_at)
        .bind(&turn.available_at)
        .bind(&turn.lease_owner)
        .bind(&turn.lease_token)
        .bind(&turn.lease_expires_at)
        .bind(fencing_token)
        .bind(version)
        .bind(&turn.created_at)
        .bind(&turn.updated_at)
        .bind(&turn.started_at)
        .bind(&turn.completed_at)
        .bind(&turn.cancel_requested_at)
        .bind(&turn.cancelled_at)
        .bind(&turn.retention_until)
        .execute(&mut **tx)
        .await?
        .rows_affected();
    Ok(inserted == 1)
}

#[cfg(feature = "postgres-sync")]
async fn get_turn_by_idempotency_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    turn: &AgentTurnRow,
) -> Result<Option<AgentTurnRow>, sqlx::Error> {
    let tenant_id = u64_to_i64(turn.tenant_id, "turn.tenant_id").map_err(transaction_error)?;
    let organization_id =
        u64_to_i64(turn.organization_id, "turn.organization_id").map_err(transaction_error)?;
    let owner_user_id =
        u64_to_i64(turn.owner_user_id, "turn.owner_user_id").map_err(transaction_error)?;
    sqlx::query(SQL_SELECT_AGENT_TURN_BY_IDEMPOTENCY)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(owner_user_id)
        .bind(&turn.idempotency_key)
        .fetch_optional(&mut **tx)
        .await?
        .map(pg_row_to_agent_turn_row)
        .transpose()
        .map_err(transaction_error)
}

#[cfg(feature = "postgres-sync")]
async fn complete_turn_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    turn: &AgentTurnRow,
    expected_version: u64,
    expected_fencing_token: u64,
    expected_lease_token: &Option<String>,
) -> Result<(), sqlx::Error> {
    let input_tokens =
        u64_to_i64(turn.input_tokens, "turn.input_tokens").map_err(transaction_error)?;
    let output_tokens =
        u64_to_i64(turn.output_tokens, "turn.output_tokens").map_err(transaction_error)?;
    let cached_tokens =
        u64_to_i64(turn.cached_tokens, "turn.cached_tokens").map_err(transaction_error)?;
    let attempt_count = i32::try_from(turn.attempt_count)
        .map_err(|_| transaction_error(KernelError::validation("turn.attempt_count overflow")))?;
    let max_attempts = i32::try_from(turn.max_attempts)
        .map_err(|_| transaction_error(KernelError::validation("turn.max_attempts overflow")))?;
    let fencing_token =
        u64_to_i64(turn.fencing_token, "turn.fencing_token").map_err(transaction_error)?;
    let version = u64_to_i64(turn.version, "turn.version").map_err(transaction_error)?;
    let tenant_id = u64_to_i64(turn.tenant_id, "turn.tenant_id").map_err(transaction_error)?;
    let organization_id =
        u64_to_i64(turn.organization_id, "turn.organization_id").map_err(transaction_error)?;
    let expected_version =
        u64_to_i64(expected_version, "turn.expected_version").map_err(transaction_error)?;
    let expected_fencing_token = u64_to_i64(expected_fencing_token, "turn.expected_fencing_token")
        .map_err(transaction_error)?;
    let affected = sqlx::query(SQL_COMPLETE_AGENT_TURN_STATE)
        .bind(&turn.response_item_id)
        .bind(&turn.runtime_binding_id)
        .bind(turn.turn_mode)
        .bind(turn.status)
        .bind(&turn.requested_model_id)
        .bind(&turn.provider_binding_id)
        .bind(&turn.model_id)
        .bind(&turn.provider_id)
        .bind(input_tokens)
        .bind(output_tokens)
        .bind(cached_tokens)
        .bind(&turn.finish_reason)
        .bind(&turn.error_code)
        .bind(&turn.error_detail)
        .bind(&turn.trace_id)
        .bind(attempt_count)
        .bind(max_attempts)
        .bind(&turn.next_retry_at)
        .bind(&turn.available_at)
        .bind(&turn.lease_owner)
        .bind(&turn.lease_token)
        .bind(&turn.lease_expires_at)
        .bind(fencing_token)
        .bind(version)
        .bind(&turn.updated_at)
        .bind(&turn.started_at)
        .bind(&turn.completed_at)
        .bind(&turn.cancel_requested_at)
        .bind(&turn.cancelled_at)
        .bind(&turn.retention_until)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(&turn.turn_id)
        .bind(expected_version)
        .bind(expected_fencing_token)
        .bind(expected_lease_token)
        .execute(&mut **tx)
        .await?
        .rows_affected();
    if affected != 1 {
        return Err(transaction_error(KernelError::conflict(
            "turn completion conflict",
        )));
    }
    Ok(())
}

#[cfg(feature = "postgres-sync")]
async fn insert_session_item_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    item: &AgentSessionItemRow,
) -> Result<(), sqlx::Error> {
    let id = u64_to_i64(item.id, "item.id").map_err(transaction_error)?;
    let tenant_id = u64_to_i64(item.tenant_id, "item.tenant_id").map_err(transaction_error)?;
    let organization_id =
        u64_to_i64(item.organization_id, "item.organization_id").map_err(transaction_error)?;
    let sequence = u64_to_i64(item.sequence, "item.sequence").map_err(transaction_error)?;
    let input_tokens =
        u64_to_i64(item.input_tokens, "item.input_tokens").map_err(transaction_error)?;
    let output_tokens =
        u64_to_i64(item.output_tokens, "item.output_tokens").map_err(transaction_error)?;
    let created_by = u64_to_i64(item.created_by, "item.created_by").map_err(transaction_error)?;
    let version = u64_to_i64(item.version, "item.version").map_err(transaction_error)?;
    let redacted_by = item
        .redacted_by
        .map(|value| u64_to_i64(value, "item.redacted_by"))
        .transpose()
        .map_err(transaction_error)?;
    sqlx::query(SQL_INSERT_AGENT_SESSION_ITEM)
        .bind(id)
        .bind(&item.uuid)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(&item.session_id)
        .bind(&item.item_id)
        .bind(item.kind)
        .bind(&item.content)
        .bind(&item.content_type)
        .bind(item.status)
        .bind(sequence)
        .bind(input_tokens)
        .bind(output_tokens)
        .bind(&item.model_id)
        .bind(&item.provider_id)
        .bind(&item.tool_name)
        .bind(&item.tool_call_id)
        .bind(&item.tool_arguments_json)
        .bind(&item.tool_result_json)
        .bind(&item.parent_item_id)
        .bind(&item.turn_id)
        .bind(created_by)
        .bind(version)
        .bind(&item.created_at)
        .bind(&item.updated_at)
        .bind(&item.completed_at)
        .bind(&item.redacted_at)
        .bind(redacted_by)
        .bind(&item.retention_until)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

#[cfg(feature = "postgres-sync")]
async fn insert_drive_ref_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    drive_ref: &AgentItemDriveRefRow,
) -> Result<(), sqlx::Error> {
    let id = u64_to_i64(drive_ref.id, "drive_ref.id").map_err(transaction_error)?;
    let tenant_id =
        u64_to_i64(drive_ref.tenant_id, "drive_ref.tenant_id").map_err(transaction_error)?;
    let organization_id = u64_to_i64(drive_ref.organization_id, "drive_ref.organization_id")
        .map_err(transaction_error)?;
    let sort_order = i32::try_from(drive_ref.sort_order)
        .map_err(|_| transaction_error(KernelError::validation("drive_ref.sort_order overflow")))?;
    let created_by =
        u64_to_i64(drive_ref.created_by, "drive_ref.created_by").map_err(transaction_error)?;
    sqlx::query(SQL_INSERT_AGENT_ITEM_DRIVE_REF)
        .bind(id)
        .bind(&drive_ref.uuid)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(&drive_ref.item_id)
        .bind(&drive_ref.resource_role)
        .bind(&drive_ref.drive_space_id)
        .bind(&drive_ref.drive_node_id)
        .bind(&drive_ref.media_resource_id)
        .bind(&drive_ref.object_blob_id)
        .bind(&drive_ref.resource_hash)
        .bind(&drive_ref.alt_text)
        .bind(sort_order)
        .bind(drive_ref.status)
        .bind(created_by)
        .bind(&drive_ref.created_at)
        .bind(&drive_ref.updated_at)
        .bind(&drive_ref.deleted_at)
        .bind(&drive_ref.retention_until)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

#[cfg(feature = "postgres-sync")]
impl AgentRepositoryAdapter for SyncPostgresAdapter {
    fn check_readiness(&self) -> KernelResult<()> {
        let pool = self.pool.pool().clone();
        self.pool
            .run_kernel(async move { sqlx::query("SELECT 1").execute(&pool).await.map(|_| ()) })
    }

    fn next_id(&self) -> KernelResult<u64> {
        self.id_generator.next_id()
    }

    fn insert_row(&self, row: AgentBusinessRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(row.owner_user_id, "owner_user_id")?;
        let version = u64_to_i64(row.version, "version")?;

        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                owner_user_id,
                row.agent_id,
                row.code,
                row.display_name,
                row.description,
                row.manifest_json,
                row.default_code_task_intent_json,
                row.implementation_provider_id,
                row.implementation_kind,
                row.implementation_type,
                row.status,
                row.visibility,
                row.tags_json,
                row.created_at,
                row.updated_at,
                row.deleted_at,
                version
            )?;
            Ok(())
        })
    }

    fn update_row(&self, row: AgentBusinessRow) -> KernelResult<()> {
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(row.owner_user_id, "owner_user_id")?;
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;

        self.with_pool(|pool| {
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT,
                organization_id,
                owner_user_id,
                row.code,
                row.display_name,
                row.description,
                row.manifest_json,
                row.default_code_task_intent_json,
                row.implementation_provider_id,
                row.implementation_kind,
                row.implementation_type,
                row.status,
                row.visibility,
                row.tags_json,
                row.updated_at,
                row.deleted_at,
                version,
                tenant_id,
                row.agent_id,
                previous_version
            )?;

            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID,
                    tenant_id,
                    row.agent_id
                )?
                .is_some();
                if exists {
                    return Err(KernelError::conflict("agent version mismatch"));
                }
                return Err(KernelError::validation("agent not found"));
            }
            Ok(())
        })
    }

    fn get_row(&self, tenant_id: u64, agent_id: &str) -> KernelResult<Option<AgentBusinessRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID,
                tenant_id,
                agent_id
            )?;
            row.map(pg_row_to_agent_business_row).transpose()
        })
    }

    fn list_rows(&self, query: &AgentListQuery) -> KernelResult<Vec<AgentBusinessRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;

        let organization_id = query
            .organization_id
            .map(|value| u64_to_i64(value, "organization_id"))
            .transpose()?;
        let owner_user_id = query
            .owner_user_id
            .map(|value| u64_to_i64(value, "owner_user_id"))
            .transpose()?;
        let include_deleted = query.include_deleted;
        let search_query: Option<String> = query
            .search_query
            .as_ref()
            .filter(|q| !is_blank(Some(q.as_str())))
            .map(|q| format!("%{}%", trim(q).to_lowercase()));
        let visibility_code = query.visibility.map(|visibility| visibility.as_db_code());
        let page_size = usize_to_i64(query.pagination.page_size, "pagination.page_size")?;
        let offset = usize_to_i64(query.pagination.offset, "pagination.offset")?;

        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT,
                tenant_id,
                organization_id,
                owner_user_id,
                include_deleted,
                search_query,
                visibility_code,
                page_size,
                offset
            )?;

            let mut mapped_rows = Vec::with_capacity(rows.len());
            for row in rows {
                mapped_rows.push(pg_row_to_agent_business_row(row)?);
            }
            Ok(mapped_rows)
        })
    }

    fn count_rows(&self, query: &AgentListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;

        let organization_id = query
            .organization_id
            .map(|value| u64_to_i64(value, "organization_id"))
            .transpose()?;
        let owner_user_id = query
            .owner_user_id
            .map(|value| u64_to_i64(value, "owner_user_id"))
            .transpose()?;
        let include_deleted = query.include_deleted;
        let search_query: Option<String> = query
            .search_query
            .as_ref()
            .filter(|q| !is_blank(Some(q.as_str())))
            .map(|q| format!("%{}%", trim(q).to_lowercase()));
        let visibility_code = query.visibility.map(|visibility| visibility.as_db_code());

        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT,
                tenant_id,
                organization_id,
                owner_user_id,
                include_deleted,
                search_query,
                visibility_code
            )?;
            let total: i64 = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    fn insert_workspace_row(&self, row: AgentWorkspaceRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(row.owner_user_id, "owner_user_id")?;
        let created_by = u64_to_i64(row.created_by, "created_by")?;
        let updated_by = u64_to_i64(row.updated_by, "updated_by")?;
        let version = u64_to_i64(row.version, "version")?;
        let archived_by = row
            .archived_by
            .map(|value| u64_to_i64(value, "archived_by"))
            .transpose()?;
        let deleted_by = row
            .deleted_by
            .map(|value| u64_to_i64(value, "deleted_by"))
            .transpose()?;
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_WORKSPACE,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.workspace_id,
                owner_user_id,
                row.name,
                row.description,
                row.is_default,
                row.status,
                created_by,
                updated_by,
                version,
                row.created_at,
                row.updated_at,
                row.archived_at,
                archived_by,
                row.deleted_at,
                deleted_by,
                row.retention_until
            )?;
            Ok(())
        })
    }

    fn update_workspace_row(&self, row: AgentWorkspaceRow) -> KernelResult<()> {
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let updated_by = u64_to_i64(row.updated_by, "updated_by")?;
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        let archived_by = row
            .archived_by
            .map(|value| u64_to_i64(value, "archived_by"))
            .transpose()?;
        let deleted_by = row
            .deleted_by
            .map(|value| u64_to_i64(value, "deleted_by"))
            .transpose()?;
        self.with_pool(|pool| {
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_WORKSPACE,
                row.name,
                row.description,
                row.status,
                updated_by,
                version,
                row.updated_at,
                row.archived_at,
                archived_by,
                row.deleted_at,
                deleted_by,
                row.retention_until,
                tenant_id,
                organization_id,
                row.workspace_id,
                previous_version
            )?;
            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_WORKSPACE,
                    tenant_id,
                    organization_id,
                    row.workspace_id
                )?
                .is_some();
                if exists {
                    return Err(KernelError::conflict("workspace version mismatch"));
                }
                return Err(KernelError::validation("workspace not found"));
            }
            Ok(())
        })
    }

    fn get_workspace_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
    ) -> KernelResult<Option<AgentWorkspaceRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_WORKSPACE,
                tenant_id,
                organization_id,
                workspace_id
            )?
            .map(pg_row_to_agent_workspace_row)
            .transpose()
        })
    }

    fn get_default_workspace_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
    ) -> KernelResult<Option<AgentWorkspaceRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(owner_user_id, "owner_user_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_DEFAULT_AGENT_WORKSPACE,
                tenant_id,
                organization_id,
                owner_user_id
            )?
            .map(pg_row_to_agent_workspace_row)
            .transpose()
        })
    }

    fn list_workspace_rows(
        &self,
        query: &WorkspaceListQuery,
    ) -> KernelResult<Vec<AgentWorkspaceRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(query.owner_user_id, "owner_user_id")?;
        let status = query.status.map(AgentWorkspaceStatus::as_db_code);
        let page_size = usize_to_i64(query.pagination.page_size, "pagination.page_size")?;
        let offset = usize_to_i64(query.pagination.offset, "pagination.offset")?;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_AGENT_WORKSPACES,
                tenant_id,
                organization_id,
                owner_user_id,
                status,
                query.include_deleted,
                page_size,
                offset
            )?
            .into_iter()
            .map(pg_row_to_agent_workspace_row)
            .collect()
        })
    }

    fn count_workspace_rows(&self, query: &WorkspaceListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(query.owner_user_id, "owner_user_id")?;
        let status = query.status.map(AgentWorkspaceStatus::as_db_code);
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_WORKSPACES,
                tenant_id,
                organization_id,
                owner_user_id,
                status,
                query.include_deleted
            )?;
            let total = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    fn insert_project_row(&self, row: AgentProjectRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(row.owner_user_id, "owner_user_id")?;
        let created_by = u64_to_i64(row.created_by, "created_by")?;
        let updated_by = u64_to_i64(row.updated_by, "updated_by")?;
        let version = u64_to_i64(row.version, "version")?;
        let archived_by = row
            .archived_by
            .map(|value| u64_to_i64(value, "archived_by"))
            .transpose()?;
        let deleted_by = row
            .deleted_by
            .map(|value| u64_to_i64(value, "deleted_by"))
            .transpose()?;
        self.with_pool(|pool| {
            let pg_pool = pool.pool().clone();
            pool.run_kernel(async move {
                retry_postgres_transaction(|| async {
                    let row = row.clone();
                    let mut tx = pg_pool.begin().await?;
                    sqlx::query(SQL_LOCK_AGENT_PROJECT_WORKSPACE_NAME)
                        .bind(tenant_id)
                        .bind(organization_id)
                        .bind(&row.workspace_id)
                        .bind(&row.name)
                        .execute(&mut *tx)
                        .await?;
                    if sqlx::query(SQL_SELECT_AGENT_PROJECT_BY_WORKSPACE_NAME)
                        .bind(tenant_id)
                        .bind(organization_id)
                        .bind(&row.workspace_id)
                        .bind(&row.name)
                        .fetch_optional(&mut *tx)
                        .await?
                        .is_some()
                    {
                        return Err(sqlx::Error::Protocol(
                            "sdkwork-domain-conflict:project name already exists in workspace"
                                .to_string(),
                        ));
                    }
                    sqlx::query(SQL_INSERT_AGENT_PROJECT)
                        .bind(id)
                        .bind(&row.uuid)
                        .bind(tenant_id)
                        .bind(organization_id)
                        .bind(&row.project_id)
                        .bind(&row.workspace_id)
                        .bind(owner_user_id)
                        .bind(&row.name)
                        .bind(&row.description)
                        .bind(row.visibility)
                        .bind(row.status)
                        .bind(row.drive_access_mode)
                        .bind(&row.default_agent_id)
                        .bind(&row.default_model_id)
                        .bind(&row.import_source_kind)
                        .bind(&row.import_source_ref)
                        .bind(&row.drive_space_id)
                        .bind(&row.drive_root_entry_id)
                        .bind(&row.drive_logical_path)
                        .bind(created_by)
                        .bind(updated_by)
                        .bind(version)
                        .bind(&row.created_at)
                        .bind(&row.updated_at)
                        .bind(&row.archived_at)
                        .bind(archived_by)
                        .bind(&row.deleted_at)
                        .bind(deleted_by)
                        .bind(&row.retention_until)
                        .execute(&mut *tx)
                        .await?;
                    tx.commit().await?;
                    Ok(())
                })
                .await
            })
        })
    }

    fn update_project_row(&self, row: AgentProjectRow) -> KernelResult<()> {
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let updated_by = u64_to_i64(row.updated_by, "updated_by")?;
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        let archived_by = row
            .archived_by
            .map(|value| u64_to_i64(value, "archived_by"))
            .transpose()?;
        let deleted_by = row
            .deleted_by
            .map(|value| u64_to_i64(value, "deleted_by"))
            .transpose()?;
        self.with_pool(|pool| {
            let pg_pool = pool.pool().clone();
            pool.run_kernel(async move {
                retry_postgres_transaction(|| async {
                    let row = row.clone();
                    let mut tx = pg_pool.begin().await?;
                    let current = sqlx::query(SQL_SELECT_AGENT_PROJECT)
                        .bind(tenant_id)
                        .bind(organization_id)
                        .bind(&row.project_id)
                        .fetch_optional(&mut *tx)
                        .await?
                        .map(pg_row_to_agent_project_row)
                        .transpose()
                        .map_err(|_| {
                            sqlx::Error::Protocol(
                                "sdkwork-domain-validation:project row is invalid".to_string(),
                            )
                        })?
                        .ok_or_else(|| {
                            sqlx::Error::Protocol(
                                "sdkwork-domain-validation:project not found".to_string(),
                            )
                        })?;
                    if !project_names_equal(&current.name, &row.name) {
                        sqlx::query(SQL_LOCK_AGENT_PROJECT_WORKSPACE_NAME)
                            .bind(tenant_id)
                            .bind(organization_id)
                            .bind(&row.workspace_id)
                            .bind(&row.name)
                            .execute(&mut *tx)
                            .await?;
                        if sqlx::query(SQL_SELECT_AGENT_PROJECT_BY_WORKSPACE_NAME)
                            .bind(tenant_id)
                            .bind(organization_id)
                            .bind(&row.workspace_id)
                            .bind(&row.name)
                            .fetch_optional(&mut *tx)
                            .await?
                            .is_some()
                        {
                            return Err(sqlx::Error::Protocol(
                                "sdkwork-domain-conflict:project name already exists in workspace"
                                    .to_string(),
                            ));
                        }
                    }
                    let updated_rows = sqlx::query(SQL_UPDATE_AGENT_PROJECT)
                        .bind(&row.workspace_id)
                        .bind(&row.name)
                        .bind(&row.description)
                        .bind(row.visibility)
                        .bind(row.status)
                        .bind(row.drive_access_mode)
                        .bind(&row.default_agent_id)
                        .bind(&row.default_model_id)
                        .bind(&row.import_source_kind)
                        .bind(&row.import_source_ref)
                        .bind(&row.drive_space_id)
                        .bind(&row.drive_root_entry_id)
                        .bind(&row.drive_logical_path)
                        .bind(updated_by)
                        .bind(version)
                        .bind(&row.updated_at)
                        .bind(&row.archived_at)
                        .bind(archived_by)
                        .bind(&row.deleted_at)
                        .bind(deleted_by)
                        .bind(&row.retention_until)
                        .bind(tenant_id)
                        .bind(organization_id)
                        .bind(&row.project_id)
                        .bind(previous_version)
                        .execute(&mut *tx)
                        .await?
                        .rows_affected();
                    if updated_rows == 0 {
                        return Err(sqlx::Error::Protocol(
                            "sdkwork-domain-conflict:project version mismatch".to_string(),
                        ));
                    }
                    tx.commit().await?;
                    Ok(())
                })
                .await
            })
        })
    }

    fn get_project_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
    ) -> KernelResult<Option<AgentProjectRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_PROJECT,
                tenant_id,
                organization_id,
                project_id
            )?
            .map(pg_row_to_agent_project_row)
            .transpose()
        })
    }

    fn get_project_row_by_workspace_name(
        &self,
        tenant_id: u64,
        organization_id: u64,
        workspace_id: &str,
        name: &str,
    ) -> KernelResult<Option<AgentProjectRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_PROJECT_BY_WORKSPACE_NAME,
                tenant_id,
                organization_id,
                workspace_id,
                name
            )?
            .map(pg_row_to_agent_project_row)
            .transpose()
        })
    }

    fn get_project_row_by_import_source(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        source_kind: &str,
        source_ref: &str,
    ) -> KernelResult<Option<AgentProjectRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(owner_user_id, "owner_user_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_PROJECT_BY_IMPORT_SOURCE,
                tenant_id,
                organization_id,
                owner_user_id,
                source_kind,
                source_ref
            )?
            .map(pg_row_to_agent_project_row)
            .transpose()
        })
    }

    fn list_project_rows(&self, query: &ProjectListQuery) -> KernelResult<Vec<AgentProjectRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let owner_user_id = query
            .owner_user_id
            .map(|value| u64_to_i64(value, "owner_user_id"))
            .transpose()?;
        let workspace_id = query.workspace_id.as_deref();
        let status = query.status.map(AgentProjectStatus::as_db_code);
        let search = query
            .search_query
            .as_ref()
            .map(|value| format!("%{}%", trim(value)));
        let page_size = usize_to_i64(query.pagination.page_size, "pagination.page_size")?;
        let offset = usize_to_i64(query.pagination.offset, "pagination.offset")?;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_AGENT_PROJECTS,
                tenant_id,
                organization_id,
                owner_user_id,
                workspace_id,
                status,
                search,
                query.include_deleted,
                page_size,
                offset
            )?
            .into_iter()
            .map(pg_row_to_agent_project_row)
            .collect()
        })
    }

    fn count_project_rows(&self, query: &ProjectListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let owner_user_id = query
            .owner_user_id
            .map(|value| u64_to_i64(value, "owner_user_id"))
            .transpose()?;
        let workspace_id = query.workspace_id.as_deref();
        let status = query.status.map(AgentProjectStatus::as_db_code);
        let search = query
            .search_query
            .as_ref()
            .map(|value| format!("%{}%", trim(value)));
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_PROJECTS,
                tenant_id,
                organization_id,
                owner_user_id,
                workspace_id,
                status,
                search,
                query.include_deleted
            )?;
            let total = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    fn insert_project_composition_slot_row(
        &self,
        row: AgentProjectCompositionSlotRow,
    ) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let created_by = u64_to_i64(row.created_by, "created_by")?;
        let updated_by = u64_to_i64(row.updated_by, "updated_by")?;
        let version = u64_to_i64(row.version, "version")?;
        let deleted_by = row
            .deleted_by
            .map(|value| u64_to_i64(value, "deleted_by"))
            .transpose()?;
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_PROJECT_COMPOSITION_SLOT,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.project_id,
                row.slot_id,
                row.slot_kind,
                row.target_module,
                row.target_ref,
                row.target_version_ref,
                row.priority,
                row.enabled,
                row.policy_json,
                created_by,
                updated_by,
                version,
                row.created_at,
                row.updated_at,
                row.deleted_at,
                deleted_by,
                row.retention_until
            )?;
            Ok(())
        })
    }

    fn update_project_composition_slot_row(
        &self,
        row: AgentProjectCompositionSlotRow,
    ) -> KernelResult<()> {
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let updated_by = u64_to_i64(row.updated_by, "updated_by")?;
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        let deleted_by = row
            .deleted_by
            .map(|value| u64_to_i64(value, "deleted_by"))
            .transpose()?;
        self.with_pool(|pool| {
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_PROJECT_COMPOSITION_SLOT,
                row.slot_kind,
                row.target_module,
                row.target_ref,
                row.target_version_ref,
                row.priority,
                row.enabled,
                row.policy_json,
                updated_by,
                version,
                row.updated_at,
                row.deleted_at,
                deleted_by,
                row.retention_until,
                tenant_id,
                organization_id,
                row.project_id,
                row.slot_id,
                previous_version
            )?;
            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_PROJECT_COMPOSITION_SLOT,
                    tenant_id,
                    organization_id,
                    row.project_id,
                    row.slot_id
                )?
                .is_some();
                if exists {
                    return Err(KernelError::conflict(
                        "project composition slot version mismatch",
                    ));
                }
                return Err(KernelError::validation(
                    "project composition slot not found",
                ));
            }
            Ok(())
        })
    }

    fn get_project_composition_slot_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentProjectCompositionSlotRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_PROJECT_COMPOSITION_SLOT,
                tenant_id,
                organization_id,
                project_id,
                slot_id
            )?
            .map(pg_row_to_agent_project_composition_slot_row)
            .transpose()
        })
    }

    fn list_project_composition_slot_rows(
        &self,
        query: &ProjectCompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentProjectCompositionSlotRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let slot_kind = query.slot_kind.map(|value| value.as_str().to_string());
        let page_size = usize_to_i64(query.pagination.page_size, "pagination.page_size")?;
        let offset = usize_to_i64(query.pagination.offset, "pagination.offset")?;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_AGENT_PROJECT_COMPOSITION_SLOTS,
                tenant_id,
                organization_id,
                query.project_id,
                slot_kind,
                query.enabled,
                page_size,
                offset
            )?
            .into_iter()
            .map(pg_row_to_agent_project_composition_slot_row)
            .collect()
        })
    }

    fn count_project_composition_slot_rows(
        &self,
        query: &ProjectCompositionSlotListQuery,
    ) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let slot_kind = query.slot_kind.map(|value| value.as_str().to_string());
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_PROJECT_COMPOSITION_SLOTS,
                tenant_id,
                organization_id,
                query.project_id,
                slot_kind,
                query.enabled
            )?;
            let total = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    fn insert_provider_binding_row(&self, row: AgentProviderBindingRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let version = u64_to_i64(row.version, "version")?;

        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_PROVIDER_BINDING,
                id,
                row.uuid,
                tenant_id,
                row.agent_id,
                row.binding_id,
                row.provider_id,
                row.implementation_kind,
                row.configuration_profile_id,
                row.capabilities_json,
                row.active,
                version,
                row.created_at,
                row.updated_at
            )?;
            Ok(())
        })
    }

    fn update_provider_binding_row(&self, row: AgentProviderBindingRow) -> KernelResult<()> {
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;

        self.with_pool(|pool| {
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_PROVIDER_BINDING,
                row.provider_id,
                row.implementation_kind,
                row.configuration_profile_id,
                row.capabilities_json,
                row.active,
                version,
                row.updated_at,
                tenant_id,
                row.agent_id,
                row.binding_id,
                previous_version
            )?;

            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_PROVIDER_BINDING,
                    tenant_id,
                    row.agent_id,
                    row.binding_id
                )?
                .is_some();
                if exists {
                    return Err(KernelError::conflict(
                        "agent provider binding version mismatch",
                    ));
                }
                return Err(KernelError::validation("agent provider binding not found"));
            }
            Ok(())
        })
    }

    fn activate_provider_binding_atomic(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
        updated_at: String,
    ) -> KernelResult<AgentProviderBindingRow> {
        let tenant_id_i64 = u64_to_i64(tenant_id, "tenant_id")?;

        fn kernel_err(error: KernelError) -> sqlx::Error {
            sqlx::Error::Protocol(error.to_string())
        }

        self.with_pool(|pool| {
            let pg_pool = pool.pool().clone();
            let agent_id = agent_id.to_string();
            let binding_id = binding_id.to_string();
            pool.run_kernel(async move {
                retry_postgres_transaction(|| async {
                    let agent_id = agent_id.clone();
                    let binding_id = binding_id.clone();
                    let updated_at = updated_at.clone();
                    let mut tx = pg_pool.begin().await?;

                    let current = sqlx::query(SQL_SELECT_AGENT_PROVIDER_BINDING)
                        .bind(tenant_id_i64)
                        .bind(&agent_id)
                        .bind(&binding_id)
                        .fetch_optional(&mut *tx)
                        .await?
                        .map(pg_row_to_agent_provider_binding_row)
                        .transpose()
                        .map_err(kernel_err)?
                        .ok_or_else(|| {
                            kernel_err(KernelError::validation("agent provider binding not found"))
                        })?;

                    if current.active {
                        tx.commit().await?;
                        return Ok(current);
                    }

                    let previous_version = current.version;
                    let next_version = previous_version.saturating_add(1);
                    let previous_version_i64 =
                        u64_to_i64(previous_version, "version").map_err(kernel_err)?;
                    let next_version_i64 =
                        u64_to_i64(next_version, "version").map_err(kernel_err)?;

                    sqlx::query(SQL_DEACTIVATE_ACTIVE_AGENT_PROVIDER_BINDINGS)
                        .bind(tenant_id_i64)
                        .bind(&agent_id)
                        .bind(&updated_at)
                        .execute(&mut *tx)
                        .await?;

                    let updated_rows = sqlx::query(SQL_ACTIVATE_AGENT_PROVIDER_BINDING)
                        .bind(tenant_id_i64)
                        .bind(&agent_id)
                        .bind(&binding_id)
                        .bind(next_version_i64)
                        .bind(&updated_at)
                        .bind(previous_version_i64)
                        .execute(&mut *tx)
                        .await?;
                    if updated_rows.rows_affected() == 0 {
                        return Err(kernel_err(KernelError::conflict(
                            "agent provider binding version mismatch",
                        )));
                    }

                    let mut activated = current;
                    activated.active = true;
                    activated.version = next_version;
                    activated.updated_at = updated_at;
                    tx.commit().await?;
                    Ok(activated)
                })
                .await
            })
        })
    }

    fn get_provider_binding_row(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_PROVIDER_BINDING,
                tenant_id,
                agent_id,
                binding_id
            )?;
            row.map(pg_row_to_agent_provider_binding_row).transpose()
        })
    }

    fn get_active_provider_binding_row(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Option<AgentProviderBindingRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_ACTIVE_AGENT_PROVIDER_BINDING,
                tenant_id,
                agent_id
            )?;
            row.map(pg_row_to_agent_provider_binding_row).transpose()
        })
    }

    fn list_provider_binding_rows(
        &self,
        query: &ProviderBindingListQuery,
    ) -> KernelResult<Vec<AgentProviderBindingRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;

        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT_PROVIDER_BINDINGS,
                tenant_id,
                query.agent_id,
                page_size,
                offset
            )?;

            let mut mapped_rows = Vec::with_capacity(rows.len());
            for row in rows {
                mapped_rows.push(pg_row_to_agent_provider_binding_row(row)?);
            }
            Ok(mapped_rows)
        })
    }

    fn count_provider_binding_rows(&self, query: &ProviderBindingListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;

        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_PROVIDER_BINDINGS,
                tenant_id,
                query.agent_id
            )?;
            let total: i64 = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    fn insert_composition_slot_row(&self, row: AgentCompositionSlotRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let version = u64_to_i64(row.version, "version")?;
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_COMPOSITION_SLOT,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.agent_id,
                row.slot_id,
                row.slot_kind,
                row.target_module,
                row.target_ref,
                row.target_version_ref,
                row.priority,
                row.enabled,
                row.policy_json,
                row.status,
                version,
                row.created_at,
                row.updated_at,
                row.deleted_at
            )?;
            Ok(())
        })
    }

    fn update_composition_slot_row(&self, row: AgentCompositionSlotRow) -> KernelResult<()> {
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let version = u64_to_i64(row.version, "version")?;
        let expected_version = u64_to_i64(row.version.saturating_sub(1), "version")?;
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_UPDATE_AGENT_COMPOSITION_SLOT,
                organization_id,
                row.slot_kind,
                row.target_module,
                row.target_ref,
                row.target_version_ref,
                row.priority,
                row.enabled,
                row.policy_json,
                row.status,
                version,
                row.updated_at,
                row.deleted_at,
                tenant_id,
                row.agent_id,
                row.slot_id,
                expected_version
            )?;
            Ok(())
        })
    }

    fn get_composition_slot_row(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str,
    ) -> KernelResult<Option<AgentCompositionSlotRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_COMPOSITION_SLOT,
                tenant_id,
                agent_id,
                slot_id
            )?;
            row.map(pg_row_to_agent_composition_slot_row).transpose()
        })
    }

    fn list_composition_slot_rows(
        &self,
        query: &CompositionSlotListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;
        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT_COMPOSITION_SLOTS,
                tenant_id,
                query.agent_id,
                page_size,
                offset
            )?;
            rows.into_iter()
                .map(pg_row_to_agent_composition_slot_row)
                .collect()
        })
    }

    fn count_composition_slot_rows(&self, query: &CompositionSlotListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_COMPOSITION_SLOTS,
                tenant_id,
                query.agent_id
            )?;
            let total: i64 = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    fn list_mcp_marketplace_slot_rows(
        &self,
        query: &McpMarketplaceListQuery,
    ) -> KernelResult<Vec<AgentCompositionSlotRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;
        let search_pattern =
            crate::mcp_marketplace::mcp_marketplace_search_pattern(query.q.as_deref());
        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_MCP_MARKETPLACE_SLOTS,
                tenant_id,
                search_pattern,
                page_size,
                offset
            )?;
            rows.into_iter()
                .map(pg_row_to_agent_composition_slot_row)
                .collect()
        })
    }

    fn count_mcp_marketplace_slot_rows(
        &self,
        query: &McpMarketplaceListQuery,
    ) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let search_pattern =
            crate::mcp_marketplace::mcp_marketplace_search_pattern(query.q.as_deref());
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_MCP_MARKETPLACE_SLOTS,
                tenant_id,
                search_pattern
            )?;
            let total: i64 = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    // -----------------------------------------------------------------------
    // Session persistence
    // -----------------------------------------------------------------------

    fn insert_session_row(&self, row: AgentSessionRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(row.owner_user_id, "owner_user_id")?;
        let item_count = u64_to_i64(row.item_count, "item_count")?;
        let last_item_sequence = u64_to_i64(row.last_item_sequence, "last_item_sequence")?;
        let total_input_tokens = u64_to_i64(row.total_input_tokens, "total_input_tokens")?;
        let total_output_tokens = u64_to_i64(row.total_output_tokens, "total_output_tokens")?;
        let created_by = u64_to_i64(row.created_by, "created_by")?;
        let updated_by = u64_to_i64(row.updated_by, "updated_by")?;
        let archived_by = row
            .archived_by
            .map(|value| u64_to_i64(value, "archived_by"))
            .transpose()?;
        let deleted_by = row
            .deleted_by
            .map(|value| u64_to_i64(value, "deleted_by"))
            .transpose()?;
        let version = u64_to_i64(row.version, "version")?;

        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_SESSION,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                owner_user_id,
                row.session_id,
                row.agent_id,
                owner_user_id,
                row.project_id,
                row.session_kind,
                row.entry_surface,
                row.source_module,
                row.source_context_kind,
                row.source_context_id,
                row.parent_session_id,
                row.forked_from_turn_id,
                row.title,
                row.title_source,
                row.status,
                item_count,
                last_item_sequence,
                total_input_tokens,
                total_output_tokens,
                row.idempotency_key,
                row.payload_hash,
                created_by,
                updated_by,
                version,
                row.created_at,
                row.updated_at,
                row.last_item_at,
                row.closed_at,
                row.archived_at,
                archived_by,
                row.deleted_at,
                deleted_by,
                row.retention_until
            )?;
            Ok(())
        })
    }

    fn update_session_row(&self, row: AgentSessionRow) -> KernelResult<()> {
        let item_count = u64_to_i64(row.item_count, "item_count")?;
        let last_item_sequence = u64_to_i64(row.last_item_sequence, "last_item_sequence")?;
        let total_input_tokens = u64_to_i64(row.total_input_tokens, "total_input_tokens")?;
        let total_output_tokens = u64_to_i64(row.total_output_tokens, "total_output_tokens")?;
        let updated_by = u64_to_i64(row.updated_by, "updated_by")?;
        let archived_by = row
            .archived_by
            .map(|value| u64_to_i64(value, "archived_by"))
            .transpose()?;
        let deleted_by = row
            .deleted_by
            .map(|value| u64_to_i64(value, "deleted_by"))
            .transpose()?;
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;

        self.with_pool(|pool| {
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_SESSION,
                row.project_id,
                row.title,
                row.title_source,
                row.status,
                item_count,
                last_item_sequence,
                total_input_tokens,
                total_output_tokens,
                updated_by,
                version,
                row.updated_at,
                row.last_item_at,
                row.closed_at,
                row.archived_at,
                archived_by,
                row.deleted_at,
                deleted_by,
                row.retention_until,
                tenant_id,
                organization_id,
                row.session_id,
                previous_version
            )?;

            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_SESSION,
                    tenant_id,
                    organization_id,
                    row.session_id
                )?
                .is_some();
                if exists {
                    return Err(KernelError::conflict("session version mismatch"));
                }
                return Err(KernelError::validation("session not found"));
            }
            Ok(())
        })
    }

    fn get_session_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_SESSION,
                tenant_id,
                organization_id,
                session_id
            )?;
            row.map(pg_row_to_agent_session_row).transpose()
        })
    }

    fn get_session_row_by_creation_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentSessionRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(owner_user_id, "owner_user_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_SESSION_BY_CREATE_IDEMPOTENCY,
                tenant_id,
                organization_id,
                owner_user_id,
                idempotency_key
            )?;
            row.map(pg_row_to_agent_session_row).transpose()
        })
    }

    fn list_session_rows(&self, query: &SessionListQuery) -> KernelResult<Vec<AgentSessionRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = query
            .organization_id
            .map(|value| u64_to_i64(value, "organization_id"))
            .transpose()?;
        let agent_id: Option<&str> = query.agent_id.as_deref();
        let project_id: Option<&str> = query.project_id.as_deref();
        let workspace_id: Option<&str> = query.workspace_id.as_deref();
        let owner_user_id = query
            .owner_user_id
            .map(|value| u64_to_i64(value, "owner_user_id"))
            .transpose()?;
        let status_code: Option<i16> = query
            .status
            .as_deref()
            .and_then(AgentSessionStatus::from_code)
            .map(|s| s.as_db_code());
        let include_archived = query.include_archived;
        let page_size = usize_to_i64(query.pagination.page_size, "pagination.page_size")?;
        let offset = usize_to_i64(query.pagination.offset, "pagination.offset")?;

        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT_SESSIONS,
                tenant_id,
                organization_id,
                agent_id,
                project_id,
                workspace_id,
                owner_user_id,
                status_code,
                include_archived,
                page_size,
                offset
            )?;
            rows.into_iter().map(pg_row_to_agent_session_row).collect()
        })
    }

    fn list_session_activity_head_rows(
        &self,
        query: &SessionActivitySummaryListQuery,
    ) -> KernelResult<Vec<AgentSessionActivityHeadRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(query.owner_user_id, "owner_user_id")?;
        let agent_id = query.agent_id.as_deref();
        let project_id = query.project_id.as_deref();
        let workspace_id = query.workspace_id.as_deref();
        let cursor_activity_at = query
            .cursor
            .as_ref()
            .map(|cursor| cursor.activity_at.as_str());
        let cursor_session_internal_id = query
            .cursor
            .as_ref()
            .map(|cursor| u64_to_i64(cursor.session_internal_id, "cursor.session_internal_id"))
            .transpose()?;
        let page_limit = query
            .page_size
            .checked_add(1)
            .ok_or_else(|| KernelError::validation("page_size is too large"))?;
        let page_limit = usize_to_i64(page_limit, "page_size")?;

        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT_SESSION_ACTIVITY_HEADS,
                tenant_id,
                organization_id,
                owner_user_id,
                agent_id,
                project_id,
                workspace_id,
                cursor_activity_at,
                cursor_session_internal_id,
                page_limit
            )?;
            rows.into_iter()
                .map(|row| {
                    let activity_at: OffsetDateTime =
                        row.try_get("activity_at").map_err(map_sqlx_error)?;
                    let activity_at = format_postgres_instant(activity_at, "Session activity")?;
                    let activity_source: String =
                        row.try_get("activity_source").map_err(map_sqlx_error)?;
                    let latest_turn = deserialize_optional_projection_row(
                        row.try_get("latest_turn_json").map_err(map_sqlx_error)?,
                        "latest Turn",
                    )?;
                    let pending_interaction = deserialize_optional_interaction_projection_row(
                        row.try_get("pending_interaction_json")
                            .map_err(map_sqlx_error)?,
                    )?;
                    let current_runtime_binding = deserialize_optional_projection_row(
                        row.try_get("current_runtime_binding_json")
                            .map_err(map_sqlx_error)?,
                        "current runtime binding",
                    )?;
                    let latest_runtime_binding = deserialize_optional_projection_row(
                        row.try_get("latest_runtime_binding_json")
                            .map_err(map_sqlx_error)?,
                        "latest runtime binding",
                    )?;
                    let user_state = deserialize_optional_projection_row(
                        row.try_get("user_state_json").map_err(map_sqlx_error)?,
                        "Session user state",
                    )?;
                    let latest_interaction_version = row
                        .try_get::<Option<i64>, _>("latest_interaction_version")
                        .map_err(map_sqlx_error)?
                        .map(|value| int64_to_u64(value, "latest_interaction_version"))
                        .transpose()?;
                    let latest_interaction_id = row
                        .try_get::<Option<String>, _>("latest_interaction_id")
                        .map_err(map_sqlx_error)?;
                    Ok(AgentSessionActivityHeadRow {
                        session: pg_row_to_agent_session_row(row)?,
                        activity_at,
                        activity_source: SessionActivitySource::from_code(&activity_source)?,
                        latest_turn,
                        pending_interaction,
                        current_runtime_binding,
                        latest_runtime_binding,
                        user_state,
                        latest_interaction_id,
                        latest_interaction_version,
                    })
                })
                .collect()
        })
    }

    fn count_session_rows(&self, query: &SessionListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = query
            .organization_id
            .map(|value| u64_to_i64(value, "organization_id"))
            .transpose()?;
        let agent_id: Option<&str> = query.agent_id.as_deref();
        let project_id: Option<&str> = query.project_id.as_deref();
        let workspace_id: Option<&str> = query.workspace_id.as_deref();
        let owner_user_id = query
            .owner_user_id
            .map(|value| u64_to_i64(value, "owner_user_id"))
            .transpose()?;
        let status_code: Option<i16> = query
            .status
            .as_deref()
            .and_then(AgentSessionStatus::from_code)
            .map(|s| s.as_db_code());
        let include_archived = query.include_archived;

        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_SESSIONS,
                tenant_id,
                organization_id,
                agent_id,
                project_id,
                workspace_id,
                owner_user_id,
                status_code,
                include_archived
            )?;
            let total: i64 = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    fn insert_session_runtime_binding_row(
        &self,
        row: AgentSessionRuntimeBindingRow,
    ) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(row.owner_user_id, "owner_user_id")?;
        let version = u64_to_i64(row.version, "version")?;
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_SESSION_RUNTIME_BINDING,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                owner_user_id,
                row.session_id,
                row.runtime_binding_id,
                row.runtime_location_id,
                row.host_mode,
                row.transport_kind,
                row.provider_binding_id,
                row.model_id,
                row.provider_id,
                row.provider_session_id,
                row.provider_session_tree_id,
                row.provider_parent_session_id,
                row.provider_forked_from_session_id,
                row.status,
                row.is_current,
                version,
                row.created_at,
                row.updated_at,
                row.activated_at,
                row.deactivated_at
            )?;
            Ok(())
        })
    }

    fn update_session_runtime_binding_row(
        &self,
        row: AgentSessionRuntimeBindingRow,
    ) -> KernelResult<()> {
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        self.with_pool(|pool| {
            let updated = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_SESSION_RUNTIME_BINDING,
                row.runtime_location_id,
                row.host_mode,
                row.transport_kind,
                row.provider_binding_id,
                row.model_id,
                row.provider_id,
                row.provider_session_id,
                row.provider_session_tree_id,
                row.provider_parent_session_id,
                row.provider_forked_from_session_id,
                row.status,
                row.is_current,
                version,
                row.updated_at,
                row.activated_at,
                row.deactivated_at,
                tenant_id,
                organization_id,
                row.session_id,
                row.runtime_binding_id,
                previous_version
            )?;
            if updated == 0 {
                return Err(KernelError::conflict(
                    "session runtime binding version mismatch",
                ));
            }
            Ok(())
        })
    }

    fn get_session_runtime_binding_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_SESSION_RUNTIME_BINDING,
                tenant_id,
                organization_id,
                session_id,
                runtime_binding_id
            )?
            .map(pg_row_to_agent_session_runtime_binding_row)
            .transpose()
        })
    }

    fn get_current_session_runtime_binding_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRuntimeBindingRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_CURRENT_AGENT_SESSION_RUNTIME_BINDING,
                tenant_id,
                organization_id,
                session_id
            )?
            .map(pg_row_to_agent_session_runtime_binding_row)
            .transpose()
        })
    }

    fn list_session_runtime_binding_rows(
        &self,
        query: &SessionRuntimeBindingListQuery,
    ) -> KernelResult<Vec<AgentSessionRuntimeBindingRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let status = query
            .status
            .as_deref()
            .and_then(|value| match value {
                "active" => Some(AgentSessionRuntimeBindingStatus::Active),
                "deactivated" => Some(AgentSessionRuntimeBindingStatus::Deactivated),
                "failed" => Some(AgentSessionRuntimeBindingStatus::Failed),
                "deleted" => Some(AgentSessionRuntimeBindingStatus::Deleted),
                _ => None,
            })
            .map(AgentSessionRuntimeBindingStatus::as_db_code);
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_AGENT_SESSION_RUNTIME_BINDINGS,
                tenant_id,
                organization_id,
                query.session_id,
                status,
                query.current_only,
                page_size,
                offset
            )?
            .into_iter()
            .map(pg_row_to_agent_session_runtime_binding_row)
            .collect()
        })
    }

    fn count_session_runtime_binding_rows(
        &self,
        query: &SessionRuntimeBindingListQuery,
    ) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let status = query
            .status
            .as_deref()
            .and_then(|value| match value {
                "active" => Some(AgentSessionRuntimeBindingStatus::Active),
                "deactivated" => Some(AgentSessionRuntimeBindingStatus::Deactivated),
                "failed" => Some(AgentSessionRuntimeBindingStatus::Failed),
                "deleted" => Some(AgentSessionRuntimeBindingStatus::Deleted),
                _ => None,
            })
            .map(AgentSessionRuntimeBindingStatus::as_db_code);
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_SESSION_RUNTIME_BINDINGS,
                tenant_id,
                organization_id,
                query.session_id,
                status,
                query.current_only
            )?;
            let total = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    fn activate_session_runtime_binding_row_atomic(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        runtime_binding_id: &str,
        expected_version: u64,
        updated_at: String,
    ) -> KernelResult<AgentSessionRuntimeBindingRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        let expected_version = u64_to_i64(expected_version, "expected_version")?;

        fn kernel_err(error: KernelError) -> sqlx::Error {
            sqlx::Error::Protocol(error.to_string())
        }

        self.with_pool(|pool| {
            let pg_pool = pool.pool().clone();
            let session_id = session_id.to_string();
            let runtime_binding_id = runtime_binding_id.to_string();
            pool.run_kernel(async move {
                retry_postgres_transaction(|| async {
                    let session_id = session_id.clone();
                    let runtime_binding_id = runtime_binding_id.clone();
                    let updated_at = updated_at.clone();
                    let mut tx = pg_pool.begin().await?;
                    let target = sqlx::query(SQL_LOCK_AGENT_SESSION_RUNTIME_BINDING)
                        .bind(tenant_id)
                        .bind(organization_id)
                        .bind(&session_id)
                        .bind(&runtime_binding_id)
                        .fetch_optional(&mut *tx)
                        .await?
                        .map(pg_row_to_agent_session_runtime_binding_row)
                        .transpose()
                        .map_err(kernel_err)?
                        .ok_or_else(|| {
                            kernel_err(KernelError::validation("session runtime binding not found"))
                        })?;
                    let target_version =
                        u64_to_i64(target.version, "version").map_err(kernel_err)?;
                    if target_version != expected_version {
                        return Err(kernel_err(KernelError::conflict(
                            "session runtime binding version mismatch",
                        )));
                    }
                    if target.is_current
                        && target.status == AgentSessionRuntimeBindingStatus::Active.as_db_code()
                    {
                        tx.commit().await?;
                        return Ok(target);
                    }
                    sqlx::query(SQL_DEACTIVATE_CURRENT_AGENT_SESSION_RUNTIME_BINDINGS)
                        .bind(tenant_id)
                        .bind(organization_id)
                        .bind(&session_id)
                        .bind(&runtime_binding_id)
                        .bind(&updated_at)
                        .execute(&mut *tx)
                        .await?;
                    let updated = sqlx::query(SQL_ACTIVATE_AGENT_SESSION_RUNTIME_BINDING)
                        .bind(tenant_id)
                        .bind(organization_id)
                        .bind(&session_id)
                        .bind(&runtime_binding_id)
                        .bind(expected_version)
                        .bind(&updated_at)
                        .execute(&mut *tx)
                        .await?;
                    if updated.rows_affected() != 1 {
                        return Err(kernel_err(KernelError::conflict(
                            "session runtime binding version mismatch",
                        )));
                    }
                    let activated_row = sqlx::query(SQL_SELECT_AGENT_SESSION_RUNTIME_BINDING)
                        .bind(tenant_id)
                        .bind(organization_id)
                        .bind(&session_id)
                        .bind(&runtime_binding_id)
                        .fetch_one(&mut *tx)
                        .await?;
                    let activated = pg_row_to_agent_session_runtime_binding_row(activated_row)
                        .map_err(kernel_err)?;
                    tx.commit().await?;
                    Ok(activated)
                })
                .await
            })
        })
    }

    fn insert_session_checkpoint_row(&self, row: AgentSessionCheckpointRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let created_by = u64_to_i64(row.created_by, "created_by")?;
        let version = u64_to_i64(row.version, "version")?;
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_SESSION_CHECKPOINT,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.session_id,
                row.checkpoint_id,
                row.turn_id,
                row.runtime_binding_id,
                row.checkpoint_kind,
                row.provider_checkpoint_ref,
                row.drive_space_id,
                row.drive_node_id,
                row.resumable,
                row.status,
                created_by,
                version,
                row.created_at,
                row.updated_at,
                row.restored_at,
                row.invalidated_at,
                row.retention_until
            )?;
            Ok(())
        })
    }

    fn update_session_checkpoint_row(&self, row: AgentSessionCheckpointRow) -> KernelResult<()> {
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        self.with_pool(|pool| {
            let updated = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_SESSION_CHECKPOINT,
                row.resumable,
                row.status,
                version,
                row.updated_at,
                row.restored_at,
                row.invalidated_at,
                row.retention_until,
                tenant_id,
                organization_id,
                row.session_id,
                row.checkpoint_id,
                previous_version
            )?;
            if updated == 0 {
                return Err(KernelError::conflict("session checkpoint version mismatch"));
            }
            Ok(())
        })
    }

    fn get_session_checkpoint_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        checkpoint_id: &str,
    ) -> KernelResult<Option<AgentSessionCheckpointRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_SESSION_CHECKPOINT,
                tenant_id,
                organization_id,
                session_id,
                checkpoint_id
            )?
            .map(pg_row_to_agent_session_checkpoint_row)
            .transpose()
        })
    }

    fn list_session_checkpoint_rows(
        &self,
        query: &SessionCheckpointListQuery,
    ) -> KernelResult<Vec<AgentSessionCheckpointRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let status = query
            .status
            .as_deref()
            .and_then(|value| match value {
                "active" => Some(AgentSessionCheckpointStatus::Active),
                "restored" => Some(AgentSessionCheckpointStatus::Restored),
                "invalidated" => Some(AgentSessionCheckpointStatus::Invalidated),
                "expired" => Some(AgentSessionCheckpointStatus::Expired),
                "deleted" => Some(AgentSessionCheckpointStatus::Deleted),
                _ => None,
            })
            .map(AgentSessionCheckpointStatus::as_db_code);
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_AGENT_SESSION_CHECKPOINTS,
                tenant_id,
                organization_id,
                query.session_id,
                status,
                page_size,
                offset
            )?
            .into_iter()
            .map(pg_row_to_agent_session_checkpoint_row)
            .collect()
        })
    }

    fn count_session_checkpoint_rows(
        &self,
        query: &SessionCheckpointListQuery,
    ) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let status = query
            .status
            .as_deref()
            .and_then(|value| match value {
                "active" => Some(AgentSessionCheckpointStatus::Active),
                "restored" => Some(AgentSessionCheckpointStatus::Restored),
                "invalidated" => Some(AgentSessionCheckpointStatus::Invalidated),
                "expired" => Some(AgentSessionCheckpointStatus::Expired),
                "deleted" => Some(AgentSessionCheckpointStatus::Deleted),
                _ => None,
            })
            .map(AgentSessionCheckpointStatus::as_db_code);
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_SESSION_CHECKPOINTS,
                tenant_id,
                organization_id,
                query.session_id,
                status
            )?;
            let total = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    fn upsert_resource_user_state_row(
        &self,
        row: AgentResourceUserStateRow,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentResourceUserStateRow> {
        let expected_record_version = expected_version
            .map(|version| version.saturating_add(1))
            .unwrap_or(0);
        if row.version != expected_record_version {
            return Err(KernelError::conflict(
                "resource user state version mismatch",
            ));
        }

        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let user_id = u64_to_i64(row.user_id, "user_id")?;
        let last_read_item_sequence = row
            .last_read_item_sequence
            .map(|value| u64_to_i64(value, "last_read_item_sequence"))
            .transpose()?;
        // -1 can never be a persisted version and therefore preserves create-only
        // semantics when the caller omits expectedVersion.
        let expected_version = expected_version
            .map(|value| u64_to_i64(value, "expected_version"))
            .transpose()?
            .unwrap_or(-1);

        self.with_pool(|pool| {
            let persisted = pg_query_optional!(
                pool,
                SQL_UPSERT_AGENT_RESOURCE_USER_STATE,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                user_id,
                row.resource_type,
                row.resource_id,
                row.pinned_at,
                row.hidden_at,
                row.last_opened_at,
                last_read_item_sequence,
                row.custom_title,
                row.created_at,
                row.updated_at,
                expected_version
            )?;
            persisted
                .map(pg_row_to_agent_resource_user_state_row)
                .transpose()?
                .ok_or_else(|| KernelError::conflict("resource user state version mismatch"))
        })
    }

    fn get_resource_user_state_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        resource_type: AgentResourceType,
        resource_id: &str,
    ) -> KernelResult<Option<AgentResourceUserStateRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        let user_id = u64_to_i64(user_id, "user_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_RESOURCE_USER_STATE,
                tenant_id,
                organization_id,
                user_id,
                resource_type.as_db_code(),
                resource_id
            )?
            .map(pg_row_to_agent_resource_user_state_row)
            .transpose()
        })
    }

    fn list_resource_user_state_rows(
        &self,
        query: &ResourceUserStateListQuery,
    ) -> KernelResult<Vec<AgentResourceUserStateRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let user_id = u64_to_i64(query.user_id, "user_id")?;
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_AGENT_RESOURCE_USER_STATES,
                tenant_id,
                organization_id,
                user_id,
                query.resource_type.as_db_code(),
                query.agent_id.as_deref(),
                query.pinned_only,
                query.include_hidden,
                query.resource_ids.as_slice(),
                page_size,
                offset
            )?
            .into_iter()
            .map(pg_row_to_agent_resource_user_state_row)
            .collect()
        })
    }

    fn count_resource_user_state_rows(
        &self,
        query: &ResourceUserStateListQuery,
    ) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let user_id = u64_to_i64(query.user_id, "user_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_RESOURCE_USER_STATES,
                tenant_id,
                organization_id,
                user_id,
                query.resource_type.as_db_code(),
                query.agent_id.as_deref(),
                query.pinned_only,
                query.include_hidden,
                query.resource_ids.as_slice()
            )?;
            let total = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    // -----------------------------------------------------------------------
    // Session-item persistence
    // -----------------------------------------------------------------------

    fn append_session_item_row(
        &self,
        row: AgentSessionItemRow,
    ) -> KernelResult<(AgentSessionRow, AgentSessionItemRow)> {
        if row.sequence != 0
            || row.turn_id.is_some()
            || row.status != AgentSessionItemStatus::Completed.as_db_code()
        {
            return Err(KernelError::validation(
                "standalone session item must be an unsequenced completed item without a turn",
            ));
        }
        self.with_pool(|pool| {
            let pg_pool = pool.pool().clone();
            pool.run_kernel(async move {
                retry_postgres_transaction(|| async {
                    let mut row = row.clone();
                    let mut tx = pg_pool.begin().await?;
                    let session = record_session_item_in_transaction(&mut tx, &row, true).await?;
                    if session.owner_user_id != row.created_by {
                        return Err(transaction_error(KernelError::validation(
                            "session item creator does not own the session",
                        )));
                    }
                    row.sequence = session.last_item_sequence;
                    insert_session_item_in_transaction(&mut tx, &row).await?;
                    tx.commit().await?;
                    Ok((session, row))
                })
                .await
            })
        })
    }

    fn update_session_item_row(&self, row: AgentSessionItemRow) -> KernelResult<()> {
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        let redacted_by = row
            .redacted_by
            .map(|value| u64_to_i64(value, "redacted_by"))
            .transpose()?;

        self.with_pool(|pool| {
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_SESSION_ITEM,
                row.content,
                row.content_type,
                row.status,
                row.model_id,
                row.provider_id,
                row.tool_name,
                row.tool_call_id,
                row.tool_arguments_json,
                row.tool_result_json,
                row.parent_item_id,
                row.turn_id,
                version,
                row.updated_at,
                row.completed_at,
                row.redacted_at,
                redacted_by,
                row.retention_until,
                tenant_id,
                organization_id,
                row.session_id,
                row.item_id,
                previous_version
            )?;

            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_SESSION_ITEM,
                    tenant_id,
                    organization_id,
                    row.session_id,
                    row.item_id
                )?
                .is_some();
                if exists {
                    return Err(KernelError::conflict("session-item update conflict"));
                }
                return Err(KernelError::validation("session item not found"));
            }
            Ok(())
        })
    }

    fn get_session_item_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        item_id: &str,
    ) -> KernelResult<Option<AgentSessionItemRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_SESSION_ITEM,
                tenant_id,
                organization_id,
                session_id,
                item_id
            )?;
            row.map(pg_row_to_agent_session_item_row).transpose()
        })
    }

    fn list_session_item_rows(
        &self,
        query: &SessionItemListQuery,
    ) -> KernelResult<Vec<AgentSessionItemRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let kind_code: Option<i16> = query
            .kind
            .as_deref()
            .and_then(AgentSessionItemKind::from_code)
            .map(|r| r.as_db_code());
        let status_code: Option<i16> = query
            .status
            .as_deref()
            .and_then(AgentSessionItemStatus::from_code)
            .map(|s| s.as_db_code());
        let page_size = query.pagination.page_size as i64;

        self.with_pool(|pool| {
            let rows = match query.sort {
                SessionItemListSort::RecentContextDesc => pg_query!(
                    pool,
                    SQL_LIST_AGENT_SESSION_ITEMS_RECENT_CONTEXT,
                    tenant_id,
                    organization_id,
                    query.session_id,
                    kind_code,
                    status_code,
                    page_size
                )?,
                SessionItemListSort::SequenceAsc | SessionItemListSort::SequenceDesc => {
                    let offset = query.pagination.offset as i64;
                    let sql = match query.sort {
                        SessionItemListSort::SequenceAsc => SQL_LIST_AGENT_SESSION_ITEMS,
                        SessionItemListSort::SequenceDesc => SQL_LIST_AGENT_SESSION_ITEMS_DESC,
                        SessionItemListSort::RecentContextDesc => unreachable!(),
                    };
                    pg_query!(
                        pool,
                        sql,
                        tenant_id,
                        organization_id,
                        query.session_id,
                        kind_code,
                        status_code,
                        page_size,
                        offset
                    )?
                }
            };
            let mut rows: Vec<AgentSessionItemRow> = rows
                .into_iter()
                .map(pg_row_to_agent_session_item_row)
                .collect::<KernelResult<Vec<_>>>()?;
            if matches!(query.sort, SessionItemListSort::RecentContextDesc) {
                rows.reverse();
            }
            Ok(rows)
        })
    }

    fn count_session_item_rows(&self, query: &SessionItemListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let kind_code: Option<i16> = query
            .kind
            .as_deref()
            .and_then(AgentSessionItemKind::from_code)
            .map(|r| r.as_db_code());
        let status_code: Option<i16> = query
            .status
            .as_deref()
            .and_then(AgentSessionItemStatus::from_code)
            .map(|s| s.as_db_code());

        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_SESSION_ITEMS,
                tenant_id,
                organization_id,
                query.session_id,
                kind_code,
                status_code
            )?;
            let total: i64 = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    fn upsert_item_feedback_row(
        &self,
        row: AgentItemFeedbackRow,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentItemFeedbackRow> {
        if let Some(expected) = expected_version {
            if row.version != expected.saturating_add(1) {
                return Err(KernelError::conflict("item feedback version mismatch"));
            }
        }
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let user_id = u64_to_i64(row.user_id, "user_id")?;
        let expected_version = expected_version
            .map(|value| u64_to_i64(value, "expected_version"))
            .transpose()?
            .unwrap_or(-1);
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_UPSERT_AGENT_ITEM_FEEDBACK,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.item_id,
                user_id,
                row.rating,
                row.reason_code,
                row.comment,
                row.created_at,
                row.updated_at,
                row.deleted_at,
                expected_version
            )?
            .map(pg_row_to_agent_item_feedback_row)
            .transpose()?
            .ok_or_else(|| KernelError::conflict("item feedback version mismatch"))
        })
    }

    fn get_item_feedback_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
        user_id: u64,
        include_deleted: bool,
    ) -> KernelResult<Option<AgentItemFeedbackRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        let user_id = u64_to_i64(user_id, "user_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_ITEM_FEEDBACK,
                tenant_id,
                organization_id,
                item_id,
                user_id,
                include_deleted
            )?
            .map(pg_row_to_agent_item_feedback_row)
            .transpose()
        })
    }

    fn list_item_feedback_rows(
        &self,
        query: &ItemFeedbackListQuery,
    ) -> KernelResult<Vec<AgentItemFeedbackRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let user_id = u64_to_i64(query.user_id, "user_id")?;
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_AGENT_ITEM_FEEDBACK,
                tenant_id,
                organization_id,
                user_id,
                query.session_id,
                page_size,
                offset
            )?
            .into_iter()
            .map(pg_row_to_agent_item_feedback_row)
            .collect()
        })
    }

    fn count_item_feedback_rows(&self, query: &ItemFeedbackListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let user_id = u64_to_i64(query.user_id, "user_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_ITEM_FEEDBACK,
                tenant_id,
                organization_id,
                user_id,
                query.session_id
            )?;
            let total = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    fn get_turn_row_by_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentTurnRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(owner_user_id, "owner_user_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_TURN_BY_IDEMPOTENCY,
                tenant_id,
                organization_id,
                owner_user_id,
                idempotency_key
            )?
            .map(pg_row_to_agent_turn_row)
            .transpose()
        })
    }

    fn get_turn_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<Option<AgentTurnRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_TURN,
                tenant_id,
                organization_id,
                turn_id
            )?
            .map(pg_row_to_agent_turn_row)
            .transpose()
        })
    }

    fn list_turn_rows(&self, query: &TurnListQuery) -> KernelResult<Vec<AgentTurnRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let status_code = query
            .status
            .as_deref()
            .and_then(AgentTurnStatus::from_code)
            .map(AgentTurnStatus::as_db_code);
        let page_size = i64::try_from(query.pagination.page_size)
            .map_err(|_| KernelError::validation("page_size overflow"))?;
        let offset = i64::try_from(query.pagination.offset)
            .map_err(|_| KernelError::validation("pagination offset overflow"))?;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_AGENT_TURNS,
                tenant_id,
                organization_id,
                query.session_id,
                status_code,
                page_size,
                offset
            )?
            .into_iter()
            .map(pg_row_to_agent_turn_row)
            .collect()
        })
    }

    fn count_turn_rows(&self, query: &TurnListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let status_code = query
            .status
            .as_deref()
            .and_then(AgentTurnStatus::from_code)
            .map(AgentTurnStatus::as_db_code);
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_TURNS,
                tenant_id,
                organization_id,
                query.session_id,
                status_code
            )?;
            let total = row
                .map(|row| row.try_get::<i64, _>("total_count").map_err(map_sqlx_error))
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    fn list_reconcilable_turn_rows(
        &self,
        stale_before: &str,
        limit: usize,
    ) -> KernelResult<Vec<AgentTurnRow>> {
        let limit = i64::try_from(limit.clamp(1, 200))
            .map_err(|_| KernelError::validation("reconciliation limit overflow"))?;
        self.with_pool(|pool| {
            pg_query!(pool, SQL_LIST_RECONCILABLE_AGENT_TURNS, stale_before, limit)?
                .into_iter()
                .map(pg_row_to_agent_turn_row)
                .collect()
        })
    }

    fn insert_turn_request_rows(
        &self,
        turn: AgentTurnRow,
        request_item: AgentSessionItemRow,
        drive_refs: Vec<AgentItemDriveRefRow>,
    ) -> KernelResult<AgentTurnRequestRowsOutcome> {
        if turn.status != AgentTurnStatus::Requested.as_db_code()
            || turn.version != 0
            || turn.response_item_id.is_some()
            || turn.request_item_id != request_item.item_id
            || turn.tenant_id != request_item.tenant_id
            || turn.organization_id != request_item.organization_id
            || turn.session_id != request_item.session_id
            || request_item.turn_id.as_deref() != Some(turn.turn_id.as_str())
            || request_item.kind != AgentSessionItemKind::UserInput.as_db_code()
            || request_item.status != AgentSessionItemStatus::Completed.as_db_code()
        {
            return Err(KernelError::validation(
                "turn request and session item scope mismatch",
            ));
        }
        if drive_refs.iter().any(|drive_ref| {
            drive_ref.tenant_id != request_item.tenant_id
                || drive_ref.organization_id != request_item.organization_id
                || drive_ref.item_id != request_item.item_id
        }) {
            return Err(KernelError::validation(
                "session item Drive reference scope mismatch",
            ));
        }

        self.with_pool(|pool| {
            let pg_pool = pool.pool().clone();
            pool.run_kernel(async move {
                retry_postgres_transaction(|| async {
                    let turn = turn.clone();
                    let mut request_item = request_item.clone();
                    let drive_refs = drive_refs.clone();
                    let mut tx = pg_pool.begin().await?;

                    let session =
                        record_session_item_in_transaction(&mut tx, &request_item, true).await?;
                    if session.agent_id != turn.agent_id
                        || session.owner_user_id != turn.owner_user_id
                    {
                        return Err(transaction_error(KernelError::validation(
                            "turn request does not belong to the session agent and owner",
                        )));
                    }
                    request_item.sequence = session.last_item_sequence;
                    if !insert_turn_in_transaction(&mut tx, &turn).await? {
                        let existing = get_turn_by_idempotency_in_transaction(&mut tx, &turn)
                            .await?
                            .ok_or_else(|| {
                                transaction_error(KernelError::Internal {
                                    message: "idempotent turn conflict has no existing row"
                                        .to_string(),
                                })
                            })?;
                        tx.rollback().await?;
                        return Ok(AgentTurnRequestRowsOutcome::Existing(Box::new(existing)));
                    }
                    insert_session_item_in_transaction(&mut tx, &request_item).await?;
                    for drive_ref in &drive_refs {
                        insert_drive_ref_in_transaction(&mut tx, drive_ref).await?;
                    }
                    tx.commit().await?;
                    Ok(AgentTurnRequestRowsOutcome::Inserted {
                        session: Box::new(session),
                        request_item: Box::new(request_item),
                    })
                })
                .await
            })
        })
    }

    fn update_turn_state_row(
        &self,
        turn: AgentTurnRow,
        expected_version: u64,
    ) -> KernelResult<AgentTurnRow> {
        let input_tokens = u64_to_i64(turn.input_tokens, "turn.input_tokens")?;
        let output_tokens = u64_to_i64(turn.output_tokens, "turn.output_tokens")?;
        let cached_tokens = u64_to_i64(turn.cached_tokens, "turn.cached_tokens")?;
        let attempt_count = i32::try_from(turn.attempt_count)
            .map_err(|_| KernelError::validation("turn.attempt_count overflow"))?;
        let max_attempts = i32::try_from(turn.max_attempts)
            .map_err(|_| KernelError::validation("turn.max_attempts overflow"))?;
        let fencing_token = u64_to_i64(turn.fencing_token, "turn.fencing_token")?;
        let version = u64_to_i64(turn.version, "turn.version")?;
        let tenant_id = u64_to_i64(turn.tenant_id, "turn.tenant_id")?;
        let organization_id = u64_to_i64(turn.organization_id, "turn.organization_id")?;
        let expected_version = u64_to_i64(expected_version, "turn.expected_version")?;
        self.with_pool(|pool| {
            let affected = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_TURN_STATE,
                &turn.response_item_id,
                &turn.runtime_binding_id,
                turn.turn_mode,
                turn.status,
                &turn.requested_model_id,
                &turn.provider_binding_id,
                &turn.model_id,
                &turn.provider_id,
                input_tokens,
                output_tokens,
                cached_tokens,
                &turn.finish_reason,
                &turn.error_code,
                &turn.error_detail,
                &turn.trace_id,
                attempt_count,
                max_attempts,
                &turn.next_retry_at,
                &turn.available_at,
                &turn.lease_owner,
                &turn.lease_token,
                &turn.lease_expires_at,
                fencing_token,
                version,
                &turn.updated_at,
                &turn.started_at,
                &turn.completed_at,
                &turn.cancel_requested_at,
                &turn.cancelled_at,
                &turn.retention_until,
                tenant_id,
                organization_id,
                &turn.turn_id,
                expected_version
            )?;
            if affected == 0 {
                return Err(KernelError::conflict("turn state update conflict"));
            }
            Ok(turn)
        })
    }

    fn complete_turn_rows(
        &self,
        turn: AgentTurnRow,
        expected_turn_version: u64,
        expected_fencing_token: u64,
        expected_lease_token: Option<String>,
        response_item: AgentSessionItemRow,
    ) -> KernelResult<(AgentSessionRow, AgentSessionItemRow)> {
        if turn.status != AgentTurnStatus::Completed.as_db_code()
            || turn.version != expected_turn_version.saturating_add(1)
            || turn.response_item_id.as_deref() != Some(response_item.item_id.as_str())
            || turn.tenant_id != response_item.tenant_id
            || turn.organization_id != response_item.organization_id
            || turn.session_id != response_item.session_id
            || response_item.turn_id.as_deref() != Some(turn.turn_id.as_str())
            || response_item.parent_item_id.as_deref() != Some(turn.request_item_id.as_str())
            || response_item.kind != AgentSessionItemKind::AssistantOutput.as_db_code()
            || response_item.status != AgentSessionItemStatus::Completed.as_db_code()
        {
            return Err(KernelError::validation(
                "completed turn and response item scope mismatch",
            ));
        }

        self.with_pool(|pool| {
            let pg_pool = pool.pool().clone();
            pool.run_kernel(async move {
                retry_postgres_transaction(|| async {
                    let turn = turn.clone();
                    let mut response_item = response_item.clone();
                    let mut tx = pg_pool.begin().await?;

                    let session =
                        record_session_item_in_transaction(&mut tx, &response_item, false).await?;
                    if session.agent_id != turn.agent_id
                        || session.owner_user_id != turn.owner_user_id
                    {
                        return Err(transaction_error(KernelError::validation(
                            "completed turn does not belong to the session agent and owner",
                        )));
                    }
                    response_item.sequence = session.last_item_sequence;
                    complete_turn_in_transaction(
                        &mut tx,
                        &turn,
                        expected_turn_version,
                        expected_fencing_token,
                        &expected_lease_token,
                    )
                    .await?;
                    insert_session_item_in_transaction(&mut tx, &response_item).await?;
                    tx.commit().await?;
                    Ok((session, response_item))
                })
                .await
            })
        })
    }

    fn list_item_drive_ref_rows(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_id: &str,
    ) -> KernelResult<Vec<AgentItemDriveRefRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_AGENT_ITEM_DRIVE_REFS,
                tenant_id,
                organization_id,
                item_id
            )?
            .into_iter()
            .map(pg_row_to_agent_item_drive_ref_row)
            .collect()
        })
    }

    fn list_item_drive_ref_rows_batch(
        &self,
        tenant_id: u64,
        organization_id: u64,
        item_ids: &[String],
    ) -> KernelResult<Vec<AgentItemDriveRefRow>> {
        if item_ids.is_empty() {
            return Ok(Vec::new());
        }
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_AGENT_ITEM_DRIVE_REFS_BATCH,
                tenant_id,
                organization_id,
                item_ids
            )?
            .into_iter()
            .map(pg_row_to_agent_item_drive_ref_row)
            .collect()
        })
    }

    // -----------------------------------------------------------------------
    // Interaction persistence
    // -----------------------------------------------------------------------

    fn insert_interaction_row(&self, row: AgentInteractionRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let fencing_token = u64_to_i64(row.fencing_token, "fencing_token")?;
        let version = u64_to_i64(row.version, "version")?;

        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_INTERACTION,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.session_id,
                row.turn_id,
                row.runtime_binding_id,
                row.interaction_id,
                row.provider_interaction_id,
                row.kind,
                row.status,
                row.prompt,
                row.options_json,
                row.resolution_json,
                row.claim_owner,
                row.claim_token_hash,
                row.claim_expires_at,
                fencing_token,
                version,
                row.created_at,
                row.updated_at,
                row.resolved_at,
                row.retention_until
            )?;
            Ok(())
        })
    }

    fn update_interaction_row(&self, row: AgentInteractionRow) -> KernelResult<()> {
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let fencing_token = u64_to_i64(row.fencing_token, "fencing_token")?;
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;

        self.with_pool(|pool| {
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_INTERACTION,
                row.kind,
                row.status,
                row.prompt,
                row.options_json,
                row.resolution_json,
                row.claim_owner,
                row.claim_token_hash,
                row.claim_expires_at,
                fencing_token,
                version,
                row.updated_at,
                row.resolved_at,
                row.retention_until,
                tenant_id,
                organization_id,
                row.session_id,
                row.interaction_id,
                previous_version
            )?;

            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_INTERACTION,
                    tenant_id,
                    organization_id,
                    row.session_id,
                    row.interaction_id
                )?
                .is_some();
                if exists {
                    return Err(KernelError::conflict("interaction version mismatch"));
                }
                return Err(KernelError::validation("interaction not found"));
            }
            Ok(())
        })
    }

    fn get_interaction_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        interaction_id: &str,
    ) -> KernelResult<Option<AgentInteractionRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_INTERACTION,
                tenant_id,
                organization_id,
                session_id,
                interaction_id
            )?;
            row.map(pg_row_to_agent_interaction_row).transpose()
        })
    }

    fn list_interaction_rows(
        &self,
        query: &InteractionListQuery,
    ) -> KernelResult<Vec<AgentInteractionRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let status_code: Option<i16> = query
            .status
            .as_deref()
            .and_then(AgentInteractionStatus::from_code)
            .map(|s| s.as_db_code());
        let kind_code: Option<i16> = query
            .kind
            .as_deref()
            .and_then(AgentInteractionKind::from_code)
            .map(|kind| kind.as_db_code());
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;

        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT_INTERACTIONS,
                tenant_id,
                organization_id,
                query.session_id,
                status_code,
                kind_code,
                page_size,
                offset
            )?;
            rows.into_iter()
                .map(pg_row_to_agent_interaction_row)
                .collect()
        })
    }

    fn count_interaction_rows(&self, query: &InteractionListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let status_code: Option<i16> = query
            .status
            .as_deref()
            .and_then(AgentInteractionStatus::from_code)
            .map(|s| s.as_db_code());
        let kind_code: Option<i16> = query
            .kind
            .as_deref()
            .and_then(AgentInteractionKind::from_code)
            .map(|kind| kind.as_db_code());

        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_INTERACTIONS,
                tenant_id,
                organization_id,
                query.session_id,
                status_code,
                kind_code
            )?;
            let total: i64 = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }

    fn insert_task_row(&self, row: AgentTaskRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(row.owner_user_id, "owner_user_id")?;
        let version = u64_to_i64(row.version, "version")?;

        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_TASK,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.agent_id,
                row.task_id,
                owner_user_id,
                row.title,
                row.prompt,
                row.status,
                row.external_ref,
                row.metadata_json,
                version,
                row.created_at,
                row.updated_at,
                row.started_at,
                row.completed_at,
                row.cancelled_at
            )?;
            Ok(())
        })
    }

    fn update_task_row(&self, row: AgentTaskRow) -> KernelResult<()> {
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;

        self.with_pool(|pool| {
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_TASK,
                row.title,
                row.prompt,
                row.status,
                row.external_ref,
                row.metadata_json,
                version,
                row.updated_at,
                row.started_at,
                row.completed_at,
                row.cancelled_at,
                tenant_id,
                row.task_id,
                previous_version
            )?;

            if updated_rows == 0 {
                let exists =
                    pg_query_optional!(pool, SQL_SELECT_AGENT_TASK, tenant_id, row.task_id)?
                        .is_some();
                if exists {
                    return Err(KernelError::conflict("task version mismatch"));
                }
                return Err(KernelError::validation("task not found"));
            }
            Ok(())
        })
    }

    fn get_task_row(&self, tenant_id: u64, task_id: &str) -> KernelResult<Option<AgentTaskRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(pool, SQL_SELECT_AGENT_TASK, tenant_id, task_id)?;
            row.map(pg_row_to_agent_task_row).transpose()
        })
    }

    fn list_task_rows(&self, query: &TaskListQuery) -> KernelResult<Vec<AgentTaskRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let agent_id: Option<&str> = query.agent_id.as_deref();
        let owner_user_id: Option<i64> = query.owner_user_id.map(|v| v as i64);
        let status_code: Option<i16> = query
            .status
            .as_deref()
            .and_then(AgentTaskStatus::from_code)
            .map(|s| s.as_db_code());
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;

        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT_TASKS,
                tenant_id,
                agent_id,
                owner_user_id,
                status_code,
                page_size,
                offset
            )?;
            rows.into_iter().map(pg_row_to_agent_task_row).collect()
        })
    }

    fn count_task_rows(&self, query: &TaskListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let agent_id: Option<&str> = query.agent_id.as_deref();
        let owner_user_id: Option<i64> = query.owner_user_id.map(|v| v as i64);
        let status_code: Option<i16> = query
            .status
            .as_deref()
            .and_then(AgentTaskStatus::from_code)
            .map(|s| s.as_db_code());

        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_TASKS,
                tenant_id,
                agent_id,
                owner_user_id,
                status_code
            )?;
            let total: i64 = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_composition_slot_row(row: PgRow) -> KernelResult<AgentCompositionSlotRow> {
    Ok(AgentCompositionSlotRow {
        id: int64_to_u64(row.try_get::<i64, _>("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get::<i64, _>("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get::<i64, _>("organization_id")
                .map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        agent_id: row.try_get("agent_id").map_err(map_sqlx_error)?,
        slot_id: row.try_get("slot_id").map_err(map_sqlx_error)?,
        slot_kind: row.try_get("slot_kind").map_err(map_sqlx_error)?,
        target_module: row.try_get("target_module").map_err(map_sqlx_error)?,
        target_ref: row.try_get("target_ref").map_err(map_sqlx_error)?,
        target_version_ref: row.try_get("target_version_ref").map_err(map_sqlx_error)?,
        priority: row.try_get("priority").map_err(map_sqlx_error)?,
        enabled: row.try_get("enabled").map_err(map_sqlx_error)?,
        policy_json: row.try_get("policy_json").map_err(map_sqlx_error)?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        version: int64_to_u64(
            row.try_get::<i64, _>("version").map_err(map_sqlx_error)?,
            "version",
        )?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        deleted_at: row.try_get("deleted_at").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
impl AgentAuditAdapter for SyncPostgresAdapter {
    fn next_id(&self) -> KernelResult<u64> {
        self.id_generator.next_id()
    }

    fn insert_audit_row(&self, row: AgentAuditEventRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let agent_internal_id = row
            .agent_internal_id
            .map(|value| u64_to_i64(value, "agent_internal_id"))
            .transpose()?;
        let actor_id = u64_to_i64(row.actor_id, "actor_id")?;

        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AUDIT_EVENT,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.aggregate_type,
                row.aggregate_id,
                agent_internal_id,
                row.agent_id,
                row.action,
                row.actor_type,
                actor_id,
                row.request_id,
                row.trace_id,
                row.payload_json,
                row.created_at
            )?;
            Ok(())
        })
    }

    fn list_audit_rows(
        &self,
        query: &AuditEventListQuery,
    ) -> KernelResult<Vec<AgentAuditEventRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;
        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID,
                tenant_id,
                query.agent_id,
                query.action,
                query.from,
                query.to,
                page_size,
                offset
            )?;
            rows.iter().map(AgentAuditEventRow::from_pg_row).collect()
        })
    }

    fn count_audit_rows(&self, query: &AuditEventListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID,
                tenant_id,
                query.agent_id,
                query.action,
                query.from,
                query.to
            )?;
            let total: i64 = row
                .map(|value| {
                    value
                        .try_get::<i64, _>("total_count")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(0);
            int64_to_u64(total, "total_count")
        })
    }
}

pub struct SqlAgentAuditSink<A>
where
    A: AgentAuditAdapter,
{
    adapter: A,
}

impl<A> SqlAgentAuditSink<A>
where
    A: AgentAuditAdapter,
{
    /// Create a global audit sink that persists audit events to PostgreSQL.
    ///
    /// The sink extracts tenant_id, organization_id, agent_id, and
    /// agent_internal_id from each event's structured context (populated by
    /// `AgentsService::emit_audit_event`), so a single sink can serve
    /// audit events for any agent in the process.
    pub fn new_global(adapter: A) -> Self {
        Self { adapter }
    }
}

impl<A> AgentAuditSink for SqlAgentAuditSink<A>
where
    A: AgentAuditAdapter,
{
    fn record(&self, event: KernelEvent) -> KernelResult<()> {
        let id = self.adapter.next_id()?;
        let row = AgentAuditEventRow::from_kernel_event(&event, id)?;
        self.adapter.insert_audit_row(row)
    }

    fn list_events(
        &self,
        query: &AuditEventListQuery,
    ) -> KernelResult<crate::ports::PaginatedResult<KernelEvent>> {
        use crate::ports::offset_paginated_result;
        let total_count = self.adapter.count_audit_rows(query)?;
        let items = self
            .adapter
            .list_audit_rows(query)?
            .into_iter()
            .map(AgentAuditEventRow::into_kernel_event)
            .collect::<KernelResult<Vec<_>>>()?;
        Ok(offset_paginated_result(
            items,
            &query.pagination,
            total_count,
        ))
    }
}

fn build_agent_business_uuid(tenant_id: u64, agent_id: &str) -> String {
    build_storage_uuid("agent-business", tenant_id, &[agent_id])
}

fn build_agent_provider_binding_uuid(tenant_id: u64, agent_id: &str, binding_id: &str) -> String {
    build_storage_uuid("provider-binding", tenant_id, &[agent_id, binding_id])
}

fn build_storage_uuid(resource_kind: &str, tenant_id: u64, identity_parts: &[&str]) -> String {
    let mut material = format!(
        "sdkwork.agents.storage.v1\n{}:{resource_kind}",
        resource_kind.len()
    );
    let tenant_id = tenant_id.to_string();
    material.push('\n');
    material.push_str(tenant_id.len().to_string().as_str());
    material.push(':');
    material.push_str(tenant_id.as_str());
    for part in identity_parts {
        material.push('\n');
        material.push_str(part.len().to_string().as_str());
        material.push(':');
        material.push_str(part);
    }
    sha256_hash(material.as_bytes())
}

const AUDIT_ACTOR_TYPE_USER: i16 = 0;
const AUDIT_ACTOR_TYPE_SERVICE: i16 = 1;
const AUDIT_ACTOR_TYPE_SYSTEM: i16 = 2;

fn audit_actor_from_subject_id(subject_id: &str) -> KernelResult<(i16, u64)> {
    if subject_id == "system.agents" || subject_id.starts_with("system.agents.") {
        return Ok((AUDIT_ACTOR_TYPE_SYSTEM, 0));
    }

    if let Some(service_id) = subject_id.strip_prefix("service.") {
        return parse_positive_audit_actor_id(service_id)
            .map(|actor_id| (AUDIT_ACTOR_TYPE_SERVICE, actor_id));
    }

    parse_positive_audit_actor_id(subject_id).map(|actor_id| (AUDIT_ACTOR_TYPE_USER, actor_id))
}

fn parse_positive_audit_actor_id(value: &str) -> KernelResult<u64> {
    value
        .parse::<u64>()
        .ok()
        .filter(|actor_id| *actor_id > 0 && *actor_id <= i64::MAX as u64)
        .ok_or_else(|| {
            KernelError::validation(
                "audit subject_id must be a positive numeric IAM subject, service.<positive numeric id>, or system.agents.*",
            )
        })
}

/// Extract a structured context value from a `KernelEvent` payload by key.
///
/// Audit events produced by `AgentsService::emit_audit_event` (and the
/// related `emit_*_audit_event` helpers) embed context metadata
/// (agent_id, tenant_id, organization_id, agent_internal_id,
/// subject_id, subject_tenant_id, etc.) under the `_context` JSON
/// field within the event payload.  This function extracts a value by
/// key from that context map.
///
/// For events that lack a `_context` field (e.g. events from external
/// sources), the function falls back to checking root-level JSON
/// fields, handling both string and numeric values.
pub fn extract_event_context(payload: &str, key: &str) -> Option<String> {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(payload) {
        if let Some(value) = parsed
            .get("_context")
            .and_then(|ctx| ctx.get(key))
            .and_then(|v| v.as_str())
        {
            return Some(value.to_string());
        }
        match parsed.get(key) {
            Some(serde_json::Value::String(s)) => return Some(s.clone()),
            Some(serde_json::Value::Number(n)) => return Some(n.to_string()),
            Some(serde_json::Value::Bool(b)) => return Some(b.to_string()),
            _ => {}
        }
    }

    extract_semicolon_payload_context(payload, key)
}

fn extract_semicolon_payload_context(payload: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=");
    payload.split(';').rev().find_map(|segment| {
        let segment = segment.trim();
        if segment.is_empty() {
            return None;
        }
        segment
            .strip_prefix(needle.as_str())
            .map(|value| value.to_string())
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AgentManifestSnapshot {
    schema_version: String,
    manifest_type: String,
    agent_id: String,
    name: String,
    display_name: String,
    description: String,
    version: String,
    domain: String,
    required_capabilities: Vec<String>,
    optional_capabilities: Vec<String>,
    event_families: Vec<String>,
    owner_name: String,
    status: String,
    implementation_provider_id: Option<String>,
    implementation_kind: Option<String>,
}

impl From<&AgentManifest> for AgentManifestSnapshot {
    fn from(value: &AgentManifest) -> Self {
        Self {
            schema_version: value.schema_version.clone(),
            manifest_type: value.manifest_type.clone(),
            agent_id: value.agent_id.clone(),
            name: value.name.clone(),
            display_name: value.display_name.clone(),
            description: value.description.clone(),
            version: value.version.clone(),
            domain: value.domain.clone(),
            required_capabilities: value.required_capabilities.clone(),
            optional_capabilities: value.optional_capabilities.clone(),
            event_families: value.event_families.clone(),
            owner_name: value.owner_name.clone(),
            status: value.status.clone(),
            implementation_provider_id: None,
            implementation_kind: None,
        }
    }
}

impl From<AgentManifestSnapshot> for AgentManifest {
    fn from(value: AgentManifestSnapshot) -> Self {
        Self {
            schema_version: value.schema_version,
            manifest_type: value.manifest_type,
            agent_id: value.agent_id,
            name: value.name,
            display_name: value.display_name,
            description: value.description,
            version: value.version,
            domain: value.domain,
            required_capabilities: value.required_capabilities,
            optional_capabilities: value.optional_capabilities,
            required_capability_requirements: Vec::new(),
            optional_capability_requirements: Vec::new(),
            event_families: value.event_families,
            owner_name: value.owner_name,
            status: value.status,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CodeTaskIntentSnapshot {
    prompt: String,
    context_paths: Vec<String>,
    constraints: Vec<String>,
}

impl From<&CodeTaskIntent> for CodeTaskIntentSnapshot {
    fn from(value: &CodeTaskIntent) -> Self {
        Self {
            prompt: value.prompt.clone(),
            context_paths: value.context_paths.clone(),
            constraints: value.constraints.clone(),
        }
    }
}

impl From<CodeTaskIntentSnapshot> for CodeTaskIntent {
    fn from(value: CodeTaskIntentSnapshot) -> Self {
        Self {
            prompt: value.prompt,
            context_paths: value.context_paths,
            constraints: value.constraints,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AuditPayloadSnapshot {
    event_id: String,
    event_type: String,
    severity: String,
    source: String,
    payload: String,
}

fn manifest_to_json(manifest: &AgentManifest) -> KernelResult<String> {
    serde_json::to_string(&AgentManifestSnapshot::from(manifest))
        .map_err(|error| KernelError::validation(format!("invalid manifest json: {error}")))
}

fn manifest_from_json(input: &str) -> KernelResult<AgentManifest> {
    let snapshot: AgentManifestSnapshot = serde_json::from_str(input)
        .map_err(|error| KernelError::validation(format!("invalid manifest json: {error}")))?;
    Ok(snapshot.into())
}

fn intent_to_json(intent: Option<&CodeTaskIntent>) -> KernelResult<Option<String>> {
    intent
        .map(|value| {
            serde_json::to_string(&CodeTaskIntentSnapshot::from(value)).map_err(|error| {
                KernelError::validation(format!("invalid default_code_task_intent json: {error}"))
            })
        })
        .transpose()
}

fn intent_from_json(input: Option<&str>) -> KernelResult<Option<CodeTaskIntent>> {
    input
        .map(|value| {
            serde_json::from_str::<CodeTaskIntentSnapshot>(value)
                .map(Into::into)
                .map_err(|error| {
                    KernelError::validation(format!(
                        "invalid default_code_task_intent json: {error}"
                    ))
                })
        })
        .transpose()
}

fn tags_to_json(tags: &[String]) -> KernelResult<String> {
    serde_json::to_string(tags)
        .map_err(|error| KernelError::validation(format!("invalid tags json: {error}")))
}

fn tags_from_json(input: &str) -> KernelResult<Vec<String>> {
    serde_json::from_str(input)
        .map_err(|error| KernelError::validation(format!("invalid tags json: {error}")))
}

fn string_list_to_json(values: &[String], field_name: &str) -> KernelResult<String> {
    serde_json::to_string(values)
        .map_err(|error| KernelError::validation(format!("invalid {field_name} json: {error}")))
}

fn string_list_from_json(input: &str, field_name: &str) -> KernelResult<Vec<String>> {
    serde_json::from_str(input)
        .map_err(|error| KernelError::validation(format!("invalid {field_name} json: {error}")))
}

fn severity_as_str(value: KernelEventSeverity) -> &'static str {
    match value {
        KernelEventSeverity::Debug => "debug",
        KernelEventSeverity::Info => "info",
        KernelEventSeverity::Warn => "warn",
        KernelEventSeverity::Error => "error",
    }
}

fn severity_from_str(value: &str) -> KernelResult<KernelEventSeverity> {
    match value {
        "debug" => Ok(KernelEventSeverity::Debug),
        "info" => Ok(KernelEventSeverity::Info),
        "warn" => Ok(KernelEventSeverity::Warn),
        "error" => Ok(KernelEventSeverity::Error),
        _ => Err(KernelError::validation(format!(
            "invalid audit severity: {value}"
        ))),
    }
}

fn source_as_str(value: KernelEventSource) -> &'static str {
    match value {
        KernelEventSource::Runtime => "runtime",
        KernelEventSource::Manifest => "manifest",
        KernelEventSource::Provider => "provider",
        KernelEventSource::Model => "model",
        KernelEventSource::Tool => "tool",
        KernelEventSource::Context => "context",
        KernelEventSource::Memory => "memory",
        KernelEventSource::Policy => "policy",
        KernelEventSource::Host => "host",
        KernelEventSource::ProtocolAdapter => "protocol_adapter",
        KernelEventSource::KernelUi => "kernel_ui",
        KernelEventSource::CodeKernel => "code_kernel",
        KernelEventSource::Telemetry => "telemetry",
        KernelEventSource::Unknown => "unknown",
    }
}

fn source_from_str(value: &str) -> KernelResult<KernelEventSource> {
    match value {
        "runtime" => Ok(KernelEventSource::Runtime),
        "manifest" => Ok(KernelEventSource::Manifest),
        "provider" => Ok(KernelEventSource::Provider),
        "model" => Ok(KernelEventSource::Model),
        "tool" => Ok(KernelEventSource::Tool),
        "context" => Ok(KernelEventSource::Context),
        "memory" => Ok(KernelEventSource::Memory),
        "policy" => Ok(KernelEventSource::Policy),
        "host" => Ok(KernelEventSource::Host),
        "protocol_adapter" => Ok(KernelEventSource::ProtocolAdapter),
        "kernel_ui" => Ok(KernelEventSource::KernelUi),
        "code_kernel" => Ok(KernelEventSource::CodeKernel),
        "telemetry" => Ok(KernelEventSource::Telemetry),
        "unknown" => Ok(KernelEventSource::Unknown),
        _ => Err(KernelError::validation(format!(
            "invalid audit source: {value}"
        ))),
    }
}

#[cfg(feature = "postgres-sync")]
fn expected_previous_version(next_version: u64) -> KernelResult<u64> {
    next_version
        .checked_sub(1)
        .ok_or_else(|| KernelError::validation("agent version must be >= 1 for update"))
}

#[cfg(feature = "postgres-sync")]
fn map_sqlx_error(error: sqlx::Error) -> KernelError {
    crate::postgres_sync_pool::map_sqlx_error(error)
}

#[cfg(feature = "postgres-sync")]
fn u64_to_i64(value: u64, field: &str) -> KernelResult<i64> {
    i64::try_from(value)
        .map_err(|_| KernelError::validation(format!("{field} exceeds postgres int64 range")))
}

#[cfg(feature = "postgres-sync")]
fn usize_to_i64(value: usize, field: &str) -> KernelResult<i64> {
    i64::try_from(value)
        .map_err(|_| KernelError::validation(format!("{field} exceeds postgres int64 range")))
}

#[cfg(feature = "postgres-sync")]
fn int64_to_u64(value: i64, field: &str) -> KernelResult<u64> {
    u64::try_from(value).map_err(|_| {
        KernelError::validation(format!("{field} must be a positive postgres int64 value"))
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_business_row(row: PgRow) -> KernelResult<AgentBusinessRow> {
    Ok(AgentBusinessRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        owner_user_id: int64_to_u64(
            row.try_get("owner_user_id").map_err(map_sqlx_error)?,
            "owner_user_id",
        )?,
        agent_id: row.try_get("agent_id").map_err(map_sqlx_error)?,
        code: row.try_get("code").map_err(map_sqlx_error)?,
        display_name: row.try_get("display_name").map_err(map_sqlx_error)?,
        description: row.try_get("description").map_err(map_sqlx_error)?,
        manifest_json: row.try_get("manifest_json").map_err(map_sqlx_error)?,
        default_code_task_intent_json: row
            .try_get("default_code_task_intent_json")
            .map_err(map_sqlx_error)?,
        implementation_provider_id: row
            .try_get("implementation_provider_id")
            .map_err(map_sqlx_error)?,
        implementation_kind: row.try_get("implementation_kind").map_err(map_sqlx_error)?,
        implementation_type: row.try_get("implementation_type").map_err(map_sqlx_error)?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        visibility: row.try_get("visibility").map_err(map_sqlx_error)?,
        tags_json: row.try_get("tags_json").map_err(map_sqlx_error)?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        deleted_at: row.try_get("deleted_at").map_err(map_sqlx_error)?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_workspace_row(row: PgRow) -> KernelResult<AgentWorkspaceRow> {
    let archived_by: Option<i64> = row.try_get("archived_by").map_err(map_sqlx_error)?;
    let deleted_by: Option<i64> = row.try_get("deleted_by").map_err(map_sqlx_error)?;
    Ok(AgentWorkspaceRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        workspace_id: row.try_get("workspace_id").map_err(map_sqlx_error)?,
        owner_user_id: int64_to_u64(
            row.try_get("owner_user_id").map_err(map_sqlx_error)?,
            "owner_user_id",
        )?,
        name: row.try_get("name").map_err(map_sqlx_error)?,
        description: row.try_get("description").map_err(map_sqlx_error)?,
        is_default: row.try_get("is_default").map_err(map_sqlx_error)?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        created_by: int64_to_u64(
            row.try_get("created_by").map_err(map_sqlx_error)?,
            "created_by",
        )?,
        updated_by: int64_to_u64(
            row.try_get("updated_by").map_err(map_sqlx_error)?,
            "updated_by",
        )?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        archived_at: row.try_get("archived_at").map_err(map_sqlx_error)?,
        archived_by: archived_by
            .map(|value| int64_to_u64(value, "archived_by"))
            .transpose()?,
        deleted_at: row.try_get("deleted_at").map_err(map_sqlx_error)?,
        deleted_by: deleted_by
            .map(|value| int64_to_u64(value, "deleted_by"))
            .transpose()?,
        retention_until: row.try_get("retention_until").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_project_row(row: PgRow) -> KernelResult<AgentProjectRow> {
    let archived_by: Option<i64> = row.try_get("archived_by").map_err(map_sqlx_error)?;
    let deleted_by: Option<i64> = row.try_get("deleted_by").map_err(map_sqlx_error)?;
    Ok(AgentProjectRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        project_id: row.try_get("project_id").map_err(map_sqlx_error)?,
        workspace_id: row.try_get("workspace_id").map_err(map_sqlx_error)?,
        owner_user_id: int64_to_u64(
            row.try_get("owner_user_id").map_err(map_sqlx_error)?,
            "owner_user_id",
        )?,
        name: row.try_get("name").map_err(map_sqlx_error)?,
        description: row.try_get("description").map_err(map_sqlx_error)?,
        visibility: row.try_get("visibility").map_err(map_sqlx_error)?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        drive_access_mode: row.try_get("drive_access_mode").map_err(map_sqlx_error)?,
        default_agent_id: row.try_get("default_agent_id").map_err(map_sqlx_error)?,
        default_model_id: row.try_get("default_model_id").map_err(map_sqlx_error)?,
        import_source_kind: row.try_get("import_source_kind").map_err(map_sqlx_error)?,
        import_source_ref: row.try_get("import_source_ref").map_err(map_sqlx_error)?,
        drive_space_id: row.try_get("drive_space_id").map_err(map_sqlx_error)?,
        drive_root_entry_id: row.try_get("drive_root_entry_id").map_err(map_sqlx_error)?,
        drive_logical_path: row.try_get("drive_logical_path").map_err(map_sqlx_error)?,
        created_by: int64_to_u64(
            row.try_get("created_by").map_err(map_sqlx_error)?,
            "created_by",
        )?,
        updated_by: int64_to_u64(
            row.try_get("updated_by").map_err(map_sqlx_error)?,
            "updated_by",
        )?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        archived_at: row.try_get("archived_at").map_err(map_sqlx_error)?,
        archived_by: archived_by
            .map(|value| int64_to_u64(value, "archived_by"))
            .transpose()?,
        deleted_at: row.try_get("deleted_at").map_err(map_sqlx_error)?,
        deleted_by: deleted_by
            .map(|value| int64_to_u64(value, "deleted_by"))
            .transpose()?,
        retention_until: row.try_get("retention_until").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_project_composition_slot_row(
    row: PgRow,
) -> KernelResult<AgentProjectCompositionSlotRow> {
    let deleted_by: Option<i64> = row.try_get("deleted_by").map_err(map_sqlx_error)?;
    Ok(AgentProjectCompositionSlotRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        project_id: row.try_get("project_id").map_err(map_sqlx_error)?,
        slot_id: row.try_get("slot_id").map_err(map_sqlx_error)?,
        slot_kind: row.try_get("slot_kind").map_err(map_sqlx_error)?,
        target_module: row.try_get("target_module").map_err(map_sqlx_error)?,
        target_ref: row.try_get("target_ref").map_err(map_sqlx_error)?,
        target_version_ref: row.try_get("target_version_ref").map_err(map_sqlx_error)?,
        priority: row.try_get("priority").map_err(map_sqlx_error)?,
        enabled: row.try_get("enabled").map_err(map_sqlx_error)?,
        policy_json: row.try_get("policy_json").map_err(map_sqlx_error)?,
        created_by: int64_to_u64(
            row.try_get("created_by").map_err(map_sqlx_error)?,
            "created_by",
        )?,
        updated_by: int64_to_u64(
            row.try_get("updated_by").map_err(map_sqlx_error)?,
            "updated_by",
        )?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        deleted_at: row.try_get("deleted_at").map_err(map_sqlx_error)?,
        deleted_by: deleted_by
            .map(|value| int64_to_u64(value, "deleted_by"))
            .transpose()?,
        retention_until: row.try_get("retention_until").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_provider_binding_row(row: PgRow) -> KernelResult<AgentProviderBindingRow> {
    Ok(AgentProviderBindingRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        agent_id: row.try_get("agent_id").map_err(map_sqlx_error)?,
        binding_id: row.try_get("binding_id").map_err(map_sqlx_error)?,
        provider_id: row.try_get("provider_id").map_err(map_sqlx_error)?,
        implementation_kind: row.try_get("implementation_kind").map_err(map_sqlx_error)?,
        configuration_profile_id: row
            .try_get("configuration_profile_id")
            .map_err(map_sqlx_error)?,
        capabilities_json: row.try_get("capabilities_json").map_err(map_sqlx_error)?,
        active: row.try_get("active").map_err(map_sqlx_error)?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn deserialize_optional_projection_row<T>(
    value: Option<String>,
    projection_name: &str,
) -> KernelResult<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    value
        .map(|value| {
            serde_json::from_str(&value).map_err(|error| KernelError::Internal {
                message: format!("stored Session activity {projection_name} is invalid: {error}"),
            })
        })
        .transpose()
}

#[cfg(feature = "postgres-sync")]
fn format_postgres_instant(value: OffsetDateTime, label: &str) -> KernelResult<String> {
    value
        .format(&Rfc3339)
        .map_err(|error| KernelError::Internal {
            message: format!("failed to format stored {label} timestamp as RFC3339: {error}"),
        })
}

fn deserialize_optional_interaction_projection_row(
    value: Option<String>,
) -> KernelResult<Option<AgentInteractionRow>> {
    value
        .map(|value| {
            let mut value: serde_json::Value =
                serde_json::from_str(&value).map_err(|error| KernelError::Internal {
                    message: format!(
                        "stored Session activity pending Interaction is invalid: {error}"
                    ),
                })?;
            if let Some(record) = value.as_object_mut() {
                for field in ["options_json", "resolution_json"] {
                    if let Some(field_value) = record.get_mut(field) {
                        if !field_value.is_null() && !field_value.is_string() {
                            *field_value = serde_json::Value::String(field_value.to_string());
                        }
                    }
                }
            }
            serde_json::from_value(value).map_err(|error| KernelError::Internal {
                message: format!("stored Session activity pending Interaction is invalid: {error}"),
            })
        })
        .transpose()
}

fn pg_row_to_agent_session_row(row: PgRow) -> KernelResult<AgentSessionRow> {
    let archived_by: Option<i64> = row.try_get("archived_by").map_err(map_sqlx_error)?;
    let deleted_by: Option<i64> = row.try_get("deleted_by").map_err(map_sqlx_error)?;
    Ok(AgentSessionRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        agent_id: row.try_get("agent_id").map_err(map_sqlx_error)?,
        session_id: row.try_get("session_id").map_err(map_sqlx_error)?,
        owner_user_id: int64_to_u64(
            row.try_get("owner_user_id").map_err(map_sqlx_error)?,
            "owner_user_id",
        )?,
        project_id: row.try_get("project_id").map_err(map_sqlx_error)?,
        session_kind: row.try_get("session_kind").map_err(map_sqlx_error)?,
        entry_surface: row.try_get("entry_surface").map_err(map_sqlx_error)?,
        source_module: row.try_get("source_module").map_err(map_sqlx_error)?,
        source_context_kind: row.try_get("source_context_kind").map_err(map_sqlx_error)?,
        source_context_id: row.try_get("source_context_id").map_err(map_sqlx_error)?,
        parent_session_id: row.try_get("parent_session_id").map_err(map_sqlx_error)?,
        forked_from_turn_id: row.try_get("forked_from_turn_id").map_err(map_sqlx_error)?,
        title: row.try_get("title").map_err(map_sqlx_error)?,
        title_source: row.try_get("title_source").map_err(map_sqlx_error)?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        item_count: int64_to_u64(
            row.try_get("item_count").map_err(map_sqlx_error)?,
            "item_count",
        )?,
        last_item_sequence: int64_to_u64(
            row.try_get("last_item_sequence").map_err(map_sqlx_error)?,
            "last_item_sequence",
        )?,
        total_input_tokens: int64_to_u64(
            row.try_get("total_input_tokens").map_err(map_sqlx_error)?,
            "total_input_tokens",
        )?,
        total_output_tokens: int64_to_u64(
            row.try_get("total_output_tokens").map_err(map_sqlx_error)?,
            "total_output_tokens",
        )?,
        idempotency_key: row.try_get("idempotency_key").map_err(map_sqlx_error)?,
        payload_hash: row.try_get("payload_hash").map_err(map_sqlx_error)?,
        created_by: int64_to_u64(
            row.try_get("created_by").map_err(map_sqlx_error)?,
            "created_by",
        )?,
        updated_by: int64_to_u64(
            row.try_get("updated_by").map_err(map_sqlx_error)?,
            "updated_by",
        )?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        last_item_at: row.try_get("last_item_at").map_err(map_sqlx_error)?,
        closed_at: row.try_get("closed_at").map_err(map_sqlx_error)?,
        archived_at: row.try_get("archived_at").map_err(map_sqlx_error)?,
        archived_by: archived_by
            .map(|value| int64_to_u64(value, "archived_by"))
            .transpose()?,
        deleted_at: row.try_get("deleted_at").map_err(map_sqlx_error)?,
        deleted_by: deleted_by
            .map(|value| int64_to_u64(value, "deleted_by"))
            .transpose()?,
        retention_until: row.try_get("retention_until").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_session_runtime_binding_row(
    row: PgRow,
) -> KernelResult<AgentSessionRuntimeBindingRow> {
    Ok(AgentSessionRuntimeBindingRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        owner_user_id: int64_to_u64(
            row.try_get("owner_user_id").map_err(map_sqlx_error)?,
            "owner_user_id",
        )?,
        session_id: row.try_get("session_id").map_err(map_sqlx_error)?,
        runtime_binding_id: row.try_get("runtime_binding_id").map_err(map_sqlx_error)?,
        runtime_location_id: row.try_get("runtime_location_id").map_err(map_sqlx_error)?,
        host_mode: row.try_get("host_mode").map_err(map_sqlx_error)?,
        transport_kind: row.try_get("transport_kind").map_err(map_sqlx_error)?,
        provider_binding_id: row.try_get("provider_binding_id").map_err(map_sqlx_error)?,
        model_id: row.try_get("model_id").map_err(map_sqlx_error)?,
        provider_id: row.try_get("provider_id").map_err(map_sqlx_error)?,
        provider_session_id: row.try_get("provider_session_id").map_err(map_sqlx_error)?,
        provider_session_tree_id: row
            .try_get("provider_session_tree_id")
            .map_err(map_sqlx_error)?,
        provider_parent_session_id: row
            .try_get("provider_parent_session_id")
            .map_err(map_sqlx_error)?,
        provider_forked_from_session_id: row
            .try_get("provider_forked_from_session_id")
            .map_err(map_sqlx_error)?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        is_current: row.try_get("is_current").map_err(map_sqlx_error)?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        activated_at: row.try_get("activated_at").map_err(map_sqlx_error)?,
        deactivated_at: row.try_get("deactivated_at").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_session_checkpoint_row(row: PgRow) -> KernelResult<AgentSessionCheckpointRow> {
    Ok(AgentSessionCheckpointRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        session_id: row.try_get("session_id").map_err(map_sqlx_error)?,
        checkpoint_id: row.try_get("checkpoint_id").map_err(map_sqlx_error)?,
        turn_id: row.try_get("turn_id").map_err(map_sqlx_error)?,
        runtime_binding_id: row.try_get("runtime_binding_id").map_err(map_sqlx_error)?,
        checkpoint_kind: row.try_get("checkpoint_kind").map_err(map_sqlx_error)?,
        provider_checkpoint_ref: row
            .try_get("provider_checkpoint_ref")
            .map_err(map_sqlx_error)?,
        drive_space_id: row.try_get("drive_space_id").map_err(map_sqlx_error)?,
        drive_node_id: row.try_get("drive_node_id").map_err(map_sqlx_error)?,
        resumable: row.try_get("resumable").map_err(map_sqlx_error)?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        created_by: int64_to_u64(
            row.try_get("created_by").map_err(map_sqlx_error)?,
            "created_by",
        )?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        restored_at: row.try_get("restored_at").map_err(map_sqlx_error)?,
        invalidated_at: row.try_get("invalidated_at").map_err(map_sqlx_error)?,
        retention_until: row.try_get("retention_until").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_resource_user_state_row(row: PgRow) -> KernelResult<AgentResourceUserStateRow> {
    Ok(AgentResourceUserStateRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        user_id: int64_to_u64(row.try_get("user_id").map_err(map_sqlx_error)?, "user_id")?,
        resource_type: row.try_get("resource_type").map_err(map_sqlx_error)?,
        resource_id: row.try_get("resource_id").map_err(map_sqlx_error)?,
        pinned_at: row.try_get("pinned_at").map_err(map_sqlx_error)?,
        hidden_at: row.try_get("hidden_at").map_err(map_sqlx_error)?,
        last_opened_at: row.try_get("last_opened_at").map_err(map_sqlx_error)?,
        last_read_item_sequence: row
            .try_get::<Option<i64>, _>("last_read_item_sequence")
            .map_err(map_sqlx_error)?
            .map(|value| int64_to_u64(value, "last_read_item_sequence"))
            .transpose()?,
        custom_title: row.try_get("custom_title").map_err(map_sqlx_error)?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_item_feedback_row(row: PgRow) -> KernelResult<AgentItemFeedbackRow> {
    Ok(AgentItemFeedbackRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        item_id: row.try_get("item_id").map_err(map_sqlx_error)?,
        user_id: int64_to_u64(row.try_get("user_id").map_err(map_sqlx_error)?, "user_id")?,
        rating: row.try_get("rating").map_err(map_sqlx_error)?,
        reason_code: row.try_get("reason_code").map_err(map_sqlx_error)?,
        comment: row.try_get("comment").map_err(map_sqlx_error)?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        deleted_at: row.try_get("deleted_at").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_item_drive_ref_row(row: PgRow) -> KernelResult<AgentItemDriveRefRow> {
    let sort_order: i32 = row.try_get("sort_order").map_err(map_sqlx_error)?;
    let created_by: i64 = row.try_get("created_by").map_err(map_sqlx_error)?;
    Ok(AgentItemDriveRefRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        item_id: row.try_get("item_id").map_err(map_sqlx_error)?,
        resource_role: row.try_get("resource_role").map_err(map_sqlx_error)?,
        drive_space_id: row.try_get("drive_space_id").map_err(map_sqlx_error)?,
        drive_node_id: row.try_get("drive_node_id").map_err(map_sqlx_error)?,
        media_resource_id: row.try_get("media_resource_id").map_err(map_sqlx_error)?,
        object_blob_id: row.try_get("object_blob_id").map_err(map_sqlx_error)?,
        resource_hash: row.try_get("resource_hash").map_err(map_sqlx_error)?,
        alt_text: row.try_get("alt_text").map_err(map_sqlx_error)?,
        sort_order: u32::try_from(sort_order)
            .map_err(|_| KernelError::validation("invalid Drive reference sort_order"))?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        created_by: int64_to_u64(created_by, "created_by")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        deleted_at: row.try_get("deleted_at").map_err(map_sqlx_error)?,
        retention_until: row.try_get("retention_until").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_session_item_row(row: PgRow) -> KernelResult<AgentSessionItemRow> {
    let redacted_by: Option<i64> = row.try_get("redacted_by").map_err(map_sqlx_error)?;
    Ok(AgentSessionItemRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        session_id: row.try_get("session_id").map_err(map_sqlx_error)?,
        item_id: row.try_get("item_id").map_err(map_sqlx_error)?,
        kind: row.try_get("kind").map_err(map_sqlx_error)?,
        content: row.try_get("content").map_err(map_sqlx_error)?,
        content_type: row.try_get("content_type").map_err(map_sqlx_error)?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        sequence: int64_to_u64(row.try_get("sequence").map_err(map_sqlx_error)?, "sequence")?,
        input_tokens: int64_to_u64(
            row.try_get("input_tokens").map_err(map_sqlx_error)?,
            "input_tokens",
        )?,
        output_tokens: int64_to_u64(
            row.try_get("output_tokens").map_err(map_sqlx_error)?,
            "output_tokens",
        )?,
        model_id: row.try_get("model_id").map_err(map_sqlx_error)?,
        provider_id: row.try_get("provider_id").map_err(map_sqlx_error)?,
        tool_name: row.try_get("tool_name").map_err(map_sqlx_error)?,
        tool_call_id: row.try_get("tool_call_id").map_err(map_sqlx_error)?,
        tool_arguments_json: row.try_get("tool_arguments_json").map_err(map_sqlx_error)?,
        tool_result_json: row.try_get("tool_result_json").map_err(map_sqlx_error)?,
        parent_item_id: row.try_get("parent_item_id").map_err(map_sqlx_error)?,
        turn_id: row.try_get("turn_id").map_err(map_sqlx_error)?,
        created_by: int64_to_u64(
            row.try_get("created_by").map_err(map_sqlx_error)?,
            "created_by",
        )?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        completed_at: row.try_get("completed_at").map_err(map_sqlx_error)?,
        redacted_at: row.try_get("redacted_at").map_err(map_sqlx_error)?,
        redacted_by: redacted_by
            .map(|value| int64_to_u64(value, "redacted_by"))
            .transpose()?,
        retention_until: row.try_get("retention_until").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_turn_row(row: PgRow) -> KernelResult<AgentTurnRow> {
    Ok(AgentTurnRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        turn_id: row.try_get("turn_id").map_err(map_sqlx_error)?,
        session_id: row.try_get("session_id").map_err(map_sqlx_error)?,
        agent_id: row.try_get("agent_id").map_err(map_sqlx_error)?,
        owner_user_id: int64_to_u64(
            row.try_get("owner_user_id").map_err(map_sqlx_error)?,
            "owner_user_id",
        )?,
        runtime_binding_id: row.try_get("runtime_binding_id").map_err(map_sqlx_error)?,
        client_request_id: row.try_get("client_request_id").map_err(map_sqlx_error)?,
        idempotency_key: row.try_get("idempotency_key").map_err(map_sqlx_error)?,
        payload_hash: row.try_get("payload_hash").map_err(map_sqlx_error)?,
        request_item_id: row.try_get("request_item_id").map_err(map_sqlx_error)?,
        response_item_id: row.try_get("response_item_id").map_err(map_sqlx_error)?,
        turn_mode: row.try_get("turn_mode").map_err(map_sqlx_error)?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        requested_model_id: row.try_get("requested_model_id").map_err(map_sqlx_error)?,
        provider_binding_id: row.try_get("provider_binding_id").map_err(map_sqlx_error)?,
        model_id: row.try_get("model_id").map_err(map_sqlx_error)?,
        provider_id: row.try_get("provider_id").map_err(map_sqlx_error)?,
        input_tokens: int64_to_u64(
            row.try_get("input_tokens").map_err(map_sqlx_error)?,
            "input_tokens",
        )?,
        output_tokens: int64_to_u64(
            row.try_get("output_tokens").map_err(map_sqlx_error)?,
            "output_tokens",
        )?,
        cached_tokens: int64_to_u64(
            row.try_get("cached_tokens").map_err(map_sqlx_error)?,
            "cached_tokens",
        )?,
        finish_reason: row.try_get("finish_reason").map_err(map_sqlx_error)?,
        error_code: row.try_get("error_code").map_err(map_sqlx_error)?,
        error_detail: row.try_get("error_detail").map_err(map_sqlx_error)?,
        trace_id: row.try_get("trace_id").map_err(map_sqlx_error)?,
        attempt_count: u32::try_from(
            row.try_get::<i32, _>("attempt_count")
                .map_err(map_sqlx_error)?,
        )
        .map_err(|_| KernelError::validation("invalid turn attempt_count"))?,
        max_attempts: u32::try_from(
            row.try_get::<i32, _>("max_attempts")
                .map_err(map_sqlx_error)?,
        )
        .map_err(|_| KernelError::validation("invalid turn max_attempts"))?,
        next_retry_at: row.try_get("next_retry_at").map_err(map_sqlx_error)?,
        available_at: row.try_get("available_at").map_err(map_sqlx_error)?,
        lease_owner: row.try_get("lease_owner").map_err(map_sqlx_error)?,
        lease_token: row.try_get("lease_token").map_err(map_sqlx_error)?,
        lease_expires_at: row.try_get("lease_expires_at").map_err(map_sqlx_error)?,
        fencing_token: int64_to_u64(
            row.try_get("fencing_token").map_err(map_sqlx_error)?,
            "fencing_token",
        )?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        started_at: row.try_get("started_at").map_err(map_sqlx_error)?,
        completed_at: row.try_get("completed_at").map_err(map_sqlx_error)?,
        cancel_requested_at: row.try_get("cancel_requested_at").map_err(map_sqlx_error)?,
        cancelled_at: row.try_get("cancelled_at").map_err(map_sqlx_error)?,
        retention_until: row.try_get("retention_until").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_interaction_row(row: PgRow) -> KernelResult<AgentInteractionRow> {
    Ok(AgentInteractionRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        session_id: row.try_get("session_id").map_err(map_sqlx_error)?,
        turn_id: row.try_get("turn_id").map_err(map_sqlx_error)?,
        runtime_binding_id: row.try_get("runtime_binding_id").map_err(map_sqlx_error)?,
        interaction_id: row.try_get("interaction_id").map_err(map_sqlx_error)?,
        provider_interaction_id: row
            .try_get("provider_interaction_id")
            .map_err(map_sqlx_error)?,
        kind: row.try_get("kind").map_err(map_sqlx_error)?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        prompt: row.try_get("prompt").map_err(map_sqlx_error)?,
        options_json: row.try_get("options_json").map_err(map_sqlx_error)?,
        resolution_json: row.try_get("resolution_json").map_err(map_sqlx_error)?,
        claim_owner: row.try_get("claim_owner").map_err(map_sqlx_error)?,
        claim_token_hash: row.try_get("claim_token_hash").map_err(map_sqlx_error)?,
        claim_expires_at: row.try_get("claim_expires_at").map_err(map_sqlx_error)?,
        fencing_token: int64_to_u64(
            row.try_get("fencing_token").map_err(map_sqlx_error)?,
            "fencing_token",
        )?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        resolved_at: row.try_get("resolved_at").map_err(map_sqlx_error)?,
        retention_until: row.try_get("retention_until").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_task_row(row: PgRow) -> KernelResult<AgentTaskRow> {
    Ok(AgentTaskRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(map_sqlx_error)?,
            "organization_id",
        )?,
        agent_id: row.try_get("agent_id").map_err(map_sqlx_error)?,
        task_id: row.try_get("task_id").map_err(map_sqlx_error)?,
        owner_user_id: int64_to_u64(
            row.try_get("owner_user_id").map_err(map_sqlx_error)?,
            "owner_user_id",
        )?,
        title: row.try_get("title").map_err(map_sqlx_error)?,
        prompt: row.try_get("prompt").map_err(map_sqlx_error)?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        external_ref: row.try_get("external_ref").map_err(map_sqlx_error)?,
        metadata_json: row.try_get("metadata_json").map_err(map_sqlx_error)?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        started_at: row.try_get("started_at").map_err(map_sqlx_error)?,
        completed_at: row.try_get("completed_at").map_err(map_sqlx_error)?,
        cancelled_at: row.try_get("cancelled_at").map_err(map_sqlx_error)?,
    })
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "postgres-sync")]
    use super::format_postgres_instant;
    use super::{
        audit_actor_from_subject_id, build_agent_business_uuid, build_agent_provider_binding_uuid,
        build_composition_slot_uuid, build_interaction_uuid, build_session_item_uuid,
        build_session_uuid, build_task_uuid, extract_event_context, AgentAuditEventRow,
        AgentProjectCompositionSlotRow,
    };
    #[cfg(feature = "postgres-sync")]
    use crate::session_activity::{
        decode_session_activity_cursor, encode_session_activity_cursor, SessionActivityCursor,
    };
    use crate::{
        AgentCompositionSlotKind, AgentCompositionTargetModule, AgentProjectCompositionSlotRecord,
    };
    use sdkwork_agent_kernel::{KernelError, KernelEvent, KernelEventSeverity, KernelEventSource};
    #[cfg(feature = "postgres-sync")]
    use time::{format_description::well_known::Rfc3339, OffsetDateTime};

    #[cfg(feature = "postgres-sync")]
    #[test]
    fn postgres_session_activity_instant_produces_decodable_cursor() {
        let value = OffsetDateTime::parse("2026-07-28T04:12:34.123456Z", &Rfc3339).unwrap();
        let cursor = SessionActivityCursor {
            activity_at: format_postgres_instant(value, "Session activity").unwrap(),
            session_internal_id: 42,
            scope_fingerprint: "scope-fingerprint".to_string(),
        };
        let encoded = encode_session_activity_cursor(&cursor).unwrap();

        assert_eq!(cursor.activity_at, "2026-07-28T04:12:34.123456Z");
        assert_eq!(decode_session_activity_cursor(&encoded).unwrap(), cursor);
    }

    #[test]
    fn document_project_composition_slot_roundtrips_through_postgres_row_mapping() {
        let record = AgentProjectCompositionSlotRecord {
            id: 7,
            tenant_id: 10,
            organization_id: 20,
            project_id: "project.alpha".to_string(),
            slot_id: "slot.documents".to_string(),
            slot_kind: AgentCompositionSlotKind::Document,
            target_module: AgentCompositionTargetModule::Documents,
            target_ref: "document.project.specification".to_string(),
            target_version_ref: Some("document.version.1".to_string()),
            priority: 0,
            enabled: true,
            policy_json: "{}".to_string(),
            created_by: 30,
            updated_by: 30,
            version: 0,
            created_at: "2026-07-19T00:00:00Z".to_string(),
            updated_at: "2026-07-19T00:00:00Z".to_string(),
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        };

        let row = AgentProjectCompositionSlotRow::from_record(&record);
        assert_eq!(row.slot_kind, "document");
        assert_eq!(row.target_module, "documents");

        let restored = row.into_record().unwrap();
        assert_eq!(restored.slot_kind, AgentCompositionSlotKind::Document);
        assert_eq!(
            restored.target_module,
            AgentCompositionTargetModule::Documents
        );
        assert_eq!(restored.target_ref, record.target_ref);
    }

    #[test]
    fn storage_uuids_are_stable_bounded_and_resource_scoped() {
        let tenant_id = 4_096_123_456_789_012_345;
        let agent_id = format!("agent.pc.{}.123456789abc", "a".repeat(48));
        let session_id = format!("session.pc.{}", "s".repeat(100));
        let item_id = format!("message.pc.{}", "m".repeat(100));
        let interaction_id = format!("interaction.pc.{}", "i".repeat(100));
        let task_id = format!("task.pc.{}", "t".repeat(100));
        let slot_id = format!("slot.pc.{}", "c".repeat(100));
        let binding_id = format!("binding.pc.{}", "b".repeat(100));

        let uuids = [
            build_agent_business_uuid(tenant_id, &agent_id),
            build_agent_provider_binding_uuid(tenant_id, &agent_id, &binding_id),
            build_composition_slot_uuid(tenant_id, &agent_id, &slot_id),
            build_session_uuid(tenant_id, &session_id),
            build_session_item_uuid(tenant_id, &session_id, &item_id),
            build_interaction_uuid(tenant_id, &session_id, &interaction_id),
            build_task_uuid(tenant_id, &task_id),
        ];

        assert!(uuids.iter().all(|uuid| uuid.len() == 64));
        assert_eq!(uuids[0], build_agent_business_uuid(tenant_id, &agent_id));
        let mut distinct = uuids.to_vec();
        distinct.sort();
        distinct.dedup();
        assert_eq!(distinct.len(), uuids.len());
    }

    #[test]
    fn storage_uuid_digest_contract_is_stable() {
        assert_eq!(
            build_agent_business_uuid(100_001, "agent.alpha"),
            "c11c4a8d54102db9c618039faae87657511c2a55846e2d9b43b6d7916d357599"
        );
    }

    #[test]
    fn long_create_audit_event_uses_bounded_storage_uuid() {
        let tenant_id = 4_096_123_456_789_012_345_u64;
        let agent_id = format!("agent.pc.{}.123456789abc", "a".repeat(48));
        let event = KernelEvent::new(
            format!("agent_audit_{agent_id}_1"),
            "agent.business.created",
            KernelEventSeverity::Info,
            serde_json::json!({
                "_context": {
                    "aggregate_type": "agent",
                    "aggregate_id": agent_id,
                    "tenant_id": tenant_id.to_string(),
                    "agent_id": agent_id,
                    "agent_internal_id": "4096123456789012346",
                    "subject_id": "700"
                }
            })
            .to_string(),
        )
        .from_source(KernelEventSource::Runtime)
        .occurred_at("2026-07-18T00:00:00Z");

        let row = AgentAuditEventRow::from_kernel_event(&event, 1)
            .expect("long create audit event should map to storage");

        assert_eq!(row.uuid.len(), 64);
        assert_eq!(row.agent_id.as_deref(), Some(agent_id.as_str()));
        assert_eq!((row.actor_type, row.actor_id), (0, 700));
    }

    #[test]
    fn turn_audit_event_keeps_agent_context_out_of_agent_scope_columns() {
        let event = KernelEvent::new(
            "agent_turn_turn.123_requested_0",
            "agent.business.turn.requested",
            KernelEventSeverity::Info,
            serde_json::json!({
                "_context": {
                    "aggregate_type": "turn",
                    "aggregate_id": "turn.123",
                    "tenant_id": "100",
                    "organization_id": "200",
                    "agent_id": "agent.birdcoder",
                    "agent_internal_id": "300",
                    "subject_id": "700"
                }
            })
            .to_string(),
        )
        .from_source(KernelEventSource::Runtime)
        .occurred_at("2026-07-28T00:00:00Z");

        let row = AgentAuditEventRow::from_kernel_event(&event, 1)
            .expect("turn audit event should satisfy the non-agent scope constraint");

        assert_eq!(row.aggregate_type, "turn");
        assert_eq!(row.aggregate_id, "turn.123");
        assert_eq!(row.agent_id, None);
        assert_eq!(row.agent_internal_id, None);
        let payload_snapshot: serde_json::Value = serde_json::from_str(row.payload_json.as_str())
            .expect("audit payload snapshot should remain valid JSON");
        let original_payload = payload_snapshot["payload"]
            .as_str()
            .expect("audit payload snapshot should retain the original payload");
        assert_eq!(
            extract_event_context(original_payload, "agent_id"),
            Some("agent.birdcoder".to_string())
        );
    }

    #[test]
    fn agent_audit_event_preserves_agent_scope_columns() {
        let event = KernelEvent::new(
            "agent_audit_agent.birdcoder_1",
            "agent.business.updated",
            KernelEventSeverity::Info,
            serde_json::json!({
                "_context": {
                    "aggregate_type": "agent",
                    "aggregate_id": "agent.birdcoder",
                    "tenant_id": "100",
                    "agent_id": "agent.birdcoder",
                    "agent_internal_id": "300",
                    "subject_id": "700"
                }
            })
            .to_string(),
        )
        .from_source(KernelEventSource::Runtime)
        .occurred_at("2026-07-28T00:00:00Z");

        let row = AgentAuditEventRow::from_kernel_event(&event, 1)
            .expect("agent audit event should preserve its agent scope");

        assert_eq!(row.agent_id.as_deref(), Some("agent.birdcoder"));
        assert_eq!(row.agent_internal_id, Some(300));
    }

    #[test]
    fn audit_event_rejects_implicit_agent_aggregate_fallback() {
        let event = KernelEvent::new(
            "agent_audit_agent.birdcoder_1",
            "agent.business.updated",
            KernelEventSeverity::Info,
            serde_json::json!({
                "_context": {
                    "tenant_id": "100",
                    "agent_id": "agent.birdcoder",
                    "agent_internal_id": "300",
                    "subject_id": "700"
                }
            })
            .to_string(),
        )
        .from_source(KernelEventSource::Runtime)
        .occurred_at("2026-07-28T00:00:00Z");

        let error = AgentAuditEventRow::from_kernel_event(&event, 1)
            .expect_err("audit aggregates must always be explicit");

        assert!(error
            .to_string()
            .contains("audit aggregate_type context is required"));
    }

    #[test]
    fn audit_actor_maps_numeric_user_subject() {
        assert_eq!(audit_actor_from_subject_id("700").unwrap(), (0, 700));
    }

    #[test]
    fn audit_actor_maps_numeric_service_subject() {
        assert_eq!(
            audit_actor_from_subject_id("service.701").unwrap(),
            (1, 701)
        );
    }

    #[test]
    fn audit_actor_maps_agents_system_subject() {
        assert_eq!(
            audit_actor_from_subject_id("system.agents.reconciliation").unwrap(),
            (2, 0)
        );
    }

    #[test]
    fn audit_actor_rejects_opaque_authenticated_subject() {
        let error = audit_actor_from_subject_id("user.700")
            .expect_err("opaque authenticated subjects must fail before SQL persistence");
        assert!(matches!(error, KernelError::Validation { .. }));
    }

    #[test]
    fn extract_returns_value_from_context_field() {
        let payload = r#"{"action":"create","agent_id":"agent.123","_context":{"agent_id":"agent.123","tenant_id":"100"}}"#;
        assert_eq!(
            extract_event_context(payload, "agent_id"),
            Some("agent.123".to_string())
        );
        assert_eq!(
            extract_event_context(payload, "tenant_id"),
            Some("100".to_string())
        );
    }

    #[test]
    fn extract_returns_none_for_missing_key() {
        let payload = r#"{"action":"create","_context":{"agent_id":"agent.123"}}"#;
        assert_eq!(extract_event_context(payload, "tenant_id"), None);
        assert_eq!(extract_event_context(payload, "missing_key"), None);
    }

    #[test]
    fn extract_returns_none_for_empty_payload() {
        assert_eq!(extract_event_context("", "agent_id"), None);
        assert_eq!(extract_event_context("   ", "agent_id"), None);
    }

    #[test]
    fn extract_returns_none_for_non_json_payload() {
        assert_eq!(extract_event_context("not json at all", "agent_id"), None);
    }

    #[test]
    fn extract_context_takes_precedence_over_root_field() {
        // The _context value should win over a root-level field with the
        // same key, mirroring the old "last occurrence wins" behaviour.
        let payload = r#"{"agent_id":"root.value","_context":{"agent_id":"context.value"}}"#;
        assert_eq!(
            extract_event_context(payload, "agent_id"),
            Some("context.value".to_string())
        );
    }

    #[test]
    fn extract_falls_back_to_root_level_string_field() {
        // Events without _context should still find values at the root level.
        let payload = r#"{"agent_id":"agent.999","tenant_id":"100"}"#;
        assert_eq!(
            extract_event_context(payload, "agent_id"),
            Some("agent.999".to_string())
        );
        assert_eq!(
            extract_event_context(payload, "tenant_id"),
            Some("100".to_string())
        );
    }

    #[test]
    fn extract_falls_back_to_root_level_numeric_field() {
        // Numeric root-level fields should be converted to strings.
        let payload = r#"{"tenant_id":100001,"organization_id":1}"#;
        assert_eq!(
            extract_event_context(payload, "tenant_id"),
            Some("100001".to_string())
        );
        assert_eq!(
            extract_event_context(payload, "organization_id"),
            Some("1".to_string())
        );
    }

    #[test]
    fn extract_does_not_match_prefixed_keys() {
        // `agent_id` must not match `agent_internal_id` and
        // `tenant_id` must not match `subject_tenant_id`.
        let payload = r#"{"_context":{"agent_internal_id":"42","subject_tenant_id":"200","agent_id":"agent.999","tenant_id":"100"}}"#;
        assert_eq!(
            extract_event_context(payload, "agent_id"),
            Some("agent.999".to_string())
        );
        assert_eq!(
            extract_event_context(payload, "tenant_id"),
            Some("100".to_string())
        );
        assert_eq!(
            extract_event_context(payload, "agent_internal_id"),
            Some("42".to_string())
        );
        assert_eq!(
            extract_event_context(payload, "subject_tenant_id"),
            Some("200".to_string())
        );
    }

    #[test]
    fn extract_handles_real_audit_event_payload_shape() {
        // Reproduces the exact payload produced by
        // `AgentsService::emit_audit_event` after `with_context` calls.
        let payload = r#"{"schema_version":"v1","action":"created","agent_id":"agent.business.001","tenant_id":100001,"organization_id":1,"owner_user_id":10,"code":"agent.business.001","status":"active","visibility":"organization","version":1,"_context":{"schema_version":"v1","subject_id":"user.42","subject_tenant_id":"100001","agent_id":"agent.business.001","tenant_id":"100001","organization_id":"1","agent_internal_id":"99"}}"#;
        assert_eq!(
            extract_event_context(payload, "tenant_id"),
            Some("100001".to_string())
        );
        assert_eq!(
            extract_event_context(payload, "organization_id"),
            Some("1".to_string())
        );
        assert_eq!(
            extract_event_context(payload, "agent_internal_id"),
            Some("99".to_string())
        );
        assert_eq!(
            extract_event_context(payload, "subject_id"),
            Some("user.42".to_string())
        );
    }
}
