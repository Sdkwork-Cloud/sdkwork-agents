use crate::chat_turn::{AgentChatTurnRecord, AgentChatTurnStatus};
use crate::domain::{
    AgentBusinessRecord, AgentBusinessStatus, AgentCompositionSlotKind, AgentCompositionSlotRecord,
    AgentCompositionTargetModule, AgentImplementationKind, AgentImplementationType,
    AgentInteractionKind, AgentInteractionRecord, AgentInteractionStatus,
    AgentMessageDriveRefRecord, AgentMessageFeedbackRating, AgentMessageFeedbackRecord,
    AgentMessageMediaRole, AgentMessageRecord, AgentMessageRole, AgentMessageStatus,
    AgentProviderBindingRecord, AgentResourceType, AgentResourceUserStateRecord,
    AgentSessionRecord, AgentSessionStatus, AgentTaskRecord, AgentTaskStatus, AgentVisibility,
};
use crate::ports::{
    AgentAuditSink, AgentListQuery, AgentRepository, AuditEventListQuery, CompositionSlotListQuery,
    InteractionListQuery, McpMarketplaceListQuery, MessageFeedbackListQuery, MessageListQuery,
    MessageListSort, ProjectCompositionSlotListQuery, ProjectListQuery, ProviderBindingListQuery,
    ResourceUserStateListQuery, SessionListQuery, TaskListQuery,
};
#[cfg(feature = "postgres-sync")]
use crate::postgres_sync_pool::{BlockingPostgresPool, PgRow};
use crate::project::{
    AgentProjectCompositionSlotRecord, AgentProjectDriveAccessMode, AgentProjectRecord,
    AgentProjectStatus, AgentProjectVisibility,
};
#[cfg(feature = "sqlite-sync")]
use crate::sqlite_sync_pool::{BlockingSqlitePool, SQLITE_MANAGED_STORE_DATABASE_SERVICE};
use crate::validation::{validate_capabilities, validate_standard_id};
#[cfg(feature = "postgres-sync")]
use crate::{pg_execute, pg_query, pg_query_optional};
use sdkwork_agent_kernel::{
    AgentManifest, KernelError, KernelEvent, KernelEventSeverity, KernelEventSource, KernelResult,
};
use sdkwork_code_kernel::CodeTaskIntent;
use sdkwork_utils_rust::{is_blank, sha256_hash, trim};
use serde::{Deserialize, Serialize};
#[cfg(feature = "sqlite-sync")]
use sqlx::sqlite::SqliteRow;
#[cfg(any(feature = "postgres-sync", feature = "sqlite-sync"))]
use sqlx::Row;

#[cfg(feature = "postgres-sync")]
use std::future::Future;

/// Maximum number of retries when a PostgreSQL deadlock (SQLSTATE 40P01) is detected.
#[cfg(feature = "postgres-sync")]
const DEADLOCK_MAX_RETRIES: usize = 3;

/// Initial backoff (milliseconds) before the first deadlock retry.
/// Backoff doubles on each retry: 10 ms, 20 ms, 40 ms.
#[cfg(feature = "postgres-sync")]
const DEADLOCK_INITIAL_BACKOFF_MS: u64 = 10;

/// Returns true when the supplied error is a PostgreSQL `deadlock_detected`
/// error (SQLSTATE 40P01). Such errors are safe to retry after the failed
/// transaction has been rolled back.
#[cfg(feature = "postgres-sync")]
fn is_postgres_deadlock(error: &sqlx::Error) -> bool {
    matches!(
        error,
        sqlx::Error::Database(db) if db.code().map(|code| code == "40P01").unwrap_or(false)
    )
}

/// Executes `operation` and retries on PostgreSQL deadlock (SQLSTATE 40P01)
/// using exponential backoff. Up to [`DEADLOCK_MAX_RETRIES`] retries are
/// attempted (10 ms, 20 ms, 40 ms). Non-deadlock errors are returned
/// immediately without retrying.
///
/// The closure must create a fresh transaction on each call — any state
/// mutated inside the transaction body must be cloned from captured
/// references so that a retry starts from a clean snapshot.
#[cfg(feature = "postgres-sync")]
async fn retry_on_deadlock<T, F, Fut>(operation: F) -> Result<T, sqlx::Error>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, sqlx::Error>>,
{
    let mut backoff_ms = DEADLOCK_INITIAL_BACKOFF_MS;
    let mut last_error: Option<sqlx::Error> = None;
    for attempt in 0..=DEADLOCK_MAX_RETRIES {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                if !is_postgres_deadlock(&error) {
                    return Err(error);
                }
                tracing::warn!(
                    target: "sdkwork.agents.persistence.deadlock_retry",
                    attempt,
                    max_retries = DEADLOCK_MAX_RETRIES,
                    backoff_ms,
                    "postgres deadlock detected, retrying transaction"
                );
                last_error = Some(error);
                if attempt < DEADLOCK_MAX_RETRIES {
                    tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                    backoff_ms = backoff_ms.saturating_mul(2);
                }
            }
        }
    }
    Err(last_error.expect("deadlock retry loop exhausted without an error"))
}

