use crate::domain::{
    AgentBusinessRecord, AgentBusinessStatus, AgentCompositionSlotKind, AgentCompositionSlotRecord,
    AgentCompositionTargetModule, AgentImplementationKind, AgentImplementationType,
    AgentInteractionKind, AgentInteractionRecord, AgentInteractionStatus, AgentMessageRecord,
    AgentMessageRole, AgentMessageStatus, AgentProviderBindingRecord, AgentSessionRecord,
    AgentSessionStatus, AgentVisibility,
};
use crate::ports::{
    AgentAuditSink, AgentListQuery, AgentRepository, InteractionListQuery, MessageListQuery,
    SessionListQuery,
};
#[cfg(feature = "postgres-sync")]
use crate::postgres_sync_pool::{BlockingPostgresPool, PgRow};
use crate::validation::{validate_capabilities, validate_standard_id};
use sdkwork_utils_rust::{is_blank, trim};
#[cfg(feature = "postgres-sync")]
use crate::{pg_execute, pg_query, pg_query_optional};
use sdkwork_agent_kernel::{
    AgentManifest, KernelError, KernelEvent, KernelEventSeverity, KernelEventSource, KernelResult,
};
use sdkwork_code_kernel::CodeTaskIntent;
use serde::{Deserialize, Serialize};
#[cfg(feature = "postgres-sync")]
use sqlx::Row;

#[cfg(feature = "postgres-sync")]
use crate::id::{AgentBusinessIdGenerator, AgentIdGenerator};

pub const SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json, default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, version FROM ai_agent WHERE tenant_id = $1 AND agent_id = $2 LIMIT 1";
pub const SQL_INSERT_AGENT: &str =
    "INSERT INTO ai_agent (id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json, default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json, created_at, updated_at, deleted_at, version) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)";
pub const SQL_UPDATE_AGENT: &str =
    "UPDATE ai_agent SET organization_id = $1, owner_user_id = $2, code = $3, display_name = $4, description = $5, manifest_json = $6, default_code_task_intent_json = $7, implementation_provider_id = $8, implementation_kind = $9, implementation_type = $10, status = $11, visibility = $12, tags_json = $13, updated_at = $14, deleted_at = $15, version = $16 WHERE tenant_id = $17 AND agent_id = $18 AND version = $19";
pub const SQL_LIST_AGENT: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json, default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, version FROM ai_agent WHERE tenant_id = $1 AND ($2 IS NULL OR organization_id = $2) AND ($3 IS NULL OR owner_user_id = $3) AND (deleted_at IS NULL OR $4::bool = true) AND ($5::text IS NULL OR LOWER(agent_id) LIKE LOWER($5::text) OR LOWER(code) LIKE LOWER($5::text) OR LOWER(display_name) LIKE LOWER($5::text) OR LOWER(COALESCE(description, '')) LIKE LOWER($5::text)) ORDER BY updated_at DESC LIMIT $6";

// All filtering (organization_id, owner_user_id, include_deleted, search_query)
// is pushed to SQL WHERE clause. Parameters: $1=tenant_id, $2=org_filter(NULL=any),
// $3=owner_filter(NULL=any), $4=include_deleted_flag, $5=search_query(NULL=none, wrapped %...% for LIKE),
// $6=page_size (mandatory limit per DATABASE_SPEC §16).
pub const SQL_INSERT_AGENT_PROVIDER_BINDING: &str =
    "INSERT INTO ai_agent_runtime_binding (id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json, active, version, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)";
pub const SQL_UPDATE_AGENT_PROVIDER_BINDING: &str =
    "UPDATE ai_agent_runtime_binding SET provider_id = $1, implementation_kind = $2, configuration_profile_id = $3, capabilities_json = $4, active = $5, version = $6, updated_at = $7 WHERE tenant_id = $8 AND agent_id = $9 AND binding_id = $10 AND version = $11";
pub const SQL_SELECT_AGENT_PROVIDER_BINDING: &str =
    "SELECT id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json, active, version, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_runtime_binding WHERE tenant_id = $1 AND agent_id = $2 AND binding_id = $3 LIMIT 1";
pub const SQL_LIST_AGENT_PROVIDER_BINDINGS: &str =
    "SELECT id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json, active, version, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_runtime_binding WHERE tenant_id = $1 AND agent_id = $2 ORDER BY active DESC, updated_at DESC, binding_id ASC";
