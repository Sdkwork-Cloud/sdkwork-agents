use crate::domain::{
    AgentBusinessRecord, AgentBusinessStatus, AgentDeploymentRecord, AgentDeploymentStatus,
    AgentImplementationKind, AgentImplementationType, AgentMcpAuthKind, AgentMcpServerRecord, AgentMcpTransportKind,
    AgentMemoryRecord, AgentProviderBindingRecord,
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
    "INSERT INTO a_agent_deployment (id, uuid, tenant_id, agent_id, deployment_id, binding_id, provider_id_snapshot, implementation_kind_snapshot, configuration_profile_id_snapshot, capabilities_snapshot_json, status, version, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)";
pub const SQL_LIST_AGENT_DEPLOYMENTS: &str =
    "SELECT id, uuid, tenant_id, agent_id, deployment_id, binding_id, provider_id_snapshot, implementation_kind_snapshot, configuration_profile_id_snapshot, capabilities_snapshot_json, status, version, created_at::text AS created_at, updated_at::text AS updated_at FROM a_agent_deployment WHERE tenant_id = $1 AND agent_id = $2 ORDER BY created_at DESC, deployment_id ASC";
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
pub const SQL_INSERT_AGENT_KNOWLEDGE_BASE: &str =
    "INSERT INTO a_agent_knowledge_base (id, uuid, tenant_id, organization_id, owner_user_id, knowledge_base_id, code, display_name, description, provider_id, base_kind, retrieval_modes_json, capability_ids_json, configuration_profile_id, status, visibility, version, created_at, updated_at, deleted_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)";
pub const SQL_UPDATE_AGENT_KNOWLEDGE_BASE: &str =
    "UPDATE a_agent_knowledge_base SET organization_id = $1, owner_user_id = $2, code = $3, display_name = $4, description = $5, provider_id = $6, base_kind = $7, retrieval_modes_json = $8, capability_ids_json = $9, configuration_profile_id = $10, status = $11, visibility = $12, version = $13, updated_at = $14, deleted_at = $15 WHERE tenant_id = $16 AND knowledge_base_id = $17 AND version = $18";
pub const SQL_SELECT_AGENT_KNOWLEDGE_BASE: &str =
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
pub struct AgentKnowledgeBaseRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub knowledge_base_id: String,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub provider_id: String,
    pub base_kind: String,
    pub retrieval_modes_json: String,
    pub capability_ids_json: String,
    pub configuration_profile_id: String,
    pub status: i16,
    pub visibility: i16,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

impl AgentKnowledgeBaseRow {
    pub fn from_record(record: &) -> KernelResult<Self> {
        validate_knowledge_base_storage_contract(record)?;
        Ok(Self {
            id: record.id,
            uuid: build_agent_knowledge_base_uuid(record.tenant_id, &record.knowledge_base_id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            owner_user_id: record.owner_user_id,
            knowledge_base_id: record.knowledge_base_id.clone(),
            code: record.code.clone(),
            display_name: record.display_name.clone(),
            description: record.description.clone(),
            provider_id: record.provider_id.clone(),
            base_kind: record.base_kind.as_str().to_string(),
            retrieval_modes_json: knowledge_index_kinds_to_json(&record.retrieval_modes)?,
            capability_ids_json: string_list_to_json(&record.capability_ids, "capability_ids")?,
            configuration_profile_id: record.configuration_profile_id.clone(),
            status: record.status.as_db_code(),
            visibility: record.visibility.as_db_code(),
            version: record.version,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            deleted_at: record.deleted_at.clone(),
        })
    }

    pub fn into_record(self) -> KernelResult<> {
        let record = {
            id: self.id,
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            owner_user_id: self.owner_user_id,
            knowledge_base_id: self.knowledge_base_id,
            code: self.code,
            display_name: self.display_name,
            description: self.description,
            provider_id: self.provider_id,
            base_kind: parse_knowledge_base_kind(&self.base_kind)?,
            retrieval_modes: knowledge_index_kinds_from_json(&self.retrieval_modes_json)?,
            capability_ids: string_list_from_json(&self.capability_ids_json, "capability_ids")?,
            configuration_profile_id: self.configuration_profile_id,
            status: AgentBusinessStatus::from_db_code(self.status).ok_or_else(|| {
                KernelError::validation(format!(
                    "invalid knowledge base status code: {}",
                    self.status
                ))
            })?,
            visibility: AgentVisibility::from_db_code(self.visibility).ok_or_else(|| {
                KernelError::validation(format!(
                    "invalid knowledge base visibility code: {}",
                    self.visibility
                ))
            })?,
            version: self.version,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
        };
        validate_knowledge_base_storage_contract(&record)?;
        Ok(record)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentKnowledgeSourceRow {
    pub id: u64,
    pub uuid: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub knowledge_source_id: String,
    pub knowledge_base_id: String,
    pub source_kind: String,
    pub source_ref: String,
    pub source_hash: String,
    pub sync_policy_json: String,
    pub metadata_json: String,
    pub status: i16,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String)?;
    validate_capabilities(record.capabilities.as_slice(), "capabilities")
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
        "capabilitiesSnapshot")
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
    validate_slug_list(record.tags.as_slice(), "tags")
}

fn validate_memory_store_storage_contract(record: &) -> KernelResult<()> {
    validate_standard_id(
        record.memory_store_id.as_str(),
        "memoryStoreId",
        Some("memory.store."))?;
    validate_slug_code(record.code.as_str(), "code")?;
    validate_non_empty_storage_text(record.display_name.as_str(), "displayName")?;
    validate_standard_id(record.provider_id.as_str(), "providerId", Some("provider."))?;
    validate_standard_id(
        record.configuration_profile_id.as_str(),
        "configurationProfileId",
        Some("profile."))?;
    validate_memory_index_kinds(record.retrieval_modes.as_slice())?;
    validate_capabilities(record.capability_ids.as_slice(), "capabilityIds")
}

fn validate_memory_profile_storage_contract(record: &) -> KernelResult<()> {
    validate_standard_id(
        record.memory_profile_id.as_str(),
        "memoryProfileId",
        Some("memory.profile."))?;
    validate_standard_id(
        record.memory_store_id.as_str(),
        "memoryStoreId",
        Some("memory.store."))?;
    validate_slug_code(record.code.as_str(), "code")?;
    validate_non_empty_storage_text(record.display_name.as_str(), "displayName")?;
    for (value, field_name) in [
        (record.write_policy_json.as_str(), "writePolicyJson"),
        (record.retrieval_policy_json.as_str(), "retrievalPolicyJson"),
        (
            record.compaction_policy_json.as_str(),
            "compactionPolicyJson"),
        (record.retention_policy_json.as_str(), "retentionPolicyJson"),
        (record.privacy_policy_json.as_str(), "privacyPolicyJson"),
    ] {
        validate_json_text(value, field_name)?;
        reject_plaintext_secret_material(value, field_name)?;
    }
    Ok(())
}

fn validate_memory_binding_storage_contract(record: &) -> KernelResult<()> {
    validate_standard_id(
        record.memory_binding_id.as_str(),
        "memoryBindingId",
        Some("memory.binding."))?;
    validate_standard_id(
        record.memory_profile_id.as_str(),
        "memoryProfileId",
        Some("memory.profile."))?;
    if let Some(agent_id) = record.agent_id.as_deref() {
        validate_standard_id(agent_id, "agentId", Some("agent."))?;
    }
    if let Some(deployment_id) = record.deployment_id.as_deref() {
        validate_standard_id(deployment_id, "deploymentId", Some("deployment."))?;
    }
    validate_non_empty_storage_text(record.scope_ref.as_str(), "scopeRef")?;
    reject_plaintext_secret_material(record.scope_ref.as_str(), "scopeRef")?;
    validate_memory_binding_storage_scope(
        record.scope_kind,
        record.scope_ref.as_str(),
        record.agent_id.as_deref(),
        record.deployment_id.as_deref())
}

fn validate_memory_binding_storage_scope(
    scope_kind: scope_ref: &str,
    agent_id: Option<&str>,
    deployment_id: Option<&str>) -> KernelResult<()> {
    match scope_kind {
        ::Agent => {
            let Some(agent_id) = agent_id else {
                return Err(KernelError::validation(
                    "agentId is required for agent memory binding scope"));
            };
            if scope_ref != agent_id {
                return Err(KernelError::validation(
                    "scopeRef must match agentId for agent memory binding scope"));
            }
        }
        ::Deployment => {
            if agent_id.is_none() {
                return Err(KernelError::validation(
                    "agentId is required for deployment memory binding scope"));
            }
            let Some(deployment_id) = deployment_id else {
                return Err(KernelError::validation(
                    "deploymentId is required for deployment memory binding scope"));
            };
            if scope_ref != deployment_id {
                return Err(KernelError::validation(
                    "scopeRef must match deploymentId for deployment memory binding scope"));
            }
        }
        ::User
        | ::Session
        | ::Organization
        | ::Tenant => {}
    }
    Ok(())
}

fn validate_memory_namespace_storage_contract(
    record: &) -> KernelResult<()> {
    validate_standard_id(
        record.memory_namespace_id.as_str(),
        "memoryNamespaceId",
        Some("memory.namespace."))?;
    if let Some(agent_id) = record.agent_id.as_deref() {
        validate_standard_id(agent_id, "agentId", Some("agent."))?;
    }
    validate_optional_plain_storage_ref(record.user_ref.as_deref(), "userRef")?;
    validate_optional_plain_storage_ref(record.session_ref.as_deref(), "sessionRef")?;
    validate_optional_plain_storage_ref(record.thread_ref.as_deref(), "threadRef")
}

fn validate_memory_record_storage_contract(record: &AgentMemoryRecord) -> KernelResult<()> {
    validate_standard_id(
        record.memory_id.as_str(),
        "memoryId",
        Some("memory.record."))?;
    validate_standard_id(
        record.memory_namespace_id.as_str(),
        "memoryNamespaceId",
        Some("memory.namespace."))?;
    if let Some(agent_id) = record.agent_id.as_deref() {
        validate_standard_id(agent_id, "agentId", Some("agent."))?;
    }
    validate_non_empty_storage_text(record.content_format.as_str(), "contentFormat")?;
    reject_plaintext_secret_material(record.content_format.as_str(), "contentFormat")?;
    validate_json_text(record.content_json.as_str(), "contentJson")?;
    reject_plaintext_secret_material(record.content_json.as_str(), "contentJson")?;
    validate_optional_storage_text(record.summary.as_deref(), "summary")?;
    validate_score_value(record.salience_score, "salienceScore")?;
    validate_score_value(record.confidence_score, "confidenceScore")?;
    validate_score_value(record.freshness_score, "freshnessScore")?;
    if !(0..=4).contains(&record.sensitivity_level) {
        return Err(KernelError::validation(
            "sensitivityLevel must be between 0 and 4"));
    }
    Ok(())
}

fn validate_memory_source_storage_contract(record: &) -> KernelResult<()> {
    validate_standard_id(
        record.memory_source_id.as_str(),
        "memorySourceId",
        Some("memory.source."))?;
    validate_standard_id(
        record.memory_id.as_str(),
        "memoryId",
        Some("memory.record."))?;
    validate_non_empty_storage_text(record.source_ref.as_str(), "sourceRef")?;
    reject_plaintext_secret_material(record.source_ref.as_str(), "sourceRef")?;
    validate_non_empty_storage_text(record.source_hash.as_str(), "sourceHash")?;
    validate_json_text(record.evidence_json.as_str(), "evidenceJson")?;
    reject_plaintext_secret_material(record.evidence_json.as_str(), "evidenceJson")
}

fn validate_memory_relation_storage_contract(
    record: &) -> KernelResult<()> {
    validate_standard_id(
        record.memory_relation_id.as_str(),
        "memoryRelationId",
        Some("memory.relation."))?;
    validate_standard_id(
        record.from_memory_id.as_str(),
        "fromMemoryId",
        Some("memory.record."))?;
    validate_standard_id(
        record.to_memory_id.as_str(),
        "toMemoryId",
        Some("memory.record."))?;
    if record.from_memory_id == record.to_memory_id {
        return Err(KernelError::validation(
            "memory relation endpoints must be different"));
    }
    validate_score_value(record.weight, "weight")
}

fn validate_memory_retrieval_index_storage_contract(
    record: &) -> KernelResult<()> {
    validate_standard_id(
        record.memory_index_id.as_str(),
        "memoryIndexId",
        Some("memory.index."))?;
    validate_standard_id(
        record.memory_id.as_str(),
        "memoryId",
        Some("memory.record."))?;
    validate_standard_id(
        record.index_provider_id.as_str(),
        "indexProviderId",
        Some("provider."))?;
    validate_non_empty_storage_text(record.external_ref.as_str(), "externalRef")?;
    reject_plaintext_secret_material(record.external_ref.as_str(), "externalRef")?;
    validate_non_empty_storage_text(record.content_hash.as_str(), "contentHash")?;
    if record.index_kind == ::Vector
        && (record.embedding_model_id.is_none() || record.vector_dimension.is_none())
    {
        return Err(KernelError::validation(
            "vector memory index requires embeddingModelId and vectorDimension"));
    }
    if let Some(embedding_model_id) = record.embedding_model_id.as_deref() {
        validate_standard_id(embedding_model_id, "embeddingModelId", Some("model."))?;
    }
    Ok(())
}

fn validate_knowledge_base_storage_contract(record: &) -> KernelResult<()> {
    validate_standard_id(
        record.knowledge_base_id.as_str(),
        "knowledgeBaseId",
        Some("knowledge.base."))?;
    validate_slug_code(record.code.as_str(), "code")?;
    validate_non_empty_storage_text(record.display_name.as_str(), "displayName")?;
    validate_standard_id(record.provider_id.as_str(), "providerId", Some("provider."))?;
    validate_standard_id(
        record.configuration_profile_id.as_str(),
        "configurationProfileId",
        Some("profile."))?;
    validate_knowledge_index_kinds(record.retrieval_modes.as_slice())?;
    validate_capabilities(record.capability_ids.as_slice(), "capabilityIds")
}

fn validate_knowledge_source_storage_contract(
    record: &) -> KernelResult<()> {
    validate_standard_id(
        record.knowledge_source_id.as_str(),
        "knowledgeSourceId",
        Some("knowledge.source."))?;
    validate_standard_id(
        record.knowledge_base_id.as_str(),
        "knowledgeBaseId",
        Some("knowledge.base."))?;
    validate_safe_storage_text_field(
        record.source_ref.as_str(),
        "sourceRef",
        MAX_KNOWLEDGE_REFERENCE_STORAGE_CHARS)?;
    validate_safe_storage_text_field(
        record.source_hash.as_str(),
        "sourceHash",
        MAX_KNOWLEDGE_HASH_STORAGE_CHARS)?;
    validate_json_text(record.sync_policy_json.as_str(), "syncPolicyJson")?;
    reject_plaintext_secret_material(record.sync_policy_json.as_str(), "syncPolicyJson")?;
    validate_json_text(record.metadata_json.as_str(), "metadataJson")?;
    reject_plaintext_secret_material(record.metadata_json.as_str(), "metadataJson")
}

fn validate_knowledge_document_storage_contract(
    record: &) -> KernelResult<()> {
    validate_standard_id(
        record.knowledge_document_id.as_str(),
        "knowledgeDocumentId",
        Some("knowledge.document."))?;
    validate_standard_id(
        record.knowledge_base_id.as_str(),
        "knowledgeBaseId",
        Some("knowledge.base."))?;
    if let Some(source_id) = record.knowledge_source_id.as_deref() {
        validate_standard_id(source_id, "knowledgeSourceId", Some("knowledge.source."))?;
    }
    validate_non_empty_storage_text(record.title.as_str(), "title")?;
    validate_safe_storage_text_field(
        record.content_ref.as_str(),
        "contentRef",
        MAX_KNOWLEDGE_REFERENCE_STORAGE_CHARS)?;
    validate_safe_storage_text_field(
        record.content_hash.as_str(),
        "contentHash",
        MAX_KNOWLEDGE_HASH_STORAGE_CHARS)?;
    validate_optional_storage_text(record.summary.as_deref(), "summary")?;
    validate_json_text(record.metadata_json.as_str(), "metadataJson")?;
    reject_plaintext_secret_material(record.metadata_json.as_str(), "metadataJson")?;
    validate_slug_list(record.tags.as_slice(), "tags")?;
    validate_slug_list(record.categories.as_slice(), "categories")?;
    if !(0..=5).contains(&record.trust_level) {
        return Err(KernelError::validation(
            "trustLevel must be between 0 and 5"));
    }
    validate_safe_storage_text_field(
        record.redaction_classification.as_str(),
        "redactionClassification",
        MAX_KNOWLEDGE_REDACTION_CLASSIFICATION_STORAGE_CHARS)
}

fn validate_knowledge_chunk_storage_contract(
    record: &) -> KernelResult<()> {
    validate_standard_id(
        record.knowledge_chunk_id.as_str(),
        "knowledgeChunkId",
        Some("knowledge.chunk."))?;
    validate_standard_id(
        record.knowledge_document_id.as_str(),
        "knowledgeDocumentId",
        Some("knowledge.document."))?;
    if let Some(parent_chunk_id) = record.parent_chunk_id.as_deref() {
        validate_standard_id(parent_chunk_id, "parentChunkId", Some("knowledge.chunk."))?;
    }
    if record.chunk_ordinal == 0 {
        return Err(KernelError::validation(
            "chunkOrdinal must be greater than 0"));
    }
    if let Some(heading) = record.heading.as_deref() {
        validate_safe_storage_text_field(heading, "heading", MAX_KNOWLEDGE_HEADING_STORAGE_CHARS)?;
    }
    validate_safe_storage_text_field(
        record.content_ref.as_str(),
        "contentRef",
        MAX_KNOWLEDGE_REFERENCE_STORAGE_CHARS)?;
    validate_safe_storage_text_field(
        record.content_hash.as_str(),
        "contentHash",
        MAX_KNOWLEDGE_HASH_STORAGE_CHARS)?;
    if record.token_estimate == 0 {
        return Err(KernelError::validation(
            "tokenEstimate must be greater than 0"));
    }
    validate_optional_storage_text(record.summary.as_deref(), "summary")?;
    validate_json_text(record.metadata_json.as_str(), "metadataJson")?;
    reject_plaintext_secret_material(record.metadata_json.as_str(), "metadataJson")
}

fn validate_knowledge_index_storage_contract(
    record: &) -> KernelResult<()> {
    validate_standard_id(
        record.knowledge_index_id.as_str(),
        "knowledgeIndexId",
        Some("knowledge.index."))?;
    validate_standard_id(
        record.knowledge_base_id.as_str(),
        "knowledgeBaseId",
        Some("knowledge.base."))?;
    if let Some(document_id) = record.knowledge_document_id.as_deref() {
        validate_standard_id(
            document_id,
            "knowledgeDocumentId",
            Some("knowledge.document."))?;
    }
    if let Some(chunk_id) = record.knowledge_chunk_id.as_deref() {
        validate_standard_id(chunk_id, "knowledgeChunkId", Some("knowledge.chunk."))?;
        if record.knowledge_document_id.is_none() {
            return Err(KernelError::validation(
                "knowledgeDocumentId is required when knowledgeChunkId is provided"));
        }
    }
    validate_standard_id(
        record.index_provider_id.as_str(),
        "indexProviderId",
        Some("provider."))?;
    validate_safe_storage_text_field(
        record.external_ref.as_str(),
        "externalRef",
        MAX_KNOWLEDGE_REFERENCE_STORAGE_CHARS)?;
    validate_safe_storage_text_field(
        record.content_hash.as_str(),
        "contentHash",
        MAX_KNOWLEDGE_HASH_STORAGE_CHARS)?;
    if record.index_kind == ::Vector {
        if record.embedding_model_id.is_none() || record.vector_dimension.is_none() {
            return Err(KernelError::validation(
                "vector knowledge index requires embeddingModelId and vectorDimension"));
        }
        if record.vector_dimension == Some(0) {
            return Err(KernelError::validation(
                "vectorDimension must be greater than 0"));
        }
    }
    if let Some(embedding_model_id) = record.embedding_model_id.as_deref() {
        validate_standard_id(embedding_model_id, "embeddingModelId", Some("model."))?;
    }
    Ok(())
}

fn validate_knowledge_binding_storage_contract(
    record: &) -> KernelResult<()> {
    validate_standard_id(
        record.knowledge_binding_id.as_str(),
        "knowledgeBindingId",
        Some("knowledge.binding."))?;
    validate_standard_id(
        record.knowledge_base_id.as_str(),
        "knowledgeBaseId",
        Some("knowledge.base."))?;
    if let Some(agent_id) = record.agent_id.as_deref() {
        validate_standard_id(agent_id, "agentId", Some("agent."))?;
    }
    if let Some(deployment_id) = record.deployment_id.as_deref() {
        validate_standard_id(deployment_id, "deploymentId", Some("deployment."))?;
    }
    validate_safe_storage_text_field(
        record.scope_ref.as_str(),
        "scopeRef",
        MAX_KNOWLEDGE_SCOPE_REF_STORAGE_CHARS)?;
    validate_knowledge_binding_storage_scope(
        record.scope_kind,
        record.scope_ref.as_str(),
        record.agent_id.as_deref(),
        record.deployment_id.as_deref())
}

fn validate_knowledge_binding_storage_scope(
    scope_kind: scope_ref: &str,
    agent_id: Option<&str>,
    deployment_id: Option<&str>) -> KernelResult<()> {
    match scope_kind {
        ::Agent => {
            let Some(agent_id) = agent_id else {
                return Err(KernelError::validation(
                    "agentId is required for agent knowledge binding scope"));
            };
            if scope_ref != agent_id {
                return Err(KernelError::validation(
                    "scopeRef must match agentId for agent knowledge binding scope"));
            }
        }
        ::Deployment => {
            if agent_id.is_none() {
                return Err(KernelError::validation(
                    "agentId is required for deployment knowledge binding scope"));
            }
            let Some(deployment_id) = deployment_id else {
                return Err(KernelError::validation(
                    "deploymentId is required for deployment knowledge binding scope"));
            };
            if scope_ref != deployment_id {
                return Err(KernelError::validation(
                    "scopeRef must match deploymentId for deployment knowledge binding scope"));
            }
        }
        ::User
        | ::Session
        | ::Organization
        | ::Tenant => {}
    }
    Ok(())
}

fn validate_knowledge_sync_job_storage_contract(
    record: &) -> KernelResult<()> {
    validate_standard_id(
        record.sync_job_id.as_str(),
        "syncJobId",
        Some("knowledge.sync."))?;
    validate_standard_id(
        record.knowledge_base_id.as_str(),
        "knowledgeBaseId",
        Some("knowledge.base."))?;
    if let Some(source_id) = record.knowledge_source_id.as_deref() {
        validate_standard_id(source_id, "knowledgeSourceId", Some("knowledge.source."))?;
    }
    validate_safe_storage_text_field(
        record.input_ref.as_str(),
        "inputRef",
        MAX_KNOWLEDGE_REFERENCE_STORAGE_CHARS)?;
    validate_json_text(record.input_json.as_str(), "inputJson")?;
    reject_plaintext_secret_material(record.input_json.as_str(), "inputJson")?;
    if let Some(output_json) = record.output_json.as_deref() {
        validate_json_text(output_json, "outputJson")?;
        reject_plaintext_secret_material(output_json, "outputJson")?;
    }
    if let Some(error_json) = record.error_json.as_deref() {
        validate_json_text(error_json, "errorJson")?;
        reject_plaintext_secret_material(error_json, "errorJson")?;
    }
    Ok(())
}

fn validate_memory_index_kinds(values: &[]) -> KernelResult<()> {
    if values.is_empty() {
        return Err(KernelError::validation(
            "retrievalModes must contain at least one mode"));
    }
    let mut seen = std::collections::HashSet::new();
    for value in values {
        if !seen.insert(value.as_str()) {
            return Err(KernelError::validation(format!(
                "retrievalModes must not contain duplicate mode: {}",
                value.as_str()
            )));
        }
    }
    Ok(())
}

fn validate_knowledge_index_kinds(values: &[]) -> KernelResult<()> {
    if values.is_empty() {
        return Err(KernelError::validation(
            "retrievalModes must contain at least one mode"));
    }
    let mut seen = std::collections::HashSet::new();
    for value in values {
        if !seen.insert(value.as_str()) {
            return Err(KernelError::validation(format!(
                "retrievalModes must not contain duplicate mode: {}",
                value.as_str()
            )));
        }
    }
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
    fn list_mcp_server_rows(&self, query: &AgentMarketplaceListQuery) -> Vec<AgentMcpServerRow>;
    fn insert_knowledge_base_row(&mut self, row: AgentKnowledgeBaseRow) -> KernelResult<()>;
    fn update_knowledge_base_row(&mut self, row: AgentKnowledgeBaseRow) -> KernelResult<()>;
    fn get_knowledge_base_row(
        &self,
        tenant_id: u64,
        knowledge_base_id: &str) -> Option<AgentKnowledgeBaseRow>;
    fn list_knowledge_base_rows(
        &self,
        query: &AgentMarketplaceListQuery) -> Vec<AgentKnowledgeBaseRow>;
    fn insert_knowledge_source_row(&mut self, row: AgentKnowledgeSourceRow) -> KernelResult<()>;
    fn update_knowledge_source_row(&mut self, row: AgentKnowledgeSourceRow) -> KernelResult<()>;
    fn get_knowledge_source_row(
        &self,
        tenant_id: u64,
        knowledge_source_id: &str) -> Option<AgentKnowledgeSourceRow>;
    fn list_knowledge_source_rows(
        &self,
        tenant_id: u64,
        knowledge_base_id: &str) -> Vec<AgentKnowledgeSourceRow>;
    fn insert_knowledge_document_row(&mut self, row: AgentKnowledgeDocumentRow)
        -> KernelResult<()>;
    fn update_knowledge_document_row(&mut self, row: AgentKnowledgeDocumentRow)
        -> KernelResult<()>;
    fn get_knowledge_document_row(
        &self,
        tenant_id: u64,
        knowledge_document_id: &str) -> Option<AgentKnowledgeDocumentRow>;
    fn list_knowledge_document_rows(
        &self,
        tenant_id: u64,
        knowledge_base_id: &str) -> Vec<AgentKnowledgeDocumentRow>;
    fn insert_knowledge_chunk_row(&mut self, row: AgentKnowledgeChunkRow) -> KernelResult<()>;
    fn get_knowledge_chunk_row(
        &self,
        tenant_id: u64,
        knowledge_chunk_id: &str) -> Option<AgentKnowledgeChunkRow>;
    fn list_knowledge_chunk_rows(
        &self,
        tenant_id: u64,
        knowledge_document_id: &str) -> Vec<AgentKnowledgeChunkRow>;
    fn upsert_knowledge_index_row(&mut self, row: AgentKnowledgeIndexRow) -> KernelResult<()>;
    fn get_knowledge_index_row(
        &self,
        tenant_id: u64,
        knowledge_index_id: &str) -> Option<AgentKnowledgeIndexRow>;
    fn list_knowledge_index_rows(
        &self,
        tenant_id: u64,
        knowledge_document_id: &str) -> Vec<AgentKnowledgeIndexRow>;
    fn list_knowledge_index_rows_by_base(
        &self,
        tenant_id: u64,
        knowledge_base_id: &str) -> Vec<AgentKnowledgeIndexRow>;
    fn insert_knowledge_binding_row(&mut self, row: AgentKnowledgeBindingRow) -> KernelResult<()>;
    fn get_knowledge_binding_row(
        &self,
        tenant_id: u64,
        knowledge_binding_id: &str) -> Option<AgentKnowledgeBindingRow>;
    fn list_knowledge_binding_rows(
        &self,
        tenant_id: u64,
        knowledge_base_id: &str) -> Vec<AgentKnowledgeBindingRow>;
    fn insert_knowledge_sync_job_row(&mut self, row: AgentKnowledgeSyncJobRow) -> KernelResult<()>;
    fn update_knowledge_sync_job_row(&mut self, row: AgentKnowledgeSyncJobRow) -> KernelResult<()>;
    fn get_knowledge_sync_job_row(
        &self,
        tenant_id: u64,
        sync_job_id: &str) -> Option<AgentKnowledgeSyncJobRow>;
    fn list_knowledge_sync_job_rows(
        &self,
        tenant_id: u64,
        knowledge_base_id: &str) -> Vec<AgentKnowledgeSyncJobRow>;
    fn insert_memory_store_row(&mut self, row: AgentMemoryStoreRow) -> KernelResult<()>;
    fn update_memory_store_row(&mut self, row: AgentMemoryStoreRow) -> KernelResult<()>;
    fn get_memory_store_row(
        &self,
        tenant_id: u64,
        memory_store_id: &str) -> Option<AgentMemoryStoreRow>;
    fn insert_memory_profile_row(&mut self, row: AgentMemoryProfileRow) -> KernelResult<()>;
    fn get_memory_profile_row(
        &self,
        tenant_id: u64,
        memory_profile_id: &str) -> Option<AgentMemoryProfileRow>;
    fn insert_memory_binding_row(&mut self, row: AgentMemoryBindingRow) -> KernelResult<()>;
    fn get_memory_binding_row(
        &self,
        tenant_id: u64,
        memory_binding_id: &str) -> Option<AgentMemoryBindingRow>;
    fn insert_memory_namespace_row(&mut self, row: AgentMemoryNamespaceRow) -> KernelResult<()>;
    fn get_memory_namespace_row(
        &self,
        tenant_id: u64,
        memory_namespace_id: &str) -> Option<AgentMemoryNamespaceRow>;
    fn insert_memory_record_row(&mut self, row: AgentMemoryRecordRow) -> KernelResult<()>;
    fn update_memory_record_row(&mut self, row: AgentMemoryRecordRow) -> KernelResult<()>;
    fn get_memory_record_row(
        &self,
        tenant_id: u64,
        memory_id: &str) -> Option<AgentMemoryRecordRow>;
    fn list_memory_record_rows(
        &self,
        tenant_id: u64,
        memory_namespace_id: &str) -> Vec<AgentMemoryRecordRow>;
    fn insert_memory_source_row(&mut self, row: AgentMemorySourceRow) -> KernelResult<()>;
    fn get_memory_source_row(
        &self,
        tenant_id: u64,
        memory_source_id: &str) -> Option<AgentMemorySourceRow>;
    fn list_memory_source_rows(&self, tenant_id: u64, memory_id: &str)
        -> Vec<AgentMemorySourceRow>;
    fn insert_memory_relation_row(&mut self, row: AgentMemoryRelationRow) -> KernelResult<()>;
    fn get_memory_relation_row(
        &self,
        tenant_id: u64,
        memory_relation_id: &str) -> Option<AgentMemoryRelationRow>;
    fn list_memory_relation_rows(
        &self,
        tenant_id: u64,
        memory_id: &str) -> Vec<AgentMemoryRelationRow>;
    fn upsert_memory_retrieval_index_row(
        &mut self,
        row: AgentMemoryRetrievalIndexRow) -> KernelResult<()>;
    fn get_memory_retrieval_index_row(
        &self,
        tenant_id: u64,
        memory_index_id: &str) -> Option<AgentMemoryRetrievalIndexRow>;
    fn list_memory_retrieval_index_rows(
        &self,
        tenant_id: u64,
        memory_id: &str) -> Vec<AgentMemoryRetrievalIndexRow>;

    fn insert_composition_slot_row(&mut self, row: AgentCompositionSlotRow) -> KernelResult<()>;
    fn update_composition_slot_row(&mut self, row: AgentCompositionSlotRow) -> KernelResult<()>;
    fn get_composition_slot_row(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str) -> Option<AgentCompositionSlotRow>;
    fn list_composition_slot_rows(
        &self,
        tenant_id: u64,
        agent_id: &str) -> Vec<AgentCompositionSlotRow>;
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
    fn next_id(&mut self) -> KernelResult<u64> {
        self.adapter.next_id()
    }

    fn insert(&mut self, record: AgentBusinessRecord) -> KernelResult<()> {
        self.adapter
            .insert_row(AgentBusinessRow::from_record(&record)?)
    }

    fn update(&mut self, record: AgentBusinessRecord) -> KernelResult<()> {
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

    fn insert_provider_binding(&mut self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        self.adapter
            .insert_provider_binding_row(AgentProviderBindingRow::from_record(&record)?)
    }

    fn update_provider_binding(&mut self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        self.adapter
            .update_provider_binding_row(AgentProviderBindingRow::from_record(&record)?)
    }

    fn get_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str) -> Option<AgentProviderBindingRecord> {
        self.adapter
            .get_provider_binding_row(tenant_id, agent_id, binding_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_provider_bindings(
        &self,
        tenant_id: u64,
        agent_id: &str) -> Vec<AgentProviderBindingRecord> {
        self.adapter
            .list_provider_binding_rows(tenant_id, agent_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn insert_deployment(&mut self, record: AgentDeploymentRecord) -> KernelResult<()> {
        self.adapter
            .insert_deployment_row(AgentDeploymentRow::from_record(&record)?)
    }

    fn list_deployments(&self, tenant_id: u64, agent_id: &str) -> Vec<AgentDeploymentRecord> {
        self.adapter
            .list_deployment_rows(tenant_id, agent_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn insert_mcp_server(&mut self, record: AgentMcpServerRecord) -> KernelResult<()> {
        self.adapter
            .insert_mcp_server_row(AgentMcpServerRow::from_record(&record)?)
    }

    fn update_mcp_server(&mut self, record: AgentMcpServerRecord) -> KernelResult<()> {
        self.adapter
            .update_mcp_server_row(AgentMcpServerRow::from_record(&record)?)
    }

    fn get_mcp_server(&self, tenant_id: u64, mcp_server_id: &str) -> Option<AgentMcpServerRecord> {
        self.adapter
            .get_mcp_server_row(tenant_id, mcp_server_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_mcp_servers(&self, query: &AgentMarketplaceListQuery) -> Vec<AgentMcpServerRecord> {
        self.adapter
            .list_mcp_server_rows(query)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn insert_knowledge_base(&mut self, record: ) -> KernelResult<()> {
        self.adapter
            .insert_knowledge_base_row(AgentKnowledgeBaseRow::from_record(&record)?)
    }

    fn update_knowledge_base(&mut self, record: ) -> KernelResult<()> {
        self.adapter
            .update_knowledge_base_row(AgentKnowledgeBaseRow::from_record(&record)?)
    }

    fn get_knowledge_base(
        &self,
        tenant_id: u64,
        knowledge_base_id: &str) -> Option<> {
        self.adapter
            .get_knowledge_base_row(tenant_id, knowledge_base_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_knowledge_bases(
        &self,
        query: &AgentMarketplaceListQuery) -> Vec<> {
        self.adapter
            .list_knowledge_base_rows(query)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn insert_knowledge_source(&mut self, record: ) -> KernelResult<()> {
        self.adapter
            .insert_knowledge_source_row(AgentKnowledgeSourceRow::from_record(&record)?)
    }

    fn update_knowledge_source(&mut self, record: ) -> KernelResult<()> {
        self.adapter
            .update_knowledge_source_row(AgentKnowledgeSourceRow::from_record(&record)?)
    }

    fn get_knowledge_source(
        &self,
        tenant_id: u64,
        knowledge_source_id: &str) -> Option<> {
        self.adapter
            .get_knowledge_source_row(tenant_id, knowledge_source_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_knowledge_sources(
        &self,
        tenant_id: u64,
        knowledge_base_id: &str) -> Vec<> {
        self.adapter
            .list_knowledge_source_rows(tenant_id, knowledge_base_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn insert_knowledge_document(
        &mut self,
        record: ) -> KernelResult<()> {
        self.adapter
            .insert_knowledge_document_row(AgentKnowledgeDocumentRow::from_record(&record)?)
    }

    fn update_knowledge_document(
        &mut self,
        record: ) -> KernelResult<()> {
        self.adapter
            .update_knowledge_document_row(AgentKnowledgeDocumentRow::from_record(&record)?)
    }

    fn get_knowledge_document(
        &self,
        tenant_id: u64,
        knowledge_document_id: &str) -> Option<> {
        self.adapter
            .get_knowledge_document_row(tenant_id, knowledge_document_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_knowledge_documents(
        &self,
        tenant_id: u64,
        knowledge_base_id: &str) -> Vec<> {
        self.adapter
            .list_knowledge_document_rows(tenant_id, knowledge_base_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn insert_knowledge_chunk(&mut self, record: ) -> KernelResult<()> {
        self.adapter
            .insert_knowledge_chunk_row(AgentKnowledgeChunkRow::from_record(&record)?)
    }

    fn get_knowledge_chunk(
        &self,
        tenant_id: u64,
        knowledge_chunk_id: &str) -> Option<> {
        self.adapter
            .get_knowledge_chunk_row(tenant_id, knowledge_chunk_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_knowledge_chunks(
        &self,
        tenant_id: u64,
        knowledge_document_id: &str) -> Vec<> {
        self.adapter
            .list_knowledge_chunk_rows(tenant_id, knowledge_document_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn upsert_knowledge_index(&mut self, record: ) -> KernelResult<()> {
        self.adapter
            .upsert_knowledge_index_row(AgentKnowledgeIndexRow::from_record(&record)?)
    }

    fn get_knowledge_index(
        &self,
        tenant_id: u64,
        knowledge_index_id: &str) -> Option<> {
        self.adapter
            .get_knowledge_index_row(tenant_id, knowledge_index_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_knowledge_indexes(
        &self,
        tenant_id: u64,
        knowledge_document_id: &str) -> Vec<> {
        self.adapter
            .list_knowledge_index_rows(tenant_id, knowledge_document_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn list_knowledge_indexes_by_base(
        &self,
        tenant_id: u64,
        knowledge_base_id: &str) -> Vec<> {
        self.adapter
            .list_knowledge_index_rows_by_base(tenant_id, knowledge_base_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn insert_knowledge_binding(
        &mut self,
        record: ) -> KernelResult<()> {
        self.adapter
            .insert_knowledge_binding_row(AgentKnowledgeBindingRow::from_record(&record)?)
    }

    fn get_knowledge_binding(
        &self,
        tenant_id: u64,
        knowledge_binding_id: &str) -> Option<> {
        self.adapter
            .get_knowledge_binding_row(tenant_id, knowledge_binding_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_knowledge_bindings(
        &self,
        tenant_id: u64,
        knowledge_base_id: &str) -> Vec<> {
        self.adapter
            .list_knowledge_binding_rows(tenant_id, knowledge_base_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn insert_knowledge_sync_job(
        &mut self,
        record: ) -> KernelResult<()> {
        self.adapter
            .insert_knowledge_sync_job_row(AgentKnowledgeSyncJobRow::from_record(&record)?)
    }

    fn update_knowledge_sync_job(
        &mut self,
        record: ) -> KernelResult<()> {
        self.adapter
            .update_knowledge_sync_job_row(AgentKnowledgeSyncJobRow::from_record(&record)?)
    }

    fn get_knowledge_sync_job(
        &self,
        tenant_id: u64,
        sync_job_id: &str) -> Option<> {
        self.adapter
            .get_knowledge_sync_job_row(tenant_id, sync_job_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_knowledge_sync_jobs(
        &self,
        tenant_id: u64,
        knowledge_base_id: &str) -> Vec<> {
        self.adapter
            .list_knowledge_sync_job_rows(tenant_id, knowledge_base_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn insert_memory_store(&mut self, record: ) -> KernelResult<()> {
        self.adapter
            .insert_memory_store_row(AgentMemoryStoreRow::from_record(&record)?)
    }

    fn update_memory_store(&mut self, record: ) -> KernelResult<()> {
        self.adapter
            .update_memory_store_row(AgentMemoryStoreRow::from_record(&record)?)
    }

    fn get_memory_store(
        &self,
        tenant_id: u64,
        memory_store_id: &str) -> Option<> {
        self.adapter
            .get_memory_store_row(tenant_id, memory_store_id)
            .and_then(|row| row.into_record().ok())
    }

    fn insert_memory_profile(&mut self, record: ) -> KernelResult<()> {
        self.adapter
            .insert_memory_profile_row(AgentMemoryProfileRow::from_record(&record)?)
    }

    fn get_memory_profile(
        &self,
        tenant_id: u64,
        memory_profile_id: &str) -> Option<> {
        self.adapter
            .get_memory_profile_row(tenant_id, memory_profile_id)
            .and_then(|row| row.into_record().ok())
    }

    fn insert_memory_binding(&mut self, record: ) -> KernelResult<()> {
        self.adapter
            .insert_memory_binding_row(AgentMemoryBindingRow::from_record(&record)?)
    }

    fn get_memory_binding(
        &self,
        tenant_id: u64,
        memory_binding_id: &str) -> Option<> {
        self.adapter
            .get_memory_binding_row(tenant_id, memory_binding_id)
            .and_then(|row| row.into_record().ok())
    }

    fn insert_memory_namespace(&mut self, record: ) -> KernelResult<()> {
        self.adapter
            .insert_memory_namespace_row(AgentMemoryNamespaceRow::from_record(&record)?)
    }

    fn get_memory_namespace(
        &self,
        tenant_id: u64,
        memory_namespace_id: &str) -> Option<> {
        self.adapter
            .get_memory_namespace_row(tenant_id, memory_namespace_id)
            .and_then(|row| row.into_record().ok())
    }

    fn insert_memory_record(&mut self, record: AgentMemoryRecord) -> KernelResult<()> {
        self.adapter
            .insert_memory_record_row(AgentMemoryRecordRow::from_record(&record)?)
    }

    fn update_memory_record(&mut self, record: AgentMemoryRecord) -> KernelResult<()> {
        self.adapter
            .update_memory_record_row(AgentMemoryRecordRow::from_record(&record)?)
    }

    fn get_memory_record(&self, tenant_id: u64, memory_id: &str) -> Option<AgentMemoryRecord> {
        self.adapter
            .get_memory_record_row(tenant_id, memory_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_memory_records(
        &self,
        tenant_id: u64,
        memory_namespace_id: &str) -> Vec<AgentMemoryRecord> {
        self.adapter
            .list_memory_record_rows(tenant_id, memory_namespace_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn insert_memory_source(&mut self, record: ) -> KernelResult<()> {
        self.adapter
            .insert_memory_source_row(AgentMemorySourceRow::from_record(&record)?)
    }

    fn list_memory_sources(&self, tenant_id: u64, memory_id: &str) -> Vec<> {
        self.adapter
            .list_memory_source_rows(tenant_id, memory_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn get_memory_source(
        &self,
        tenant_id: u64,
        memory_source_id: &str) -> Option<> {
        self.adapter
            .get_memory_source_row(tenant_id, memory_source_id)
            .and_then(|row| row.into_record().ok())
    }

    fn insert_memory_relation(&mut self, record: ) -> KernelResult<()> {
        self.adapter
            .insert_memory_relation_row(AgentMemoryRelationRow::from_record(&record)?)
    }

    fn list_memory_relations(
        &self,
        tenant_id: u64,
        memory_id: &str) -> Vec<> {
        self.adapter
            .list_memory_relation_rows(tenant_id, memory_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn get_memory_relation(
        &self,
        tenant_id: u64,
        memory_relation_id: &str) -> Option<> {
        self.adapter
            .get_memory_relation_row(tenant_id, memory_relation_id)
            .and_then(|row| row.into_record().ok())
    }

    fn upsert_memory_retrieval_index(
        &mut self,
        record: ) -> KernelResult<()> {
        self.adapter
            .upsert_memory_retrieval_index_row(AgentMemoryRetrievalIndexRow::from_record(&record)?)
    }

    fn list_memory_retrieval_indexes(
        &self,
        tenant_id: u64,
        memory_id: &str) -> Vec<> {
        self.adapter
            .list_memory_retrieval_index_rows(tenant_id, memory_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

    fn get_memory_retrieval_index(
        &self,
        tenant_id: u64,
        retrieval_index_id: &str) -> Option<> {
        self.adapter
            .get_memory_retrieval_index_row(tenant_id, retrieval_index_id)
            .and_then(|row| row.into_record().ok())
    }
    fn insert_composition_slot(&mut self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        self.adapter
            .insert_composition_slot_row(AgentCompositionSlotRow::from_record(&record)?)
    }

    fn update_composition_slot(&mut self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        self.adapter
            .update_composition_slot_row(AgentCompositionSlotRow::from_record(&record)?)
    }

    fn get_composition_slot(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str) -> Option<AgentCompositionSlotRecord> {
        self.adapter
            .get_composition_slot_row(tenant_id, agent_id, slot_id)
            .and_then(|row| row.into_record().ok())
    }

    fn list_composition_slots(
        &self,
        tenant_id: u64,
        agent_id: &str) -> Vec<AgentCompositionSlotRecord> {
        self.adapter
            .list_composition_slot_rows(tenant_id, agent_id)
            .into_iter()
            .filter_map(|row| row.into_record().ok())
            .collect()
    }

}

pub trait PostgresAuditAdapter {
    fn next_id(&mut self) -> KernelResult<u64>;
    fn insert_audit_row(&mut self, row: AgentAuditEventRow) -> KernelResult<()>;
    fn list_audit_rows(
        &self,
        tenant_id: u64,
        agent_id: &str) -> KernelResult<Vec<AgentAuditEventRow>> {
        let _ = (tenant_id, agent_id);
        Ok(Vec::new())
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
        slot_id: &str) -> Option<AgentCompositionSlotRow> {
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
        agent_id: &str) -> Vec<AgentCompositionSlotRow> {
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

#[cfg(feature = "postgres-sync")]
fn pg_row_to_agent_composition_slot_row(row: PgRow) -> KernelResult<AgentCompositionSlotRow> {
    Ok(AgentCompositionSlotRow {
        id: int64_to_u64(row.try_get::<i64, _>("id").map_err(map_sqlx_error)?, "id")?,
        uuid: row.try_get("uuid").map_err(map_sqlx_error)?,
        tenant_id: int64_to_u64(row.try_get::<i64, _>("tenant_id").map_err(map_sqlx_error)?, "tenant_id")?,
        organization_id: int64_to_u64(
            row.try_get::<i64, _>("organization_id").map_err(map_sqlx_error)?,
            "organization_id")?,
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
        id_generator: AgentBusinessIdGenerator) -> Self {
        Self { pool, id_generator }
    }

    pub fn apply_managed_store_schema(&self) -> KernelResult<()> {
        let ddl = include_str!("../specs/sql/agents_managed_store_postgres.sql");
        self.pool.execute_batch_sql(ddl)
    }

    fn with_pool<T>(
        &self,
        action: impl FnOnce(&BlockingPostgresPool) -> KernelResult<T>) -> KernelResult<T> {
        action(&self.pool)
    }
}

#[cfg(feature = "postgres-sync")]
impl PostgresAgentRepositoryAdapter for SyncPostgresAdapter {
    fn next_id(&mut self) -> KernelResult<u64> {
        self.id_generator.next_id()
    }

    fn insert_row(&mut self, row: AgentBusinessRow) -> KernelResult<()> {
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

    fn update_row(&mut self, row: AgentBusinessRow) -> KernelResult<()> {
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

        self.with_pool(|pool| {
            let rows = pg_query!(pool, SQL_LIST_AGENT, tenant_id)?;

            let mut mapped_rows = Vec::with_capacity(rows.len());
            for row in rows {
                mapped_rows.push(pg_row_to_agent_business_row(row)?);
            }
            Ok(mapped_rows)
        })
        .map(|rows| {
            rows.into_iter()
                .filter(|row| {
                    if let Some(organization_id) = query.organization_id {
                        row.organization_id == organization_id
                    } else {
                        true
                    }
                })
                .filter(|row| {
                    if let Some(owner_user_id) = query.owner_user_id {
                        row.owner_user_id == owner_user_id
                    } else {
                        true
                    }
                })
                .filter(|row| {
                    if query.include_deleted {
                        true
                    } else {
                        row.status != AgentBusinessStatus::Deleted.as_db_code()
                            && row.deleted_at.is_none()
                    }
                })
                .filter(|row| {
                    let Some(search_query) = query.search_query.as_ref() else {
                        return true;
                    };
                    let normalized_query = search_query.trim().to_lowercase();
                    if normalized_query.is_empty() {
                        return true;
                    }

                    let description = row.description.as_deref().unwrap_or("");
                    row.agent_id
                        .to_lowercase()
                        .contains(normalized_query.as_str())
                        || row.code.to_lowercase().contains(normalized_query.as_str())
                        || row
                            .display_name
                            .to_lowercase()
                            .contains(normalized_query.as_str())
                        || description
                            .to_lowercase()
                            .contains(normalized_query.as_str())
                })
                .collect()
        })
        .unwrap_or_default()
    }

    fn insert_provider_binding_row(&mut self, row: AgentProviderBindingRow) -> KernelResult<()> {
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

    fn update_provider_binding_row(&mut self, row: AgentProviderBindingRow) -> KernelResult<()> {
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
                        "agent provider binding version mismatch"));
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
        binding_id: &str) -> Option<AgentProviderBindingRow> {
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
        agent_id: &str) -> Vec<AgentProviderBindingRow> {
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

    fn insert_deployment_row(&mut self, row: AgentDeploymentRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let version = u64_to_i64(row.version, "version")?;

        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_DEPLOYMENT,
                id,
                row.uuid,
                tenant_id,
                row.agent_id,
                row.deployment_id,
                row.binding_id,
                row.provider_id_snapshot,
                row.implementation_kind_snapshot,
                row.configuration_profile_id_snapshot,
                row.capabilities_snapshot_json,
                row.status,
                version,
                row.created_at,
                row.updated_at
            )?;
            Ok(())
        })
    }

    fn list_deployment_rows(&self, tenant_id: u64, agent_id: &str) -> Vec<AgentDeploymentRow> {
        let tenant_id = match u64_to_i64(tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };

        self.with_pool(|pool| {
            let rows = pg_query!(pool, SQL_LIST_AGENT_DEPLOYMENTS, tenant_id, agent_id)?;

            let mut mapped_rows = Vec::with_capacity(rows.len());
            for row in rows {
                mapped_rows.push(pg_row_to_agent_deployment_row(row)?);
            }
            Ok(mapped_rows)
        })
        .unwrap_or_default()
    }

    fn insert_mcp_server_row(&mut self, row: AgentMcpServerRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(row.owner_user_id, "owner_user_id")?;
        let version = u64_to_i64(row.version, "version")?;
        let tool_count = i64::from(row.tool_count);
        let resource_count = i64::from(row.resource_count);
        let prompt_count = i64::from(row.prompt_count);
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_MCP_SERVER,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                owner_user_id,
                row.mcp_server_id,
                row.code,
                row.display_name,
                row.description,
                row.protocol_version,
                row.transport_kind,
                row.endpoint_ref,
                row.command_ref,
                row.auth_kind,
                row.auth_profile_id,
                row.capability_ids_json,
                tool_count,
                resource_count,
                prompt_count,
                row.capabilities_json,
                row.categories_json,
                row.tags_json,
                row.security_profile_id,
                row.status,
                row.visibility,
                version,
                row.created_at,
                row.updated_at,
                row.deleted_at
            )?;
            Ok(())
        })
    }

    fn update_mcp_server_row(&mut self, row: AgentMcpServerRow) -> KernelResult<()> {
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let owner_user_id = u64_to_i64(row.owner_user_id, "owner_user_id")?;
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        let tool_count = i64::from(row.tool_count);
        let resource_count = i64::from(row.resource_count);
        let prompt_count = i64::from(row.prompt_count);
        self.with_pool(|pool| {
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_MCP_SERVER,
                organization_id,
                owner_user_id,
                row.code,
                row.display_name,
                row.description,
                row.protocol_version,
                row.transport_kind,
                row.endpoint_ref,
                row.command_ref,
                row.auth_kind,
                row.auth_profile_id,
                row.capability_ids_json,
                tool_count,
                resource_count,
                prompt_count,
                row.capabilities_json,
                row.categories_json,
                row.tags_json,
                row.security_profile_id,
                row.status,
                row.visibility,
                version,
                row.updated_at,
                row.deleted_at,
                tenant_id,
                row.mcp_server_id,
                previous_version
            )?;
            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_MCP_SERVER,
                    tenant_id,
                    row.mcp_server_id
                )?
                .is_some();
                if exists {
                    return Err(KernelError::conflict("agent mcp server version mismatch"));
                }
                return Err(KernelError::validation("agent mcp server not found"));
            }
            Ok(())
        })
    }

    fn get_mcp_server_row(&self, tenant_id: u64, mcp_server_id: &str) -> Option<AgentMcpServerRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
        self.with_pool(|pool| {
            let row =
                pg_query_optional!(pool, SQL_SELECT_AGENT_MCP_SERVER, tenant_id, mcp_server_id)?;
            row.map(pg_row_to_agent_mcp_server_row).transpose()
        })
        .ok()
        .flatten()
    }

    fn list_mcp_server_rows(&self, query: &AgentMarketplaceListQuery) -> Vec<AgentMcpServerRow> {
        let tenant_id = match u64_to_i64(query.tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        self.with_pool(|pool| {
            let rows = pg_query!(pool, SQL_LIST_AGENT_MCP_SERVERS, tenant_id)?;
            rows.into_iter()
                .map(pg_row_to_agent_mcp_server_row)
                .collect()
        })
        .map(|rows: Vec<AgentMcpServerRow>| {
            rows.into_iter()
                .filter(|row| {
                    marketplace_row_matches(
                        query,
                        row.organization_id,
                        row.owner_user_id,
                        row.status,
                        row.visibility,
                        row.deleted_at.as_deref(),
                        row.mcp_server_id.as_str(),
                row.source_kind,
                row.source_ref,
                row.source_hash,
                row.sync_policy_json,
                row.metadata_json,
                row.status,
                version,
                row.created_at,
                row.updated_at,
                row.deleted_at
            )?;
            Ok(())
        })
    }

    fn update_knowledge_source_row(&mut self, row: AgentKnowledgeSourceRow) -> KernelResult<()> {
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        self.with_pool(|pool| {
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_KNOWLEDGE_SOURCE,
                organization_id,
                row.knowledge_base_id,
                row.source_kind,
                row.source_ref,
                row.source_hash,
                row.sync_policy_json,
                row.metadata_json,
                row.status,
                version,
                row.updated_at,
                row.deleted_at,
                tenant_id,
                row.knowledge_source_id,
                previous_version
            )?;
            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_KNOWLEDGE_SOURCE,
                    tenant_id,
                    row.knowledge_source_id
                )?
                .is_some();
                if exists {
                    return Err(KernelError::conflict(
                        "agent knowledge source version mismatch"));
                }
                return Err(KernelError::validation("agent knowledge source not found"));
            }
            Ok(())
        })
    }

    fn get_knowledge_source_row(
        &self,
        tenant_id: u64,
        knowledge_source_id: &str) -> Option<AgentKnowledgeSourceRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_KNOWLEDGE_SOURCE,
                tenant_id,
                knowledge_source_id
            )?;
            row.map(pg_row_to_agent_knowledge_source_row).transpose()
        })
        .ok()
        .flatten()
    }

    fn list_knowledge_source_rows(
        &self,
        tenant_id: u64,
        knowledge_base_id: &str) -> Vec<AgentKnowledgeSourceRow> {
        let tenant_id = match u64_to_i64(tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT_KNOWLEDGE_SOURCES,
                tenant_id,
                knowledge_base_id
            )?;
            rows.into_iter()
                .map(pg_row_to_agent_knowledge_source_row)
                .collect()
        })
        .unwrap_or_default()
    }

    fn insert_knowledge_document_row(
        &mut self,
        row: AgentKnowledgeDocumentRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let chunk_count = i64::from(row.chunk_count);
        let version = u64_to_i64(row.version, "version")?;
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_KNOWLEDGE_DOCUMENT,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.knowledge_document_id,
                row.knowledge_base_id,
                row.knowledge_source_id,
                row.document_kind,
                row.title,
                row.content_ref,
                row.content_hash,
                row.summary,
                row.metadata_json,
                row.tags_json,
                row.categories_json,
                row.trust_level,
                row.redaction_classification,
                chunk_count,
                row.status,
                row.visibility,
                version,
                row.created_at,
                row.updated_at,
                row.deleted_at
            )?;
            Ok(())
        })
    }

    fn update_knowledge_document_row(
        &mut self,
        row: AgentKnowledgeDocumentRow) -> KernelResult<()> {
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let chunk_count = i64::from(row.chunk_count);
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        self.with_pool(|pool| {
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_KNOWLEDGE_DOCUMENT,
                row.knowledge_base_id,
                row.knowledge_source_id,
                row.document_kind,
                row.title,
                row.content_ref,
                row.content_hash,
                row.summary,
                row.metadata_json,
                row.tags_json,
                row.categories_json,
                row.trust_level,
                row.redaction_classification,
                chunk_count,
                row.status,
                row.visibility,
                version,
                row.updated_at,
                row.deleted_at,
                tenant_id,
                row.knowledge_document_id,
                previous_version
            )?;
            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_KNOWLEDGE_DOCUMENT,
                    tenant_id,
                    row.knowledge_document_id
                )?
                .is_some();
                if exists {
                    return Err(KernelError::conflict(
                        "agent knowledge document version mismatch"));
                }
                return Err(KernelError::validation(
                    "agent knowledge document not found"));
            }
            Ok(())
        })
    }

    fn get_knowledge_document_row(
        &self,
        tenant_id: u64,
        knowledge_document_id: &str) -> Option<AgentKnowledgeDocumentRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_KNOWLEDGE_DOCUMENT,
                tenant_id,
                knowledge_document_id
            )?;
            row.map(pg_row_to_agent_knowledge_document_row).transpose()
        })
        .ok()
        .flatten()
    }

    fn list_knowledge_document_rows(
        &self,
        tenant_id: u64,
        knowledge_base_id: &str) -> Vec<AgentKnowledgeDocumentRow> {
        let tenant_id = match u64_to_i64(tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT_KNOWLEDGE_DOCUMENTS,
                tenant_id,
                knowledge_base_id
            )?;
            rows.into_iter()
                .map(pg_row_to_agent_knowledge_document_row)
                .collect()
        })
        .unwrap_or_default()
    }

    fn insert_knowledge_chunk_row(&mut self, row: AgentKnowledgeChunkRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let chunk_ordinal = i64::from(row.chunk_ordinal);
        let token_estimate = i64::from(row.token_estimate);
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_KNOWLEDGE_CHUNK,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.knowledge_chunk_id,
                row.knowledge_document_id,
                row.parent_chunk_id,
                chunk_ordinal,
                row.heading,
                row.content_ref,
                row.content_hash,
                token_estimate,
                row.summary,
                row.metadata_json,
                row.status,
                row.created_at
            )?;
            pg_execute!(
                pool,
                SQL_INCREMENT_AGENT_KNOWLEDGE_DOCUMENT_CHUNK_COUNT,
                row.created_at,
                tenant_id,
                row.knowledge_document_id
            )?;
            Ok(())
        })
    }

    fn get_knowledge_chunk_row(
        &self,
        tenant_id: u64,
        knowledge_chunk_id: &str) -> Option<AgentKnowledgeChunkRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                row.uuid,
                tenant_id,
                organization_id,
                owner_user_id,
                row.memory_profile_id,
                row.memory_store_id,
                row.code,
                row.display_name,
                row.description,
                row.write_policy_json,
                row.retrieval_policy_json,
                row.compaction_policy_json,
                row.retention_policy_json,
                row.privacy_policy_json,
                row.status,
                row.visibility,
                version,
                row.created_at,
                row.updated_at,
                row.deleted_at
            )?;
            Ok(())
        })
    }

    fn get_memory_profile_row(
        &self,
        tenant_id: u64,
        memory_profile_id: &str) -> Option<AgentMemoryProfileRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_MEMORY_PROFILE,
                tenant_id,
                memory_profile_id
            )?;
            row.map(pg_row_to_agent_memory_profile_row).transpose()
        })
        .ok()
        .flatten()
    }

    fn insert_memory_binding_row(&mut self, row: AgentMemoryBindingRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let version = u64_to_i64(row.version, "version")?;
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_MEMORY_BINDING,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.memory_binding_id,
                row.memory_profile_id,
                row.agent_id,
                row.deployment_id,
                row.scope_kind,
                row.scope_ref,
                row.active,
                row.default_binding,
                version,
                row.created_at,
                row.updated_at
            )?;
            Ok(())
        })
    }

    fn get_memory_binding_row(
        &self,
        tenant_id: u64,
        memory_binding_id: &str) -> Option<AgentMemoryBindingRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_MEMORY_BINDING,
                tenant_id,
                memory_binding_id
            )?;
            row.map(pg_row_to_agent_memory_binding_row).transpose()
        })
        .ok()
        .flatten()
    }

    fn insert_memory_namespace_row(&mut self, row: AgentMemoryNamespaceRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let version = u64_to_i64(row.version, "version")?;
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_MEMORY_NAMESPACE,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.memory_namespace_id,
                row.agent_id,
                row.user_ref,
                row.session_ref,
                row.thread_ref,
                row.namespace_kind,
                row.status,
                row.visibility,
                version,
                row.created_at,
                row.updated_at,
                row.deleted_at
            )?;
            Ok(())
        })
    }

    fn get_memory_namespace_row(
        &self,
        tenant_id: u64,
        memory_namespace_id: &str) -> Option<AgentMemoryNamespaceRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_MEMORY_NAMESPACE,
                tenant_id,
                memory_namespace_id
            )?;
            row.map(pg_row_to_agent_memory_namespace_row).transpose()
        })
        .ok()
        .flatten()
    }

    fn insert_memory_record_row(&mut self, row: AgentMemoryRecordRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let organization_id = u64_to_i64(row.organization_id, "organization_id")?;
        let source_count = i64::from(row.source_count);
        let use_count = u64_to_i64(row.use_count, "use_count")?;
        let version = u64_to_i64(row.version, "version")?;
        let salience_score = row.salience_score;
        let confidence_score = row.confidence_score;
        let freshness_score = row.freshness_score;
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_MEMORY_RECORD,
                id,
                row.uuid,
                tenant_id,
                organization_id,
                row.memory_id,
                row.memory_namespace_id,
                row.agent_id,
                row.memory_kind,
                row.content_format,
                row.content_json,
                row.summary,
                salience_score,
                confidence_score,
                freshness_score,
                row.sensitivity_level,
                source_count,
                row.effective_at,
                row.expires_at,
                row.last_used_at,
                use_count,
                row.status,
                version,
                row.created_at,
                row.updated_at,
                row.deleted_at,
                row.redacted_at
            )?;
            Ok(())
        })
    }

    fn update_memory_record_row(&mut self, row: AgentMemoryRecordRow) -> KernelResult<()> {
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let source_count = i64::from(row.source_count);
        let use_count = u64_to_i64(row.use_count, "use_count")?;
        let version = u64_to_i64(row.version, "version")?;
        let previous_version =
            u64_to_i64(expected_previous_version(row.version)?, "previous_version")?;
        let salience_score = row.salience_score;
        let confidence_score = row.confidence_score;
        let freshness_score = row.freshness_score;
        self.with_pool(|pool| {
            let updated_rows = pg_execute!(
                pool,
                SQL_UPDATE_AGENT_MEMORY_RECORD,
                row.content_format,
                row.content_json,
                row.summary,
                salience_score,
                confidence_score,
                freshness_score,
                row.sensitivity_level,
                source_count,
                row.effective_at,
                row.expires_at,
                row.last_used_at,
                use_count,
                row.status,
                version,
                row.updated_at,
                row.deleted_at,
                row.redacted_at,
                tenant_id,
                row.memory_id,
                previous_version
            )?;
            if updated_rows == 0 {
                let exists = pg_query_optional!(
                    pool,
                    SQL_SELECT_AGENT_MEMORY_RECORD,
                    tenant_id,
                    row.memory_id
                )?
                .is_some();
                if exists {
                    return Err(KernelError::conflict(
                        "agent memory record version mismatch"));
                }
                return Err(KernelError::validation("agent memory record not found"));
            }
            Ok(())
        })
    }

    fn get_memory_record_row(
        &self,
        tenant_id: u64,
        memory_id: &str) -> Option<AgentMemoryRecordRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
        self.with_pool(|pool| {
            let row =
                pg_query_optional!(pool, SQL_SELECT_AGENT_MEMORY_RECORD, tenant_id, memory_id)?;
            row.map(pg_row_to_agent_memory_record_row).transpose()
        })
        .ok()
        .flatten()
    }

    fn list_memory_record_rows(
        &self,
        tenant_id: u64,
        memory_namespace_id: &str) -> Vec<AgentMemoryRecordRow> {
        let tenant_id = match u64_to_i64(tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT_MEMORY_RECORDS,
                tenant_id,
                memory_namespace_id
            )?;
            rows.into_iter()
                .map(pg_row_to_agent_memory_record_row)
                .collect()
        })
        .unwrap_or_default()
    }

    fn insert_memory_source_row(&mut self, row: AgentMemorySourceRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_MEMORY_SOURCE,
                id,
                row.uuid,
                tenant_id,
                row.memory_source_id,
                row.memory_id,
                row.source_kind,
                row.source_ref,
                row.source_hash,
                row.evidence_json,
                row.captured_at,
                row.created_at
            )?;
            pg_execute!(
                pool,
                SQL_INCREMENT_AGENT_MEMORY_RECORD_SOURCE_COUNT,
                row.created_at,
                tenant_id,
                row.memory_id
            )?;
            Ok(())
        })
    }

    fn get_memory_source_row(
        &self,
        tenant_id: u64,
        memory_source_id: &str) -> Option<AgentMemorySourceRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_MEMORY_SOURCE,
                tenant_id,
                memory_source_id
            )?;
            row.map(pg_row_to_agent_memory_source_row).transpose()
        })
        .ok()
        .flatten()
    }

    fn list_memory_source_rows(
        &self,
        tenant_id: u64,
        memory_id: &str) -> Vec<AgentMemorySourceRow> {
        let tenant_id = match u64_to_i64(tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        self.with_pool(|pool| {
            let rows = pg_query!(pool, SQL_LIST_AGENT_MEMORY_SOURCES, tenant_id, memory_id)?;
            rows.into_iter()
                .map(pg_row_to_agent_memory_source_row)
                .collect()
        })
        .unwrap_or_default()
    }

    fn insert_memory_relation_row(&mut self, row: AgentMemoryRelationRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let weight = row.weight;
        let valid_from = optional_rfc3339_timestamp(&row.valid_from)?;
        let valid_until = optional_rfc3339_timestamp(&row.valid_until)?;
        let created_at = parse_rfc3339_timestamp(row.created_at.as_str())?;
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_INSERT_AGENT_MEMORY_RELATION,
                id,
                row.uuid,
                tenant_id,
                row.memory_relation_id,
                row.from_memory_id,
                row.to_memory_id,
                row.relation_kind,
                weight,
                valid_from,
                valid_until,
                created_at
            )?;
            Ok(())
        })
    }

    fn get_memory_relation_row(
        &self,
        tenant_id: u64,
        memory_relation_id: &str) -> Option<AgentMemoryRelationRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_MEMORY_RELATION,
                tenant_id,
                memory_relation_id
            )?;
            row.map(pg_row_to_agent_memory_relation_row).transpose()
        })
        .ok()
        .flatten()
    }

    fn list_memory_relation_rows(
        &self,
        tenant_id: u64,
        memory_id: &str) -> Vec<AgentMemoryRelationRow> {
        let tenant_id = match u64_to_i64(tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        self.with_pool(|pool| {
            let rows = pg_query!(pool, SQL_LIST_AGENT_MEMORY_RELATIONS, tenant_id, memory_id)?;
            rows.into_iter()
                .map(pg_row_to_agent_memory_relation_row)
                .collect()
        })
        .unwrap_or_default()
    }

    fn upsert_memory_retrieval_index_row(
        &mut self,
        row: AgentMemoryRetrievalIndexRow) -> KernelResult<()> {
        let id = u64_to_i64(row.id, "id")?;
        let tenant_id = u64_to_i64(row.tenant_id, "tenant_id")?;
        let vector_dimension = row.vector_dimension.map(i64::from);
        self.with_pool(|pool| {
            pg_execute!(
                pool,
                SQL_UPSERT_AGENT_MEMORY_RETRIEVAL_INDEX,
                id,
                row.uuid,
                tenant_id,
                row.memory_index_id,
                row.memory_id,
                row.index_kind,
                row.index_provider_id,
                row.external_ref,
                row.embedding_model_id,
                vector_dimension,
                row.content_hash,
                row.indexed_at,
                row.status
            )?;
            Ok(())
        })
    }

    fn get_memory_retrieval_index_row(
        &self,
        tenant_id: u64,
        memory_index_id: &str) -> Option<AgentMemoryRetrievalIndexRow> {
        let tenant_id = u64_to_i64(tenant_id, "tenant_id").ok()?;
        self.with_pool(|pool| {
            let row = pg_query_optional!(
                pool,
                SQL_SELECT_AGENT_MEMORY_RETRIEVAL_INDEX,
                tenant_id,
                memory_index_id
            )?;
            row.map(pg_row_to_agent_memory_retrieval_index_row)
                .transpose()
        })
        .ok()
        .flatten()
    }

    fn list_memory_retrieval_index_rows(
        &self,
        tenant_id: u64,
        memory_id: &str) -> Vec<AgentMemoryRetrievalIndexRow> {
        let tenant_id = match u64_to_i64(tenant_id, "tenant_id") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        self.with_pool(|pool| {
            let rows = pg_query!(
                pool,
                SQL_LIST_AGENT_MEMORY_RETRIEVAL_INDEXES,
                tenant_id,
                memory_id
            )?;
            rows.into_iter()
                .map(pg_row_to_agent_memory_retrieval_index_row)
                .collect()
        })
        .unwrap_or_default()
    }
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

fn build_agent_memory_store_uuid(tenant_id: u64, memory_store_id: &str) -> String {
    format!("agent_memory_store_{}_{}", tenant_id, memory_store_id)
}

fn build_agent_memory_profile_uuid(tenant_id: u64, memory_profile_id: &str) -> String {
    format!("agent_memory_profile_{}_{}", tenant_id, memory_profile_id)
}

fn build_agent_memory_binding_uuid(tenant_id: u64, memory_binding_id: &str) -> String {
    format!("agent_memory_binding_{}_{}", tenant_id, memory_binding_id)
}

fn build_agent_memory_namespace_uuid(tenant_id: u64, memory_namespace_id: &str) -> String {
    format!(
        "agent_memory_namespace_{}_{}",
        tenant_id, memory_namespace_id
    )
}

fn build_agent_memory_record_uuid(tenant_id: u64, memory_id: &str) -> String {
    format!("agent_memory_record_{}_{}", tenant_id, memory_id)
}

fn build_agent_memory_source_uuid(tenant_id: u64, memory_source_id: &str) -> String {
    format!("agent_memory_source_{}_{}", tenant_id, memory_source_id)
}

fn build_agent_memory_relation_uuid(tenant_id: u64, memory_relation_id: &str) -> String {
    format!("agent_memory_relation_{}_{}", tenant_id, memory_relation_id)
}

fn build_agent_memory_retrieval_index_uuid(tenant_id: u64, memory_index_id: &str) -> String {
    format!(
        "agent_memory_retrieval_index_{}_{}",
        tenant_id, memory_index_id
    )
}

fn build_agent_knowledge_base_uuid(tenant_id: u64, knowledge_base_id: &str) -> String {
    format!("agent_knowledge_base_{}_{}", tenant_id, knowledge_base_id)
}

fn build_agent_knowledge_source_uuid(tenant_id: u64, knowledge_source_id: &str) -> String {
    format!(
        "agent_knowledge_source_{}_{}",
        tenant_id, knowledge_source_id
    )
}

fn build_agent_knowledge_document_uuid(tenant_id: u64, knowledge_document_id: &str) -> String {
    format!(
        "agent_knowledge_document_{}_{}",
        tenant_id, knowledge_document_id
    )
}

fn build_agent_knowledge_chunk_uuid(tenant_id: u64, knowledge_chunk_id: &str) -> String {
    format!("agent_knowledge_chunk_{}_{}", tenant_id, knowledge_chunk_id)
}

fn build_agent_knowledge_index_uuid(tenant_id: u64, knowledge_index_id: &str) -> String {
    format!("agent_knowledge_index_{}_{}", tenant_id, knowledge_index_id)
}

fn build_agent_knowledge_binding_uuid(tenant_id: u64, knowledge_binding_id: &str) -> String {
    format!(
        "agent_knowledge_binding_{}_{}",
        tenant_id, knowledge_binding_id
    )
}

fn build_agent_knowledge_sync_job_uuid(tenant_id: u64, sync_job_id: &str) -> String {
    format!("agent_knowledge_sync_job_{}_{}", tenant_id, sync_job_id)
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

fn memory_index_kinds_to_json(values: &[]) -> KernelResult<String> {
    validate_memory_index_kinds(values)?;
    let serialized: Vec<&str> = values.iter().map(::as_str).collect();
    serde_json::to_string(&serialized)
        .map_err(|error| KernelError::validation(format!("invalid retrieval_modes json: {error}")))
}

fn memory_index_kinds_from_json(input: &str) -> KernelResult<Vec<>> {
    let values: Vec<String> = serde_json::from_str(input).map_err(|error| {
        KernelError::validation(format!("invalid retrieval_modes json: {error}"))
    })?;
    let mut parsed = Vec::with_capacity(values.len());
    for value in values {
        parsed.push(parse_memory_index_kind(value.as_str())?);
    }
    validate_memory_index_kinds(parsed.as_slice())?;
    Ok(parsed)
}

fn knowledge_index_kinds_to_json(values: &[]) -> KernelResult<String> {
    validate_knowledge_index_kinds(values)?;
    let serialized: Vec<&str> = values.iter().map(::as_str).collect();
    serde_json::to_string(&serialized)
        .map_err(|error| KernelError::validation(format!("invalid retrieval_modes json: {error}")))
}

fn knowledge_index_kinds_from_json(input: &str) -> KernelResult<Vec<>> {
    let values: Vec<String> = serde_json::from_str(input).map_err(|error| {
        KernelError::validation(format!("invalid retrieval_modes json: {error}"))
    })?;
    let mut parsed = Vec::with_capacity(values.len());
    for value in values {
        parsed.push(parse_knowledge_index_kind(value.as_str())?);
    }
    validate_knowledge_index_kinds(parsed.as_slice())?;
    Ok(parsed)
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

#[cfg(feature = "postgres-sync")]
        assert!(SQL_INSERT_AGENT_KNOWLEDGE_SOURCE.contains("$16"));
        assert!(
            SQL_INSERT_AGENT_KNOWLEDGE_DOCUMENT.contains("INSERT INTO a_agent_knowledge_document")
        );
        assert!(SQL_INSERT_AGENT_KNOWLEDGE_DOCUMENT.contains("$24"));
        assert!(SQL_UPDATE_AGENT_KNOWLEDGE_DOCUMENT
            .contains("WHERE tenant_id = $19 AND knowledge_document_id = $20 AND version = $21"));
        assert!(SQL_LIST_AGENT_KNOWLEDGE_DOCUMENTS.contains("status <> 4"));
        assert!(SQL_LIST_AGENT_KNOWLEDGE_DOCUMENTS.contains("deleted_at IS NULL"));
        assert!(SQL_INSERT_AGENT_KNOWLEDGE_CHUNK.contains("INSERT INTO a_agent_knowledge_chunk"));
        assert!(SQL_INSERT_AGENT_KNOWLEDGE_CHUNK.contains("$16"));
        assert!(SQL_INCREMENT_AGENT_KNOWLEDGE_DOCUMENT_CHUNK_COUNT.contains("chunk_count + 1"));
        assert!(SQL_UPSERT_AGENT_KNOWLEDGE_INDEX.contains("$15"));
        assert!(SQL_UPSERT_AGENT_KNOWLEDGE_INDEX
            .contains("ON CONFLICT (tenant_id, knowledge_index_id) DO UPDATE"));
        assert!(SQL_SELECT_AGENT_KNOWLEDGE_INDEX.contains("knowledge_index_id = $2"));
        assert!(SQL_SELECT_AGENT_KNOWLEDGE_INDEX.contains("LIMIT 1"));
        assert!(SQL_LIST_AGENT_KNOWLEDGE_INDEXES.contains("knowledge_document_id = $2"));
        assert!(SQL_LIST_AGENT_KNOWLEDGE_INDEXES.contains("status <> 4"));
        assert!(SQL_LIST_AGENT_KNOWLEDGE_INDEXES_BY_BASE.contains("knowledge_base_id = $2"));
        assert!(SQL_LIST_AGENT_KNOWLEDGE_INDEXES_BY_BASE.contains("status <> 4"));
        assert!(SQL_LIST_AGENT_KNOWLEDGE_INDEXES_BY_BASE
            .contains("ORDER BY indexed_at DESC, knowledge_index_id ASC"));
        assert!(
            SQL_INSERT_AGENT_KNOWLEDGE_BINDING.contains("INSERT INTO a_agent_knowledge_binding")
        );
        assert!(SQL_INSERT_AGENT_KNOWLEDGE_BINDING.contains("$15"));
        assert!(
            SQL_INSERT_AGENT_KNOWLEDGE_SYNC_JOB.contains("INSERT INTO a_agent_knowledge_sync_job")
        );
        assert!(SQL_INSERT_AGENT_KNOWLEDGE_SYNC_JOB.contains("$18"));
        assert!(SQL_UPDATE_AGENT_KNOWLEDGE_SYNC_JOB.contains("UPDATE a_agent_knowledge_sync_job"));
        assert!(SQL_UPDATE_AGENT_KNOWLEDGE_SYNC_JOB
            .contains("WHERE tenant_id = $7 AND sync_job_id = $8"));
        assert!(SQL_INSERT_AGENT_MEMORY_STORE.contains("INSERT INTO a_agent_memory_store"));
        assert!(SQL_INSERT_AGENT_MEMORY_STORE.contains("$20"));
        assert!(SQL_UPDATE_AGENT_MEMORY_STORE
            .contains("WHERE tenant_id = $16 AND memory_store_id = $17 AND version = $18"));
        assert!(SQL_INSERT_AGENT_MEMORY_PROFILE.contains("INSERT INTO a_agent_memory_profile"));
        assert!(SQL_INSERT_AGENT_MEMORY_PROFILE.contains("$21"));
        assert!(SQL_INSERT_AGENT_MEMORY_BINDING.contains("INSERT INTO a_agent_memory_binding"));
        assert!(SQL_INSERT_AGENT_MEMORY_BINDING.contains("$15"));
        assert!(SQL_INSERT_AGENT_MEMORY_NAMESPACE.contains("INSERT INTO a_agent_memory_namespace"));
        assert!(SQL_INSERT_AGENT_MEMORY_NAMESPACE.contains("$16"));
        assert!(SQL_INSERT_AGENT_MEMORY_RECORD.contains("INSERT INTO a_agent_memory_record"));
        assert!(SQL_INSERT_AGENT_MEMORY_RECORD.contains("$26"));
        assert!(SQL_UPDATE_AGENT_MEMORY_RECORD
            .contains("WHERE tenant_id = $18 AND memory_id = $19 AND version = $20"));
        assert!(SQL_LIST_AGENT_MEMORY_RECORDS.contains("status <> 4"));
        assert!(SQL_LIST_AGENT_MEMORY_RECORDS.contains("deleted_at IS NULL"));
        assert!(SQL_INSERT_AGENT_MEMORY_SOURCE.contains("INSERT INTO a_agent_memory_source"));
        assert!(SQL_INSERT_AGENT_MEMORY_SOURCE.contains("$11"));
        assert!(SQL_INCREMENT_AGENT_MEMORY_RECORD_SOURCE_COUNT.contains("source_count + 1"));
        assert!(SQL_INSERT_AGENT_MEMORY_RELATION.contains("INSERT INTO a_agent_memory_relation"));
        assert!(SQL_INSERT_AGENT_MEMORY_RELATION.contains("$11"));
        assert!(SQL_UPSERT_AGENT_MEMORY_RETRIEVAL_INDEX.contains("$13"));
        assert!(SQL_UPSERT_AGENT_MEMORY_RETRIEVAL_INDEX
            .contains("ON CONFLICT (tenant_id, memory_index_id) DO UPDATE"));

        for sql in [
            SQL_INSERT_AGENT,
            SQL_INSERT_AGENT_PROVIDER_BINDING,
            SQL_INSERT_AGENT_DEPLOYMENT,
            SQL_INSERT_AUDIT_EVENT,
            SQL_INSERT_AGENT_MCP_SERVER,
            SQL_INSERT_AGENT_KNOWLEDGE_BASE,
            SQL_INSERT_AGENT_KNOWLEDGE_SOURCE,
            SQL_INSERT_AGENT_KNOWLEDGE_DOCUMENT,
            SQL_INSERT_AGENT_KNOWLEDGE_CHUNK,
            SQL_UPSERT_AGENT_KNOWLEDGE_INDEX,
            SQL_INSERT_AGENT_KNOWLEDGE_BINDING,
            SQL_INSERT_AGENT_KNOWLEDGE_SYNC_JOB,
            SQL_INSERT_AGENT_MEMORY_STORE,
            SQL_INSERT_AGENT_MEMORY_PROFILE,
            SQL_INSERT_AGENT_MEMORY_BINDING,
            SQL_INSERT_AGENT_MEMORY_NAMESPACE,
            SQL_INSERT_AGENT_MEMORY_RECORD,
            SQL_INSERT_AGENT_MEMORY_SOURCE,
            SQL_INSERT_AGENT_MEMORY_RELATION,
            SQL_UPSERT_AGENT_MEMORY_RETRIEVAL_INDEX,
        ] {
            assert!(!sql.contains("nextval"));
            assert!(!sql.contains("RETURNING id"));
        }

        for required in [            "CREATE TABLE IF NOT EXISTS a_agent_mcp_server",
            "CREATE TABLE IF NOT EXISTS a_agent_knowledge_base",
            "CREATE TABLE IF NOT EXISTS a_agent_knowledge_source",
            "CREATE TABLE IF NOT EXISTS a_agent_knowledge_document",
            "CREATE TABLE IF NOT EXISTS a_agent_knowledge_chunk",
            "CREATE TABLE IF NOT EXISTS a_agent_knowledge_index",
            "idx_a_agent_knowledge_index_tenant_base_indexed",
            "CREATE TABLE IF NOT EXISTS a_agent_knowledge_binding",
            "CREATE TABLE IF NOT EXISTS a_agent_knowledge_sync_job",
            "CREATE TABLE IF NOT EXISTS a_agent_memory_store",
            "CREATE TABLE IF NOT EXISTS a_agent_memory_profile",
            "CREATE TABLE IF NOT EXISTS a_agent_memory_binding",
            "CREATE TABLE IF NOT EXISTS a_agent_memory_namespace",
            "CREATE TABLE IF NOT EXISTS a_agent_memory_record",
            "CREATE TABLE IF NOT EXISTS a_agent_memory_source",
            "CREATE TABLE IF NOT EXISTS a_agent_memory_relation",
            "CREATE TABLE IF NOT EXISTS a_agent_memory_retrieval_index",
            "CREATE TABLE IF NOT EXISTS a_agent_memory_access_event",
            "CREATE TABLE IF NOT EXISTS a_agent_memory_compaction_job",
            "ck_ai_agent_implementation_provider_id_standard",
            "implementation_provider_id ~ '^provider\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'",
            "implementation_type VARCHAR(64) NOT NULL DEFAULT 'sdkwork-native'",
            "ck_ai_agent_implementation_type",
            "'sdkwork-native'",
            "'openai-agents'",
            "'semantic-kernel'",
            "ck_ai_agent_runtime_binding_binding_id_standard",
            "binding_id ~ '^binding\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'",
            "ck_ai_agent_runtime_binding_provider_id_standard",
            "provider_id ~ '^provider\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'",
            "ck_ai_agent_runtime_binding_configuration_profile_id_standard",
            "configuration_profile_id ~ '^profile\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'",
            "ck_ai_agent_runtime_binding_capabilities_standard",
            "sdkwork_intelligence_agents_service_capabilities_json_is_standard(capabilities_json)",
            "ck_a_agent_deployment_deployment_id_standard",
            "deployment_id ~ '^deployment\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'",
            "ck_a_agent_deployment_binding_id_standard",
            "binding_id ~ '^binding\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'",
            "ck_a_agent_deployment_provider_id_snapshot_standard",
            "provider_id_snapshot ~ '^provider\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'",
            "ck_a_agent_deployment_configuration_profile_id_snapshot_standard",
            "configuration_profile_id_snapshot ~ '^profile\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'",
            "ck_a_agent_deployment_capabilities_snapshot_standard",
            "sdkwork_intelligence_agents_service_capabilities_json_is_standard(capabilities_snapshot_json)",
            "ck_a_agent_mcp_server_id_standard",
            "mcp_server_id ~ '^mcp\\.server\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'",
            "ck_a_agent_mcp_server_transport_refs",
            "ck_a_agent_mcp_server_endpoint_ref_standard",
            "ck_a_agent_mcp_server_auth_kind",
            "sdkwork_intelligence_agents_service_capabilities_json_is_standard(capability_ids_json)",
            "sdkwork_intelligence_agents_service_knowledge_modes_json_is_standard(retrieval_modes_json)",
            "ck_a_agent_knowledge_base_id_standard",
            "knowledge_base_id ~ '^knowledge\\.base\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'",
            "ck_a_agent_knowledge_base_kind",
            "ck_a_agent_knowledge_source_kind",
            "ck_a_agent_knowledge_document_kind",
            "ck_a_agent_knowledge_document_trust_level",
            "ck_a_agent_knowledge_chunk_ordinal",
            "ck_a_agent_knowledge_index_vector_contract",
            "ck_a_agent_knowledge_index_chunk_requires_document",
            "WHERE knowledge_document_id IS NOT NULL AND status <> 4",
            "ck_a_agent_knowledge_binding_scope_kind",
            "ck_a_agent_knowledge_sync_job_kind",
            "ck_a_agent_knowledge_sync_job_status",
            "sdkwork_intelligence_agents_service_memory_modes_json_is_standard(retrieval_modes_json)",
            "ck_a_agent_memory_store_id_standard",
            "memory_store_id ~ '^memory\\.store\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'",
            "ck_a_agent_memory_store_kind",
            "ck_a_agent_memory_profile_id_standard",
            "memory_profile_id ~ '^memory\\.profile\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'",
            "ck_a_agent_memory_binding_scope_kind",
            "ck_a_agent_memory_namespace_kind",
            "ck_a_agent_memory_record_kind",
            "ck_a_agent_memory_record_scores",
            "ck_a_agent_memory_record_sensitivity",
            "idx_a_agent_memory_record_tenant_namespace_updated",
            "WHERE status <> 4 AND deleted_at IS NULL",
            "ck_a_agent_memory_source_kind",
            "'knowledge-ref'",
            "ck_a_agent_memory_relation_distinct_endpoints",
            "ck_a_agent_memory_retrieval_index_kind",
            "'keyword', 'sparse', 'vector', 'graph', 'wiki', 'rule', 'hybrid'",
            "ck_a_agent_memory_retrieval_index_vector_contract",
            "embedding_model_id IS NOT NULL AND vector_dimension IS NOT NULL",
            "ck_a_agent_memory_access_event_kind",
            "ck_a_agent_memory_compaction_job_kind",
                        "mcp_created",
            "prompt_created",
            "memory_store_created",
            "memory_record_created",
            "memory_retrieval_index_upserted",
            "jsonb_typeof(capability_values.value) = 'string'",
            "capability_values.value #>> '{}'",
            "char_length(capability_values.value #>> '{}') <= 128",
            "~ '^[a-z0-9_-]+(\\.[a-z0-9_-]+)+$'",
            "COUNT(DISTINCT capability_values.value #>> '{}')",
        ] {
            assert!(
                postgres_schema.contains(required),
                "postgres schema must contain {required}"
            );
        }
        for retrieval_mode in [
            "'exact'",
            "'keyword'",
            "'full_text'",
            "'structured'",
            "'graph'",
            "'wiki'",
            "'rule'",
            "'vector'",
            "'hybrid'",
            "'llm_rerank'",
            "'external'",
        ] {
            assert!(
                postgres_schema.contains(retrieval_mode),
                "postgres schema must contain knowledge retrieval mode {retrieval_mode}"
            );
        }
    }

    #[test]
    fn expected_previous_version_maps_incremented_version() {
        let previous = expected_previous_version(3).expect("version should map");
        assert_eq!(previous, 2);
    }

    #[test]
    fn expected_previous_version_rejects_zero() {
        let error = expected_previous_version(0)
            .expect_err("version=0 cannot be used for update precondition");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains(">= 1"));
            }
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn row_roundtrip_preserves_record_contract() {
        let record = AgentBusinessRecord {
            id: 1,
            agent_id: "agent.alpha".to_string(),
            tenant_id: 100_001,
            organization_id: 0,
            owner_user_id: 700,
            code: "alpha".to_string(),
            display_name: "Alpha".to_string(),
            description: Some("desc".to_string()),
            manifest: sample_manifest("agent.alpha"),
            default_code_task_intent: Some(CodeTaskIntent::new("Refactor runtime")),
            implementation_provider_id: Some("provider.model.rig-rust".to_string()),
            implementation_kind: Some(AgentImplementationKind::TypedLocalProvider),
            implementation_type: AgentImplementationType::LangGraph,
            status: AgentBusinessStatus::Active,
            visibility: AgentVisibility::Tenant,
            tags: vec!["starter".to_string()],
            version: 3,
            created_at: "2026-06-01T00:00:00Z".to_string(),
            updated_at: "2026-06-01T01:00:00Z".to_string(),
            deleted_at: None,
        };

        let row = AgentBusinessRow::from_record(&record).expect("row mapping should succeed");
        assert_eq!(row.implementation_type, "langgraph");
        let rebuilt = row.into_record().expect("record mapping should succeed");

        assert_eq!(rebuilt, record);
    }

    #[test]
    fn agent_business_row_roundtrip_preserves_implementation_type() {
        let mut row = sample_agent_business_row();
        row.implementation_type = "openai-agents".to_string();

        let rebuilt = row.into_record().expect("record mapping should succeed");

        assert_eq!(
            rebuilt.implementation_type,
            AgentImplementationType::OpenAiAgents
        );
    }

    #[test]
    fn agent_business_row_rejects_invalid_implementation_type_from_storage() {
        let mut row = sample_agent_business_row();
        row.implementation_type = "unsupported-framework".to_string();

        let error = row
            .into_record()
            .expect_err("invalid implementation type should fail");

        assert_validation_contains(error, "implementationType");
    }

    #[test]
    fn agent_business_row_rejects_non_standard_implementation_provider_id_from_storage() {
        let mut row = sample_agent_business_row();
        row.implementation_provider_id = Some("model.rig-rust".to_string());

        let error = row
            .into_record()
            .expect_err("implementation provider id without provider prefix should fail");

        assert_validation_contains(error, "implementationProviderId");
    }

    #[test]
    fn provider_binding_row_roundtrip_preserves_standard_snapshots() {
        let record = AgentProviderBindingRecord {
            id: 31,
            tenant_id: 100_001,
            agent_id: "agent.alpha".to_string(),
            binding_id: "binding.rig.default".to_string(),
            provider_id: "provider.model.rig-rust".to_string(),
            implementation_kind: AgentImplementationKind::TypedLocalProvider,
            configuration_profile_id: "profile.rig.local".to_string(),
            capabilities: vec!["model.chat".to_string(), "tool.invoke".to_string()],
            active: true,
            version: 2,
            created_at: "2026-06-01T00:00:00Z".to_string(),
            updated_at: "2026-06-01T02:00:00Z".to_string(),
        };

        let row = AgentProviderBindingRow::from_record(&record)
            .expect("provider binding row mapping should succeed");

        assert_eq!(
            row.uuid,
            "agent_provider_binding_100001_agent.alpha_binding.rig.default"
        );
        assert_eq!(row.implementation_kind, "typed-local-provider");
        assert!(row.active);
        assert!(row.capabilities_json.contains("model.chat"));

        let rebuilt = row.into_record().expect("record mapping should succeed");

        assert_eq!(rebuilt, record);
    }

    #[test]
    fn deployment_row_roundtrip_preserves_provider_binding_snapshot() {
        let record = AgentDeploymentRecord {
            id: 32,
            tenant_id: 100_001,
            agent_id: "agent.alpha".to_string(),
            deployment_id: "deployment.rig.local.001".to_string(),
            binding_id: "binding.rig.default".to_string(),
            provider_id_snapshot: "provider.model.rig-rust".to_string(),
            implementation_kind_snapshot: AgentImplementationKind::TypedLocalProvider,
            configuration_profile_id_snapshot: "profile.rig.local".to_string(),
            capabilities_snapshot: vec!["model.chat".to_string(), "planning.create".to_string()],
            status: AgentDeploymentStatus::Created,
            version: 1,
            created_at: "2026-06-01T03:00:00Z".to_string(),
            updated_at: "2026-06-01T03:00:00Z".to_string(),
        };

        let row = AgentDeploymentRow::from_record(&record)
            .expect("deployment row mapping should succeed");

        assert_eq!(
            row.uuid,
            "agent_deployment_100001_agent.alpha_deployment.rig.local.001"
        );
        assert_eq!(row.status, 0);
        assert_eq!(row.implementation_kind_snapshot, "typed-local-provider");
        assert!(row.capabilities_snapshot_json.contains("planning.create"));

        let rebuilt = row.into_record().expect("record mapping should succeed");

        assert_eq!(rebuilt, record);
    }

    #[test]
    fn mcp_server_row_roundtrip_preserves_protocol_marketplace_contract() {
        let record = sample_mcp_server_record();
        let row =
            AgentMcpServerRow::from_record(&record).expect("mcp server row mapping should succeed");

        assert_eq!(row.uuid, "agent_mcp_server_100001_mcp.server.filesystem");
        assert_eq!(row.transport_kind, "http");
        assert_eq!(row.auth_kind, "oauth2");
        assert_eq!(row.endpoint_ref.as_deref(), Some("endpoint.mcp.filesystem"));
        assert!(row.capabilities_json.contains("tools"));

        let rebuilt = row
            .into_record()
            .expect("mcp server record mapping should succeed");

        assert_eq!(rebuilt, record);
    }

    #[test]
    fn memory_rows_roundtrip_preserve_standard_memory_contracts() {
        let store = sample_memory_store_record();
        let store_row =
            AgentMemoryStoreRow::from_record(&store).expect("memory store row should build");
        assert_eq!(store_row.uuid, "agent_memory_store_100001_memory.store.primary");
        assert!(store_row.retrieval_modes_json.contains("wiki"));
        assert_eq!(
            store_row
                .into_record()
                .expect("memory store should rebuild"),
            store
        );

        let profile = sample_memory_profile_record();
        let profile_row =
            AgentMemoryProfileRow::from_record(&profile).expect("memory profile row should build");
        assert_eq!(
            profile_row.uuid,
            "agent_memory_profile_100001_memory.profile.default"
        );
        assert_eq!(
            profile_row
                .into_record()
                .expect("memory profile should rebuild"),
            profile
        );

        let binding = sample_memory_binding_record();
        let binding_row =
            AgentMemoryBindingRow::from_record(&binding).expect("memory binding row should build");
        assert_eq!(
            binding_row.uuid,
            "agent_memory_binding_100001_memory.binding.agent.default"
        );
        assert_eq!(
            binding_row
                .into_record()
                .expect("memory binding should rebuild"),
            binding
        );

        let namespace = sample_memory_namespace_record();
        let namespace_row = AgentMemoryNamespaceRow::from_record(&namespace)
            .expect("memory namespace row should build");
        assert_eq!(
            namespace_row.uuid,
            "agent_memory_namespace_100001_memory.namespace.agent.alpha"
        );
        assert_eq!(
            namespace_row
                .into_record()
                .expect("memory namespace should rebuild"),
            namespace
        );

        let memory = sample_memory_record();
        let memory_row =
            AgentMemoryRecordRow::from_record(&memory).expect("memory record row should build");
        assert_eq!(
            memory_row.uuid,
            "agent_memory_record_100001_memory.record.fact.prefix"
        );
        assert_eq!(
            memory_row
                .into_record()
                .expect("memory record should rebuild"),
            memory
        );

        let source = sample_memory_source_record();
        let source_row =
            AgentMemorySourceRow::from_record(&source).expect("memory source row should build");
        assert_eq!(
            source_row.uuid,
            "agent_memory_source_100001_memory.source.fact.prefix.doc"
        );
        assert_eq!(
            source_row
                .into_record()
                .expect("memory source should rebuild"),
            source
        );

        let relation = sample_memory_relation_record();
        let relation_row = AgentMemoryRelationRow::from_record(&relation)
            .expect("memory relation row should build");
        assert_eq!(
            relation_row.uuid,
            "agent_memory_relation_100001_memory.relation.prefix.supports.marketplace"
        );
        assert_eq!(
            relation_row
                .into_record()
                .expect("memory relation should rebuild"),
            relation
        );

        let retrieval_index = sample_memory_retrieval_index_record();
        let retrieval_index_row = AgentMemoryRetrievalIndexRow::from_record(&retrieval_index)
            .expect("memory retrieval index row should build");
        assert_eq!(
            retrieval_index_row.uuid,
            "agent_memory_retrieval_index_100001_memory.index.fact.prefix.wiki"
        );
        assert_eq!(retrieval_index_row.vector_dimension, None);
        assert_eq!(
            retrieval_index_row
                .into_record()
                .expect("memory retrieval index should rebuild"),
            retrieval_index
        );
    }

    #[test]
    fn memory_rows_reject_non_standard_storage_values() {
        let mut store =
            AgentMemoryStoreRow::from_record(&sample_memory_store_record()).expect("row builds");
        store.memory_store_id = "memory.bad".to_string();
        let error = store
            .into_record()
            .expect_err("invalid memory store id should fail");
        assert_validation_contains(error, "memoryStoreId");

        let mut store =
            AgentMemoryStoreRow::from_record(&sample_memory_store_record()).expect("row builds");
        store.retrieval_modes_json = r#"["wiki","wiki"]"#.to_string();
        let error = store
            .into_record()
            .expect_err("duplicate retrieval modes should fail");
        assert_validation_contains(error, "retrievalModes");

        let mut profile = AgentMemoryProfileRow::from_record(&sample_memory_profile_record())
            .expect("row builds");
        profile.privacy_policy_json = r#"{"secret":"sk-live"}"#.to_string();
        let error = profile
            .into_record()
            .expect_err("secret material in privacy policy should fail");
        assert_validation_contains(error, "privacyPolicyJson");

        let mut memory =
            AgentMemoryRecordRow::from_record(&sample_memory_record()).expect("row builds");
        memory.content_json = r#"{"password":"secret=plain"}"#.to_string();
        let error = memory
            .into_record()
            .expect_err("secret material in memory content should fail");
        assert_validation_contains(error, "contentJson");

        let mut relation = AgentMemoryRelationRow::from_record(&sample_memory_relation_record())
            .expect("row builds");
        relation.to_memory_id = relation.from_memory_id.clone();
        let error = relation
            .into_record()
            .expect_err("self relation should fail");
        assert_validation_contains(error, "endpoints");

        let retrieval_index =
            AgentMemoryRetrievalIndexRow::from_record(&{
                index_kind: ::Vector,
                embedding_model_id: None,
                vector_dimension: None,
                ..sample_memory_retrieval_index_record()
            })
            .expect_err("vector index without embedding metadata should fail");
        assert_validation_contains(retrieval_index, "vector memory index");

        let agent_scope_mismatch = AgentMemoryBindingRow::from_record(&{
            agent_id: Some("agent.alpha".to_string()),
            scope_kind: ::Agent,
            scope_ref: "agent.beta".to_string(),
            ..sample_memory_binding_record()
        })
        .expect_err("agent-scoped memory binding row should require matching scope ref");
        assert_validation_contains(agent_scope_mismatch, "scopeRef");

        let deployment_scope_mismatch =
            AgentMemoryBindingRow::from_record(&{
                agent_id: Some("agent.alpha".to_string()),
                deployment_id: Some("deployment.alpha.local".to_string()),
                scope_kind: ::Deployment,
                scope_ref: "deployment.beta.local".to_string(),
                ..sample_memory_binding_record()
            })
            .expect_err("deployment-scoped memory binding row should require matching scope ref");
        assert_validation_contains(deployment_scope_mismatch, "scopeRef");
    }

    #[test]
    fn knowledge_rows_roundtrip_preserve_standard_rag_contracts() {
        let base = sample_knowledge_base_record();
        let base_row =
            AgentKnowledgeBaseRow::from_record(&base).expect("knowledge base row should build");
        assert_eq!(
            base_row.uuid,
            "agent_knowledge_base_100001_knowledge.base.kernel"
        );
        assert!(base_row.retrieval_modes_json.contains("wiki"));
        assert_eq!(
            base_row
                .into_record()
                .expect("knowledge base should rebuild"),
            base
        );

        let source = sample_knowledge_source_record();
        let source_row = AgentKnowledgeSourceRow::from_record(&source)
            .expect("knowledge source row should build");
        assert_eq!(
            source_row.uuid,
            "agent_knowledge_source_100001_knowledge.source.kernel.wiki"
        );
        assert_eq!(
            source_row
                .into_record()
                .expect("knowledge source should rebuild"),
            source
        );

        let document = sample_knowledge_document_record();
        let document_row = AgentKnowledgeDocumentRow::from_record(&document)
            .expect("knowledge document row should build");
        assert_eq!(
            document_row.uuid,
            "agent_knowledge_document_100001_knowledge.document.kernel.spi"
        );
        assert!(document_row.tags_json.contains("rag"));
        assert_eq!(
            document_row
                .into_record()
                .expect("knowledge document should rebuild"),
            document
        );

        let chunk = sample_knowledge_chunk_record();
        let chunk_row =
            AgentKnowledgeChunkRow::from_record(&chunk).expect("knowledge chunk row should build");
        assert_eq!(
            chunk_row.uuid,
            "agent_knowledge_chunk_100001_knowledge.chunk.kernel.spi.intro"
        );
        assert_eq!(
            chunk_row
                .into_record()
                .expect("knowledge chunk should rebuild"),
            chunk
        );

        let index = sample_knowledge_index_record();
        let index_row =
            AgentKnowledgeIndexRow::from_record(&index).expect("knowledge index row should build");
        assert_eq!(
            index_row.uuid,
            "agent_knowledge_index_100001_knowledge.index.kernel.spi.wiki"
        );
        assert_eq!(index_row.vector_dimension, None);
        assert_eq!(
            index_row
                .into_record()
                .expect("knowledge index should rebuild"),
            index
        );

        let binding = sample_knowledge_binding_record();
        let binding_row = AgentKnowledgeBindingRow::from_record(&binding)
            .expect("knowledge binding row should build");
        assert_eq!(
            binding_row.uuid,
            "agent_knowledge_binding_100001_knowledge.binding.agent.default"
        );
        assert_eq!(
            binding_row
                .into_record()
                .expect("knowledge binding should rebuild"),
            binding
        );

        let sync_job = sample_knowledge_sync_job_record();
        let sync_job_row = AgentKnowledgeSyncJobRow::from_record(&sync_job)
            .expect("knowledge sync job row should build");
        assert_eq!(
            sync_job_row.uuid,
            "agent_knowledge_sync_job_100001_knowledge.sync.kernel.reindex.1"
        );
        assert_eq!(
            sync_job_row
                .into_record()
                .expect("knowledge sync job should rebuild"),
            sync_job
        );
    }

    #[test]
    fn knowledge_rows_reject_non_standard_storage_values() {
        let mut base = AgentKnowledgeBaseRow::from_record(&sample_knowledge_base_record())
            .expect("row builds");
        base.knowledge_base_id = "knowledge.bad".to_string();
        let error = base
            .into_record()
            .expect_err("invalid knowledge base id should fail");
        assert_validation_contains(error, "knowledgeBaseId");

        let mut base = AgentKnowledgeBaseRow::from_record(&sample_knowledge_base_record())
            .expect("row builds");
        base.retrieval_modes_json = r#"["wiki","wiki"]"#.to_string();
        let error = base
            .into_record()
            .expect_err("duplicate knowledge retrieval modes should fail");
        assert_validation_contains(error, "retrievalModes");

        let mut source = AgentKnowledgeSourceRow::from_record(&sample_knowledge_source_record())
            .expect("row builds");
        source.source_ref = "https://example.test?api_key=plain".to_string();
        let error = source
            .into_record()
            .expect_err("secret material in source ref should fail");
        assert_validation_contains(error, "sourceRef");

        let mut source = AgentKnowledgeSourceRow::from_record(&sample_knowledge_source_record())
            .expect("row builds");
        source.source_hash = "h".repeat(129);
        let error = source
            .into_record()
            .expect_err("oversized source hash should fail before domain rebuild");
        assert_validation_contains(error, "sourceHash");

        let mut document =
            AgentKnowledgeDocumentRow::from_record(&sample_knowledge_document_record())
                .expect("row builds");
        document.trust_level = 9;
        let error = document
            .into_record()
            .expect_err("invalid document trust level should fail");
        assert_validation_contains(error, "trustLevel");

        let mut document =
            AgentKnowledgeDocumentRow::from_record(&sample_knowledge_document_record())
                .expect("row builds");
        document.content_hash = "h".repeat(129);
        let error = document
            .into_record()
            .expect_err("oversized document content hash should fail before domain rebuild");
        assert_validation_contains(error, "contentHash");

        let oversized_chunk_hash =
            AgentKnowledgeChunkRow::from_record(&{
                content_hash: "h".repeat(129),
                ..sample_knowledge_chunk_record()
            })
            .expect_err("oversized chunk content hash should fail before row build");
        assert_validation_contains(oversized_chunk_hash, "contentHash");

        let vector_index = AgentKnowledgeIndexRow::from_record(&{
            index_kind: ::Vector,
            embedding_model_id: None,
            vector_dimension: None,
            ..sample_knowledge_index_record()
        })
        .expect_err("vector knowledge index without embedding metadata should fail");
        assert_validation_contains(vector_index, "vector knowledge index");

        let chunk_without_document =
            AgentKnowledgeIndexRow::from_record(&{
                knowledge_document_id: None,
                knowledge_chunk_id: Some("knowledge.chunk.kernel.spi.intro".to_string()),
                ..sample_knowledge_index_record()
            })
            .expect_err("chunk-scoped knowledge index without document should fail");
        assert_validation_contains(chunk_without_document, "knowledgeDocumentId");

        let oversized_index_hash =
            AgentKnowledgeIndexRow::from_record(&{
                content_hash: "h".repeat(129),
                ..sample_knowledge_index_record()
            })
            .expect_err("oversized index content hash should fail before row build");
        assert_validation_contains(oversized_index_hash, "contentHash");

        let oversized_scope_ref =
            AgentKnowledgeBindingRow::from_record(&{
                scope_ref: "s".repeat(129),
                ..sample_knowledge_binding_record()
            })
            .expect_err("oversized knowledge binding scope ref should fail before row build");
        assert_validation_contains(oversized_scope_ref, "scopeRef");

        let agent_scope_mismatch =
            AgentKnowledgeBindingRow::from_record(&{
                agent_id: Some("agent.alpha".to_string()),
                scope_kind: ::Agent,
                scope_ref: "agent.beta".to_string(),
                ..sample_knowledge_binding_record()
            })
            .expect_err("agent-scoped knowledge binding row should require matching scope ref");
        assert_validation_contains(agent_scope_mismatch, "scopeRef");

        let deployment_scope_mismatch =
            AgentKnowledgeBindingRow::from_record(&{
                agent_id: Some("agent.alpha".to_string()),
                deployment_id: Some("deployment.alpha.local".to_string()),
                scope_kind: ::Deployment,
                scope_ref: "deployment.beta.local".to_string(),
                ..sample_knowledge_binding_record()
            })
            .expect_err(
                "deployment-scoped knowledge binding row should require matching scope ref");
        assert_validation_contains(deployment_scope_mismatch, "scopeRef");
    }

    #[test]
    fn marketplace_rows_reject_non_standard_ids_from_storage() {
let mut mcp = AgentMcpServerRow::from_record(&sample_mcp_server_record())
            .expect("mcp row should build");
        mcp.endpoint_ref = Some("https://example.test".to_string());
        let error = mcp
            .into_record()
            .expect_err("invalid endpoint ref should fail");
        assert_validation_contains(error, "endpointRef");
    }

    #[test]
    fn provider_binding_row_rejects_non_standard_ids_from_storage() {
        let mut row = sample_provider_binding_row();
        row.binding_id = "rig.default".to_string();
        let error = row
            .into_record()
            .expect_err("binding id without binding prefix should fail");
        assert_validation_contains(error, "bindingId");

        let mut row = sample_provider_binding_row();
        row.provider_id = "model.rig-rust".to_string();
        let error = row
            .into_record()
            .expect_err("provider id without provider prefix should fail");
        assert_validation_contains(error, "providerId");

        let mut row = sample_provider_binding_row();
        row.configuration_profile_id = "config.rig.local".to_string();
        let error = row
            .into_record()
            .expect_err("configuration profile id without profile prefix should fail");
        assert_validation_contains(error, "configurationProfileId");
    }

    #[test]
    fn provider_binding_row_rejects_non_standard_capabilities_from_storage() {
        let mut row = sample_provider_binding_row();
        row.capabilities_json = r#"["model.chat","model.chat"]"#.to_string();
        let error = row
            .into_record()
            .expect_err("duplicate capability ids should fail");
        assert_validation_contains(error, "capabilities");

        let mut row = sample_provider_binding_row();
        row.capabilities_json = r#"["Model.Chat"]"#.to_string();
        let error = row
            .into_record()
            .expect_err("uppercase capability id should fail");
        assert_validation_contains(error, "capabilities");

        let mut row = sample_provider_binding_row();
        row.capabilities_json = r#"["chat"]"#.to_string();
        let error = row
            .into_record()
            .expect_err("unnamespaced capability id should fail");
        assert_validation_contains(error, "capabilities");
    }

    #[test]
    fn deployment_row_rejects_non_standard_snapshots_from_storage() {
        let mut row = sample_deployment_row();
        row.deployment_id = "rig.local.001".to_string();
        let error = row
            .into_record()
            .expect_err("deployment id without deployment prefix should fail");
        assert_validation_contains(error, "deploymentId");

        let mut row = sample_deployment_row();
        row.binding_id = "rig.default".to_string();
        let error = row
            .into_record()
            .expect_err("binding id without binding prefix should fail");
        assert_validation_contains(error, "bindingId");

        let mut row = sample_deployment_row();
        row.provider_id_snapshot = "model.rig-rust".to_string();
        let error = row
            .into_record()
            .expect_err("provider snapshot without provider prefix should fail");
        assert_validation_contains(error, "providerId");

        let mut row = sample_deployment_row();
        row.configuration_profile_id_snapshot = "config.rig.local".to_string();
        let error = row
            .into_record()
            .expect_err("profile snapshot without profile prefix should fail");
        assert_validation_contains(error, "configurationProfileId");

        let mut row = sample_deployment_row();
        row.capabilities_snapshot_json = r#"["planning.create","planning.create"]"#.to_string();
        let error = row
            .into_record()
            .expect_err("duplicate capability snapshot ids should fail");
        assert_validation_contains(error, "capabilitiesSnapshot");
    }

    #[test]
    fn invalid_deployment_status_code_is_rejected() {
        let row = AgentDeploymentRow {
            id: 1,
            uuid: "deployment.invalid".to_string(),
            tenant_id: 100_001,
            agent_id: "agent.alpha".to_string(),
            deployment_id: "deployment.invalid".to_string(),
            binding_id: "binding.rig.default".to_string(),
            provider_id_snapshot: "provider.model.rig-rust".to_string(),
            implementation_kind_snapshot: "typed-local-provider".to_string(),
            configuration_profile_id_snapshot: "profile.rig.local".to_string(),
            capabilities_snapshot_json: "[]".to_string(),
            status: 99,
            version: 1,
            created_at: "2026-06-01T03:00:00Z".to_string(),
            updated_at: "2026-06-01T03:00:00Z".to_string(),
        };

        let error = row
            .into_record()
            .expect_err("invalid deployment status should fail");

        match error {
            KernelError::Validation { message } => assert!(message.contains("deployment status")),
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn invalid_status_code_is_rejected() {
        let row = AgentBusinessRow {
            id: 1,
            uuid: "uuid".to_string(),
            tenant_id: 100_001,
            organization_id: 0,
            owner_user_id: 1,
            agent_id: "agent.alpha".to_string(),
            code: "alpha".to_string(),
            display_name: "Alpha".to_string(),
            description: None,
            manifest_json: manifest_to_json(&sample_manifest("agent.alpha"))
                .expect("manifest json should be valid"),
            default_code_task_intent_json: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: "sdkwork-native".to_string(),
            status: 9,
            visibility: 0,
            tags_json: "[]".to_string(),
            created_at: "2026-06-01T00:00:00Z".to_string(),
            updated_at: "2026-06-01T00:00:00Z".to_string(),
            deleted_at: None,
            version: 0,
        };

        let error = row
            .into_record()
            .expect_err("invalid db status should fail");
        match error {
            KernelError::Validation { message } => assert!(message.contains("status")),
            _ => panic!("expected validation error"),
        }
    }
}#[derive(Debug, Clone, PartialEq, Eq)]
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