#[cfg(feature = "postgres-sync")]
use crate::id::{AgentBusinessIdGenerator, AgentIdGenerator};
#[cfg(all(feature = "sqlite-sync", not(feature = "postgres-sync")))]
use crate::id::{AgentBusinessIdGenerator, AgentIdGenerator};
mod sql;
#[cfg(feature = "sqlite-sync")]
pub mod sqlite_sql;

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
    SQL_COUNT_AGENT_INTERACTIONS, SQL_COUNT_AGENT_MESSAGES, SQL_COUNT_AGENT_MESSAGE_FEEDBACK,
    SQL_COUNT_AGENT_PROJECTS, SQL_COUNT_AGENT_PROJECT_COMPOSITION_SLOTS,
    SQL_COUNT_AGENT_RESOURCE_USER_STATES, SQL_COUNT_AGENT_SESSIONS, SQL_COUNT_AGENT_TASKS,
    SQL_INSERT_AGENT_CHAT_TURN, SQL_INSERT_AGENT_INTERACTION, SQL_INSERT_AGENT_MESSAGE,
    SQL_INSERT_AGENT_MESSAGE_DRIVE_REF, SQL_INSERT_AGENT_PROJECT,
    SQL_INSERT_AGENT_PROJECT_COMPOSITION_SLOT, SQL_INSERT_AGENT_SESSION, SQL_INSERT_AGENT_TASK,
    SQL_LIST_AGENT_INTERACTIONS, SQL_LIST_AGENT_MESSAGES, SQL_LIST_AGENT_MESSAGES_RECENT_CONTEXT,
    SQL_LIST_AGENT_MESSAGE_DRIVE_REFS, SQL_LIST_AGENT_MESSAGE_DRIVE_REFS_BATCH,
    SQL_LIST_AGENT_MESSAGE_FEEDBACK, SQL_LIST_AGENT_PROJECTS,
    SQL_LIST_AGENT_PROJECT_COMPOSITION_SLOTS, SQL_LIST_AGENT_RESOURCE_USER_STATES,
    SQL_LIST_AGENT_SESSIONS, SQL_LIST_AGENT_TASKS, SQL_LIST_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID,
    SQL_LIST_RECONCILABLE_AGENT_CHAT_TURNS, SQL_LOCK_AGENT_SESSION_FOR_UPDATE,
    SQL_NEXT_MESSAGE_SEQUENCE, SQL_SELECT_AGENT_CHAT_TURN,
    SQL_SELECT_AGENT_CHAT_TURN_BY_IDEMPOTENCY, SQL_SELECT_AGENT_INTERACTION,
    SQL_SELECT_AGENT_MESSAGE, SQL_SELECT_AGENT_MESSAGE_FEEDBACK, SQL_SELECT_AGENT_PROJECT,
    SQL_SELECT_AGENT_PROJECT_COMPOSITION_SLOT, SQL_SELECT_AGENT_RESOURCE_USER_STATE,
    SQL_SELECT_AGENT_SESSION, SQL_SELECT_AGENT_TASK, SQL_UPDATE_AGENT_CHAT_TURN_STATE,
    SQL_UPDATE_AGENT_INTERACTION, SQL_UPDATE_AGENT_MESSAGE, SQL_UPDATE_AGENT_PROJECT,
    SQL_UPDATE_AGENT_PROJECT_COMPOSITION_SLOT, SQL_UPDATE_AGENT_SESSION, SQL_UPDATE_AGENT_TASK,
    SQL_UPSERT_AGENT_MESSAGE_FEEDBACK, SQL_UPSERT_AGENT_RESOURCE_USER_STATE,
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
pub struct AgentProjectRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub project_id: String,
    pub owner_user_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub visibility: i16,
    pub status: i16,
    pub drive_access_mode: i16,
    pub default_agent_id: Option<String>,
    pub default_model_id: Option<String>,
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
            owner_user_id: record.owner_user_id,
            name: record.name.clone(),
            description: record.description.clone(),
            visibility: record.visibility.as_db_code(),
            status: record.status.as_db_code(),
            drive_access_mode: record.drive_access_mode.as_db_code(),
            default_agent_id: record.default_agent_id.clone(),
            default_model_id: record.default_model_id.clone(),
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
    pub title: Option<String>,
    pub status: i16,
    pub provider_binding_id: Option<String>,
    pub model_id: Option<String>,
    pub message_count: u64,
    pub last_message_sequence: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub metadata_json: String,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub last_message_at: Option<String>,
    pub closed_at: Option<String>,
    pub archived_at: Option<String>,
    pub deleted_at: Option<String>,
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
            title: record.title.clone(),
            status: record.status.as_db_code(),
            provider_binding_id: record.provider_binding_id.clone(),
            model_id: record.model_id.clone(),
            message_count: record.message_count,
            last_message_sequence: record.last_message_sequence,
            total_input_tokens: record.total_input_tokens,
            total_output_tokens: record.total_output_tokens,
            metadata_json: record.metadata_json.clone(),
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            last_message_at: record.last_message_at.clone(),
            closed_at: record.closed_at.clone(),
            archived_at: record.archived_at.clone(),
            deleted_at: record.deleted_at.clone(),
        })
    }

    pub fn into_record(self) -> KernelResult<AgentSessionRecord> {
        let status = AgentSessionStatus::from_db_code(self.status).ok_or_else(|| {
            KernelError::validation(format!("invalid session status db code: {}", self.status))
        })?;
        Ok(AgentSessionRecord {
            id: self.id,
            session_id: self.session_id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            agent_id: self.agent_id,
            owner_user_id: self.owner_user_id,
            project_id: self.project_id,
            title: self.title,
            status,
            provider_binding_id: self.provider_binding_id,
            model_id: self.model_id,
            message_count: self.message_count,
            last_message_sequence: self.last_message_sequence,
            total_input_tokens: self.total_input_tokens,
            total_output_tokens: self.total_output_tokens,
            metadata_json: self.metadata_json,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            last_message_at: self.last_message_at,
            closed_at: self.closed_at,
            archived_at: self.archived_at,
            deleted_at: self.deleted_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub last_read_message_sequence: Option<u64>,
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
            last_read_message_sequence: record.last_read_message_sequence,
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
            last_read_message_sequence: self.last_read_message_sequence,
            custom_title: self.custom_title,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentMessageFeedbackRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub message_id: String,
    pub user_id: u64,
    pub rating: i16,
    pub reason_code: Option<String>,
    pub comment: Option<String>,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

impl AgentMessageFeedbackRow {
    pub fn from_record(record: &AgentMessageFeedbackRecord) -> Self {
        Self {
            id: record.id,
            uuid: format!("agents-message-feedback-{}", record.id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            message_id: record.message_id.clone(),
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

    pub fn into_record(self) -> KernelResult<AgentMessageFeedbackRecord> {
        Ok(AgentMessageFeedbackRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            message_id: self.message_id,
            user_id: self.user_id,
            rating: AgentMessageFeedbackRating::from_db_code(self.rating)
                .ok_or_else(|| KernelError::validation("invalid message feedback rating"))?,
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
pub struct AgentMessageDriveRefRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub message_id: String,
    pub media_role: String,
    pub drive_space_id: String,
    pub drive_node_id: String,
    pub drive_uri: String,
    pub media_resource_id: Option<String>,
    pub object_blob_id: Option<String>,
    pub resource_snapshot_json: String,
    pub resource_hash: String,
    pub alt_text: Option<String>,
    pub sort_order: u32,
    pub status: i16,
    pub created_by: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub retention_until: Option<String>,
}

impl AgentMessageDriveRefRow {
    pub fn from_record(record: &AgentMessageDriveRefRecord) -> Self {
        Self {
            id: record.id,
            uuid: format!("agents-message-drive-ref-{}", record.id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            message_id: record.message_id.clone(),
            media_role: record.media_role.as_str().to_string(),
            drive_space_id: record.drive_space_id.clone(),
            drive_node_id: record.drive_node_id.clone(),
            drive_uri: record.drive_uri.clone(),
            media_resource_id: record.media_resource_id.clone(),
            object_blob_id: record.object_blob_id.clone(),
            resource_snapshot_json: record.resource_snapshot_json.clone(),
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

    pub fn into_record(self) -> KernelResult<AgentMessageDriveRefRecord> {
        let media_role = match self.media_role.as_str() {
            "attachment" => AgentMessageMediaRole::Attachment,
            "image" => AgentMessageMediaRole::Image,
            "voice" => AgentMessageMediaRole::Voice,
            "generated_output" => AgentMessageMediaRole::GeneratedOutput,
            "artifact" => AgentMessageMediaRole::Artifact,
            _ => return Err(KernelError::validation("invalid message media role")),
        };
        Ok(AgentMessageDriveRefRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            message_id: self.message_id,
            media_role,
            drive_space_id: self.drive_space_id,
            drive_node_id: self.drive_node_id,
            drive_uri: self.drive_uri,
            media_resource_id: self.media_resource_id,
            object_blob_id: self.object_blob_id,
            resource_snapshot_json: self.resource_snapshot_json,
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
// AgentMessageRow — persistence row for ai_agent_message
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentMessageRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub session_id: String,
    pub agent_id: String,
    pub message_id: String,
    pub role: i16,
    pub content: String,
    pub content_type: String,
    pub status: i16,
    pub sequence: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub artifacts_json: String,
    pub metadata_json: String,
    pub parent_message_id: Option<String>,
    pub turn_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl AgentMessageRow {
    pub fn from_record(record: &AgentMessageRecord) -> KernelResult<Self> {
        Ok(Self {
            id: record.id,
            uuid: build_message_uuid(record.tenant_id, &record.session_id, &record.message_id),
            tenant_id: record.tenant_id,
            session_id: record.session_id.clone(),
            agent_id: record.agent_id.clone(),
            message_id: record.message_id.clone(),
            role: record.role.as_db_code(),
            content: record.content.clone(),
            content_type: record.content_type.clone(),
            status: record.status.as_db_code(),
            sequence: record.sequence,
            input_tokens: record.input_tokens,
            output_tokens: record.output_tokens,
            model_id: record.model_id.clone(),
            provider_id: record.provider_id.clone(),
            artifacts_json: record.artifacts_json.clone(),
            metadata_json: record.metadata_json.clone(),
            parent_message_id: record.parent_message_id.clone(),
            turn_id: record.turn_id.clone(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        })
    }

    pub fn into_record(self) -> KernelResult<AgentMessageRecord> {
        let role = AgentMessageRole::from_db_code(self.role).ok_or_else(|| {
            KernelError::validation(format!("invalid message role db code: {}", self.role))
        })?;
        let status = AgentMessageStatus::from_db_code(self.status).ok_or_else(|| {
            KernelError::validation(format!("invalid message status db code: {}", self.status))
        })?;
        Ok(AgentMessageRecord {
            id: self.id,
            message_id: self.message_id,
            tenant_id: self.tenant_id,
            session_id: self.session_id,
            agent_id: self.agent_id,
            role,
            content: self.content,
            content_type: self.content_type,
            status,
            sequence: self.sequence,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            model_id: self.model_id,
            provider_id: self.provider_id,
            artifacts_json: self.artifacts_json,
            metadata_json: self.metadata_json,
            parent_message_id: self.parent_message_id,
            turn_id: self.turn_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

// ============================================================================
// AgentInteractionRow — persistence row for ai_agent_interaction
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentChatTurnRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub turn_id: String,
    pub session_id: String,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub client_request_id: Option<String>,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub request_message_id: String,
    pub response_message_id: Option<String>,
    pub status: i16,
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

impl AgentChatTurnRow {
    pub fn from_record(record: &AgentChatTurnRecord) -> Self {
        Self {
            id: record.id,
            uuid: build_chat_turn_uuid(record.tenant_id, record.organization_id, &record.turn_id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            turn_id: record.turn_id.clone(),
            session_id: record.session_id.clone(),
            agent_id: record.agent_id.clone(),
            owner_user_id: record.owner_user_id,
            client_request_id: record.client_request_id.clone(),
            idempotency_key: record.idempotency_key.clone(),
            payload_hash: record.payload_hash.clone(),
            request_message_id: record.request_message_id.clone(),
            response_message_id: record.response_message_id.clone(),
            status: record.status.as_db_code(),
            requested_model_id: record.requested_model_id.clone(),
            provider_binding_id: record.provider_binding_id.clone(),
            model_id: record.model_id.clone(),
            provider_id: record.provider_id.clone(),
            input_tokens: record.input_tokens,
            output_tokens: record.output_tokens,
            finish_reason: record.finish_reason.clone(),
            error_code: record.error_code.clone(),
            error_detail: record.error_detail.clone(),
            trace_id: record.trace_id.clone(),
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

    pub fn into_record(self) -> KernelResult<AgentChatTurnRecord> {
        Ok(AgentChatTurnRecord {
            id: self.id,
            turn_id: self.turn_id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            session_id: self.session_id,
            agent_id: self.agent_id,
            owner_user_id: self.owner_user_id,
            client_request_id: self.client_request_id,
            idempotency_key: self.idempotency_key,
            payload_hash: self.payload_hash,
            request_message_id: self.request_message_id,
            response_message_id: self.response_message_id,
            status: AgentChatTurnStatus::from_db_code(self.status)
                .ok_or_else(|| KernelError::validation("invalid chat turn status"))?,
            requested_model_id: self.requested_model_id,
            provider_binding_id: self.provider_binding_id,
            model_id: self.model_id,
            provider_id: self.provider_id,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            finish_reason: self.finish_reason,
            error_code: self.error_code,
            error_detail: self.error_detail,
            trace_id: self.trace_id,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentInteractionRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub agent_id: String,
    pub engine_key: String,
    pub interaction_id: String,
    pub kind: i16,
    pub status: i16,
    pub prompt: String,
    pub options_json: String,
    pub resolution_json: String,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub resolved_at: Option<String>,
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
            agent_id: record.agent_id.clone(),
            engine_key: record.engine_key.clone(),
            interaction_id: record.interaction_id.clone(),
            kind: record.kind.as_db_code(),
            status: record.status.as_db_code(),
            prompt: record.prompt.clone(),
            options_json: record.options_json.clone(),
            resolution_json: record.resolution_json.clone(),
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            resolved_at: record.resolved_at.clone(),
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
            agent_id: self.agent_id,
            engine_key: self.engine_key,
            kind,
            status,
            prompt: self.prompt,
            options_json: self.options_json,
            resolution_json: self.resolution_json,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            resolved_at: self.resolved_at,
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

fn build_message_uuid(tenant_id: u64, session_id: &str, message_id: &str) -> String {
    build_storage_uuid("message", tenant_id, &[session_id, message_id])
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

fn build_chat_turn_uuid(tenant_id: u64, organization_id: u64, turn_id: &str) -> String {
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
    pub subject_id: String,
    pub subject_tenant_id: String,
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
    /// `agent_internal_id`, `subject_id`, `subject_tenant_id`.
    ///
    /// Missing context values fall back to safe defaults (`0` for numeric
    /// fields, `"unknown"` for string fields) so that audit recording never
    /// fails due to incomplete metadata.
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
        let agent_internal_id = extract_event_context(event.payload.as_str(), "agent_internal_id")
            .and_then(|value| value.parse::<u64>().ok());
        let agent_id = extract_event_context(event.payload.as_str(), "agent_id");
        let aggregate_type = extract_event_context(event.payload.as_str(), "aggregate_type")
            .unwrap_or_else(|| "agent".to_string());
        let aggregate_id = extract_event_context(event.payload.as_str(), "aggregate_id")
            .or_else(|| agent_id.clone())
            .ok_or_else(|| KernelError::validation("audit aggregate_id context is required"))?;
        if aggregate_type == "agent" && agent_id.is_none() {
            return Err(KernelError::validation(
                "agent audit context requires agent_id",
            ));
        }
        let subject_id = extract_event_context(event.payload.as_str(), "subject_id")
            .or_else(|| event.correlation_id.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let subject_tenant_id = extract_event_context(event.payload.as_str(), "subject_tenant_id")
            .unwrap_or_else(|| "unknown".to_string());

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
            subject_id,
            subject_tenant_id,
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
            subject_id: row.try_get("subject_id").map_err(map_sqlx_error)?,
            subject_tenant_id: row.try_get("subject_tenant_id").map_err(map_sqlx_error)?,
            request_id: row.try_get("request_id").map_err(map_sqlx_error)?,
            trace_id: row.try_get("trace_id").map_err(map_sqlx_error)?,
            payload_json: row.try_get("payload_json").map_err(map_sqlx_error)?,
            created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        })
    }
}

/// Thread-safe PostgreSQL adapter trait.
///
/// All methods use `&self` — implementations MUST use interior mutability
/// (e.g. an `Arc<Mutex<...>>` wrapped pool or a connection pool that
/// internally manages transactional state). This aligns with the stateless
/// `AgentRepository` trait and eliminates the global Mutex bottleneck.
pub trait AgentRepositoryAdapter: Send + Sync {
    fn next_id(&self) -> KernelResult<u64>;
    fn insert_row(&self, row: AgentBusinessRow) -> KernelResult<()>;
    fn update_row(&self, row: AgentBusinessRow) -> KernelResult<()>;
    fn get_row(&self, tenant_id: u64, agent_id: &str) -> KernelResult<Option<AgentBusinessRow>>;
    fn list_rows(&self, query: &AgentListQuery) -> KernelResult<Vec<AgentBusinessRow>>;
    fn count_rows(&self, query: &AgentListQuery) -> KernelResult<u64>;
    fn insert_project_row(&self, row: AgentProjectRow) -> KernelResult<()>;
    fn update_project_row(&self, row: AgentProjectRow) -> KernelResult<()>;
    fn get_project_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
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
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRow>>;
    fn list_session_rows(&self, query: &SessionListQuery) -> KernelResult<Vec<AgentSessionRow>>;
    fn count_session_rows(&self, query: &SessionListQuery) -> KernelResult<u64>;
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

    // Message operations
    fn insert_message_row(&self, row: AgentMessageRow) -> KernelResult<()>;
    fn update_message_row(&self, row: AgentMessageRow) -> KernelResult<()>;
    fn get_message_row(
        &self,
        tenant_id: u64,
        session_id: &str,
        message_id: &str,
    ) -> KernelResult<Option<AgentMessageRow>>;
    fn list_message_rows(&self, query: &MessageListQuery) -> KernelResult<Vec<AgentMessageRow>>;
    fn count_message_rows(&self, query: &MessageListQuery) -> KernelResult<u64>;
    fn upsert_message_feedback_row(
        &self,
        row: AgentMessageFeedbackRow,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentMessageFeedbackRow>;
    fn get_message_feedback_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        message_id: &str,
        user_id: u64,
        include_deleted: bool,
    ) -> KernelResult<Option<AgentMessageFeedbackRow>>;
    fn list_message_feedback_rows(
        &self,
        query: &MessageFeedbackListQuery,
    ) -> KernelResult<Vec<AgentMessageFeedbackRow>>;
    fn count_message_feedback_rows(&self, query: &MessageFeedbackListQuery) -> KernelResult<u64>;
    fn next_message_sequence(&self, tenant_id: u64, session_id: &str) -> KernelResult<u64>;
    fn get_chat_turn_row_by_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentChatTurnRow>>;
    fn get_chat_turn_row(
        &self,
        _tenant_id: u64,
        _organization_id: u64,
        _turn_id: &str,
    ) -> KernelResult<Option<AgentChatTurnRow>> {
        Err(KernelError::Internal {
            message: "get_chat_turn_row requires an adapter override".to_string(),
        })
    }
    fn list_reconcilable_chat_turn_rows(
        &self,
        _stale_before: &str,
        _limit: usize,
    ) -> KernelResult<Vec<AgentChatTurnRow>> {
        Err(KernelError::Internal {
            message: "list_reconcilable_chat_turn_rows requires an adapter override".to_string(),
        })
    }
    fn insert_chat_turn_reservation_row(&self, _turn: AgentChatTurnRow) -> KernelResult<()> {
        Err(KernelError::Internal {
            message: "insert_chat_turn_reservation_row requires an adapter override".to_string(),
        })
    }
    fn update_chat_turn_state_row(
        &self,
        _turn: AgentChatTurnRow,
        _expected_version: u64,
    ) -> KernelResult<AgentChatTurnRow> {
        Err(KernelError::Internal {
            message: "update_chat_turn_state_row requires an adapter override".to_string(),
        })
    }
    fn insert_chat_turn_rows(
        &self,
        turn: AgentChatTurnRow,
        session: AgentSessionRow,
        user: AgentMessageRow,
        assistant: AgentMessageRow,
    ) -> KernelResult<(AgentSessionRow, AgentMessageRow, AgentMessageRow)>;
    fn insert_chat_turn_with_drive_ref_rows(
        &self,
        _turn: AgentChatTurnRow,
        _session: AgentSessionRow,
        _user: AgentMessageRow,
        _assistant: AgentMessageRow,
        _drive_refs: Vec<AgentMessageDriveRefRow>,
    ) -> KernelResult<(AgentSessionRow, AgentMessageRow, AgentMessageRow)> {
        Err(KernelError::Internal {
            message:
                "insert_chat_turn_with_drive_ref_rows requires a transactional adapter override"
                    .to_string(),
        })
    }
    fn list_message_drive_ref_rows(
        &self,
        _tenant_id: u64,
        _organization_id: u64,
        _message_id: &str,
    ) -> KernelResult<Vec<AgentMessageDriveRefRow>> {
        Err(KernelError::Internal {
            message: "list_message_drive_ref_rows requires an adapter override".to_string(),
        })
    }
    fn list_message_drive_ref_rows_batch(
        &self,
        tenant_id: u64,
        organization_id: u64,
        message_ids: &[String],
    ) -> KernelResult<Vec<AgentMessageDriveRefRow>> {
        let mut rows = Vec::new();
        for message_id in message_ids {
            rows.extend(self.list_message_drive_ref_rows(
                tenant_id,
                organization_id,
                message_id,
            )?);
        }
        Ok(rows)
    }

    // Interaction operations
    fn insert_interaction_row(&self, row: AgentInteractionRow) -> KernelResult<()>;
    fn update_interaction_row(&self, row: AgentInteractionRow) -> KernelResult<()>;
    fn get_interaction_row(
        &self,
        tenant_id: u64,
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
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRecord>> {
        self.adapter
            .get_session_row(tenant_id, session_id)?
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
    // Message persistence
    // -----------------------------------------------------------------------

    fn insert_message(&self, record: AgentMessageRecord) -> KernelResult<()> {
        self.adapter
            .insert_message_row(AgentMessageRow::from_record(&record)?)
    }

    fn update_message(&self, record: AgentMessageRecord) -> KernelResult<()> {
        self.adapter
            .update_message_row(AgentMessageRow::from_record(&record)?)
    }

    fn get_message(
        &self,
        tenant_id: u64,
        session_id: &str,
        message_id: &str,
    ) -> KernelResult<Option<AgentMessageRecord>> {
        self.adapter
            .get_message_row(tenant_id, session_id, message_id)?
            .map(AgentMessageRow::into_record)
            .transpose()
    }

    fn list_messages(&self, query: &MessageListQuery) -> KernelResult<Vec<AgentMessageRecord>> {
        self.adapter
            .list_message_rows(query)?
            .into_iter()
            .map(AgentMessageRow::into_record)
            .collect()
    }

    fn count_messages(&self, query: &MessageListQuery) -> KernelResult<u64> {
        self.adapter.count_message_rows(query)
    }

    fn upsert_message_feedback(
        &self,
        record: AgentMessageFeedbackRecord,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentMessageFeedbackRecord> {
        self.adapter
            .upsert_message_feedback_row(
                AgentMessageFeedbackRow::from_record(&record),
                expected_version,
            )?
            .into_record()
    }

    fn get_message_feedback(
        &self,
        tenant_id: u64,
        organization_id: u64,
        message_id: &str,
        user_id: u64,
        include_deleted: bool,
    ) -> KernelResult<Option<AgentMessageFeedbackRecord>> {
        self.adapter
            .get_message_feedback_row(
                tenant_id,
                organization_id,
                message_id,
                user_id,
                include_deleted,
            )?
            .map(AgentMessageFeedbackRow::into_record)
            .transpose()
    }

    fn list_message_feedback(
        &self,
        query: &MessageFeedbackListQuery,
    ) -> KernelResult<Vec<AgentMessageFeedbackRecord>> {
        self.adapter
            .list_message_feedback_rows(query)?
            .into_iter()
            .map(AgentMessageFeedbackRow::into_record)
            .collect()
    }

    fn count_message_feedback(&self, query: &MessageFeedbackListQuery) -> KernelResult<u64> {
        self.adapter.count_message_feedback_rows(query)
    }

    fn next_message_sequence(&self, tenant_id: u64, session_id: &str) -> KernelResult<u64> {
        self.adapter.next_message_sequence(tenant_id, session_id)
    }

    fn get_chat_turn_by_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentChatTurnRecord>> {
        self.adapter
            .get_chat_turn_row_by_idempotency(
                tenant_id,
                organization_id,
                owner_user_id,
                idempotency_key,
            )?
            .map(AgentChatTurnRow::into_record)
            .transpose()
    }

    fn get_chat_turn(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<Option<AgentChatTurnRecord>> {
        self.adapter
            .get_chat_turn_row(tenant_id, organization_id, turn_id)?
            .map(AgentChatTurnRow::into_record)
            .transpose()
    }

    fn list_reconcilable_chat_turns(
        &self,
        stale_before: &str,
        limit: usize,
    ) -> KernelResult<Vec<AgentChatTurnRecord>> {
        self.adapter
            .list_reconcilable_chat_turn_rows(stale_before, limit)?
            .into_iter()
            .map(AgentChatTurnRow::into_record)
            .collect()
    }

    fn insert_chat_turn_reservation(&self, turn: AgentChatTurnRecord) -> KernelResult<()> {
        self.adapter
            .insert_chat_turn_reservation_row(AgentChatTurnRow::from_record(&turn))
    }

    fn update_chat_turn_state(
        &self,
        turn: AgentChatTurnRecord,
        expected_version: u64,
    ) -> KernelResult<AgentChatTurnRecord> {
        self.adapter
            .update_chat_turn_state_row(AgentChatTurnRow::from_record(&turn), expected_version)?
            .into_record()
    }

    fn insert_chat_turn(
        &self,
        turn: AgentChatTurnRecord,
        session: AgentSessionRecord,
        user_message: AgentMessageRecord,
        assistant_message: AgentMessageRecord,
    ) -> KernelResult<(AgentSessionRecord, AgentMessageRecord, AgentMessageRecord)> {
        let turn_row = AgentChatTurnRow::from_record(&turn);
        let session_row = AgentSessionRow::from_record(&session)?;
        let user_row = AgentMessageRow::from_record(&user_message)?;
        let assistant_row = AgentMessageRow::from_record(&assistant_message)?;
        let (session_row, user_row, assistant_row) =
            self.adapter
                .insert_chat_turn_rows(turn_row, session_row, user_row, assistant_row)?;
        Ok((
            session_row.into_record()?,
            user_row.into_record()?,
            assistant_row.into_record()?,
        ))
    }

    fn insert_chat_turn_with_drive_refs(
        &self,
        turn: AgentChatTurnRecord,
        session: AgentSessionRecord,
        user_message: AgentMessageRecord,
        assistant_message: AgentMessageRecord,
        drive_refs: Vec<AgentMessageDriveRefRecord>,
    ) -> KernelResult<(AgentSessionRecord, AgentMessageRecord, AgentMessageRecord)> {
        let turn_row = AgentChatTurnRow::from_record(&turn);
        let session_row = AgentSessionRow::from_record(&session)?;
        let user_row = AgentMessageRow::from_record(&user_message)?;
        let assistant_row = AgentMessageRow::from_record(&assistant_message)?;
        let drive_ref_rows = drive_refs
            .iter()
            .map(AgentMessageDriveRefRow::from_record)
            .collect();
        let (session_row, user_row, assistant_row) =
            self.adapter.insert_chat_turn_with_drive_ref_rows(
                turn_row,
                session_row,
                user_row,
                assistant_row,
                drive_ref_rows,
            )?;
        Ok((
            session_row.into_record()?,
            user_row.into_record()?,
            assistant_row.into_record()?,
        ))
    }

    fn list_message_drive_refs(
        &self,
        tenant_id: u64,
        organization_id: u64,
        message_id: &str,
    ) -> KernelResult<Vec<AgentMessageDriveRefRecord>> {
        self.adapter
            .list_message_drive_ref_rows(tenant_id, organization_id, message_id)?
            .into_iter()
            .map(AgentMessageDriveRefRow::into_record)
            .collect()
    }

    fn list_message_drive_refs_batch(
        &self,
        tenant_id: u64,
        organization_id: u64,
        message_ids: &[String],
    ) -> KernelResult<Vec<AgentMessageDriveRefRecord>> {
        self.adapter
            .list_message_drive_ref_rows_batch(tenant_id, organization_id, message_ids)?
            .into_iter()
            .map(AgentMessageDriveRefRow::into_record)
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
        session_id: &str,
        interaction_id: &str,
    ) -> KernelResult<Option<AgentInteractionRecord>> {
        self.adapter
            .get_interaction_row(tenant_id, session_id, interaction_id)?
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

#[cfg(feature = "sqlite-sync")]
pub struct SyncSqliteAdapter {
    pool: BlockingSqlitePool,
    id_generator: AgentBusinessIdGenerator,
}

#[cfg(feature = "sqlite-sync")]
impl SyncSqliteAdapter {
    pub fn connect(connection_uri: &str) -> KernelResult<Self> {
        Ok(Self {
            pool: BlockingSqlitePool::connect(connection_uri)?,
            id_generator: AgentBusinessIdGenerator::new_default()?,
        })
    }

    pub fn connect_from_sdkwork_env(service_name: &str) -> KernelResult<Self> {
        Ok(Self {
            pool: BlockingSqlitePool::connect_from_sdkwork_env(service_name)?,
            id_generator: AgentBusinessIdGenerator::new_default()?,
        })
    }

    pub fn connect_from_agents_managed_store_env() -> KernelResult<Self> {
        Self::connect_from_sdkwork_env(SQLITE_MANAGED_STORE_DATABASE_SERVICE)
    }

    pub fn from_pool(pool: BlockingSqlitePool) -> KernelResult<Self> {
        Ok(Self {
            pool,
            id_generator: AgentBusinessIdGenerator::new_default()?,
        })
    }

    pub fn with_pool_and_id_generator(
        pool: BlockingSqlitePool,
        id_generator: AgentBusinessIdGenerator,
    ) -> Self {
        Self { pool, id_generator }
    }

    pub fn pool(&self) -> &BlockingSqlitePool {
        &self.pool
    }

    pub fn next_id(&self) -> KernelResult<u64> {
        self.id_generator.next_id()
    }

    pub fn insert_agent(&self, record: AgentBusinessRecord) -> KernelResult<()> {
        let row = AgentBusinessRow::from_record(&record)?;
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(row.owner_user_id, "owner_user_id")?;
        let version = u64_to_i64(row.version, "version")?;
        let sqlite = self.pool.pool().clone();
        self.pool.run_kernel(async move {
            sqlx::query(sqlite_sql::INSERT_AGENT)
                .bind(id)
                .bind(row.uuid)
                .bind(tenant_id)
                .bind(organization_id)
                .bind(owner_user_id)
                .bind(row.agent_id)
                .bind(row.code)
                .bind(row.display_name)
                .bind(row.description)
                .bind(row.manifest_json)
                .bind(row.default_code_task_intent_json)
                .bind(row.implementation_provider_id)
                .bind(row.implementation_kind)
                .bind(row.implementation_type)
                .bind(row.status)
                .bind(row.visibility)
                .bind(row.tags_json)
                .bind(row.created_at)
                .bind(row.updated_at)
                .bind(row.deleted_at)
                .bind(version)
                .execute(&sqlite)
                .await
                .map(|_| ())
        })
    }

    pub fn get_agent(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Option<AgentBusinessRecord>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let sqlite = self.pool.pool().clone();
        let agent_id = agent_id.to_owned();
        let row = self.pool.run_kernel(async move {
            sqlx::query(sqlite_sql::SELECT_AGENT)
                .bind(tenant_id)
                .bind(agent_id)
                .fetch_optional(&sqlite)
                .await
        })?;
        row.map(sqlite_row_to_agent_business_row)
            .transpose()?
            .map(AgentBusinessRow::into_record)
            .transpose()
    }

    pub fn update_agent(&self, record: AgentBusinessRecord) -> KernelResult<()> {
        let row = AgentBusinessRow::from_record(&record)?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(row.owner_user_id, "owner_user_id")?;
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let lookup_agent_id = row.agent_id.clone();
        let sqlite = self.pool.pool().clone();
        let affected = self.pool.run_kernel(async move {
            sqlx::query(sqlite_sql::UPDATE_AGENT)
                .bind(organization_id)
                .bind(owner_user_id)
                .bind(row.code)
                .bind(row.display_name)
                .bind(row.description)
                .bind(row.manifest_json)
                .bind(row.default_code_task_intent_json)
                .bind(row.implementation_provider_id)
                .bind(row.implementation_kind)
                .bind(row.implementation_type)
                .bind(row.status)
                .bind(row.visibility)
                .bind(row.tags_json)
                .bind(row.updated_at)
                .bind(row.deleted_at)
                .bind(version)
                .bind(tenant_id)
                .bind(row.agent_id)
                .bind(previous_version)
                .execute(&sqlite)
                .await
                .map(|result| result.rows_affected())
        })?;
        if affected > 0 {
            return Ok(());
        }
        if self
            .get_agent(record.tenant_id, &lookup_agent_id)?
            .is_some()
        {
            return Err(KernelError::conflict("agent version mismatch"));
        }
        Err(KernelError::validation("agent not found"))
    }
}

#[cfg(feature = "postgres-sync")]
pub const AGENTS_MANAGED_STORE_DATABASE_SERVICE: &str = "AGENTS_STORE";

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

    /// Connects using `sdkwork-database-config` env resolution for the given service name.
    ///
    /// Honors the legacy `SDKWORK_{SERVICE}_POSTGRES_URI` variable when set, then falls back to
    /// `DatabaseConfig::from_env` (`SDKWORK_{SERVICE}_DATABASE_URL` and unified claw profile keys).
    pub fn connect_from_sdkwork_env(service_name: &str) -> KernelResult<Self> {
        Ok(Self {
            pool: BlockingPostgresPool::connect_from_sdkwork_env(service_name)?,
            id_generator: AgentBusinessIdGenerator::new_default()?,
        })
    }

    /// Connects using platform database config for agents managed store persistence.
    pub fn connect_from_agents_managed_store_env() -> KernelResult<Self> {
        Self::connect_from_sdkwork_env(AGENTS_MANAGED_STORE_DATABASE_SERVICE)
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
impl AgentRepositoryAdapter for SyncPostgresAdapter {
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

        let organization_id: Option<i64> = query.organization_id.map(|v| v as i64);
        let owner_user_id: Option<i64> = query.owner_user_id.map(|v| v as i64);
        let include_deleted = query.include_deleted;
        let search_query: Option<String> = query
            .search_query
            .as_ref()
            .filter(|q| !is_blank(Some(q.as_str())))
            .map(|q| format!("%{}%", trim(q).to_lowercase()));
        let visibility_code = query.visibility.map(|visibility| visibility.as_db_code());
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;

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

        let organization_id: Option<i64> = query.organization_id.map(|v| v as i64);
        let owner_user_id: Option<i64> = query.owner_user_id.map(|v| v as i64);
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
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_PROJECT,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.project_id,
                owner_user_id,
                row.name,
                row.description,
                row.visibility,
                row.status,
                row.drive_access_mode,
                row.default_agent_id,
                row.default_model_id,
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
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_PROJECT,
                row.name,
                row.description,
                row.visibility,
                row.status,
                row.drive_access_mode,
                row.default_agent_id,
                row.default_model_id,
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
                row.project_id,
                previous_version
            )?;
            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_PROJECT,
                    tenant_id,
                    organization_id,
                    row.project_id
                )?
                .is_some();
                if exists {
                    return Err(KernelError::conflict("project version mismatch"));
                }
                return Err(KernelError::validation("project not found"));
            }
            Ok(())
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

    fn list_project_rows(&self, query: &ProjectListQuery) -> KernelResult<Vec<AgentProjectRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let owner_user_id = query
            .owner_user_id
            .map(|value| u64_to_i64(value, "owner_user_id"))
            .transpose()?;
        let status = query.status.map(AgentProjectStatus::as_db_code);
        let search = query
            .search_query
            .as_ref()
            .map(|value| format!("%{}%", trim(value)));
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_AGENT_PROJECTS,
                tenant_id,
                organization_id,
                owner_user_id,
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
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;
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
                retry_on_deadlock(|| async {
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
        let message_count = u64_to_i64(row.message_count, "message_count")?;
        let last_message_sequence = u64_to_i64(row.last_message_sequence, "last_message_sequence")?;
        let total_input_tokens = u64_to_i64(row.total_input_tokens, "total_input_tokens")?;
        let total_output_tokens = u64_to_i64(row.total_output_tokens, "total_output_tokens")?;
        let version = u64_to_i64(row.version, "version")?;

        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_SESSION,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.agent_id,
                owner_user_id,
                row.session_id,
                row.project_id,
                row.title,
                row.status,
                row.provider_binding_id,
                row.model_id,
                message_count,
                last_message_sequence,
                total_input_tokens,
                total_output_tokens,
                row.metadata_json,
                version,
                row.created_at,
                row.updated_at,
                row.last_message_at,
                row.closed_at,
                row.archived_at,
                row.deleted_at
            )?;
            Ok(())
        })
    }

    fn update_session_row(&self, row: AgentSessionRow) -> KernelResult<()> {
        let message_count = u64_to_i64(row.message_count, "message_count")?;
        let last_message_sequence = u64_to_i64(row.last_message_sequence, "last_message_sequence")?;
        let total_input_tokens = u64_to_i64(row.total_input_tokens, "total_input_tokens")?;
        let total_output_tokens = u64_to_i64(row.total_output_tokens, "total_output_tokens")?;
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
                row.status,
                row.provider_binding_id,
                row.model_id,
                message_count,
                last_message_sequence,
                total_input_tokens,
                total_output_tokens,
                row.metadata_json,
                version,
                row.updated_at,
                row.last_message_at,
                row.closed_at,
                row.archived_at,
                row.deleted_at,
                tenant_id,
                organization_id,
                row.session_id,
                previous_version
            )?;

            if updated_rows == 0 {
                let exists =
                    pg_query_optional!(pool, SQL_SELECT_AGENT_SESSION, tenant_id, row.session_id)?
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
        session_id: &str,
    ) -> KernelResult<Option<AgentSessionRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(pool, SQL_SELECT_AGENT_SESSION, tenant_id, session_id)?;
            row.map(pg_row_to_agent_session_row).transpose()
        })
    }

    fn list_session_rows(&self, query: &SessionListQuery) -> KernelResult<Vec<AgentSessionRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id: Option<i64> = query.organization_id.map(|v| v as i64);
        let agent_id: Option<&str> = query.agent_id.as_deref();
        let project_id: Option<&str> = query.project_id.as_deref();
        let owner_user_id: Option<i64> = query.owner_user_id.map(|v| v as i64);
        let status_code: Option<i16> = query
            .status
            .as_deref()
            .and_then(AgentSessionStatus::from_code)
            .map(|s| s.as_db_code());
        let include_archived = query.include_archived;
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;

        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT_SESSIONS,
                tenant_id,
                organization_id,
                agent_id,
                project_id,
                owner_user_id,
                status_code,
                include_archived,
                page_size,
                offset
            )?;
            rows.into_iter().map(pg_row_to_agent_session_row).collect()
        })
    }

    fn count_session_rows(&self, query: &SessionListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id: Option<i64> = query.organization_id.map(|v| v as i64);
        let agent_id: Option<&str> = query.agent_id.as_deref();
        let project_id: Option<&str> = query.project_id.as_deref();
        let owner_user_id: Option<i64> = query.owner_user_id.map(|v| v as i64);
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
        let last_read_message_sequence = row
            .last_read_message_sequence
            .map(|value| u64_to_i64(value, "last_read_message_sequence"))
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
                last_read_message_sequence,
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
                query.include_hidden
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
    // Message persistence
    // -----------------------------------------------------------------------

    fn insert_message_row(&self, row: AgentMessageRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let sequence = u64_to_i64(row.sequence, "sequence")?;
        let input_tokens = u64_to_i64(row.input_tokens, "input_tokens")?;
        let output_tokens = u64_to_i64(row.output_tokens, "output_tokens")?;

        self.with_pool(|pool| {
            let inserted_rows = pg_execute!(
                pool,
                SQL_INSERT_AGENT_MESSAGE,
                id,
                row.uuid,
                tenant_id,
                row.session_id,
                row.agent_id,
                row.role,
                row.message_id,
                row.content,
                row.content_type,
                row.status,
                sequence,
                input_tokens,
                output_tokens,
                row.model_id,
                row.provider_id,
                row.artifacts_json,
                row.metadata_json,
                row.parent_message_id,
                row.created_at,
                row.updated_at,
                row.turn_id
            )?;
            if inserted_rows == 0 {
                return Err(KernelError::validation("message parent session not found"));
            }
            Ok(())
        })
    }

    fn update_message_row(&self, row: AgentMessageRow) -> KernelResult<()> {
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;

        self.with_pool(|pool| {
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_MESSAGE,
                row.content,
                row.content_type,
                row.status,
                row.model_id,
                row.provider_id,
                row.artifacts_json,
                row.metadata_json,
                row.updated_at,
                tenant_id,
                row.session_id,
                row.message_id
            )?;

            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_MESSAGE,
                    tenant_id,
                    row.session_id,
                    row.message_id
                )?
                .is_some();
                if exists {
                    return Err(KernelError::conflict("message update conflict"));
                }
                return Err(KernelError::validation("message not found"));
            }
            Ok(())
        })
    }

    fn get_message_row(
        &self,
        tenant_id: u64,
        session_id: &str,
        message_id: &str,
    ) -> KernelResult<Option<AgentMessageRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_MESSAGE,
                tenant_id,
                session_id,
                message_id
            )?;
            row.map(pg_row_to_agent_message_row).transpose()
        })
    }

    fn list_message_rows(&self, query: &MessageListQuery) -> KernelResult<Vec<AgentMessageRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let role_code: Option<i16> = query
            .role
            .as_deref()
            .and_then(AgentMessageRole::from_code)
            .map(|r| r.as_db_code());
        let status_code: Option<i16> = query
            .status
            .as_deref()
            .and_then(AgentMessageStatus::from_code)
            .map(|s| s.as_db_code());
        let page_size = query.pagination.page_size as i64;

        self.with_pool(|pool| {
            let rows = match query.sort {
                MessageListSort::RecentContextDesc => pg_query!(
                    pool,
                    SQL_LIST_AGENT_MESSAGES_RECENT_CONTEXT,
                    tenant_id,
                    query.session_id,
                    role_code,
                    status_code,
                    page_size
                )?,
                MessageListSort::SequenceAsc => {
                    let offset = query.pagination.offset as i64;
                    pg_query!(
                        pool,
                        SQL_LIST_AGENT_MESSAGES,
                        tenant_id,
                        query.session_id,
                        role_code,
                        status_code,
                        page_size,
                        offset
                    )?
                }
            };
            let mut rows: Vec<AgentMessageRow> = rows
                .into_iter()
                .map(pg_row_to_agent_message_row)
                .collect::<KernelResult<Vec<_>>>()?;
            if matches!(query.sort, MessageListSort::RecentContextDesc) {
                rows.reverse();
            }
            Ok(rows)
        })
    }

    fn count_message_rows(&self, query: &MessageListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let role_code: Option<i16> = query
            .role
            .as_deref()
            .and_then(AgentMessageRole::from_code)
            .map(|r| r.as_db_code());
        let status_code: Option<i16> = query
            .status
            .as_deref()
            .and_then(AgentMessageStatus::from_code)
            .map(|s| s.as_db_code());

        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_MESSAGES,
                tenant_id,
                query.session_id,
                role_code,
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

    fn upsert_message_feedback_row(
        &self,
        row: AgentMessageFeedbackRow,
        expected_version: Option<u64>,
    ) -> KernelResult<AgentMessageFeedbackRow> {
        if let Some(expected) = expected_version {
            if row.version != expected.saturating_add(1) {
                return Err(KernelError::conflict("message feedback version mismatch"));
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
                SQL_UPSERT_AGENT_MESSAGE_FEEDBACK,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.message_id,
                user_id,
                row.rating,
                row.reason_code,
                row.comment,
                row.created_at,
                row.updated_at,
                row.deleted_at,
                expected_version
            )?
            .map(pg_row_to_agent_message_feedback_row)
            .transpose()?
            .ok_or_else(|| KernelError::conflict("message feedback version mismatch"))
        })
    }

    fn get_message_feedback_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        message_id: &str,
        user_id: u64,
        include_deleted: bool,
    ) -> KernelResult<Option<AgentMessageFeedbackRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        let user_id = u64_to_i64(user_id, "user_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_MESSAGE_FEEDBACK,
                tenant_id,
                organization_id,
                message_id,
                user_id,
                include_deleted
            )?
            .map(pg_row_to_agent_message_feedback_row)
            .transpose()
        })
    }

    fn list_message_feedback_rows(
        &self,
        query: &MessageFeedbackListQuery,
    ) -> KernelResult<Vec<AgentMessageFeedbackRow>> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let user_id = u64_to_i64(query.user_id, "user_id")?;
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_AGENT_MESSAGE_FEEDBACK,
                tenant_id,
                organization_id,
                user_id,
                query.session_id,
                page_size,
                offset
            )?
            .into_iter()
            .map(pg_row_to_agent_message_feedback_row)
            .collect()
        })
    }

    fn count_message_feedback_rows(&self, query: &MessageFeedbackListQuery) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(query.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(query.organization_id, "organization_id")?;
        let user_id = u64_to_i64(query.user_id, "user_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_MESSAGE_FEEDBACK,
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

    fn next_message_sequence(&self, tenant_id: u64, session_id: &str) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(pool, SQL_NEXT_MESSAGE_SEQUENCE, tenant_id, session_id)?;
            let next: i64 = row
                .map(|r| r.try_get::<i64, _>("next_sequence").map_err(map_sqlx_error))
                .transpose()?
                .unwrap_or(1);
            int64_to_u64(next, "next_sequence")
        })
    }

    fn get_chat_turn_row_by_idempotency(
        &self,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        idempotency_key: &str,
    ) -> KernelResult<Option<AgentChatTurnRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(owner_user_id, "owner_user_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_CHAT_TURN_BY_IDEMPOTENCY,
                tenant_id,
                organization_id,
                owner_user_id,
                idempotency_key
            )?
            .map(pg_row_to_agent_chat_turn_row)
            .transpose()
        })
    }

    fn get_chat_turn_row(
        &self,
        tenant_id: u64,
        organization_id: u64,
        turn_id: &str,
    ) -> KernelResult<Option<AgentChatTurnRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_CHAT_TURN,
                tenant_id,
                organization_id,
                turn_id
            )?
            .map(pg_row_to_agent_chat_turn_row)
            .transpose()
        })
    }

    fn list_reconcilable_chat_turn_rows(
        &self,
        stale_before: &str,
        limit: usize,
    ) -> KernelResult<Vec<AgentChatTurnRow>> {
        let limit = i64::try_from(limit.clamp(1, 200))
            .map_err(|_| KernelError::validation("reconciliation limit overflow"))?;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_RECONCILABLE_AGENT_CHAT_TURNS,
                stale_before,
                limit
            )?
            .into_iter()
            .map(pg_row_to_agent_chat_turn_row)
            .collect()
        })
    }

    fn insert_chat_turn_reservation_row(&self, turn: AgentChatTurnRow) -> KernelResult<()> {
        let id = u64_to_i64(turn.id, "turn.id")?;
        let tenant_id = u64_to_i64(turn.tenant_id, "turn.tenant_id")?;
        let organization_id = u64_to_i64(turn.organization_id, "turn.organization_id")?;
        let owner_user_id = u64_to_i64(turn.owner_user_id, "turn.owner_user_id")?;
        let input_tokens = u64_to_i64(turn.input_tokens, "turn.input_tokens")?;
        let output_tokens = u64_to_i64(turn.output_tokens, "turn.output_tokens")?;
        let version = u64_to_i64(turn.version, "turn.version")?;
        self.with_pool(|pool| {
            let affected = pg_execute!(
                pool,
                SQL_INSERT_AGENT_CHAT_TURN,
                id,
                turn.uuid,
                tenant_id,
                organization_id,
                turn.turn_id,
                turn.session_id,
                turn.agent_id,
                owner_user_id,
                turn.client_request_id,
                turn.idempotency_key,
                turn.payload_hash,
                turn.request_message_id,
                &turn.response_message_id,
                turn.status,
                &turn.requested_model_id,
                &turn.provider_binding_id,
                &turn.model_id,
                &turn.provider_id,
                input_tokens,
                output_tokens,
                turn.finish_reason,
                turn.error_code,
                turn.error_detail,
                turn.trace_id,
                version,
                turn.created_at,
                turn.updated_at,
                turn.started_at,
                turn.completed_at,
                turn.cancel_requested_at,
                turn.cancelled_at,
                turn.retention_until
            )?;
            if affected == 0 {
                return Err(KernelError::conflict("chat turn reservation conflict"));
            }
            Ok(())
        })
    }

    fn update_chat_turn_state_row(
        &self,
        turn: AgentChatTurnRow,
        expected_version: u64,
    ) -> KernelResult<AgentChatTurnRow> {
        let input_tokens = u64_to_i64(turn.input_tokens, "turn.input_tokens")?;
        let output_tokens = u64_to_i64(turn.output_tokens, "turn.output_tokens")?;
        let version = u64_to_i64(turn.version, "turn.version")?;
        let tenant_id = u64_to_i64(turn.tenant_id, "turn.tenant_id")?;
        let organization_id = u64_to_i64(turn.organization_id, "turn.organization_id")?;
        let expected_version = u64_to_i64(expected_version, "turn.expected_version")?;
        self.with_pool(|pool| {
            let affected = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_CHAT_TURN_STATE,
                &turn.response_message_id,
                turn.status,
                &turn.requested_model_id,
                &turn.provider_binding_id,
                &turn.model_id,
                &turn.provider_id,
                input_tokens,
                output_tokens,
                &turn.finish_reason,
                &turn.error_code,
                &turn.error_detail,
                &turn.trace_id,
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
                return Err(KernelError::conflict("chat turn state update conflict"));
            }
            Ok(turn)
        })
    }

    fn insert_chat_turn_rows(
        &self,
        turn: AgentChatTurnRow,
        session: AgentSessionRow,
        user: AgentMessageRow,
        assistant: AgentMessageRow,
    ) -> KernelResult<(AgentSessionRow, AgentMessageRow, AgentMessageRow)> {
        self.insert_chat_turn_with_drive_ref_rows(turn, session, user, assistant, Vec::new())
    }

    fn insert_chat_turn_with_drive_ref_rows(
        &self,
        turn: AgentChatTurnRow,
        session: AgentSessionRow,
        user: AgentMessageRow,
        assistant: AgentMessageRow,
        drive_refs: Vec<AgentMessageDriveRefRow>,
    ) -> KernelResult<(AgentSessionRow, AgentMessageRow, AgentMessageRow)> {
        let tenant_id = u64_to_i64(session.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(session.organization_id, "organization_id")?;
        let session_id = session.session_id.clone();
        let message_count = u64_to_i64(session.message_count, "message_count")?;
        let last_message_sequence =
            u64_to_i64(session.last_message_sequence, "last_message_sequence")?;
        let total_input_tokens = u64_to_i64(session.total_input_tokens, "total_input_tokens")?;
        let total_output_tokens = u64_to_i64(session.total_output_tokens, "total_output_tokens")?;
        let version = u64_to_i64(session.version, "version")?;
        let previous_version = u64_to_i64(
            expected_previous_version(session.version)?,
            "previous_version",
        )?;

        let user_id = u64_to_i64(user.id, "id")?;
        let assistant_id = u64_to_i64(assistant.id, "id")?;
        let turn_internal_id = u64_to_i64(turn.id, "turn_id")?;
        let turn_owner_user_id = u64_to_i64(turn.owner_user_id, "owner_user_id")?;
        let turn_input_tokens = u64_to_i64(turn.input_tokens, "input_tokens")?;
        let turn_output_tokens = u64_to_i64(turn.output_tokens, "output_tokens")?;
        let turn_version = u64_to_i64(turn.version, "version")?;

        fn kernel_err(error: KernelError) -> sqlx::Error {
            sqlx::Error::Protocol(error.to_string())
        }

        self.with_pool(|pool| {
            let pg_pool = pool.pool().clone();
            pool.run_kernel(async move {
                retry_on_deadlock(|| async {
                    let mut user = user.clone();
                    let mut assistant = assistant.clone();
                    let turn = turn.clone();
                    let session = session.clone();
                    let session_id = session_id.clone();
                    let drive_refs = drive_refs.clone();
                    let mut tx = pg_pool.begin().await?;

                    let locked = sqlx::query(SQL_LOCK_AGENT_SESSION_FOR_UPDATE)
                        .bind(tenant_id)
                        .bind(&session_id)
                        .bind(organization_id)
                        .fetch_optional(&mut *tx)
                        .await?;
                    if locked.is_none() {
                        return Err(kernel_err(KernelError::validation("session not found")));
                    }

                    let row = sqlx::query(SQL_NEXT_MESSAGE_SEQUENCE)
                        .bind(tenant_id)
                        .bind(&session_id)
                        .fetch_optional(&mut *tx)
                        .await?;
                    let user_seq: i64 = row
                        .and_then(|r| r.try_get::<i64, _>("next_sequence").ok())
                        .unwrap_or(1);
                    let assistant_seq = user_seq.saturating_add(1);
                    user.sequence = int64_to_u64(user_seq, "sequence").map_err(kernel_err)?;
                    assistant.sequence =
                        int64_to_u64(assistant_seq, "sequence").map_err(kernel_err)?;
                    user.turn_id = Some(turn.turn_id.clone());
                    assistant.turn_id = Some(turn.turn_id.clone());

                    let user_sequence =
                        u64_to_i64(user.sequence, "sequence").map_err(kernel_err)?;
                    let assistant_sequence =
                        u64_to_i64(assistant.sequence, "sequence").map_err(kernel_err)?;
                    let user_input_tokens =
                        u64_to_i64(user.input_tokens, "input_tokens").map_err(kernel_err)?;
                    let user_output_tokens =
                        u64_to_i64(user.output_tokens, "output_tokens").map_err(kernel_err)?;
                    let assistant_input_tokens =
                        u64_to_i64(assistant.input_tokens, "input_tokens").map_err(kernel_err)?;
                    let assistant_output_tokens =
                        u64_to_i64(assistant.output_tokens, "output_tokens").map_err(kernel_err)?;

                    let turn_write = sqlx::query(SQL_INSERT_AGENT_CHAT_TURN)
                        .bind(turn_internal_id)
                        .bind(&turn.uuid)
                        .bind(tenant_id)
                        .bind(organization_id)
                        .bind(&turn.turn_id)
                        .bind(&turn.session_id)
                        .bind(&turn.agent_id)
                        .bind(turn_owner_user_id)
                        .bind(&turn.client_request_id)
                        .bind(&turn.idempotency_key)
                        .bind(&turn.payload_hash)
                        .bind(&turn.request_message_id)
                        .bind(&turn.response_message_id)
                        .bind(turn.status)
                        .bind(&turn.requested_model_id)
                        .bind(&turn.provider_binding_id)
                        .bind(&turn.model_id)
                        .bind(&turn.provider_id)
                        .bind(turn_input_tokens)
                        .bind(turn_output_tokens)
                        .bind(&turn.finish_reason)
                        .bind(&turn.error_code)
                        .bind(&turn.error_detail)
                        .bind(&turn.trace_id)
                        .bind(turn_version)
                        .bind(&turn.created_at)
                        .bind(&turn.updated_at)
                        .bind(&turn.started_at)
                        .bind(&turn.completed_at)
                        .bind(&turn.cancel_requested_at)
                        .bind(&turn.cancelled_at)
                        .bind(&turn.retention_until)
                        .execute(&mut *tx)
                        .await?;
                    if turn_write.rows_affected() == 0 {
                        return Err(kernel_err(KernelError::conflict(
                            "chat turn completion conflict",
                        )));
                    }

                    sqlx::query(SQL_INSERT_AGENT_MESSAGE)
                        .bind(user_id)
                        .bind(&user.uuid)
                        .bind(tenant_id)
                        .bind(&user.session_id)
                        .bind(&user.agent_id)
                        .bind(user.role)
                        .bind(&user.message_id)
                        .bind(&user.content)
                        .bind(&user.content_type)
                        .bind(user.status)
                        .bind(user_sequence)
                        .bind(user_input_tokens)
                        .bind(user_output_tokens)
                        .bind(&user.model_id)
                        .bind(&user.provider_id)
                        .bind(&user.artifacts_json)
                        .bind(&user.metadata_json)
                        .bind(&user.parent_message_id)
                        .bind(&user.created_at)
                        .bind(&user.updated_at)
                        .bind(&user.turn_id)
                        .execute(&mut *tx)
                        .await?;

                    for drive_ref in drive_refs {
                        if drive_ref.tenant_id != session.tenant_id
                            || drive_ref.organization_id != session.organization_id
                            || drive_ref.message_id != user.message_id
                        {
                            return Err(kernel_err(KernelError::validation(
                                "message Drive reference scope mismatch",
                            )));
                        }
                        let drive_ref_id =
                            u64_to_i64(drive_ref.id, "drive_ref.id").map_err(kernel_err)?;
                        let drive_ref_tenant_id =
                            u64_to_i64(drive_ref.tenant_id, "drive_ref.tenant_id")
                                .map_err(kernel_err)?;
                        let drive_ref_organization_id =
                            u64_to_i64(drive_ref.organization_id, "drive_ref.organization_id")
                                .map_err(kernel_err)?;
                        let drive_ref_sort_order =
                            i32::try_from(drive_ref.sort_order).map_err(|_| {
                                kernel_err(KernelError::validation(
                                    "drive_ref.sort_order exceeds integer range",
                                ))
                            })?;
                        let drive_ref_created_by =
                            u64_to_i64(drive_ref.created_by, "drive_ref.created_by")
                                .map_err(kernel_err)?;
                        sqlx::query(SQL_INSERT_AGENT_MESSAGE_DRIVE_REF)
                            .bind(drive_ref_id)
                            .bind(&drive_ref.uuid)
                            .bind(drive_ref_tenant_id)
                            .bind(drive_ref_organization_id)
                            .bind(&drive_ref.message_id)
                            .bind(&drive_ref.media_role)
                            .bind(&drive_ref.drive_space_id)
                            .bind(&drive_ref.drive_node_id)
                            .bind(&drive_ref.drive_uri)
                            .bind(&drive_ref.media_resource_id)
                            .bind(&drive_ref.object_blob_id)
                            .bind(&drive_ref.resource_snapshot_json)
                            .bind(&drive_ref.resource_hash)
                            .bind(&drive_ref.alt_text)
                            .bind(drive_ref_sort_order)
                            .bind(drive_ref.status)
                            .bind(drive_ref_created_by)
                            .bind(&drive_ref.created_at)
                            .bind(&drive_ref.updated_at)
                            .bind(&drive_ref.deleted_at)
                            .bind(&drive_ref.retention_until)
                            .execute(&mut *tx)
                            .await?;
                    }

                    sqlx::query(SQL_INSERT_AGENT_MESSAGE)
                        .bind(assistant_id)
                        .bind(&assistant.uuid)
                        .bind(tenant_id)
                        .bind(&assistant.session_id)
                        .bind(&assistant.agent_id)
                        .bind(assistant.role)
                        .bind(&assistant.message_id)
                        .bind(&assistant.content)
                        .bind(&assistant.content_type)
                        .bind(assistant.status)
                        .bind(assistant_sequence)
                        .bind(assistant_input_tokens)
                        .bind(assistant_output_tokens)
                        .bind(&assistant.model_id)
                        .bind(&assistant.provider_id)
                        .bind(&assistant.artifacts_json)
                        .bind(&assistant.metadata_json)
                        .bind(&assistant.parent_message_id)
                        .bind(&assistant.created_at)
                        .bind(&assistant.updated_at)
                        .bind(&assistant.turn_id)
                        .execute(&mut *tx)
                        .await?;

                    let updated_rows = sqlx::query(SQL_UPDATE_AGENT_SESSION)
                        .bind(&session.project_id)
                        .bind(&session.title)
                        .bind(session.status)
                        .bind(&session.provider_binding_id)
                        .bind(&session.model_id)
                        .bind(message_count)
                        .bind(last_message_sequence)
                        .bind(total_input_tokens)
                        .bind(total_output_tokens)
                        .bind(&session.metadata_json)
                        .bind(version)
                        .bind(&session.updated_at)
                        .bind(&session.last_message_at)
                        .bind(&session.closed_at)
                        .bind(&session.archived_at)
                        .bind(&session.deleted_at)
                        .bind(tenant_id)
                        .bind(organization_id)
                        .bind(&session_id)
                        .bind(previous_version)
                        .execute(&mut *tx)
                        .await?;
                    if updated_rows.rows_affected() == 0 {
                        return Err(kernel_err(KernelError::conflict("session update conflict")));
                    }

                    tx.commit().await?;
                    Ok((session, user, assistant))
                })
                .await
            })
        })
    }

    fn list_message_drive_ref_rows(
        &self,
        tenant_id: u64,
        organization_id: u64,
        message_id: &str,
    ) -> KernelResult<Vec<AgentMessageDriveRefRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_AGENT_MESSAGE_DRIVE_REFS,
                tenant_id,
                organization_id,
                message_id
            )?
            .into_iter()
            .map(pg_row_to_agent_message_drive_ref_row)
            .collect()
        })
    }

    fn list_message_drive_ref_rows_batch(
        &self,
        tenant_id: u64,
        organization_id: u64,
        message_ids: &[String],
    ) -> KernelResult<Vec<AgentMessageDriveRefRow>> {
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(organization_id, "organization_id")?;
        self.with_pool(|pool| {
            pg_query!(
                pool,
                SQL_LIST_AGENT_MESSAGE_DRIVE_REFS_BATCH,
                tenant_id,
                organization_id,
                message_ids
            )?
            .into_iter()
            .map(pg_row_to_agent_message_drive_ref_row)
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
                row.agent_id,
                row.engine_key,
                row.interaction_id,
                row.kind,
                row.status,
                row.prompt,
                row.options_json,
                row.resolution_json,
                version,
                row.created_at,
                row.updated_at,
                row.resolved_at
            )?;
            Ok(())
        })
    }

    fn update_interaction_row(&self, row: AgentInteractionRow) -> KernelResult<()> {
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
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
                version,
                row.updated_at,
                row.resolved_at,
                tenant_id,
                row.session_id,
                row.interaction_id,
                previous_version
            )?;

            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_INTERACTION,
                    tenant_id,
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
        session_id: &str,
        interaction_id: &str,
    ) -> KernelResult<Option<AgentInteractionRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_INTERACTION,
                tenant_id,
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
        let status_code: Option<i16> = query
            .status
            .as_deref()
            .and_then(AgentInteractionStatus::from_code)
            .map(|s| s.as_db_code());
        let page_size = query.pagination.page_size as i64;
        let offset = query.pagination.offset as i64;

        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT_INTERACTIONS,
                tenant_id,
                query.session_id,
                status_code,
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
        let status_code: Option<i16> = query
            .status
            .as_deref()
            .and_then(AgentInteractionStatus::from_code)
            .map(|s| s.as_db_code());

        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_COUNT_AGENT_INTERACTIONS,
                tenant_id,
                query.session_id,
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
                row.subject_id,
                row.subject_tenant_id,
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
fn int64_to_u64(value: i64, field: &str) -> KernelResult<u64> {
    u64::try_from(value).map_err(|_| {
        KernelError::validation(format!("{field} must be a positive postgres int64 value"))
    })
}