pub const SQL_INSERT_AUDIT_EVENT: &str =
    "INSERT INTO ai_agent_audit_event (id, uuid, tenant_id, organization_id, agent_internal_id, agent_id, action, subject_id, subject_tenant_id, request_id, trace_id, payload_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_internal_id, agent_id, action, subject_id, subject_tenant_id, request_id, trace_id, payload_json, created_at::text AS created_at FROM ai_agent_audit_event WHERE tenant_id = $1 AND agent_id = $2 ORDER BY created_at DESC, id DESC";
pub const SQL_INSERT_AGENT_COMPOSITION_SLOT: &str =
    "INSERT INTO ai_agent_composition_slot (id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json, status, version, created_at, updated_at, deleted_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::jsonb, $14, $15, $16, $17, $18)";
pub const SQL_UPDATE_AGENT_COMPOSITION_SLOT: &str =
    "UPDATE ai_agent_composition_slot SET organization_id = $1, slot_kind = $2, target_module = $3, target_ref = $4, target_version_ref = $5, priority = $6, enabled = $7, policy_json = $8::jsonb, status = $9, version = $10, updated_at = $11, deleted_at = $12 WHERE tenant_id = $13 AND agent_id = $14 AND slot_id = $15 AND version = $16";
pub const SQL_SELECT_AGENT_COMPOSITION_SLOT: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json::text AS policy_json, status, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM ai_agent_composition_slot WHERE tenant_id = $1 AND agent_id = $2 AND slot_id = $3 LIMIT 1";
pub const SQL_LIST_AGENT_COMPOSITION_SLOTS: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json::text AS policy_json, status, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM ai_agent_composition_slot WHERE tenant_id = $1 AND agent_id = $2 ORDER BY priority ASC, slot_id ASC";

// Session SQL constants
// Session SQL constants (postgres-sync only)
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_SESSION: &str =
    "INSERT INTO ai_agent_session (id, uuid, tenant_id, organization_id, agent_id, owner_user_id, session_id, title, status, provider_binding_id, model_id, message_count, total_input_tokens, total_output_tokens, metadata_json, version, created_at, updated_at, last_message_at, closed_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15::jsonb, $16, $17, $18, $19, $20)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_SESSION: &str =
    "UPDATE ai_agent_session SET title = $1, status = $2, provider_binding_id = $3, model_id = $4, message_count = $5, total_input_tokens = $6, total_output_tokens = $7, metadata_json = $8::jsonb, version = $9, updated_at = $10, last_message_at = $11, closed_at = $12 WHERE tenant_id = $13 AND session_id = $14 AND version = $15";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_SESSION: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, owner_user_id, session_id, title, status, provider_binding_id, model_id, message_count, total_input_tokens, total_output_tokens, metadata_json::text AS metadata_json, version, created_at::text AS created_at, updated_at::text AS updated_at, last_message_at::text AS last_message_at, closed_at::text AS closed_at FROM ai_agent_session WHERE tenant_id = $1 AND session_id = $2 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_SESSIONS: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, owner_user_id, session_id, title, status, provider_binding_id, model_id, message_count, total_input_tokens, total_output_tokens, metadata_json, version, created_at::text AS created_at, updated_at::text AS updated_at, last_message_at::text AS last_message_at, closed_at::text AS closed_at FROM ai_agent_session WHERE tenant_id = $1 AND ($2::text IS NULL OR agent_id = $2) AND ($3::bigint IS NULL OR owner_user_id = $3) AND ($4::smallint IS NULL OR status = $4) AND ($5::bool = true OR status != 3) ORDER BY updated_at DESC LIMIT $6 OFFSET $7";

// Message SQL constants (postgres-sync only)
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_MESSAGE: &str =
    "INSERT INTO ai_agent_message (id, uuid, tenant_id, session_id, agent_id, role, message_id, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, artifacts_json, metadata_json, parent_message_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16::jsonb, $17::jsonb, $18, $19, $20)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_MESSAGE: &str =
    "UPDATE ai_agent_message SET content = $1, content_type = $2, status = $3, model_id = $4, provider_id = $5, artifacts_json = $6::jsonb, metadata_json = $7::jsonb, updated_at = $8 WHERE tenant_id = $9 AND session_id = $10 AND message_id = $11";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_MESSAGE: &str =
    "SELECT id, uuid, tenant_id, session_id, agent_id, role, message_id, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, artifacts_json::text AS artifacts_json, metadata_json::text AS metadata_json, parent_message_id, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_message WHERE tenant_id = $1 AND session_id = $2 AND message_id = $3 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_MESSAGES: &str =
    "SELECT id, uuid, tenant_id, session_id, agent_id, role, message_id, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, artifacts_json, metadata_json, parent_message_id, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_message WHERE tenant_id = $1 AND session_id = $2 AND ($3::smallint IS NULL OR role = $3) AND ($4::smallint IS NULL OR status = $4) ORDER BY sequence ASC LIMIT $5 OFFSET $6";
