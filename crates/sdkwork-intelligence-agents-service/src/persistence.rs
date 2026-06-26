use crate::domain::{
    AgentBusinessRecord, AgentBusinessStatus, AgentDeploymentRecord, AgentDeploymentStatus,
    AgentImplementationKind, AgentImplementationType, AgentMcpAuthKind, AgentMcpServerRecord, AgentMcpTransportKind,
    AgentCompositionSlotKind, AgentCompositionSlotRecord, AgentCompositionTargetModule,
    AgentProviderBindingRecord,
    AgentVisibility,
};
use crate::ports::{AgentAuditSink, AgentListQuery, AgentMarketplaceListQuery, AgentRepository};
#[cfg(feature = "postgres-sync")]
use crate::postgres_sync_pool::{BlockingPostgresPool, PgRow};
use crate::validation::{validate_capabilities, validate_standard_id};
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
use time::{OffsetDateTime, PrimitiveDateTime};

#[cfg(feature = "postgres-sync")]
use crate::id::{AgentBusinessIdGenerator, AgentIdGenerator};


pub const SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json, default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, version FROM ai_agent WHERE tenant_id = $1 AND agent_id = $2 LIMIT 1";
pub const SQL_INSERT_AGENT: &str =
    "INSERT INTO ai_agent (id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json, default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json, created_at, updated_at, deleted_at, version) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)";
pub const SQL_UPDATE_AGENT: &str =
    "UPDATE ai_agent SET organization_id = $1, owner_user_id = $2, code = $3, display_name = $4, description = $5, manifest_json = $6, default_code_task_intent_json = $7, implementation_provider_id = $8, implementation_kind = $9, implementation_type = $10, status = $11, visibility = $12, tags_json = $13, updated_at = $14, deleted_at = $15, version = $16 WHERE tenant_id = $17 AND agent_id = $18 AND version = $19";
pub const SQL_LIST_AGENT: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json, default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, version FROM ai_agent WHERE tenant_id = $1 ORDER BY updated_at DESC";
pub const SQL_INSERT_AGENT_PROVIDER_BINDING: &str =
    "INSERT INTO ai_agent_runtime_binding (id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json, active, version, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)";
pub const SQL_UPDATE_AGENT_PROVIDER_BINDING: &str =
    "UPDATE ai_agent_runtime_binding SET provider_id = $1, implementation_kind = $2, configuration_profile_id = $3, capabilities_json = $4, active = $5, version = $6, updated_at = $7 WHERE tenant_id = $8 AND agent_id = $9 AND binding_id = $10 AND version = $11";
pub const SQL_SELECT_AGENT_PROVIDER_BINDING: &str =
    "SELECT id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json, active, version, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_runtime_binding WHERE tenant_id = $1 AND agent_id = $2 AND binding_id = $3 LIMIT 1";
pub const SQL_LIST_AGENT_PROVIDER_BINDINGS: &str =
    "SELECT id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json, active, version, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_runtime_binding WHERE tenant_id = $1 AND agent_id = $2 ORDER BY active DESC, updated_at DESC, binding_id ASC";
pub const SQL_INSERT_AGENT_DEPLOYMENT: &str =
    "INSERT INTO ai_agent_deployment (id, uuid, tenant_id, agent_id, deployment_id, binding_id, provider_id_snapshot, implementation_kind_snapshot, configuration_profile_id_snapshot, capabilities_snapshot_json, status, version, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)";
pub const SQL_LIST_AGENT_DEPLOYMENTS: &str =
    "SELECT id, uuid, tenant_id, agent_id, deployment_id, binding_id, provider_id_snapshot, implementation_kind_snapshot, configuration_profile_id_snapshot, capabilities_snapshot_json, status, version, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_deployment WHERE tenant_id = $1 AND agent_id = $2 ORDER BY created_at DESC, deployment_id ASC";
pub const SQL_INSERT_AUDIT_EVENT: &str =
    "INSERT INTO ai_agent_audit_event (id, uuid, tenant_id, organization_id, agent_internal_id, agent_id, action, subject_id, subject_tenant_id, request_id, trace_id, payload_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_internal_id, agent_id, action, subject_id, subject_tenant_id, request_id, trace_id, payload_json, created_at::text AS created_at FROM ai_agent_audit_event WHERE tenant_id = $1 AND agent_id = $2 ORDER BY created_at DESC, id DESC";