#[cfg(feature = "sqlite-sync")]
fn sqlite_row_to_agent_business_row(row: SqliteRow) -> KernelResult<AgentBusinessRow> {
    let read = |error: sqlx::Error| {
        KernelError::provider_error("sqlite_row_decode_error", error.to_string())
    };
    Ok(AgentBusinessRow {
        id: int64_to_u64(row.try_get("id").map_err(&read)?, "id")?,
        uuid: row.try_get("uuid").map_err(&read)?,
        tenant_id: int64_to_u64(row.try_get("tenant_id").map_err(&read)?, "tenant_id")?,
        organization_id: int64_to_u64(
            row.try_get("organization_id").map_err(&read)?,
            "organization_id",
        )?,
        owner_user_id: int64_to_u64(
            row.try_get("owner_user_id").map_err(&read)?,
            "owner_user_id",
        )?,
        agent_id: row.try_get("agent_id").map_err(&read)?,
        code: row.try_get("code").map_err(&read)?,
        display_name: row.try_get("display_name").map_err(&read)?,
        description: row.try_get("description").map_err(&read)?,
        manifest_json: row.try_get("manifest_json").map_err(&read)?,
        default_code_task_intent_json: row
            .try_get("default_code_task_intent_json")
            .map_err(&read)?,
        implementation_provider_id: row.try_get("implementation_provider_id").map_err(&read)?,
        implementation_kind: row.try_get("implementation_kind").map_err(&read)?,
        implementation_type: row.try_get("implementation_type").map_err(&read)?,
        status: row.try_get("status").map_err(&read)?,
        visibility: row.try_get("visibility").map_err(&read)?,
        tags_json: row.try_get("tags_json").map_err(&read)?,
        created_at: row.try_get("created_at").map_err(&read)?,
        updated_at: row.try_get("updated_at").map_err(&read)?,
        deleted_at: row.try_get("deleted_at").map_err(&read)?,
        version: int64_to_u64(row.try_get("version").map_err(&read)?, "version")?,
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
fn pg_row_to_agent_session_row(row: PgRow) -> KernelResult<AgentSessionRow> {
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
        title: row.try_get("title").map_err(map_sqlx_error)?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        provider_binding_id: row.try_get("provider_binding_id").map_err(map_sqlx_error)?,
        model_id: row.try_get("model_id").map_err(map_sqlx_error)?,
        message_count: int64_to_u64(
            row.try_get("message_count").map_err(map_sqlx_error)?,
            "message_count",
        )?,
        last_message_sequence: int64_to_u64(
            row.try_get("last_message_sequence")
                .map_err(map_sqlx_error)?,
            "last_message_sequence",
        )?,
        total_input_tokens: int64_to_u64(
            row.try_get("total_input_tokens").map_err(map_sqlx_error)?,
            "total_input_tokens",
        )?,
        total_output_tokens: int64_to_u64(
            row.try_get("total_output_tokens").map_err(map_sqlx_error)?,
            "total_output_tokens",
        )?,
        metadata_json: row.try_get("metadata_json").map_err(map_sqlx_error)?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        last_message_at: row.try_get("last_message_at").map_err(map_sqlx_error)?,
        closed_at: row.try_get("closed_at").map_err(map_sqlx_error)?,
        archived_at: row.try_get("archived_at").map_err(map_sqlx_error)?,
        deleted_at: row.try_get("deleted_at").map_err(map_sqlx_error)?,
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
        last_read_message_sequence: row
            .try_get::<Option<i64>, _>("last_read_message_sequence")
            .map_err(map_sqlx_error)?
            .map(|value| int64_to_u64(value, "last_read_message_sequence"))
            .transpose()?,
        custom_title: row.try_get("custom_title").map_err(map_sqlx_error)?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_message_feedback_row(row: PgRow) -> KernelResult<AgentMessageFeedbackRow> {
    Ok(AgentMessageFeedbackRow {
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
        message_id: row.try_get("message_id").map_err(map_sqlx_error)?,
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
fn pg_row_to_agent_message_drive_ref_row(row: PgRow) -> KernelResult<AgentMessageDriveRefRow> {
    let sort_order: i32 = row.try_get("sort_order").map_err(map_sqlx_error)?;
    let created_by: i64 = row.try_get("created_by").map_err(map_sqlx_error)?;
    Ok(AgentMessageDriveRefRow {
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
        message_id: row.try_get("message_id").map_err(map_sqlx_error)?,
        media_role: row.try_get("media_role").map_err(map_sqlx_error)?,
        drive_space_id: row.try_get("drive_space_id").map_err(map_sqlx_error)?,
        drive_node_id: row.try_get("drive_node_id").map_err(map_sqlx_error)?,
        drive_uri: row.try_get("drive_uri").map_err(map_sqlx_error)?,
        media_resource_id: row.try_get("media_resource_id").map_err(map_sqlx_error)?,
        object_blob_id: row.try_get("object_blob_id").map_err(map_sqlx_error)?,
        resource_snapshot_json: row
            .try_get("resource_snapshot_json")
            .map_err(map_sqlx_error)?,
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
fn pg_row_to_agent_message_row(row: PgRow) -> KernelResult<AgentMessageRow> {
    Ok(AgentMessageRow {
        id: int64_to_u64(row.try_get("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(
            row.try_get("tenant_id").map_err(map_sqlx_error)?,
            "tenant_id",
        )?,
        session_id: row.try_get("session_id").map_err(map_sqlx_error)?,
        agent_id: row.try_get("agent_id").map_err(map_sqlx_error)?,
        message_id: row.try_get("message_id").map_err(map_sqlx_error)?,
        role: row.try_get("role").map_err(map_sqlx_error)?,
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
        artifacts_json: row.try_get("artifacts_json").map_err(map_sqlx_error)?,
        metadata_json: row.try_get("metadata_json").map_err(map_sqlx_error)?,
        parent_message_id: row.try_get("parent_message_id").map_err(map_sqlx_error)?,
        turn_id: row.try_get("turn_id").map_err(map_sqlx_error)?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
    })
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_chat_turn_row(row: PgRow) -> KernelResult<AgentChatTurnRow> {
    Ok(AgentChatTurnRow {
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
        client_request_id: row.try_get("client_request_id").map_err(map_sqlx_error)?,
        idempotency_key: row.try_get("idempotency_key").map_err(map_sqlx_error)?,
        payload_hash: row.try_get("payload_hash").map_err(map_sqlx_error)?,
        request_message_id: row.try_get("request_message_id").map_err(map_sqlx_error)?,
        response_message_id: row.try_get("response_message_id").map_err(map_sqlx_error)?,
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
        finish_reason: row.try_get("finish_reason").map_err(map_sqlx_error)?,
        error_code: row.try_get("error_code").map_err(map_sqlx_error)?,
        error_detail: row.try_get("error_detail").map_err(map_sqlx_error)?,
        trace_id: row.try_get("trace_id").map_err(map_sqlx_error)?,
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
        agent_id: row.try_get("agent_id").map_err(map_sqlx_error)?,
        engine_key: row.try_get("engine_key").map_err(map_sqlx_error)?,
        interaction_id: row.try_get("interaction_id").map_err(map_sqlx_error)?,
        kind: row.try_get("kind").map_err(map_sqlx_error)?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        prompt: row.try_get("prompt").map_err(map_sqlx_error)?,
        options_json: row.try_get("options_json").map_err(map_sqlx_error)?,
        resolution_json: row.try_get("resolution_json").map_err(map_sqlx_error)?,
        version: int64_to_u64(row.try_get("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        resolved_at: row.try_get("resolved_at").map_err(map_sqlx_error)?,
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
    use super::{
        build_agent_business_uuid, build_agent_provider_binding_uuid, build_composition_slot_uuid,
        build_interaction_uuid, build_message_uuid, build_session_uuid, build_task_uuid,
        extract_event_context, AgentAuditEventRow,
    };
    use sdkwork_agent_kernel::{KernelEvent, KernelEventSeverity, KernelEventSource};

    #[test]
    fn storage_uuids_are_stable_bounded_and_resource_scoped() {
        let tenant_id = 4_096_123_456_789_012_345;
        let agent_id = format!("agent.pc.{}.123456789abc", "a".repeat(48));
        let session_id = format!("session.pc.{}", "s".repeat(100));
        let message_id = format!("message.pc.{}", "m".repeat(100));
        let interaction_id = format!("interaction.pc.{}", "i".repeat(100));
        let task_id = format!("task.pc.{}", "t".repeat(100));
        let slot_id = format!("slot.pc.{}", "c".repeat(100));
        let binding_id = format!("binding.pc.{}", "b".repeat(100));

        let uuids = [
            build_agent_business_uuid(tenant_id, &agent_id),
            build_agent_provider_binding_uuid(tenant_id, &agent_id, &binding_id),
            build_composition_slot_uuid(tenant_id, &agent_id, &slot_id),
            build_session_uuid(tenant_id, &session_id),
            build_message_uuid(tenant_id, &session_id, &message_id),
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
                    "tenant_id": tenant_id.to_string(),
                    "agent_id": agent_id,
                    "agent_internal_id": "4096123456789012346"
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
    }

    #[cfg(feature = "sqlite-sync")]
    fn sqlite_agent_record() -> crate::domain::AgentBusinessRecord {
        crate::domain::AgentBusinessRecord {
            id: 101,
            agent_id: "agent.sqlite.test".to_string(),
            tenant_id: 100_001,
            organization_id: 0,
            owner_user_id: 20,
            code: "sqlite-test".to_string(),
            display_name: "SQLite Test".to_string(),
            description: Some("SQLite round trip".to_string()),
            manifest: sdkwork_agent_kernel::AgentManifest {
                schema_version: "1.0.0".to_string(),
                manifest_type: "agent".to_string(),
                agent_id: "agent.sqlite.test".to_string(),
                name: "sqlite-test".to_string(),
                display_name: "SQLite Test".to_string(),
                description: "SQLite round trip".to_string(),
                version: "1.0.0".to_string(),
                domain: "intelligence".to_string(),
                required_capabilities: vec![],
                optional_capabilities: vec![],
                required_capability_requirements: vec![],
                optional_capability_requirements: vec![],
                event_families: vec![],
                owner_name: "sdkwork".to_string(),
                status: "active".to_string(),
            },
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: crate::domain::AgentImplementationType::SdkworkNative,
            status: crate::domain::AgentBusinessStatus::Draft,
            visibility: crate::domain::AgentVisibility::Private,
            tags: vec!["sqlite".to_string()],
            version: 1,
            created_at: "2026-07-14T00:00:00Z".to_string(),
            updated_at: "2026-07-14T00:00:00Z".to_string(),
            deleted_at: None,
        }
    }

    #[cfg(feature = "sqlite-sync")]
    fn sqlite_adapter_with_schema() -> super::SyncSqliteAdapter {
        let adapter = super::SyncSqliteAdapter::connect("sqlite::memory:")
            .expect("SQLite adapter should connect");
        let sqlite = adapter.pool().pool().clone();
        adapter
            .pool()
            .run_kernel(async move {
                sqlx::raw_sql(include_str!(
                    "../../../database/ddl/baseline/sqlite/0001_agents_baseline.sql"
                ))
                .execute(&sqlite)
                .await
                .map(|_| ())
            })
            .expect("SQLite baseline should execute");
        adapter
    }

    #[cfg(feature = "sqlite-sync")]
    #[test]
    fn sqlite_adapter_constructs_with_real_pool_and_generates_ids() {
        let adapter = super::SyncSqliteAdapter::connect("sqlite::memory:")
            .expect("SQLite adapter should connect");
        let first = adapter.next_id().expect("first ID should be generated");
        let second = adapter.next_id().expect("second ID should be generated");

        assert!(second > first);
        assert_eq!(
            adapter.pool().database_pool().engine(),
            sdkwork_database_config::DatabaseEngine::Sqlite
        );
    }

    #[cfg(feature = "sqlite-sync")]
    #[test]
    fn sqlite_adapter_rejects_postgres_urls() {
        assert!(super::SyncSqliteAdapter::connect("postgres://localhost/agents").is_err());
    }

    #[cfg(feature = "sqlite-sync")]
    #[test]
    fn sqlite_agent_insert_and_get_round_trip_is_tenant_scoped() {
        let adapter = sqlite_adapter_with_schema();
        let record = sqlite_agent_record();
        adapter
            .insert_agent(record.clone())
            .expect("SQLite agent insert should succeed");

        assert_eq!(
            adapter
                .get_agent(record.tenant_id, &record.agent_id)
                .expect("SQLite agent read should succeed"),
            Some(record.clone())
        );
        assert_eq!(
            adapter
                .get_agent(record.tenant_id + 1, &record.agent_id)
                .expect("cross-tenant read should not fail"),
            None
        );
        assert!(adapter.insert_agent(record).is_err());
    }

    #[cfg(feature = "sqlite-sync")]
    #[test]
    fn sqlite_agent_update_enforces_optimistic_version() {
        let adapter = sqlite_adapter_with_schema();
        let original = sqlite_agent_record();
        adapter
            .insert_agent(original.clone())
            .expect("SQLite agent insert should succeed");

        let mut updated = original.clone();
        updated.display_name = "Updated SQLite Agent".to_string();
        updated.version = 2;
        updated.updated_at = "2026-07-14T00:01:00Z".to_string();
        adapter
            .update_agent(updated.clone())
            .expect("matching previous version should update");
        assert_eq!(
            adapter
                .get_agent(updated.tenant_id, &updated.agent_id)
                .expect("updated agent should load"),
            Some(updated)
        );

        let mut stale = original;
        stale.version = 2;
        let error = adapter
            .update_agent(stale)
            .expect_err("stale SQLite agent update must fail");
        assert_eq!(
            error.kind(),
            sdkwork_agent_kernel::KernelErrorKind::Conflict
        );
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