#[cfg(feature = "postgres-sync")]
pub const SQL_NEXT_MESSAGE_SEQUENCE: &str =
    "SELECT COALESCE(MAX(sequence), 0) + 1 AS next_sequence FROM ai_agent_message WHERE tenant_id = $1 AND session_id = $2";

// Interaction SQL constants (postgres-sync only)
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_INTERACTION: &str =
    "INSERT INTO ai_agent_interaction (id, uuid, tenant_id, organization_id, session_id, agent_id, engine_key, interaction_id, kind, status, prompt, options_json, resolution_json, version, created_at, updated_at, resolved_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::jsonb, $13::jsonb, $14, $15, $16, $17)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_INTERACTION: &str =
    "UPDATE ai_agent_interaction SET kind = $1, status = $2, prompt = $3, options_json = $4::jsonb, resolution_json = $5::jsonb, version = $6, updated_at = $7, resolved_at = $8 WHERE tenant_id = $9 AND session_id = $10 AND interaction_id = $11";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_INTERACTION: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, agent_id, engine_key, interaction_id, kind, status, prompt, options_json::text AS options_json, resolution_json::text AS resolution_json, version, created_at::text AS created_at, updated_at::text AS updated_at, resolved_at::text AS resolved_at FROM ai_agent_interaction WHERE tenant_id = $1 AND session_id = $2 AND interaction_id = $3 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_INTERACTIONS: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, agent_id, engine_key, interaction_id, kind, status, prompt, options_json::text AS options_json, resolution_json::text AS resolution_json, version, created_at::text AS created_at, updated_at::text AS updated_at, resolved_at::text AS resolved_at FROM ai_agent_interaction WHERE tenant_id = $1 AND session_id = $2 AND ($3::smallint IS NULL OR status = $3) ORDER BY created_at DESC LIMIT $4 OFFSET $5";

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
    pub title: Option<String>,
    pub status: i16,
    pub provider_binding_id: Option<String>,
    pub model_id: Option<String>,
    pub message_count: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub metadata_json: String,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub last_message_at: Option<String>,
    pub closed_at: Option<String>,
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
            title: record.title.clone(),
            status: record.status.as_db_code(),
            provider_binding_id: record.provider_binding_id.clone(),
            model_id: record.model_id.clone(),
            message_count: record.message_count,
            total_input_tokens: record.total_input_tokens,
            total_output_tokens: record.total_output_tokens,
            metadata_json: record.metadata_json.clone(),
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            last_message_at: record.last_message_at.clone(),
            closed_at: record.closed_at.clone(),
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
            title: self.title,
            status,
            provider_binding_id: self.provider_binding_id,
            model_id: self.model_id,
            message_count: self.message_count,
            total_input_tokens: self.total_input_tokens,
            total_output_tokens: self.total_output_tokens,
            metadata_json: self.metadata_json,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            last_message_at: self.last_message_at,
            closed_at: self.closed_at,
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
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        })
    }

    pub fn into_record(self) -> KernelResult<AgentMessageRecord> {
        let role = AgentMessageRole::from_db_code(self.role)
            .ok_or_else(|| KernelError::validation(format!("invalid message role db code: {}", self.role)))?;
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
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

// ============================================================================
// AgentInteractionRow — persistence row for ai_agent_interaction
// ============================================================================

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
            KernelError::validation(format!("invalid interaction status db code: {}", self.status))
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

fn build_session_uuid(tenant_id: u64, session_id: &str) -> String {
    format!("agent_session_{tenant_id}_{session_id}")
}

fn build_message_uuid(tenant_id: u64, session_id: &str, message_id: &str) -> String {
    format!("agent_message_{tenant_id}_{session_id}_{message_id}")
}

fn build_interaction_uuid(tenant_id: u64, session_id: &str, interaction_id: &str) -> String {
    format!("agent_interaction_{tenant_id}_{session_id}_{interaction_id}")
}

fn build_composition_slot_uuid(tenant_id: u64, agent_id: &str, slot_id: &str) -> String {
    format!("composition_slot_{tenant_id}_{agent_id}_{slot_id}")
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
    pub agent_internal_id: u64,
    pub agent_id: String,
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
        let agent_internal_id =
            extract_event_context(event.payload.as_str(), "agent_internal_id")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0);
        let agent_id = extract_event_context(event.payload.as_str(), "agent_id")
            .unwrap_or_else(|| "unknown".to_string());
        let subject_id = extract_event_context(event.payload.as_str(), "subject_id")
            .or_else(|| event.correlation_id.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let subject_tenant_id =
            extract_event_context(event.payload.as_str(), "subject_tenant_id")
                .unwrap_or_else(|| "unknown".to_string());

        Ok(Self {
            id,
            uuid: format!("audit_{}_{}", tenant_id, event.event_id),
            tenant_id,
            organization_id,
            agent_internal_id,
            agent_id,
            action: event
                .event_type
                .rsplit('.')
                .next()
                .unwrap_or("unknown")
                .to_string(),
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
            agent_internal_id: int64_to_u64(
                row.try_get::<i64, _>("agent_internal_id")
                    .map_err(map_sqlx_error)?,
                "agent_internal_id",
            )?,
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
pub trait PostgresAgentRepositoryAdapter: Send + Sync {
    fn next_id(&self) -> KernelResult<u64>;
    fn insert_row(&self, row: AgentBusinessRow) -> KernelResult<()>;
    fn update_row(&self, row: AgentBusinessRow) -> KernelResult<()>;
    fn get_row(&self, tenant_id: u64, agent_id: &str) -> Option<AgentBusinessRow>;
    fn list_rows(&self, query: &AgentListQuery) -> Vec<AgentBusinessRow>;
    fn insert_provider_binding_row(&self, row: AgentProviderBindingRow) -> KernelResult<()>;
    fn update_provider_binding_row(&self, row: AgentProviderBindingRow) -> KernelResult<()>;
    fn get_provider_binding_row(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
    ) -> Option<AgentProviderBindingRow>;
    fn list_provider_binding_rows(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> Vec<AgentProviderBindingRow>;
    fn insert_composition_slot_row(&self, row: AgentCompositionSlotRow) -> KernelResult<()>;
    fn update_composition_slot_row(&self, row: AgentCompositionSlotRow) -> KernelResult<()>;
    fn get_composition_slot_row(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str,
    ) -> Option<AgentCompositionSlotRow>;
    fn list_composition_slot_rows(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> Vec<AgentCompositionSlotRow>;

    // Session operations
    fn insert_session_row(&self, row: AgentSessionRow) -> KernelResult<()>;
    fn update_session_row(&self, row: AgentSessionRow) -> KernelResult<()>;
    fn get_session_row(&self, tenant_id: u64, session_id: &str) -> Option<AgentSessionRow>;
    fn list_session_rows(&self, query: &SessionListQuery) -> Vec<AgentSessionRow>;

    // Message operations
    fn insert_message_row(&self, row: AgentMessageRow) -> KernelResult<()>;
    fn update_message_row(&self, row: AgentMessageRow) -> KernelResult<()>;
    fn get_message_row(
        &self,
        tenant_id: u64,
        session_id: &str,
        message_id: &str,
    ) -> Option<AgentMessageRow>;
    fn list_message_rows(&self, query: &MessageListQuery) -> Vec<AgentMessageRow>;
    fn next_message_sequence(&self, tenant_id: u64, session_id: &str) -> KernelResult<u64>;

    // Interaction operations
    fn insert_interaction_row(&self, row: AgentInteractionRow) -> KernelResult<()>;
    fn update_interaction_row(&self, row: AgentInteractionRow) -> KernelResult<()>;
    fn get_interaction_row(
        &self,
        tenant_id: u64,
        session_id: &str,
        interaction_id: &str,
    ) -> Option<AgentInteractionRow>;
    fn list_interaction_rows(&self, query: &InteractionListQuery) -> Vec<AgentInteractionRow>;
}

pub struct PostgresAgentRepository<A>
where
    A: PostgresAgentRepositoryAdapter,
{
    adapter: A,
}

impl<A> PostgresAgentRepository<A>
where
    A: PostgresAgentRepositoryAdapter,
{
    pub fn new(adapter: A) -> Self {
        Self { adapter }
    }
}

impl<A> AgentRepository for PostgresAgentRepository<A>
where
    A: PostgresAgentRepositoryAdapter,
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

    fn get(&self, tenant_id: u64, agent_id: &str) -> Option<AgentBusinessRecord> {
        self.adapter
            .get_row(tenant_id, agent_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list(&self, query: &AgentListQuery) -> Vec<AgentBusinessRecord> {
        self.adapter
            .list_rows(query)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
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
    ) -> Option<AgentProviderBindingRecord> {
        self.adapter
            .get_provider_binding_row(tenant_id, agent_id, binding_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_provider_bindings(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> Vec<AgentProviderBindingRecord> {
        self.adapter
            .list_provider_binding_rows(tenant_id, agent_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
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
    ) -> Option<AgentCompositionSlotRecord> {
        self.adapter
            .get_composition_slot_row(tenant_id, agent_id, slot_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_composition_slots(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> Vec<AgentCompositionSlotRecord> {
        self.adapter
            .list_composition_slot_rows(tenant_id, agent_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
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

    fn get_session(&self, tenant_id: u64, session_id: &str) -> Option<AgentSessionRecord> {
        self.adapter
            .get_session_row(tenant_id, session_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_sessions(&self, query: &SessionListQuery) -> Vec<AgentSessionRecord> {
        self.adapter
            .list_session_rows(query)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
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
    ) -> Option<AgentMessageRecord> {
        self.adapter
            .get_message_row(tenant_id, session_id, message_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_messages(&self, query: &MessageListQuery) -> Vec<AgentMessageRecord> {
        self.adapter
            .list_message_rows(query)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn next_message_sequence(&self, tenant_id: u64, session_id: &str) -> KernelResult<u64> {
        self.adapter.next_message_sequence(tenant_id, session_id)
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
    ) -> Option<AgentInteractionRecord> {
        self.adapter
            .get_interaction_row(tenant_id, session_id, interaction_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_interactions(&self, query: &InteractionListQuery) -> Vec<AgentInteractionRecord> {
        self.adapter
            .list_interaction_rows(query)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }
}

pub trait PostgresAuditAdapter: Send + Sync {
    fn next_id(&self) -> KernelResult<u64>;
    fn insert_audit_row(&self, row: AgentAuditEventRow) -> KernelResult<()>;
    fn list_audit_rows(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Vec<AgentAuditEventRow>> {
        let _ = (tenant_id, agent_id);
        Ok(Vec::new())
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
            id_generator: AgentBusinessIdGenerator::new_default()
                .expect("default agents managed store snowflake node id is valid"),
        })
    }

    /// Connects using `sdkwork-database-config` env resolution for the given service name.
    ///
    /// Honors the legacy `SDKWORK_{SERVICE}_POSTGRES_URI` variable when set, then falls back to
    /// `DatabaseConfig::from_env` (`SDKWORK_{SERVICE}_DATABASE_URL` and unified claw profile keys).
    pub fn connect_from_sdkwork_env(service_name: &str) -> KernelResult<Self> {
        Ok(Self {
            pool: BlockingPostgresPool::connect_from_sdkwork_env(service_name)?,
            id_generator: AgentBusinessIdGenerator::new_default()
                .expect("default agents managed store snowflake node id is valid"),
        })
    }

    /// Connects using platform database config for agents managed store persistence.
    pub fn connect_from_agents_managed_store_env() -> KernelResult<Self> {
        Self::connect_from_sdkwork_env(AGENTS_MANAGED_STORE_DATABASE_SERVICE)
    }

    pub fn from_pool(pool: BlockingPostgresPool) -> Self {
        Self {
            pool,
            id_generator: AgentBusinessIdGenerator::new_default()
                .expect("default agents managed store snowflake node id is valid"),
        }
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

    pub fn apply_managed_store_schema(&self) -> KernelResult<()> {
        let ddl = include_str!("../specs/sql/agents_managed_store_postgres.sql");
        self.pool.execute_batch_sql(ddl)
    }

    fn with_pool<T>(
        &self,
        action: impl FnOnce(&BlockingPostgresPool) -> KernelResult<T>,
    ) -> KernelResult<T> {
        action(&self.pool)
    }
}

#[cfg(feature = "postgres-sync")]
impl PostgresAgentRepositoryAdapter for SyncPostgresAdapter {
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

    fn get_row(&self, tenant_id: u64, agent_id: &str) -> Option<AgentBusinessRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID,
                tenant_id,
                agent_id
            )?;
            row.map(pg_row_to_agent_business_row).transpose()
        })
        .ok()
        .flatten()
    }

    fn list_rows(&self, query: &AgentListQuery) -> Vec<AgentBusinessRow> {
        let tenant_id = match u64_to_i64(query.tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };

        let organization_id: Option<i64> = query.organization_id.map(|v| v as i64);
        let owner_user_id: Option<i64> = query.owner_user_id.map(|v| v as i64);
        let include_deleted = query.include_deleted;
        let search_query: Option<String> = query
            .search_query
            .as_ref()
            .filter(|q| !is_blank(Some(q.as_str())))
            .map(|q| format!("%{}%", trim(q).to_lowercase()));
        
        // Mandatory pagination per DATABASE_SPEC §16
        let page_size = query.pagination.page_size as i64;

        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT,
                tenant_id,
                organization_id,
                owner_user_id,
                include_deleted,
                search_query,
                page_size
            )?;

            let mut mapped_rows = Vec::with_capacity(rows.len());
            for row in rows {
                mapped_rows.push(pg_row_to_agent_business_row(row)?);
            }
            Ok(mapped_rows)
        })
        .unwrap_or_default()
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

    fn get_provider_binding_row(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
    ) -> Option<AgentProviderBindingRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
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
        .ok()
        .flatten()
    }

    fn list_provider_binding_rows(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> Vec<AgentProviderBindingRow> {
        let tenant_id = match u64_to_i64(tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };

        self.with_pool(|pool| {
            let rows = pg_query!(pool, SQL_LIST_AGENT_PROVIDER_BINDINGS, tenant_id, agent_id)?;

            let mut mapped_rows = Vec::with_capacity(rows.len());
            for row in rows {
                mapped_rows.push(pg_row_to_agent_provider_binding_row(row)?);
            }
            Ok(mapped_rows)
        })
        .unwrap_or_default()
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
    ) -> Option<AgentCompositionSlotRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
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
        .ok()
        .flatten()
    }

    fn list_composition_slot_rows(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> Vec<AgentCompositionSlotRow> {
        let tenant_id = match u64_to_i64(tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        self.with_pool(|pool| {
            let rows = pg_query!(pool, SQL_LIST_AGENT_COMPOSITION_SLOTS, tenant_id, agent_id)?;
            rows.into_iter()
                .map(pg_row_to_agent_composition_slot_row)
                .collect()
        })
        .unwrap_or_default()
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
                row.title,
                row.status,
                row.provider_binding_id,
                row.model_id,
                message_count,
                total_input_tokens,
                total_output_tokens,
                row.metadata_json,
                version,
                row.created_at,
                row.updated_at,
                row.last_message_at,
                row.closed_at
            )?;
            Ok(())
        })
    }

    fn update_session_row(&self, row: AgentSessionRow) -> KernelResult<()> {
        let message_count = u64_to_i64(row.message_count, "message_count")?;
        let total_input_tokens = u64_to_i64(row.total_input_tokens, "total_input_tokens")?;
        let total_output_tokens = u64_to_i64(row.total_output_tokens, "total_output_tokens")?;
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;

        self.with_pool(|pool| {
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_SESSION,
                row.title,
                row.status,
                row.provider_binding_id,
                row.model_id,
                message_count,
                total_input_tokens,
                total_output_tokens,
                row.metadata_json,
                version,
                row.updated_at,
                row.last_message_at,
                row.closed_at,
                tenant_id,
                row.session_id,
                previous_version
            )?;

            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_SESSION,
                    tenant_id,
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

    fn get_session_row(&self, tenant_id: u64, session_id: &str) -> Option<AgentSessionRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
        self.with_pool(|pool| {
            let row =
                pg_query_optional!(pool, SQL_SELECT_AGENT_SESSION, tenant_id, session_id)?;
            row.map(pg_row_to_agent_session_row).transpose()
        })
        .ok()
        .flatten()
    }

    fn list_session_rows(&self, query: &SessionListQuery) -> Vec<AgentSessionRow> {
        let tenant_id = match u64_to_i64(query.tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        let agent_id: Option<&str> = query.agent_id.as_deref();
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
                agent_id,
                owner_user_id,
                status_code,
                include_archived,
                page_size,
                offset
            )?;
            rows.into_iter()
                .map(pg_row_to_agent_session_row)
                .collect()
        })
        .unwrap_or_default()
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
            pg_execute!(
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
                row.updated_at
            )?;
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
    ) -> Option<AgentMessageRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
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
        .ok()
        .flatten()
    }

    fn list_message_rows(&self, query: &MessageListQuery) -> Vec<AgentMessageRow> {
        let tenant_id = match u64_to_i64(query.tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
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
        let offset = query.pagination.offset as i64;

        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT_MESSAGES,
                tenant_id,
                query.session_id,
                role_code,
                status_code,
                page_size,
                offset
            )?;
            rows.into_iter()
                .map(pg_row_to_agent_message_row)
                .collect()
        })
        .unwrap_or_default()
    }

    fn next_message_sequence(&self, tenant_id: u64, session_id: &str) -> KernelResult<u64> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_NEXT_MESSAGE_SEQUENCE,
                tenant_id,
                session_id
            )?;
            let next: i64 = row
                .map(|r| {
                    r.try_get::<i64, _>("next_sequence")
                        .map_err(map_sqlx_error)
                })
                .transpose()?
                .unwrap_or(1);
            int64_to_u64(next, "next_sequence")
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
                row.interaction_id
            )?;

            if updated_rows == 0 {
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
    ) -> Option<AgentInteractionRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
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
        .ok()
        .flatten()
    }

    fn list_interaction_rows(&self, query: &InteractionListQuery) -> Vec<AgentInteractionRow> {
        let tenant_id = match u64_to_i64(query.tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
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
        .unwrap_or_default()
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
impl PostgresAuditAdapter for SyncPostgresAdapter {
    fn next_id(&self) -> KernelResult<u64> {
        self.id_generator.next_id()
    }

    fn insert_audit_row(&self, row: AgentAuditEventRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let agent_internal_id = u64_to_i64(row.agent_internal_id, "agent_internal_id")?;

        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AUDIT_EVENT,
                id,
                row.uuid,
                tenant_id,
                organization_id,
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
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Vec<AgentAuditEventRow>> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id")?;
        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID,
                tenant_id,
                agent_id
            )?;
            rows.iter().map(AgentAuditEventRow::from_pg_row).collect()
        })
    }
}

pub struct PostgresAgentAuditSink<A>
where
    A: PostgresAuditAdapter,
{
    adapter: A,
}

impl<A> PostgresAgentAuditSink<A>
where
    A: PostgresAuditAdapter,
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

impl<A> AgentAuditSink for PostgresAgentAuditSink<A>
where
    A: PostgresAuditAdapter,
{
    fn record(&self, event: KernelEvent) -> KernelResult<()> {
        let id = self.adapter.next_id()?;
        let row = AgentAuditEventRow::from_kernel_event(&event, id)?;
        self.adapter.insert_audit_row(row)
    }

    fn list_events(&self, tenant_id: u64, agent_id: &str) -> KernelResult<Vec<KernelEvent>> {
        self.adapter
            .list_audit_rows(tenant_id, agent_id)?
            .into_iter()
            .map(AgentAuditEventRow::into_kernel_event)
            .collect()
    }
}

fn build_agent_business_uuid(tenant_id: u64, agent_id: &str) -> String {
    format!("agent_business_{}_{}", tenant_id, agent_id)
}

fn build_agent_provider_binding_uuid(tenant_id: u64, agent_id: &str, binding_id: &str) -> String {
    format!(
        "agent_provider_binding_{}_{}_{}",
        tenant_id, agent_id, binding_id
    )
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
        title: row.try_get("title").map_err(map_sqlx_error)?,
        status: row.try_get("status").map_err(map_sqlx_error)?,
        provider_binding_id: row.try_get("provider_binding_id").map_err(map_sqlx_error)?,
        model_id: row.try_get("model_id").map_err(map_sqlx_error)?,
        message_count: int64_to_u64(
            row.try_get("message_count").map_err(map_sqlx_error)?,
            "message_count",
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
        sequence: int64_to_u64(
            row.try_get("sequence").map_err(map_sqlx_error)?,
            "sequence",
        )?,
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
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
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
            row.try_get("organization_id")
                .map_err(map_sqlx_error)?,
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

#[cfg(test)]
mod tests {
    use super::extract_event_context;

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
