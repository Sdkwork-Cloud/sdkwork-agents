use serde::{Deserialize, Serialize};

use crate::application::{
    AgentMcpServerCreateCommand, AgentMcpServerUpdateCommand, AgentsService, GetAgentMarketplaceItemCommand,
};
use crate::domain::{
    AgentMcpAuthKind, AgentMcpServerRecord, AgentMcpTransportKind, AgentVisibility,
};
use crate::ports::AgentMarketplaceListQuery;
use sdkwork_agent_kernel::{KernelError, PolicySubject};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerRecordDto {
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
    pub capability_ids: Vec<String>,
    pub tool_count: u32,
    pub resource_count: u32,
    pub prompt_count: u32,
    pub capabilities_json: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub security_profile_id: Option<String>,
    pub status: String,
    pub visibility: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerListResponseDto {
    pub items: Vec<McpServerRecordDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerResponseDto {
    pub data: McpServerRecordDto,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMcpServerBody {
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
    pub capability_ids: Vec<String>,
    pub tool_count: u32,
    pub resource_count: u32,
    pub prompt_count: u32,
    pub capabilities_json: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub security_profile_id: Option<String>,
    pub visibility: String,
    pub requested_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMcpServerBody {
    pub expected_version: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub protocol_version: Option<String>,
    pub transport_kind: Option<String>,
    pub endpoint_ref: Option<Option<String>>,
    pub command_ref: Option<Option<String>>,
    pub auth_kind: Option<String>,
    pub auth_profile_id: Option<Option<String>>,
    pub capability_ids: Option<Vec<String>>,
    pub tool_count: Option<u32>,
    pub resource_count: Option<u32>,
    pub prompt_count: Option<u32>,
    pub capabilities_json: Option<String>,
    pub categories: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub security_profile_id: Option<Option<String>>,
    pub visibility: Option<String>,
    pub requested_at: String,
}

pub fn to_mcp_server_dto(record: &AgentMcpServerRecord) -> McpServerRecordDto {
    McpServerRecordDto {
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
        capability_ids: record.capability_ids.clone(),
        tool_count: record.tool_count,
        resource_count: record.resource_count,
        prompt_count: record.prompt_count,
        capabilities_json: record.capabilities_json.clone(),
        categories: record.categories.clone(),
        tags: record.tags.clone(),
        security_profile_id: record.security_profile_id.clone(),
        status: record.status.as_str().to_string(),
        visibility: record.visibility.as_str().to_string(),
        version: record.version.to_string(),
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
    }
}

pub fn parse_transport_kind(value: &str) -> Result<AgentMcpTransportKind, KernelError> {
    match value {
        "http" => Ok(AgentMcpTransportKind::Http),
        "stdio" => Ok(AgentMcpTransportKind::Stdio),
        "sse" => Ok(AgentMcpTransportKind::Sse),
        other => Err(KernelError::validation(format!(
            "unsupported transportKind: {other}"
        ))),
    }
}

pub fn parse_auth_kind(value: &str) -> Result<AgentMcpAuthKind, KernelError> {
    match value {
        "none" => Ok(AgentMcpAuthKind::None),
        "host_secret_ref" | "host-secret-ref" => Ok(AgentMcpAuthKind::HostSecretRef),
        "api_key_ref" | "api-key-ref" => Ok(AgentMcpAuthKind::ApiKeyRef),
        "oauth2" => Ok(AgentMcpAuthKind::OAuth2),
        other => Err(KernelError::validation(format!(
            "unsupported authKind: {other}"
        ))),
    }
}

pub fn parse_visibility(value: &str) -> Result<AgentVisibility, KernelError> {
    match value {
        "private" => Ok(AgentVisibility::Private),
        "tenant" => Ok(AgentVisibility::Tenant),
        "public" => Ok(AgentVisibility::Public),
        other => Err(KernelError::validation(format!(
            "unsupported visibility: {other}"
        ))),
    }
}

pub fn build_create_command(
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    subject: PolicySubject,
    body: CreateMcpServerBody,
) -> Result<AgentMcpServerCreateCommand, KernelError> {
    Ok(AgentMcpServerCreateCommand {
        tenant_id,
        organization_id,
        owner_user_id,
        mcp_server_id: body.mcp_server_id,
        code: body.code,
        display_name: body.display_name,
        description: body.description,
        protocol_version: body.protocol_version,
        transport_kind: parse_transport_kind(body.transport_kind.as_str())?,
        endpoint_ref: body.endpoint_ref,
        command_ref: body.command_ref,
        auth_kind: parse_auth_kind(body.auth_kind.as_str())?,
        auth_profile_id: body.auth_profile_id,
        capability_ids: body.capability_ids,
        tool_count: body.tool_count,
        resource_count: body.resource_count,
        prompt_count: body.prompt_count,
        capabilities_json: body.capabilities_json,
        categories: body.categories,
        tags: body.tags,
        security_profile_id: body.security_profile_id,
        visibility: parse_visibility(body.visibility.as_str())?,
        requested_by: subject,
        requested_at: body.requested_at,
    })
}

pub fn build_update_command(
    tenant_id: u64,
    mcp_server_id: String,
    subject: PolicySubject,
    body: UpdateMcpServerBody,
) -> Result<AgentMcpServerUpdateCommand, KernelError> {
    Ok(AgentMcpServerUpdateCommand {
        tenant_id,
        mcp_server_id,
        expected_version: body
            .expected_version
            .as_deref()
            .map(str::parse)
            .transpose()
            .map_err(|error| KernelError::validation(format!("expectedVersion invalid: {error}")))?,
        display_name: body.display_name,
        description: body.description,
        protocol_version: body.protocol_version,
        transport_kind: body
            .transport_kind
            .as_deref()
            .map(parse_transport_kind)
            .transpose()?,
        endpoint_ref: body.endpoint_ref,
        command_ref: body.command_ref,
        auth_kind: body.auth_kind.as_deref().map(parse_auth_kind).transpose()?,
        auth_profile_id: body.auth_profile_id,
        capability_ids: body.capability_ids,
        tool_count: body.tool_count,
        resource_count: body.resource_count,
        prompt_count: body.prompt_count,
        capabilities_json: body.capabilities_json,
        categories: body.categories,
        tags: body.tags,
        security_profile_id: body.security_profile_id,
        visibility: body
            .visibility
            .as_deref()
            .map(parse_visibility)
            .transpose()?,
        requested_by: subject,
        requested_at: body.requested_at,
    })
}

pub fn list_query(tenant_id: u64) -> AgentMarketplaceListQuery {
    AgentMarketplaceListQuery::for_tenant(tenant_id)
}

pub fn list_mcp_servers<S, A, P>(
    service: &mut AgentsService<S, A, P>,
    tenant_id: u64,
    subject: PolicySubject,
) -> Result<Vec<AgentMcpServerRecord>, KernelError>
where
    S: crate::ports::AgentRepository,
    A: crate::ports::AgentAuditSink,
    P: sdkwork_agent_kernel::PolicyProvider,
{
    service.list_mcp_servers(list_query(tenant_id), subject)
}

pub fn get_mcp_server<S, A, P>(
    service: &mut AgentsService<S, A, P>,
    tenant_id: u64,
    mcp_server_id: &str,
    subject: PolicySubject,
) -> Result<AgentMcpServerRecord, KernelError>
where
    S: crate::ports::AgentRepository,
    A: crate::ports::AgentAuditSink,
    P: sdkwork_agent_kernel::PolicyProvider,
{
    service.get_mcp_server(GetAgentMarketplaceItemCommand {
        tenant_id,
        item_id: mcp_server_id.to_string(),
        requested_by: subject,
    })
}