pub const SQL_INSERT_AGENT_MCP_SERVER: &str =
    "INSERT INTO a_agent_mcp_server (id, uuid, tenant_id, organization_id, owner_user_id, mcp_server_id, code, display_name, description, protocol_version, transport_kind, endpoint_ref, command_ref, auth_kind, auth_profile_id, capability_ids_json, tool_count, resource_count, prompt_count, capabilities_json, categories_json, tags_json, security_profile_id, status, visibility, version, created_at, updated_at, deleted_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29)";
pub const SQL_UPDATE_AGENT_MCP_SERVER: &str =
    "UPDATE a_agent_mcp_server SET organization_id = $1, owner_user_id = $2, code = $3, display_name = $4, description = $5, protocol_version = $6, transport_kind = $7, endpoint_ref = $8, command_ref = $9, auth_kind = $10, auth_profile_id = $11, capability_ids_json = $12, tool_count = $13, resource_count = $14, prompt_count = $15, capabilities_json = $16, categories_json = $17, tags_json = $18, security_profile_id = $19, status = $20, visibility = $21, version = $22, updated_at = $23, deleted_at = $24 WHERE tenant_id = $25 AND mcp_server_id = $26 AND version = $27";
pub const SQL_SELECT_AGENT_MCP_SERVER: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, mcp_server_id, code, display_name, description, protocol_version, transport_kind, endpoint_ref, command_ref, auth_kind, auth_profile_id, capability_ids_json, tool_count, resource_count, prompt_count, capabilities_json, categories_json, tags_json, security_profile_id, status, visibility, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM a_agent_mcp_server WHERE tenant_id = $1 AND mcp_server_id = $2 LIMIT 1";
pub const SQL_LIST_AGENT_MCP_SERVERS: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, mcp_server_id, code, display_name, description, protocol_version, transport_kind, endpoint_ref, command_ref, auth_kind, auth_profile_id, capability_ids_json, tool_count, resource_count, prompt_count, capabilities_json, categories_json, tags_json, security_profile_id, status, visibility, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM a_agent_mcp_server WHERE tenant_id = $1 ORDER BY updated_at DESC, code ASC";

pub const SQL_INSERT_AGENT_COMPOSITION_SLOT: &str =
    "INSERT INTO ai_agent_composition_slot (id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json, status, version, created_at, updated_at, deleted_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::jsonb, $14, $15, $16, $17, $18)";
pub const SQL_UPDATE_AGENT_COMPOSITION_SLOT: &str =
    "UPDATE ai_agent_composition_slot SET organization_id = $1, slot_kind = $2, target_module = $3, target_ref = $4, target_version_ref = $5, priority = $6, enabled = $7, policy_json = $8::jsonb, status = $9, version = $10, updated_at = $11, deleted_at = $12 WHERE tenant_id = $13 AND agent_id = $14 AND slot_id = $15 AND version = $16";
pub const SQL_SELECT_AGENT_COMPOSITION_SLOT: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json::text AS policy_json, status, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM ai_agent_composition_slot WHERE tenant_id = $1 AND agent_id = $2 AND slot_id = $3 LIMIT 1";
pub const SQL_LIST_AGENT_COMPOSITION_SLOTS: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json::text AS policy_json, status, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM ai_agent_composition_slot WHERE tenant_id = $1 AND agent_id = $2 ORDER BY priority ASC, slot_id ASC";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
                record.default_code_task_intent.as_ref())?,
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
                self.default_code_task_intent_json.as_deref())?,
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
                &record.binding_id),
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
pub struct AgentDeploymentRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub agent_id: String,
    pub deployment_id: String,
    pub binding_id: String,
    pub provider_id_snapshot: String,
    pub implementation_kind_snapshot: String,
    pub configuration_profile_id_snapshot: String,
    pub capabilities_snapshot_json: String,
    pub status: i16,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
}

impl AgentDeploymentRow {
    pub fn from_record(record: &AgentDeploymentRecord) -> KernelResult<Self> {
        validate_deployment_storage_contract(record)?;
        Ok(Self {
            id: record.id,
            uuid: build_agent_deployment_uuid(
                record.tenant_id,
                &record.agent_id,
                &record.deployment_id),
            tenant_id: record.tenant_id,
            agent_id: record.agent_id.clone(),
            deployment_id: record.deployment_id.clone(),
            binding_id: record.binding_id.clone(),
            provider_id_snapshot: record.provider_id_snapshot.clone(),
            implementation_kind_snapshot: record.implementation_kind_snapshot.as_str().to_string(),
            configuration_profile_id_snapshot: record.configuration_profile_id_snapshot.clone(),
            capabilities_snapshot_json: string_list_to_json(
                &record.capabilities_snapshot,
                "capabilities_snapshot")?,
            status: record.status.as_db_code(),
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        })
    }

    pub fn into_record(self) -> KernelResult<AgentDeploymentRecord> {
        let capabilities_snapshot =
            string_list_from_json(&self.capabilities_snapshot_json, "capabilities_snapshot")?;
        let record = AgentDeploymentRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            agent_id: self.agent_id,
            deployment_id: self.deployment_id,
            binding_id: self.binding_id,
            provider_id_snapshot: self.provider_id_snapshot,
            implementation_kind_snapshot: parse_implementation_kind(
                &self.implementation_kind_snapshot)?,
            configuration_profile_id_snapshot: self.configuration_profile_id_snapshot,
            capabilities_snapshot,
            status: AgentDeploymentStatus::from_db_code(self.status).ok_or_else(|| {
                KernelError::validation(format!("invalid deployment status code: {}", self.status))
            })?,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
        };
        validate_deployment_storage_contract(&record)?;
        Ok(record)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentMcpServerRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub mcp_server_id: String,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub protocol_version: String,
    pub transport_kind: String,
    pub endpoint_ref: Option<String>,
    pub command_ref: Option<String>,
    pub auth_kind: String,
    pub auth_profile_id: Option<String>,
    pub capability_ids_json: String,
    pub tool_count: u32,
    pub resource_count: u32,
    pub prompt_count: u32,
    pub capabilities_json: String,
    pub categories_json: String,
    pub tags_json: String,
    pub security_profile_id: Option<String>,
    pub status: i16,
    pub visibility: i16,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

impl AgentMcpServerRow {
    pub fn from_record(record: &AgentMcpServerRecord) -> KernelResult<Self> {
        validate_mcp_server_storage_contract(record)?;
        Ok(Self {
            id: record.id,
            uuid: build_agent_mcp_server_uuid(record.tenant_id, &record.mcp_server_id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            owner_user_id: record.owner_user_id,
            mcp_server_id: record.mcp_server_id.clone(),
            code: record.code.clone(),
            display_name: record.display_name.clone(),
            description: record.description.clone(),
            protocol_version: record.protocol_version.clone(),
            transport_kind: record.transport_kind.as_str().to_string(),
            endpoint_ref: record.endpoint_ref.clone(),
            command_ref: record.command_ref.clone(),
            auth_kind: record.auth_kind.as_str().to_string(),
            auth_profile_id: record.auth_profile_id.clone(),
            capability_ids_json: string_list_to_json(&record.capability_ids, "capability_ids")?,
            tool_count: record.tool_count,
            resource_count: record.resource_count,
            prompt_count: record.prompt_count,
            capabilities_json: record.capabilities_json.clone(),
            categories_json: string_list_to_json(&record.categories, "categories")?,
            tags_json: string_list_to_json(&record.tags, "tags")?,
            security_profile_id: record.security_profile_id.clone(),
            status: record.status.as_db_code(),
            visibility: record.visibility.as_db_code(),
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            deleted_at: record.deleted_at.clone(),
        })
    }

    pub fn into_record(self) -> KernelResult<AgentMcpServerRecord> {
        let record = AgentMcpServerRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            owner_user_id: self.owner_user_id,
            mcp_server_id: self.mcp_server_id,
            code: self.code,
            display_name: self.display_name,
            description: self.description,
            protocol_version: self.protocol_version,
            transport_kind: parse_mcp_transport_kind(&self.transport_kind)?,
            endpoint_ref: self.endpoint_ref,
            command_ref: self.command_ref,
            auth_kind: parse_mcp_auth_kind(&self.auth_kind)?,
            auth_profile_id: self.auth_profile_id,
            capability_ids: string_list_from_json(&self.capability_ids_json, "capability_ids")?,
            tool_count: self.tool_count,
            resource_count: self.resource_count,
            prompt_count: self.prompt_count,
            capabilities_json: self.capabilities_json,
            categories: string_list_from_json(&self.categories_json, "categories")?,
            tags: string_list_from_json(&self.tags_json, "tags")?,
            security_profile_id: self.security_profile_id,
            status: AgentBusinessStatus::from_db_code(self.status).ok_or_else(|| {
                KernelError::validation(format!("invalid mcp server status code: {}", self.status))
            })?,
            visibility: AgentVisibility::from_db_code(self.visibility).ok_or_else(|| {
                KernelError::validation(format!(
                    "invalid mcp server visibility code: {}",
                    self.visibility
                ))
            })?,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        };
        validate_mcp_server_storage_contract(&record)?;
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
            slot_kind: AgentCompositionSlotKind::from_str(self.slot_kind.as_str())
                .ok_or_else(|| KernelError::validation(format!("invalid slot_kind: {}", self.slot_kind)))?,
            target_module: AgentCompositionTargetModule::from_str(self.target_module.as_str())
                .ok_or_else(|| KernelError::validation(format!("invalid target_module: {}", self.target_module)))?,
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

fn build_composition_slot_uuid(tenant_id: u64, agent_id: &str, slot_id: &str) -> String {
    format!("composition_slot_{tenant_id}_{agent_id}_{slot_id}")
}

fn validate_deployment_storage_contract(record: &AgentDeploymentRecord) -> KernelResult<()> {
    validate_standard_id(
        record.deployment_id.as_str(),
        "deploymentId",
        Some("deployment."))?;
    validate_standard_id(record.binding_id.as_str(), "bindingId", Some("binding."))?;
    validate_standard_id(
        record.provider_id_snapshot.as_str(),
        "providerId",
        Some("provider."))?;
    validate_standard_id(
        record.configuration_profile_id_snapshot.as_str(),
        "configurationProfileId",
        Some("profile."))?;
    validate_capabilities(
        record.capabilities_snapshot.as_slice(),
        "capabilitiesSnapshot")?;
    Ok(())
}

fn validate_mcp_server_storage_contract(record: &AgentMcpServerRecord) -> KernelResult<()> {
    validate_standard_id(
        record.mcp_server_id.as_str(),
        "mcpServerId",
        Some("mcp.server."))?;
    if let Some(endpoint_ref) = record.endpoint_ref.as_deref() {
        validate_standard_id(endpoint_ref, "endpointRef", Some("endpoint."))?;
    }
    if let Some(command_ref) = record.command_ref.as_deref() {
        validate_standard_id(command_ref, "commandRef", Some("command."))?;
    }
    if let Some(auth_profile_id) = record.auth_profile_id.as_deref() {
        validate_standard_id(auth_profile_id, "authProfileId", Some("profile."))?;
    }
    if let Some(security_profile_id) = record.security_profile_id.as_deref() {
        validate_standard_id(security_profile_id, "securityProfileId", Some("profile."))?;
    }
    validate_capabilities(record.capability_ids.as_slice(), "capabilityIds")?;
    validate_json_text(record.capabilities_json.as_str(), "capabilitiesJson")?;
    validate_slug_list(record.categories.as_slice(), "categories")?;
    validate_slug_list(record.tags.as_slice(), "tags")?;
    Ok(())
}






fn validate_score_value(value: f32, field_name: &str) -> KernelResult<()> {
    if !(0.0..=1.0).contains(&value) || !value.is_finite() {
        return Err(KernelError::validation(format!(
            "{field_name} must be between 0 and 1"
        )));
    }
    Ok(())
}

fn validate_non_empty_storage_text(value: &str, field_name: &str) -> KernelResult<()> {
    crate::validation::require_trimmed_non_blank(value, field_name)
}

fn validate_optional_plain_storage_ref(value: Option<&str>, field_name: &str) -> KernelResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    validate_non_empty_storage_text(value, field_name)?;
    reject_plaintext_secret_material(value, field_name)?;
    if value.chars().count() > 128 {
        return Err(KernelError::validation(format!(
            "{field_name} must be at most 128 characters"
        )));
    }
    Ok(())
}

fn validate_safe_storage_text_field(
    value: &str,
    field_name: &str,
    max_chars: usize) -> KernelResult<()> {
    validate_non_empty_storage_text(value, field_name)?;
    reject_plaintext_secret_material(value, field_name)?;
    if value.chars().count() > max_chars {
        return Err(KernelError::validation(format!(
            "{field_name} must be at most {max_chars} characters"
        )));
    }
    Ok(())
}

fn validate_optional_storage_text(value: Option<&str>, field_name: &str) -> KernelResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    validate_non_empty_storage_text(value, field_name)?;
    reject_plaintext_secret_material(value, field_name)
}

fn validate_slug_code(value: &str, field_name: &str) -> KernelResult<()> {
    validate_non_empty_storage_text(value, field_name)?;
    if value.chars().count() > 128 {
        return Err(KernelError::validation(format!(
            "{field_name} must be at most 128 characters"
        )));
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_'))
    {
        return Err(KernelError::validation(format!(
            "{field_name} must use lowercase slug characters"
        )));
    }
    Ok(())
}

fn reject_plaintext_secret_material(value: &str, field_name: &str) -> KernelResult<()> {
    let normalized = value.to_lowercase();
    for marker in [
        "api_key=",
        "apikey=",
        "access_token=",
        "refresh_token=",
        "secret=",
        "password=",
        "bearer ",
        "sk-",
    ] {
        if normalized.contains(marker) {
            return Err(KernelError::validation(format!(
                "{field_name} must not contain plaintext secret material"
            )));
        }
    }
    Ok(())
}

fn validate_json_text(input: &str, field_name: &str) -> KernelResult<()> {
    let _: serde_json::Value = serde_json::from_str(input)
        .map_err(|error| KernelError::validation(format!("invalid {field_name} json: {error}")))?;
    Ok(())
}

fn validate_slug_list(values: &[String], field_name: &str) -> KernelResult<()> {
    let mut seen = std::collections::HashSet::new();
    for value in values {
        validate_non_empty_storage_text(value, field_name)?;
        if !value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_'))
        {
            return Err(KernelError::validation(format!(
                "{field_name} values must use lowercase slug characters"
            )));
        }
        if !seen.insert(value.as_str()) {
            return Err(KernelError::validation(format!(
                "{field_name} must not contain duplicate value: {value}"
            )));
        }
    }
    Ok(())
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
    pub fn from_kernel_event(
        event: &KernelEvent,
        id: u64,
        tenant_id: u64,
        organization_id: u64,
        agent_internal_id: u64,
        agent_id: &str) -> KernelResult<Self> {
        let occurred_at = event
            .occurred_at
            .clone()
            .ok_or_else(|| KernelError::validation("audit event occurred_at is required"))?;

        Ok(Self {
            id,
            uuid: format!("audit_{}_{}", tenant_id, event.event_id),
            tenant_id,
            organization_id,
            agent_internal_id,
            agent_id: agent_id.to_string(),
            action: event
                .event_type
                .rsplit('.')
                .next()
                .unwrap_or("unknown")
                .to_string(),
            subject_id: event
                .correlation_id
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            subject_tenant_id: "unknown".to_string(),
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
            payload.payload)
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
                "tenant_id")?,
            organization_id: int64_to_u64(
                row.try_get::<i64, _>("organization_id")
                    .map_err(map_sqlx_error)?,
                "organization_id")?,
            agent_internal_id: int64_to_u64(
                row.try_get::<i64, _>("agent_internal_id")
                    .map_err(map_sqlx_error)?,
                "agent_internal_id")?,
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

pub trait PostgresAgentRepositoryAdapter {
    fn next_id(&mut self) -> KernelResult<u64>;
    fn insert_row(&mut self, row: AgentBusinessRow) -> KernelResult<()>;
    fn update_row(&mut self, row: AgentBusinessRow) -> KernelResult<()>;
    fn get_row(&self, tenant_id: u64, agent_id: &str) -> Option<AgentBusinessRow>;
    fn list_rows(&self, query: &AgentListQuery) -> Vec<AgentBusinessRow>;
    fn insert_provider_binding_row(&mut self, row: AgentProviderBindingRow) -> KernelResult<()>;
    fn update_provider_binding_row(&mut self, row: AgentProviderBindingRow) -> KernelResult<()>;
    fn get_provider_binding_row(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str) -> Option<AgentProviderBindingRow>;
    fn list_provider_binding_rows(
        &self,
        tenant_id: u64,
        agent_id: &str) -> Vec<AgentProviderBindingRow>;
    fn insert_deployment_row(&mut self, row: AgentDeploymentRow) -> KernelResult<()>;
    fn list_deployment_rows(&self, tenant_id: u64, agent_id: &str) -> Vec<AgentDeploymentRow>;
    fn insert_mcp_server_row(&mut self, row: AgentMcpServerRow) -> KernelResult<()>;
    fn update_mcp_server_row(&mut self, row: AgentMcpServerRow) -> KernelResult<()>;
    fn get_mcp_server_row(&self, tenant_id: u64, mcp_server_id: &str) -> Option<AgentMcpServerRow>;

    fn list_mcp_server_rows(&self, query: &AgentMarketplaceListQuery) -> Vec<AgentMcpServerRow> {
        let tenant_id = match u64_to_i64(query.tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        let mut rows = self
            .with_pool(|pool| {
                let rows = pg_query!(pool, SQL_LIST_AGENT_MCP_SERVERS, tenant_id)?;
                rows.into_iter()
                    .map(pg_row_to_agent_mcp_server_row)
                    .collect::<KernelResult<Vec<_>>>()
            })
            .unwrap_or_default()
            .into_iter()
            .filter_map(|row| row.ok())
            .filter(|row| {
                marketplace_row_matches(
                    query,
                    row.organization_id,
                    row.owner_user_id,
                    row.status,
                    row.visibility,
                    row.deleted_at.as_deref(),
                    row.mcp_server_id.as_str(),
                    row.code.as_str(),
                    row.display_name.as_str(),
                    row.description.as_deref(),
                    row.categories_json.as_str(),
                    row.tags_json.as_str(),
                )
            })
            .collect::<Vec<_>>();
        rows.sort_by(|left, right| {
            compare_marketplace_standard_order(
                left.updated_at.as_str(),
                left.code.as_str(),
                right.updated_at.as_str(),
                right.code.as_str(),
            )
        });
        rows
    }


    fn insert_composition_slot_row(&mut self, row: AgentCompositionSlotRow) -> KernelResult<()> {
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

    fn update_composition_slot_row(&mut self, row: AgentCompositionSlotRow) -> KernelResult<()> {
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
}

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_composition_slot_row(row: PgRow) -> KernelResult<AgentCompositionSlotRow> {
    Ok(AgentCompositionSlotRow {
        id: int64_to_u64(row.try_get::<i64, _>("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(row.try_get::<i64, _>("tenant_id").map_err(map_sqlx_error)?, "tenant_id")?,
        organization_id: int64_to_u64(
            row.try_get::<i64, _>("organization_id").map_err(map_sqlx_error)?,
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
        version: int64_to_u64(row.try_get::<i64, _>("version").map_err(map_sqlx_error)?, "version")?,
        created_at: row.try_get("created_at").map_err(map_sqlx_error)?,
        updated_at: row.try_get("updated_at").map_err(map_sqlx_error)?,
        deleted_at: row.try_get("deleted_at").map_err(map_sqlx_error)?,
    })
}


#[cfg(feature = "postgres-sync")]
impl PostgresAuditAdapter for SyncPostgresAdapter {
    fn next_id(&mut self) -> KernelResult<u64> {
        self.id_generator.next_id()
    }

    fn insert_audit_row(&mut self, row: AgentAuditEventRow) -> KernelResult<()> {
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
        agent_id: &str) -> KernelResult<Vec<AgentAuditEventRow>> {
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
    tenant_id: u64,
    organization_id: u64,
    agent_internal_id: u64,
    agent_id: String,
}

impl<A> PostgresAgentAuditSink<A>
where
    A: PostgresAuditAdapter,
{
    pub fn new(
        adapter: A,
        tenant_id: u64,
        organization_id: u64,
        agent_internal_id: u64,
        agent_id: impl Into<String>) -> Self {
        Self {
            adapter,
            tenant_id,
            organization_id,
            agent_internal_id,
            agent_id: agent_id.into(),
        }
    }
}

impl<A> AgentAuditSink for PostgresAgentAuditSink<A>
where
    A: PostgresAuditAdapter,
{
    fn record(&mut self, event: KernelEvent) -> KernelResult<()> {
        let id = self.adapter.next_id()?;
        let row = AgentAuditEventRow::from_kernel_event(
            &event,
            id,
            self.tenant_id,
            self.organization_id,
            self.agent_internal_id,
            self.agent_id.as_str())?;
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


struct MarketplaceRecordView<'a> {
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    status: AgentBusinessStatus,
    visibility: AgentVisibility,
    deleted: bool,
    id: &'a str,
    code: &'a str,
    display_name: &'a str,
    description: Option<&'a str>,
    categories: &'a [String],
    tags: &'a [String],
}

fn marketplace_record_matches(query: &AgentMarketplaceListQuery, record: MarketplaceRecordView<'_>) -> bool {
    if record.tenant_id != query.tenant_id {
        return false;
    }
    if let Some(organization_id) = query.organization_id {
        if record.organization_id != organization_id {
            return false;
        }
    }
    if let Some(owner_user_id) = query.owner_user_id {
        if record.owner_user_id != owner_user_id {
            return false;
        }
    }
    if let Some(status) = query.status {
        if record.status != status {
            return false;
        }
    }
    if let Some(visibility) = query.visibility {
        if record.visibility != visibility {
            return false;
        }
    }
    if !query.include_deleted && record.deleted {
        return false;
    }
    if let Some(category) = query.category.as_deref() {
        if !record.categories.iter().any(|value| value == category) {
            return false;
        }
    }
    if let Some(tag) = query.tag.as_deref() {
        if !record.tags.iter().any(|value| value == tag) {
            return false;
        }
    }
    if let Some(search) = query.search_query.as_deref() {
        let needle = search.to_ascii_lowercase();
        let haystacks = [
            record.id,
            record.code,
            record.display_name,
            record.description.unwrap_or_default(),
        ];
        if !haystacks
            .iter()
            .any(|value| value.to_ascii_lowercase().contains(needle.as_str()))
        {
            return false;
        }
    }
    true
}

fn compare_marketplace_standard_order(
    left_updated_at: &str,
    left_code: &str,
    right_updated_at: &str,
    right_code: &str,
) -> std::cmp::Ordering {
    right_updated_at
        .cmp(left_updated_at)
        .then_with(|| left_code.cmp(right_code))
}

fn marketplace_row_matches(
    query: &AgentMarketplaceListQuery,
    organization_id: u64,
    owner_user_id: u64,
    status: i16,
    visibility: i16,
    deleted_at: Option<&str>,
    id: &str,
    code: &str,
    display_name: &str,
    description: Option<&str>,
    categories_json: &str,
    tags_json: &str,
) -> bool {
    let categories = string_list_from_json(categories_json, "categories").unwrap_or_default();
    let tags = string_list_from_json(tags_json, "tags").unwrap_or_default();
    marketplace_record_matches(
        query,
        MarketplaceRecordView {
            tenant_id: query.tenant_id,
            organization_id,
            owner_user_id,
            status: AgentBusinessStatus::from_db_code(status)
                .unwrap_or(AgentBusinessStatus::Draft),
            visibility: AgentVisibility::from_db_code(visibility)
                .unwrap_or(AgentVisibility::Private),
            deleted: deleted_at.is_some(),
            id,
            code,
            display_name,
            description,
            categories: categories.as_slice(),
            tags: tags.as_slice(),
        },
    )
}

fn build_agent_provider_binding_uuid(tenant_id: u64, agent_id: &str, binding_id: &str) -> String {
    format!(
        "agent_provider_binding_{}_{}_{}",
        tenant_id, agent_id, binding_id
    )
}

fn build_agent_deployment_uuid(tenant_id: u64, agent_id: &str, deployment_id: &str) -> String {
    format!(
        "agent_deployment_{}_{}_{}",
        tenant_id, agent_id, deployment_id
    )
}

fn build_agent_mcp_server_uuid(tenant_id: u64, mcp_server_id: &str) -> String {
    format!("agent_mcp_server_{}_{}", tenant_id, mcp_server_id)
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

#[cfg(any(feature = "postgres-sync", test))]
fn expected_previous_version(next_version: u64) -> KernelResult<u64> {
    next_version
        .checked_sub(1)
        .ok_or_else(|| KernelError::validation("agent version must be >= 1 for update"))
}

#[cfg(feature = "postgres-sync")]
fn parse_rfc3339_timestamp(value: &str) -> KernelResult<PrimitiveDateTime> {
    let parsed = OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
        .map_err(|error| {
            KernelError::validation(format!("invalid RFC3339 timestamp `{value}`: {error}"))
        })?;
    Ok(PrimitiveDateTime::new(parsed.date(), parsed.time()))
}

#[cfg(feature = "postgres-sync")]
fn optional_rfc3339_timestamp(value: &Option<String>) -> KernelResult<Option<PrimitiveDateTime>> {
    value.as_deref().map(parse_rfc3339_timestamp).transpose()
}

#[cfg(feature = "postgres-sync")]
fn map_sqlx_error(error: sqlx::Error) -> KernelError {
    crate::postgres_sync_pool::map_sqlx_error(error)
}
