use std::collections::HashMap;
use std::sync::Arc;

mod commands;
pub use commands::*;

use crate::agent_turn::{AgentTurnRecord, AgentTurnStatus};
use crate::domain::{
    AgentAuditAction, AgentAuditPayload, AgentBusinessRecord, AgentBusinessStatus,
    AgentCompositionSlotKind, AgentCompositionSlotRecord, AgentCompositionTargetModule,
    AgentInteractionKind, AgentInteractionRecord, AgentInteractionStatus, AgentItemDriveRefRecord,
    AgentItemFeedbackRecord, AgentItemResourceRole, AgentProviderBindingRecord, AgentResourceType,
    AgentResourceUserStateRecord, AgentRuntimeExecutionOperation, AgentRuntimeExecutionRecord,
    AgentRuntimeExecutionStatus, AgentSessionCheckpointRecord, AgentSessionCheckpointStatus,
    AgentSessionEntrySurface, AgentSessionItemKind, AgentSessionItemRecord, AgentSessionItemStatus,
    AgentSessionKind, AgentSessionRecord, AgentSessionRuntimeBindingRecord,
    AgentSessionRuntimeBindingStatus, AgentSessionStatus, AgentTaskRecord, AgentTaskStatus,
    AgentVisibility, MarketplaceAuditPayload, ProviderBindingAuditPayload,
    RuntimeExecutionAuditPayload, SessionAuditPayload, SessionItemAuditPayload, TaskAuditPayload,
    DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY,
};
use crate::dto::AgentManagementProfileDto;
use crate::ports::{
    offset_paginated_result, AgentAuditSink, AgentRepository, PaginatedResult, PaginationParams,
    ProviderBindingListQuery, SessionItemListQuery, MAX_PAGE_SIZE, MAX_TURN_INPUT_CONTENT_BYTES,
    TURN_CONTEXT_ITEM_LIMIT,
};
use crate::project::{
    AgentProjectCompositionSlotRecord, AgentProjectDriveAccessMode, AgentProjectRecord,
    AgentProjectStatus, AgentProjectVisibility,
};
use crate::runtime_facade_bridge::{
    execute_preview_response, execute_prompt_optimization, RUNTIME_MODE_CONTRACT_FALLBACK,
};
use crate::turn_runtime::{
    complete_with_timeout, is_capacity_error, is_inference_error, ContractTurnExecutor,
    TurnExecutionInput, TurnExecutor, TURN_EXECUTION_TIMEOUT,
};
use crate::validation::{
    default_json_array_if_blank, default_json_object_if_blank, default_plain_text_if_blank,
    is_trimmed_blank, parse_optional_rfc3339_datetime, parse_rfc3339_datetime, require_non_blank,
    validate_capabilities, validate_requested_at, validate_standard_id,
};
use sdkwork_agent_kernel::{
    KernelError, KernelErrorKind, KernelEvent, KernelEventRedaction, KernelEventSeverity,
    KernelEventSource, KernelResult, PolicyCategory, PolicyDecisionValue, PolicyProvider,
    PolicyRequest, PolicySubject,
};
use sdkwork_agents_contract::agents_allow_contract_runtime_fallback;
use sdkwork_utils_rust::{sha256_hash, trim};
use time::OffsetDateTime;

const MAX_TURN_DRIVE_REFS: usize = 64;
const MAX_INTERACTION_LEASE_SECONDS: u32 = 300;

fn validate_runtime_token(value: &str, field_name: &str, max_bytes: usize) -> KernelResult<()> {
    require_non_blank(value, field_name)?;
    if value.len() > max_bytes
        || !value.bytes().enumerate().all(|(index, byte)| match byte {
            b'a'..=b'z' => true,
            b'0'..=b'9' | b'_' | b'-' => index > 0,
            _ => false,
        })
    {
        return Err(KernelError::validation(format!("{field_name} is invalid")));
    }
    Ok(())
}

fn validate_optional_bounded(
    value: &Option<String>,
    field_name: &str,
    max_bytes: usize,
) -> KernelResult<()> {
    if let Some(value) = value {
        require_non_blank(value, field_name)?;
        if value.len() > max_bytes {
            return Err(KernelError::validation(format!(
                "{field_name} exceeds {max_bytes} bytes"
            )));
        }
    }
    Ok(())
}

fn format_utc_seconds(value: OffsetDateTime) -> String {
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        value.year(),
        u8::from(value.month()),
        value.day(),
        value.hour(),
        value.minute(),
        value.second()
    )
}

fn validate_interaction_options(value: &serde_json::Value) -> KernelResult<()> {
    let options = value
        .as_array()
        .ok_or_else(|| KernelError::validation("options must be an array"))?;
    if options.len() > 128 {
        return Err(KernelError::validation("options exceeds 128 items"));
    }
    let mut values = std::collections::HashSet::with_capacity(options.len());
    for option in options {
        let object = option
            .as_object()
            .ok_or_else(|| KernelError::validation("each option must be an object"))?;
        if object.keys().any(|key| key != "value" && key != "label") {
            return Err(KernelError::validation(
                "options contains an unsupported field",
            ));
        }
        let option_value = object
            .get("value")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| KernelError::validation("options.value is required"))?;
        let label = object
            .get("label")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| KernelError::validation("options.label is required"))?;
        require_non_blank(option_value, "options.value")?;
        require_non_blank(label, "options.label")?;
        if option_value.len() > 256 || label.len() > 512 {
            return Err(KernelError::validation("interaction option is too long"));
        }
        if !values.insert(option_value) {
            return Err(KernelError::conflict(
                "interaction option values must be unique",
            ));
        }
    }
    Ok(())
}

fn validate_interaction_claim(
    record: &AgentInteractionRecord,
    claim_token: &str,
    fencing_token: u64,
) -> KernelResult<()> {
    if !(32..=256).contains(&claim_token.len()) {
        return Err(KernelError::validation("claimToken is invalid"));
    }
    if record.fencing_token != fencing_token {
        return Err(KernelError::conflict("interaction fencing token mismatch"));
    }
    let expected_hash = record
        .claim_token_hash
        .as_deref()
        .ok_or_else(|| KernelError::conflict("interaction must be claimed before resolution"))?;
    if sha256_hash(claim_token.as_bytes()) != expected_hash {
        return Err(KernelError::conflict("interaction claim token mismatch"));
    }
    let expires_at = record
        .claim_expires_at
        .as_deref()
        .ok_or_else(|| KernelError::conflict("interaction claim has no expiration"))?;
    if parse_rfc3339_datetime(expires_at, "claimExpiresAt")? <= OffsetDateTime::now_utc() {
        return Err(KernelError::conflict("interaction claim has expired"));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedItemDriveRef {
    resource_role: AgentItemResourceRole,
    drive_space_id: String,
    drive_node_id: String,
    sort_order: u32,
}

fn normalize_item_drive_resources(
    resources: &[AgentItemDriveRefInput],
) -> KernelResult<Vec<NormalizedItemDriveRef>> {
    if resources.len() > MAX_TURN_DRIVE_REFS {
        return Err(KernelError::validation(format!(
            "driveRefs exceeds maximum item count of {MAX_TURN_DRIVE_REFS}"
        )));
    }

    let mut normalized = Vec::with_capacity(resources.len());
    let mut uniqueness = std::collections::HashSet::with_capacity(resources.len());
    for (sort_order, resource) in resources.iter().enumerate() {
        let drive_space_id = resource.drive_space_id.trim();
        let drive_node_id = resource.drive_node_id.trim();
        if drive_space_id.is_empty() || drive_space_id.len() > 128 {
            return Err(KernelError::validation("driveRefs.driveSpaceId is invalid"));
        }
        if drive_node_id.is_empty() || drive_node_id.len() > 128 {
            return Err(KernelError::validation("driveRefs.driveNodeId is invalid"));
        }
        if !uniqueness.insert((
            drive_space_id.to_string(),
            drive_node_id.to_string(),
            resource.resource_role.as_str(),
        )) {
            return Err(KernelError::conflict(
                "duplicate session-item Drive reference",
            ));
        }
        normalized.push(NormalizedItemDriveRef {
            resource_role: resource.resource_role,
            drive_space_id: drive_space_id.to_string(),
            drive_node_id: drive_node_id.to_string(),
            sort_order: u32::try_from(sort_order)
                .map_err(|_| KernelError::validation("driveRefs sort order overflow"))?,
        });
    }
    Ok(normalized)
}

/// Stateless agent business service.
///
/// All methods take `&self` because `AgentRepository`, `AgentAuditSink`, and
/// `PolicyProvider` are all `Send + Sync`. This eliminates the global Mutex
/// bottleneck and allows true concurrent request processing. Optimistic
/// concurrency (version checks) ensures data integrity without serialization.
pub struct AgentsService<R, A, P>
where
    R: AgentRepository,
    A: AgentAuditSink,
    P: PolicyProvider,
{
    repository: R,
    audit_sink: A,
    policy_provider: P,
    turn_executor: Arc<dyn TurnExecutor>,
}

impl<R, A, P> AgentsService<R, A, P>
where
    R: AgentRepository,
    A: AgentAuditSink,
    P: PolicyProvider,
{
    pub fn new(repository: R, audit_sink: A, policy_provider: P) -> Self {
        Self {
            repository,
            audit_sink,
            policy_provider,
            turn_executor: Arc::new(ContractTurnExecutor),
        }
    }

    /// Replace the default contract completer with a kernel-backed implementation
    /// at gateway bootstrap (production) without changing HTTP handlers.
    pub fn with_turn_executor(mut self, turn_executor: Arc<dyn TurnExecutor>) -> Self {
        self.turn_executor = turn_executor;
        self
    }

    fn ensure_session_owner_scope(
        session: &AgentSessionRecord,
        owner_scope: Option<u64>,
    ) -> KernelResult<()> {
        if let Some(required_owner) = owner_scope {
            if session.owner_user_id != required_owner {
                return Err(KernelError::validation("session not found"));
            }
        }
        Ok(())
    }

    fn ensure_project_owner_scope(
        project: &AgentProjectRecord,
        owner_scope: Option<u64>,
    ) -> KernelResult<()> {
        if let Some(required_owner) = owner_scope {
            if project.owner_user_id != required_owner {
                return Err(KernelError::validation("project not found"));
            }
        }
        Ok(())
    }

    fn load_active_project_for_composition(
        &self,
        tenant_id: u64,
        organization_id: u64,
        project_id: &str,
        owner_scope: Option<u64>,
    ) -> KernelResult<AgentProjectRecord> {
        validate_standard_id(project_id, "projectId", Some("project."))?;
        let project = self
            .repository
            .get_project(tenant_id, organization_id, project_id)?
            .ok_or_else(|| KernelError::validation("project not found"))?;
        Self::ensure_project_owner_scope(&project, owner_scope)?;
        if project.status != AgentProjectStatus::Active {
            return Err(KernelError::validation("project is not active"));
        }
        Ok(project)
    }

    fn ensure_task_owner_scope(
        task: &AgentTaskRecord,
        owner_scope: Option<u64>,
    ) -> KernelResult<()> {
        if let Some(required_owner) = owner_scope {
            if task.owner_user_id != required_owner {
                return Err(KernelError::validation("task not found"));
            }
        }
        Ok(())
    }

    fn ensure_nested_agent_id(
        record_agent_id: &str,
        path_agent_id: &str,
        resource_label: &str,
    ) -> KernelResult<()> {
        if record_agent_id != path_agent_id {
            return Err(KernelError::validation(format!(
                "{resource_label} not found"
            )));
        }
        Ok(())
    }

    fn load_session_for_nested_route(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        path_agent_id: &str,
        owner_scope: Option<u64>,
    ) -> KernelResult<AgentSessionRecord> {
        let session = self
            .repository
            .get_session(tenant_id, organization_id, session_id)?
            .ok_or_else(|| KernelError::validation("session not found"))?;
        Self::ensure_session_owner_scope(&session, owner_scope)?;
        Self::ensure_nested_agent_id(&session.agent_id, path_agent_id, "session")?;
        Ok(session)
    }

    pub fn create_agent(&self, command: CreateAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;

        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.create",
            command.requested_by.clone(),
            policy_resource,
            "create",
        )?;

        if self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .is_some()
        {
            return Err(KernelError::conflict("agent already exists"));
        }

        require_non_blank(command.code.as_str(), "code")?;
        require_non_blank(command.display_name.as_str(), "displayName")?;
        if let Some(provider_id) = command.implementation_provider_id.as_deref() {
            validate_standard_id(provider_id, "implementationProviderId", Some("provider."))?;
        }

        let mut record = AgentBusinessRecord {
            id: self.repository.next_id()?,
            agent_id: command.agent_id,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            owner_user_id: command.owner_user_id,
            code: command.code,
            display_name: command.display_name,
            description: command.description,
            manifest: command.manifest,
            default_code_task_intent: command.default_code_task_intent,
            implementation_provider_id: command.implementation_provider_id,
            implementation_kind: command.implementation_kind,
            implementation_type: command.implementation_type.unwrap_or_default(),
            status: AgentBusinessStatus::Draft,
            visibility: command.visibility,
            tags: command.tags,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            deleted_at: None,
        };
        record.mark_updated(command.requested_at.clone());

        self.repository.insert(record.clone())?;
        self.emit_audit_event(
            AgentAuditAction::Create,
            &record,
            None,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn add_provider_binding(
        &self,
        command: AgentProviderBindingCommand,
    ) -> KernelResult<AgentProviderBindingRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        self.authorize(
            "agent.business.provider_binding.add",
            command.requested_by.clone(),
            format!("agent.business.{}", command.agent_id),
            "provider_binding.add",
        )?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        validate_standard_id(command.binding_id.as_str(), "bindingId", Some("binding."))?;
        validate_standard_id(
            command.provider_id.as_str(),
            "providerId",
            Some("provider."),
        )?;
        validate_standard_id(
            command.configuration_profile_id.as_str(),
            "configurationProfileId",
            Some("profile."),
        )?;
        validate_capabilities(command.capabilities.as_slice(), "capabilities")?;

        if self
            .repository
            .get_provider_binding(
                command.tenant_id,
                command.agent_id.as_str(),
                command.binding_id.as_str(),
            )?
            .is_some()
        {
            return Err(KernelError::conflict(
                "agent provider binding already exists",
            ));
        }

        if command.make_default {
            self.deactivate_provider_bindings(
                command.tenant_id,
                command.agent_id.as_str(),
                command.requested_at.clone(),
            )?;
        }

        let record = AgentProviderBindingRecord {
            id: self.repository.next_id()?,
            tenant_id: command.tenant_id,
            agent_id: command.agent_id.clone(),
            binding_id: command.binding_id,
            provider_id: command.provider_id,
            implementation_kind: command.implementation_kind,
            configuration_profile_id: command.configuration_profile_id,
            capabilities: command.capabilities,
            active: command.make_default,
            version: 1,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
        };

        self.repository.insert_provider_binding(record.clone())?;
        self.emit_binding_audit_event(
            AgentAuditAction::ProviderBindingChanged,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn activate_provider_binding(
        &self,
        command: ActivateAgentProviderBindingCommand,
    ) -> KernelResult<AgentProviderBindingRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        self.authorize(
            "agent.business.provider_binding.activate",
            command.requested_by.clone(),
            format!("agent.business.{}", command.agent_id),
            "provider_binding.activate",
        )?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        validate_standard_id(command.binding_id.as_str(), "bindingId", Some("binding."))?;

        let record = self.repository.activate_provider_binding_atomic(
            command.tenant_id,
            command.agent_id.as_str(),
            command.binding_id.as_str(),
            command.requested_at.clone(),
        )?;
        self.emit_binding_audit_event(
            AgentAuditAction::ProviderBindingChanged,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    fn all_provider_bindings_for_agent(
        &self,
        tenant_id: u64,
        agent_id: &str,
    ) -> KernelResult<Vec<AgentProviderBindingRecord>> {
        let mut all = Vec::new();
        let mut page = 1usize;
        loop {
            let batch = self.repository.list_provider_bindings(
                &ProviderBindingListQuery::for_agent(tenant_id, agent_id).with_pagination(
                    PaginationParams::default()
                        .with_page_size(MAX_PAGE_SIZE)
                        .with_page(page),
                ),
            )?;
            if batch.is_empty() {
                break;
            }
            let batch_len = batch.len();
            all.extend(batch);
            if batch_len < MAX_PAGE_SIZE {
                break;
            }
            page = page.saturating_add(1);
        }
        Ok(all)
    }

    pub fn list_provider_bindings(
        &self,
        command: ProviderBindingListCommand,
    ) -> KernelResult<PaginatedResult<AgentProviderBindingRecord>> {
        validate_agent_id(command.query.agent_id.as_str())?;
        self.authorize(
            "agent.business.provider_binding.list",
            command.requested_by,
            format!("agent.business.{}", command.query.agent_id),
            "provider_binding.list",
        )?;
        self.repository
            .get(command.query.tenant_id, command.query.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        let total_count = self.repository.count_provider_bindings(&command.query)?;
        let items = self.repository.list_provider_bindings(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn get_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
        requested_by: PolicySubject,
    ) -> KernelResult<AgentProviderBindingRecord> {
        validate_agent_id(agent_id)?;
        self.authorize(
            "agent.business.provider_binding.retrieve",
            requested_by,
            format!("agent.business.{}", agent_id),
            "provider_binding.retrieve",
        )?;
        self.repository
            .get(tenant_id, agent_id)?
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        self.repository
            .get_provider_binding(tenant_id, agent_id, binding_id)?
            .ok_or_else(|| KernelError::validation("provider binding not found"))
    }

    pub fn deactivate_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str,
        requested_at: String,
        requested_by: PolicySubject,
    ) -> KernelResult<AgentProviderBindingRecord> {
        validate_agent_id(agent_id)?;
        self.authorize(
            "agent.business.provider_binding.deactivate",
            requested_by.clone(),
            format!("agent.business.{}", agent_id),
            "provider_binding.deactivate",
        )?;
        let mut record = self
            .repository
            .get_provider_binding(tenant_id, agent_id, binding_id)?
            .ok_or_else(|| KernelError::validation("provider binding not found"))?;
        if !record.active {
            return Err(KernelError::validation(
                "provider binding is already inactive",
            ));
        }
        record.active = false;
        record.mark_updated(requested_at.as_str());
        self.repository.update_provider_binding(record.clone())?;
        self.emit_binding_audit_event(
            AgentAuditAction::ProviderBindingChanged,
            &record,
            requested_by,
            record.updated_at.clone(),
        )?;
        Ok(record)
    }

    pub fn update_session_metadata(
        &self,
        tenant_id: u64,
        organization_id: u64,
        session_id: &str,
        title: Option<String>,
        requested_at: String,
        requested_by: PolicySubject,
    ) -> KernelResult<AgentSessionRecord> {
        self.authorize(
            "agent.business.session.update",
            requested_by,
            format!("agent.business.session.{}", session_id),
            "session.update",
        )?;
        let mut record = self
            .repository
            .get_session(tenant_id, organization_id, session_id)?
            .ok_or_else(|| KernelError::validation("session not found"))?;
        if let Some(title) = title {
            record.title = Some(title);
        }
        record.mark_updated(requested_at.as_str());
        self.repository.update_session(record.clone())?;
        Ok(record)
    }

    pub fn list_code_engine_catalog(
        &self,
        requested_by: PolicySubject,
    ) -> KernelResult<sdkwork_agents_runtime_facade::CodeEngineCatalog> {
        self.authorize(
            "agent.business.code_engine.list",
            requested_by,
            "agent.business".to_string(),
            "code_engine.list",
        )?;
        Ok(crate::code_engine_catalog::list_code_engine_catalog())
    }

    pub fn list_mcp_marketplace(
        &self,
        command: ListMcpMarketplaceCommand,
    ) -> KernelResult<PaginatedResult<crate::mcp_marketplace::McpServerMarketplaceRecord>> {
        self.authorize(
            "agent.business.mcp_server.list",
            command.requested_by,
            "agent.business".to_string(),
            "mcp_server.list",
        )?;
        let total_count = self
            .repository
            .count_mcp_marketplace_slots(&command.query)?;
        let slots = self.repository.list_mcp_marketplace_slots(&command.query)?;
        let items = slots
            .iter()
            .map(crate::mcp_marketplace::project_mcp_slot)
            .collect();
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn create_preview_response(
        &self,
        command: AgentPreviewResponseCommand,
    ) -> KernelResult<AgentRuntimeExecutionRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        self.authorize(
            "agent.business.runtime.preview_response",
            command.requested_by.clone(),
            format!("agent.business.{}", command.agent_id),
            "runtime.preview_response",
        )?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        validate_standard_id(
            command.execution_id.as_str(),
            "executionId",
            Some("execution."),
        )?;
        validate_non_empty(command.content.as_str(), "content")?;
        validate_json_payload(command.input_payload_json.as_str(), "inputPayload")?;
        if let Some(model) = command.model.as_deref() {
            validate_optional_plain_ref(Some(model), "model")?;
        }
        if let Some(temperature) = command.temperature {
            if !(0.0..=2.0).contains(&temperature) || !temperature.is_finite() {
                return Err(KernelError::validation(
                    "temperature must be between 0 and 2",
                ));
            }
        }

        let active_binding = self
            .repository
            .get_active_provider_binding(command.tenant_id, command.agent_id.as_str())?;

        let preview = execute_preview_response(
            active_binding.as_ref(),
            command.content.as_str(),
            command.model.as_deref(),
        );
        if preview.runtime_mode == RUNTIME_MODE_CONTRACT_FALLBACK
            && !agents_allow_contract_runtime_fallback()
        {
            return Err(KernelError::validation(
                "preview response requires an active code-engine provider binding",
            ));
        }

        let output_payload_json = serde_json::json!({
            "content": preview.content,
            "debugMode": command.debug_mode,
            "model": preview.model_id,
            "temperature": command.temperature,
            "runtimeMode": preview.runtime_mode,
        })
        .to_string();

        let record = AgentRuntimeExecutionRecord {
            tenant_id: command.tenant_id,
            agent_id: command.agent_id,
            execution_id: command.execution_id,
            operation: AgentRuntimeExecutionOperation::PreviewResponse,
            status: AgentRuntimeExecutionStatus::Completed,
            input_payload_json: command.input_payload_json,
            output_payload_json,
            requested_at: command.requested_at.clone(),
            completed_at: command.requested_at.clone(),
        };

        self.emit_runtime_execution_audit_event(
            AgentAuditAction::RuntimeExecutionCompleted,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn create_prompt_optimization(
        &self,
        command: AgentPromptOptimizationCommand,
    ) -> KernelResult<AgentRuntimeExecutionRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        self.authorize(
            "agent.business.runtime.prompt_optimization",
            command.requested_by.clone(),
            format!("agent.business.{}", command.agent_id),
            "runtime.prompt_optimization",
        )?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        validate_standard_id(
            command.execution_id.as_str(),
            "executionId",
            Some("execution."),
        )?;
        validate_non_empty(command.prompt.as_str(), "prompt")?;
        validate_json_payload(command.input_payload_json.as_str(), "inputPayload")?;

        let active_binding = self
            .repository
            .get_active_provider_binding(command.tenant_id, command.agent_id.as_str())?;

        let optimization =
            execute_prompt_optimization(active_binding.as_ref(), command.prompt.as_str());
        if optimization.runtime_mode == RUNTIME_MODE_CONTRACT_FALLBACK
            && !agents_allow_contract_runtime_fallback()
        {
            return Err(KernelError::validation(
                "prompt optimization requires an active code-engine provider binding",
            ));
        }

        let output_payload_json = serde_json::json!({
            "optimizedPrompt": optimization.optimized_prompt,
            "runtimeMode": optimization.runtime_mode,
        })
        .to_string();

        let record = AgentRuntimeExecutionRecord {
            tenant_id: command.tenant_id,
            agent_id: command.agent_id,
            execution_id: command.execution_id,
            operation: AgentRuntimeExecutionOperation::PromptOptimization,
            status: AgentRuntimeExecutionStatus::Completed,
            input_payload_json: command.input_payload_json,
            output_payload_json,
            requested_at: command.requested_at.clone(),
            completed_at: command.requested_at.clone(),
        };

        self.emit_runtime_execution_audit_event(
            AgentAuditAction::RuntimeExecutionCompleted,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn create_composition_slot(
        &self,
        command: AgentCompositionSlotCreateCommand,
    ) -> KernelResult<AgentCompositionSlotRecord> {
        self.authorize(
            "agent.business.composition_slot.create",
            command.requested_by.clone(),
            format!(
                "agent.business.composition_slot.{}.{}",
                command.agent_id, command.slot_id
            ),
            "composition_slot.create",
        )?;
        validate_agent_id(command.agent_id.as_str())?;
        validate_standard_id(command.slot_id.as_str(), "slotId", Some("slot."))?;
        require_non_blank(command.target_ref.as_str(), "targetRef")?;
        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        if self
            .repository
            .get_composition_slot(
                command.tenant_id,
                command.agent_id.as_str(),
                command.slot_id.as_str(),
            )?
            .is_some()
        {
            return Err(KernelError::conflict("composition slot already exists"));
        }
        let record = AgentCompositionSlotRecord {
            id: self.repository.next_id()?,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            agent_id: command.agent_id,
            slot_id: command.slot_id,
            slot_kind: command.slot_kind,
            target_module: command.target_module,
            target_ref: command.target_ref,
            target_version_ref: command.target_version_ref,
            priority: command.priority,
            enabled: command.enabled,
            policy_json: command.policy_json,
            status: AgentBusinessStatus::Active,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            deleted_at: None,
        };
        self.repository.insert_composition_slot(record.clone())?;
        self.emit_marketplace_audit_event(AgentBusinessAuditEventInput {
            action: AgentAuditAction::CompositionSlotCreated,
            item_kind: "composition_slot",
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            item_id: record.slot_id.as_str(),
            status: record.status,
            visibility: AgentVisibility::Tenant,
            version: record.version,
            subject: command.requested_by,
            occurred_at: command.requested_at,
        })?;
        Ok(record)
    }

    pub fn list_composition_slots(
        &self,
        command: AgentCompositionSlotListCommand,
    ) -> KernelResult<PaginatedResult<AgentCompositionSlotRecord>> {
        self.authorize(
            "agent.business.composition_slot.list",
            command.requested_by,
            format!("agent.business.{}", command.query.agent_id),
            "composition_slot.list",
        )?;
        validate_agent_id(command.query.agent_id.as_str())?;
        self.repository
            .get(command.query.tenant_id, command.query.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        let total_count = self.repository.count_composition_slots(&command.query)?;
        let items = self.repository.list_composition_slots(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn get_composition_slot(
        &self,
        command: AgentCompositionSlotGetCommand,
    ) -> KernelResult<AgentCompositionSlotRecord> {
        self.authorize(
            "agent.business.composition_slot.retrieve",
            command.requested_by,
            format!(
                "agent.business.composition_slot.{}.{}",
                command.agent_id, command.slot_id
            ),
            "composition_slot.retrieve",
        )?;
        validate_agent_id(command.agent_id.as_str())?;
        validate_standard_id(command.slot_id.as_str(), "slotId", Some("slot."))?;
        self.repository
            .get_composition_slot(
                command.tenant_id,
                command.agent_id.as_str(),
                command.slot_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("composition slot not found"))
    }

    pub fn update_composition_slot(
        &self,
        command: AgentCompositionSlotUpdateCommand,
    ) -> KernelResult<AgentCompositionSlotRecord> {
        self.authorize(
            "agent.business.composition_slot.update",
            command.requested_by.clone(),
            format!(
                "agent.business.composition_slot.{}.{}",
                command.agent_id, command.slot_id
            ),
            "composition_slot.update",
        )?;
        validate_agent_id(command.agent_id.as_str())?;
        validate_standard_id(command.slot_id.as_str(), "slotId", Some("slot."))?;
        let mut record = self
            .repository
            .get_composition_slot(
                command.tenant_id,
                command.agent_id.as_str(),
                command.slot_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("composition slot not found"))?;
        if record.is_deleted() {
            return Err(KernelError::validation("composition slot is deleted"));
        }
        if let Some(expected_version) = command.expected_version {
            if record.version != expected_version {
                return Err(KernelError::conflict(format!(
                    "composition slot version mismatch: expected={expected_version}, actual={}",
                    record.version
                )));
            }
        }
        if let Some(slot_kind) = command.slot_kind {
            record.slot_kind = slot_kind;
        }
        if let Some(target_module) = command.target_module {
            record.target_module = target_module;
        }
        if let Some(target_ref) = command.target_ref {
            require_non_blank(target_ref.as_str(), "targetRef")?;
            record.target_ref = target_ref;
        }
        if let Some(target_version_ref) = command.target_version_ref {
            record.target_version_ref = target_version_ref;
        }
        if let Some(priority) = command.priority {
            record.priority = priority;
        }
        if let Some(enabled) = command.enabled {
            record.enabled = enabled;
        }
        if let Some(policy_json) = command.policy_json {
            record.policy_json = policy_json;
        }
        record.mark_updated(command.requested_at.clone());
        self.repository.update_composition_slot(record.clone())?;
        self.emit_marketplace_audit_event(AgentBusinessAuditEventInput {
            action: AgentAuditAction::CompositionSlotUpdated,
            item_kind: "composition_slot",
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            item_id: record.slot_id.as_str(),
            status: record.status,
            visibility: AgentVisibility::Tenant,
            version: record.version,
            subject: command.requested_by,
            occurred_at: command.requested_at,
        })?;
        Ok(record)
    }

    pub fn delete_composition_slot(
        &self,
        command: AgentCompositionSlotDeleteCommand,
    ) -> KernelResult<AgentCompositionSlotRecord> {
        self.authorize(
            "agent.business.composition_slot.delete",
            command.requested_by.clone(),
            format!(
                "agent.business.composition_slot.{}.{}",
                command.agent_id, command.slot_id
            ),
            "composition_slot.delete",
        )?;
        validate_agent_id(command.agent_id.as_str())?;
        validate_standard_id(command.slot_id.as_str(), "slotId", Some("slot."))?;
        let mut record = self
            .repository
            .get_composition_slot(
                command.tenant_id,
                command.agent_id.as_str(),
                command.slot_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("composition slot not found"))?;
        if record.is_deleted() {
            return Err(KernelError::validation("composition slot already deleted"));
        }
        if let Some(expected_version) = command.expected_version {
            if record.version != expected_version {
                return Err(KernelError::conflict(format!(
                    "composition slot version mismatch: expected={expected_version}, actual={}",
                    record.version
                )));
            }
        }
        record.mark_deleted(command.requested_at.clone());
        self.repository.update_composition_slot(record.clone())?;
        self.emit_marketplace_audit_event(AgentBusinessAuditEventInput {
            action: AgentAuditAction::CompositionSlotDeleted,
            item_kind: "composition_slot",
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            item_id: record.slot_id.as_str(),
            status: record.status,
            visibility: AgentVisibility::Tenant,
            version: record.version,
            subject: command.requested_by,
            occurred_at: command.requested_at,
        })?;
        Ok(record)
    }

    pub fn update_agent(&self, command: UpdateAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.update",
            command.requested_by.clone(),
            policy_resource,
            "update",
        )?;

        let mut record = self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        if record.is_deleted() {
            return Err(KernelError::validation("deleted agent cannot be updated"));
        }
        ensure_expected_version(record.version, command.expected_version, "agent")?;

        if let Some(display_name) = command.display_name {
            require_non_blank(display_name.as_str(), "displayName")?;
            record.display_name = display_name;
        }
        if let Some(description) = command.description {
            record.description = Some(description);
        }
        if let Some(manifest) = command.manifest {
            record.manifest = manifest;
        }
        if let Some(visibility) = command.visibility {
            record.visibility = visibility;
        }
        if let Some(tags) = command.tags {
            record.tags = tags;
        }
        if let Some(intent) = command.default_code_task_intent {
            record.default_code_task_intent = Some(intent);
        }
        if let Some(provider_id) = command.implementation_provider_id {
            if let Some(provider_id) = provider_id.as_deref() {
                validate_standard_id(provider_id, "implementationProviderId", Some("provider."))?;
            }
            record.implementation_provider_id = provider_id;
        }
        if let Some(implementation_kind) = command.implementation_kind {
            record.implementation_kind = implementation_kind;
        }
        if let Some(implementation_type) = command.implementation_type {
            record.implementation_type = implementation_type;
        }
        record.mark_updated(command.requested_at.clone());

        self.repository.update(record.clone())?;
        self.emit_audit_event(
            AgentAuditAction::Update,
            &record,
            None,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn change_status(
        &self,
        command: ChangeAgentStatusCommand,
    ) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.status.update",
            command.requested_by.clone(),
            policy_resource,
            "change_status",
        )?;

        let mut record = self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        if record.is_deleted() {
            return Err(KernelError::validation(
                "deleted agent status cannot be changed",
            ));
        }
        ensure_expected_version(record.version, command.expected_version, "agent")?;

        if !is_valid_status_transition(record.status, command.target_status) {
            return Err(KernelError::validation("invalid agent status transition"));
        }

        let previous_status = record.status;
        record.status = command.target_status;
        record.mark_updated(command.requested_at.clone());
        self.repository.update(record.clone())?;
        self.emit_audit_event(
            AgentAuditAction::ChangeStatus,
            &record,
            Some(previous_status),
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn delete_agent(&self, command: DeleteAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.delete",
            command.requested_by.clone(),
            policy_resource,
            "delete",
        )?;

        let mut record = self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        if record.is_deleted() {
            return Err(KernelError::validation("agent already deleted"));
        }
        if let Some(expected_version) = command.expected_version {
            if record.version != expected_version {
                return Err(KernelError::conflict(format!(
                    "agent version mismatch: expected={expected_version}, actual={}",
                    record.version
                )));
            }
        }

        record.mark_deleted(command.requested_at.clone());
        self.repository.update(record.clone())?;
        self.emit_audit_event(
            AgentAuditAction::Delete,
            &record,
            None,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn restore_agent(&self, command: RestoreAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.restore",
            command.requested_by.clone(),
            policy_resource,
            "restore",
        )?;

        let mut record = self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        if !record.is_deleted() {
            return Err(KernelError::validation("agent is not deleted"));
        }
        ensure_expected_version(record.version, command.expected_version, "agent")?;

        record.mark_restored(command.requested_at.clone());
        self.repository.update(record.clone())?;
        self.emit_audit_event(
            AgentAuditAction::Restore,
            &record,
            None,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn get_agent(&self, command: GetAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.retrieve",
            command.requested_by,
            policy_resource,
            "retrieve",
        )?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))
            .and_then(|record| {
                if record.is_deleted() {
                    return Err(KernelError::validation("agent not found"));
                }
                Ok(record)
            })
    }

    pub fn list_agents(
        &self,
        command: ListAgentsCommand,
    ) -> KernelResult<PaginatedResult<AgentBusinessRecord>> {
        self.authorize(
            "agent.business.list",
            command.requested_by,
            format!("agent.business.tenant.{}", command.query.tenant_id),
            "list",
        )?;
        self.repository.list_paginated(&command.query)
    }

    pub fn list_agent_audit_events(
        &self,
        command: ListAgentAuditEventsCommand,
    ) -> KernelResult<PaginatedResult<KernelEvent>> {
        validate_agent_id(command.query.agent_id.as_str())?;
        self.authorize(
            "agent.business.audit.read",
            command.requested_by,
            format!("agent.business.{}", command.query.agent_id),
            "audit.read",
        )?;
        self.audit_sink.list_events(&command.query)
    }

    // -----------------------------------------------------------------------
    // Project management
    // -----------------------------------------------------------------------

    pub fn create_project(
        &self,
        command: CreateProjectCommand,
    ) -> KernelResult<AgentProjectRecord> {
        let project_id = if is_trimmed_blank(command.project_id.as_str()) {
            format!("project.{}", self.repository.next_id()?)
        } else {
            command.project_id.clone()
        };
        validate_standard_id(project_id.as_str(), "projectId", Some("project."))?;
        require_non_blank(command.name.as_str(), "name")?;
        if command.name.len() > 255 {
            return Err(KernelError::validation("name exceeds 255 bytes"));
        }
        validate_project_drive_access(command.visibility, command.drive_access_mode)?;
        self.authorize(
            "agent.business.project.create",
            command.requested_by.clone(),
            format!("agent.business.project.{project_id}"),
            "project.create",
        )?;
        if let Some(agent_id) = command.default_agent_id.as_deref() {
            validate_agent_id(agent_id)?;
            let agent = self
                .repository
                .get(command.tenant_id, agent_id)?
                .ok_or_else(|| KernelError::validation("default agent not found"))?;
            if agent.organization_id != command.organization_id {
                return Err(KernelError::validation("default agent not found"));
            }
        }
        if self
            .repository
            .get_project(command.tenant_id, command.organization_id, &project_id)?
            .is_some()
        {
            return Err(KernelError::conflict("project already exists"));
        }
        let record = AgentProjectRecord {
            id: self.repository.next_id()?,
            project_id,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            owner_user_id: command.owner_user_id,
            name: trim(command.name.as_str()).to_string(),
            description: command.description,
            visibility: command.visibility,
            status: AgentProjectStatus::Active,
            drive_access_mode: command.drive_access_mode,
            default_agent_id: command.default_agent_id,
            default_model_id: command.default_model_id,
            created_by: command.owner_user_id,
            updated_by: command.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        };
        self.repository.insert_project(record.clone())?;
        self.emit_project_audit_event(
            AgentAuditAction::ProjectCreated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn update_project(
        &self,
        command: UpdateProjectCommand,
    ) -> KernelResult<AgentProjectRecord> {
        validate_standard_id(&command.project_id, "projectId", Some("project."))?;
        self.authorize(
            "agent.business.project.update",
            command.requested_by.clone(),
            format!("agent.business.project.{}", command.project_id),
            "project.update",
        )?;
        let mut record = self
            .repository
            .get_project(
                command.tenant_id,
                command.organization_id,
                &command.project_id,
            )?
            .ok_or_else(|| KernelError::validation("project not found"))?;
        Self::ensure_project_owner_scope(&record, command.owner_scope)?;
        ensure_expected_version(record.version, command.expected_version, "project")?;
        if record.status != AgentProjectStatus::Active {
            return Err(KernelError::validation("project is not active"));
        }
        if let Some(name) = command.name {
            require_non_blank(&name, "name")?;
            if name.len() > 255 {
                return Err(KernelError::validation("name exceeds 255 bytes"));
            }
            record.name = trim(&name).to_string();
        }
        if let Some(description) = command.description {
            record.description = description;
        }
        if let Some(visibility) = command.visibility {
            record.visibility = visibility;
        }
        if let Some(drive_access_mode) = command.drive_access_mode {
            record.drive_access_mode = drive_access_mode;
        }
        validate_project_drive_access(record.visibility, record.drive_access_mode)?;
        if let Some(default_agent_id) = command.default_agent_id {
            if let Some(agent_id) = default_agent_id.as_deref() {
                validate_agent_id(agent_id)?;
                let agent = self
                    .repository
                    .get(command.tenant_id, agent_id)?
                    .ok_or_else(|| KernelError::validation("default agent not found"))?;
                if agent.organization_id != command.organization_id {
                    return Err(KernelError::validation("default agent not found"));
                }
            }
            record.default_agent_id = default_agent_id;
        }
        if let Some(default_model_id) = command.default_model_id {
            record.default_model_id = default_model_id;
        }
        record.mark_updated(command.requested_user_id, command.requested_at.clone());
        self.repository.update_project(record.clone())?;
        self.emit_project_audit_event(
            AgentAuditAction::ProjectUpdated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn archive_project(
        &self,
        command: ProjectMutationCommand,
    ) -> KernelResult<AgentProjectRecord> {
        self.mutate_project_status(command, AgentProjectStatus::Archived)
    }

    pub fn delete_project(
        &self,
        command: ProjectMutationCommand,
    ) -> KernelResult<AgentProjectRecord> {
        self.mutate_project_status(command, AgentProjectStatus::Deleted)
    }

    fn mutate_project_status(
        &self,
        command: ProjectMutationCommand,
        target: AgentProjectStatus,
    ) -> KernelResult<AgentProjectRecord> {
        validate_standard_id(&command.project_id, "projectId", Some("project."))?;
        let (request_id, action, audit_action) = match target {
            AgentProjectStatus::Archived => (
                "agent.business.project.archive",
                "project.archive",
                AgentAuditAction::ProjectArchived,
            ),
            AgentProjectStatus::Deleted => (
                "agent.business.project.delete",
                "project.delete",
                AgentAuditAction::ProjectDeleted,
            ),
            AgentProjectStatus::Active => {
                return Err(KernelError::validation("unsupported project transition"));
            }
        };
        self.authorize(
            request_id,
            command.requested_by.clone(),
            format!("agent.business.project.{}", command.project_id),
            action,
        )?;
        let mut record = self
            .repository
            .get_project(
                command.tenant_id,
                command.organization_id,
                &command.project_id,
            )?
            .ok_or_else(|| KernelError::validation("project not found"))?;
        Self::ensure_project_owner_scope(&record, command.owner_scope)?;
        if target != AgentProjectStatus::Deleted || command.expected_version.is_some() {
            ensure_expected_version(record.version, command.expected_version, "project")?;
        }
        match target {
            AgentProjectStatus::Archived => {
                record.archive(command.requested_user_id, command.requested_at.clone())
            }
            AgentProjectStatus::Deleted => {
                record.soft_delete(command.requested_user_id, command.requested_at.clone())
            }
            AgentProjectStatus::Active => unreachable!(),
        }
        self.repository.update_project(record.clone())?;
        self.emit_project_audit_event(
            audit_action,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn get_project(&self, command: GetProjectCommand) -> KernelResult<AgentProjectRecord> {
        self.authorize(
            "agent.business.project.retrieve",
            command.requested_by,
            format!("agent.business.project.{}", command.project_id),
            "project.retrieve",
        )?;
        validate_standard_id(&command.project_id, "projectId", Some("project."))?;
        self.repository
            .get_project(
                command.tenant_id,
                command.organization_id,
                &command.project_id,
            )?
            .ok_or_else(|| KernelError::validation("project not found"))
            .and_then(|record| {
                Self::ensure_project_owner_scope(&record, command.owner_scope)?;
                Ok(record)
            })
    }

    pub fn list_projects(
        &self,
        command: ListProjectsCommand,
    ) -> KernelResult<PaginatedResult<AgentProjectRecord>> {
        self.authorize(
            "agent.business.project.list",
            command.requested_by,
            format!(
                "agent.business.project.tenant.{}.organization.{}",
                command.query.tenant_id, command.query.organization_id
            ),
            "project.list",
        )?;
        let total_count = self.repository.count_projects(&command.query)?;
        let items = self.repository.list_projects(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn create_project_composition_slot(
        &self,
        command: CreateProjectCompositionSlotCommand,
    ) -> KernelResult<AgentProjectCompositionSlotRecord> {
        validate_standard_id(&command.slot_id, "slotId", Some("slot."))?;
        self.authorize(
            "agent.business.project.composition_slot.create",
            command.requested_by.clone(),
            format!(
                "agent.business.project.{}.composition_slot.{}",
                command.project_id, command.slot_id
            ),
            "project.composition_slot.create",
        )?;
        self.load_active_project_for_composition(
            command.tenant_id,
            command.organization_id,
            &command.project_id,
            command.owner_scope,
        )?;
        validate_project_composition_slot_fields(
            command.slot_kind,
            command.target_module,
            &command.target_ref,
            command.target_version_ref.as_deref(),
            command.priority,
            &command.policy_json,
        )?;
        if self
            .repository
            .get_project_composition_slot(
                command.tenant_id,
                command.organization_id,
                &command.project_id,
                &command.slot_id,
            )?
            .is_some()
        {
            return Err(KernelError::conflict(
                "project composition slot already exists",
            ));
        }
        let record = AgentProjectCompositionSlotRecord {
            id: self.repository.next_id()?,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            project_id: command.project_id,
            slot_id: command.slot_id,
            slot_kind: command.slot_kind,
            target_module: command.target_module,
            target_ref: trim(&command.target_ref).to_string(),
            target_version_ref: command.target_version_ref,
            priority: command.priority,
            enabled: command.enabled,
            policy_json: command.policy_json,
            created_by: command.requested_user_id,
            updated_by: command.requested_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        };
        self.repository
            .insert_project_composition_slot(record.clone())?;
        self.emit_project_composition_slot_audit_event(
            AgentAuditAction::ProjectCompositionSlotCreated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn get_project_composition_slot(
        &self,
        command: GetProjectCompositionSlotCommand,
    ) -> KernelResult<AgentProjectCompositionSlotRecord> {
        validate_standard_id(&command.slot_id, "slotId", Some("slot."))?;
        self.authorize(
            "agent.business.project.composition_slot.retrieve",
            command.requested_by,
            format!(
                "agent.business.project.{}.composition_slot.{}",
                command.project_id, command.slot_id
            ),
            "project.composition_slot.retrieve",
        )?;
        self.load_active_project_for_composition(
            command.tenant_id,
            command.organization_id,
            &command.project_id,
            command.owner_scope,
        )?;
        self.repository
            .get_project_composition_slot(
                command.tenant_id,
                command.organization_id,
                &command.project_id,
                &command.slot_id,
            )?
            .ok_or_else(|| KernelError::validation("project composition slot not found"))
    }

    pub fn list_project_composition_slots(
        &self,
        command: ListProjectCompositionSlotsCommand,
    ) -> KernelResult<PaginatedResult<AgentProjectCompositionSlotRecord>> {
        self.authorize(
            "agent.business.project.composition_slot.list",
            command.requested_by,
            format!(
                "agent.business.project.{}.composition_slots",
                command.query.project_id
            ),
            "project.composition_slot.list",
        )?;
        self.load_active_project_for_composition(
            command.query.tenant_id,
            command.query.organization_id,
            &command.query.project_id,
            command.owner_scope,
        )?;
        let total_count = self
            .repository
            .count_project_composition_slots(&command.query)?;
        let items = self
            .repository
            .list_project_composition_slots(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn update_project_composition_slot(
        &self,
        command: UpdateProjectCompositionSlotCommand,
    ) -> KernelResult<AgentProjectCompositionSlotRecord> {
        validate_standard_id(&command.slot_id, "slotId", Some("slot."))?;
        self.authorize(
            "agent.business.project.composition_slot.update",
            command.requested_by.clone(),
            format!(
                "agent.business.project.{}.composition_slot.{}",
                command.project_id, command.slot_id
            ),
            "project.composition_slot.update",
        )?;
        self.load_active_project_for_composition(
            command.tenant_id,
            command.organization_id,
            &command.project_id,
            command.owner_scope,
        )?;
        let mut record = self
            .repository
            .get_project_composition_slot(
                command.tenant_id,
                command.organization_id,
                &command.project_id,
                &command.slot_id,
            )?
            .ok_or_else(|| KernelError::validation("project composition slot not found"))?;
        ensure_expected_version(
            record.version,
            command.expected_version,
            "project composition slot",
        )?;
        if let Some(slot_kind) = command.slot_kind {
            record.slot_kind = slot_kind;
        }
        if let Some(target_module) = command.target_module {
            record.target_module = target_module;
        }
        if let Some(target_ref) = command.target_ref {
            record.target_ref = trim(&target_ref).to_string();
        }
        if let Some(target_version_ref) = command.target_version_ref {
            record.target_version_ref = target_version_ref;
        }
        if let Some(priority) = command.priority {
            record.priority = priority;
        }
        if let Some(enabled) = command.enabled {
            record.enabled = enabled;
        }
        if let Some(policy_json) = command.policy_json {
            record.policy_json = policy_json;
        }
        validate_project_composition_slot_fields(
            record.slot_kind,
            record.target_module,
            &record.target_ref,
            record.target_version_ref.as_deref(),
            record.priority,
            &record.policy_json,
        )?;
        record.mark_updated(command.requested_user_id, command.requested_at.clone());
        self.repository
            .update_project_composition_slot(record.clone())?;
        self.emit_project_composition_slot_audit_event(
            AgentAuditAction::ProjectCompositionSlotUpdated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn delete_project_composition_slot(
        &self,
        command: DeleteProjectCompositionSlotCommand,
    ) -> KernelResult<AgentProjectCompositionSlotRecord> {
        validate_standard_id(&command.slot_id, "slotId", Some("slot."))?;
        self.authorize(
            "agent.business.project.composition_slot.delete",
            command.requested_by.clone(),
            format!(
                "agent.business.project.{}.composition_slot.{}",
                command.project_id, command.slot_id
            ),
            "project.composition_slot.delete",
        )?;
        self.load_active_project_for_composition(
            command.tenant_id,
            command.organization_id,
            &command.project_id,
            command.owner_scope,
        )?;
        let mut record = self
            .repository
            .get_project_composition_slot(
                command.tenant_id,
                command.organization_id,
                &command.project_id,
                &command.slot_id,
            )?
            .ok_or_else(|| KernelError::validation("project composition slot not found"))?;
        ensure_expected_version(
            record.version,
            command.expected_version,
            "project composition slot",
        )?;
        record.soft_delete(command.requested_user_id, command.requested_at.clone());
        self.repository
            .update_project_composition_slot(record.clone())?;
        self.emit_project_composition_slot_audit_event(
            AgentAuditAction::ProjectCompositionSlotDeleted,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    // -----------------------------------------------------------------------
    // Session management
    // -----------------------------------------------------------------------

    pub fn create_session(
        &self,
        command: CreateSessionCommand,
    ) -> KernelResult<AgentSessionRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let session_id = if is_trimmed_blank(command.session_id.as_str()) {
            format!("session.{}", self.repository.next_id()?)
        } else {
            command.session_id.clone()
        };
        validate_standard_id(session_id.as_str(), "sessionId", Some("session."))?;
        self.authorize(
            "agent.business.session.create",
            command.requested_by.clone(),
            format!("agent.business.session.{}", session_id),
            "session.create",
        )?;

        // Ensure the agent exists
        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        // Ensure session does not already exist
        if self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                session_id.as_str(),
            )?
            .is_some()
        {
            return Err(KernelError::conflict("session already exists"));
        }

        if command.source_module.is_some()
            || command.source_context_kind.is_some()
            || command.source_context_id.is_some()
        {
            for (value, field) in [
                (command.source_module.as_deref(), "sourceModule"),
                (command.source_context_kind.as_deref(), "sourceContextKind"),
                (command.source_context_id.as_deref(), "sourceContextId"),
            ] {
                require_non_blank(
                    value.ok_or_else(|| {
                        KernelError::validation(
                            "sourceModule, sourceContextKind and sourceContextId must be supplied together",
                        )
                    })?,
                    field,
                )?;
            }
        }
        if command.parent_session_id.is_some() != command.forked_from_turn_id.is_some() {
            return Err(KernelError::validation(
                "parentSessionId and forkedFromTurnId must be supplied together",
            ));
        }
        if let (Some(parent_session_id), Some(forked_from_turn_id)) = (
            command.parent_session_id.as_deref(),
            command.forked_from_turn_id.as_deref(),
        ) {
            if parent_session_id == session_id {
                return Err(KernelError::validation(
                    "parentSessionId must differ from sessionId",
                ));
            }
            let parent = self
                .repository
                .get_session(
                    command.tenant_id,
                    command.organization_id,
                    parent_session_id,
                )?
                .ok_or_else(|| KernelError::validation("parent session not found"))?;
            if parent.organization_id != command.organization_id
                || parent.owner_user_id != command.owner_user_id
            {
                return Err(KernelError::validation("parent session scope mismatch"));
            }
            let fork_turn = self
                .repository
                .get_turn(
                    command.tenant_id,
                    command.organization_id,
                    forked_from_turn_id,
                )?
                .ok_or_else(|| KernelError::validation("fork turn not found"))?;
            if fork_turn.session_id != parent_session_id {
                return Err(KernelError::validation(
                    "forkedFromTurnId does not belong to parentSessionId",
                ));
            }
        }
        if command.idempotency_key.is_some() != command.payload_hash.is_some() {
            return Err(KernelError::validation(
                "idempotencyKey and payloadHash must be supplied together",
            ));
        }

        if let Some(project_id) = command.project_id.as_deref() {
            validate_standard_id(project_id, "projectId", Some("project."))?;
            let project = self
                .repository
                .get_project(command.tenant_id, command.organization_id, project_id)?
                .ok_or_else(|| KernelError::validation("project not found"))?;
            Self::ensure_project_owner_scope(&project, Some(command.owner_user_id))?;
            if project.status != AgentProjectStatus::Active {
                return Err(KernelError::validation("project is not active"));
            }
        }

        let record = AgentSessionRecord {
            id: self.repository.next_id()?,
            session_id,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            agent_id: command.agent_id,
            owner_user_id: command.owner_user_id,
            project_id: command.project_id,
            session_kind: command.session_kind,
            entry_surface: command.entry_surface,
            source_module: command.source_module,
            source_context_kind: command.source_context_kind,
            source_context_id: command.source_context_id,
            parent_session_id: command.parent_session_id,
            forked_from_turn_id: command.forked_from_turn_id,
            title: command.title,
            status: AgentSessionStatus::Active,
            item_count: 0,
            last_item_sequence: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            idempotency_key: command.idempotency_key,
            payload_hash: command.payload_hash,
            created_by: command.owner_user_id,
            updated_by: command.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            last_item_at: None,
            closed_at: None,
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        };

        self.repository.insert_session(record.clone())?;
        self.emit_session_audit_event(
            AgentAuditAction::SessionCreated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn update_session(
        &self,
        command: UpdateSessionCommand,
    ) -> KernelResult<AgentSessionRecord> {
        validate_standard_id(command.session_id.as_str(), "sessionId", Some("session."))?;
        self.authorize(
            "agent.business.session.update",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "session.update",
        )?;
        let mut record = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("session not found"))?;
        Self::ensure_session_owner_scope(&record, command.owner_scope)?;
        if record.organization_id != command.organization_id {
            return Err(KernelError::validation("session organization mismatch"));
        }
        if command.expected_version.is_some() {
            ensure_expected_version(record.version, command.expected_version, "session")?;
        }
        if record.deleted_at.is_some() {
            return Err(KernelError::validation("session not found"));
        }

        let mut audit_action = AgentAuditAction::SessionRenamed;
        if let Some(title) = command.title {
            require_non_blank(&title, "title")?;
            if title.len() > 512 {
                return Err(KernelError::validation("title exceeds 512 bytes"));
            }
            record.title = Some(trim(&title).to_string());
        }
        if let Some(project_id) = command.project_id {
            audit_action = AgentAuditAction::SessionMoved;
            if let Some(project_id) = project_id.as_deref() {
                validate_standard_id(project_id, "projectId", Some("project."))?;
                let project = self
                    .repository
                    .get_project(command.tenant_id, command.organization_id, project_id)?
                    .ok_or_else(|| KernelError::validation("project not found"))?;
                Self::ensure_project_owner_scope(&project, command.owner_scope)?;
                if project.status != AgentProjectStatus::Active {
                    return Err(KernelError::validation("project is not active"));
                }
            }
            record.project_id = project_id;
        }
        record.mark_updated(command.requested_at.clone());
        self.repository.update_session(record.clone())?;
        self.emit_session_audit_event(
            audit_action,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn delete_session(
        &self,
        command: DeleteSessionCommand,
    ) -> KernelResult<AgentSessionRecord> {
        validate_standard_id(command.session_id.as_str(), "sessionId", Some("session."))?;
        self.authorize(
            "agent.business.session.delete",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "session.delete",
        )?;
        let mut record = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("session not found"))?;
        Self::ensure_session_owner_scope(&record, command.owner_scope)?;
        if record.organization_id != command.organization_id {
            return Err(KernelError::validation("session organization mismatch"));
        }
        record.soft_delete(command.requested_at.clone());
        self.repository.update_session(record.clone())?;
        self.emit_session_audit_event(
            AgentAuditAction::SessionDeleted,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn close_session(&self, command: CloseSessionCommand) -> KernelResult<AgentSessionRecord> {
        self.authorize(
            "agent.business.session.close",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "session.close",
        )?;
        validate_standard_id(command.session_id.as_str(), "sessionId", Some("session."))?;

        let mut record = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("session not found"))?;

        Self::ensure_session_owner_scope(&record, command.owner_scope)?;

        if !record.status.is_active() {
            return Err(KernelError::validation("session is not active"));
        }

        ensure_expected_version(record.version, command.expected_version, "session")?;

        record.close(command.requested_at.clone());
        self.repository.update_session(record.clone())?;
        self.emit_session_audit_event(
            AgentAuditAction::SessionClosed,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn archive_session(
        &self,
        command: ArchiveSessionCommand,
    ) -> KernelResult<AgentSessionRecord> {
        self.authorize(
            "agent.business.session.archive",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "session.archive",
        )?;
        validate_standard_id(command.session_id.as_str(), "sessionId", Some("session."))?;

        let mut record = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("session not found"))?;

        Self::ensure_session_owner_scope(&record, command.owner_scope)?;

        if record.status == AgentSessionStatus::Archived {
            return Err(KernelError::validation("session is already archived"));
        }
        if record.status.is_active() {
            return Err(KernelError::validation(
                "session must be closed before archiving",
            ));
        }

        ensure_expected_version(record.version, command.expected_version, "session")?;

        record.archive(command.requested_at.clone());
        self.repository.update_session(record.clone())?;
        self.emit_session_audit_event(
            AgentAuditAction::SessionArchived,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn get_session(&self, command: GetSessionCommand) -> KernelResult<AgentSessionRecord> {
        self.authorize(
            "agent.business.session.retrieve",
            command.requested_by,
            format!("agent.business.session.{}", command.session_id),
            "session.retrieve",
        )?;
        validate_standard_id(command.session_id.as_str(), "sessionId", Some("session."))?;
        self.repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("session not found"))
            .and_then(|record| {
                Self::ensure_session_owner_scope(&record, command.owner_scope)?;
                Self::ensure_nested_agent_id(
                    &record.agent_id,
                    command.path_agent_id.as_str(),
                    "session",
                )?;
                Ok(record)
            })
    }

    pub fn list_sessions(
        &self,
        command: ListSessionsCommand,
    ) -> KernelResult<PaginatedResult<AgentSessionRecord>> {
        self.authorize(
            "agent.business.session.list",
            command.requested_by,
            format!("agent.business.tenant.{}", command.query.tenant_id),
            "session.list",
        )?;
        let total_count = self.repository.count_sessions(&command.query)?;
        let items = self.repository.list_sessions(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn list_session_user_states(
        &self,
        command: ListSessionUserStatesCommand,
    ) -> KernelResult<PaginatedResult<AgentResourceUserStateRecord>> {
        validate_agent_id(command.path_agent_id.as_str())?;
        self.authorize(
            "agent.business.session.user_state.list",
            command.requested_by,
            format!(
                "agent.business.agent.{}.session.user_state",
                command.path_agent_id
            ),
            "session.user_state.list",
        )?;
        let mut query = command.query;
        query.resource_type = AgentResourceType::Session;
        query.agent_id = Some(command.path_agent_id);
        let total_count = self.repository.count_resource_user_states(&query)?;
        let items = self.repository.list_resource_user_states(&query)?;
        Ok(offset_paginated_result(
            items,
            &query.pagination,
            total_count,
        ))
    }

    pub fn get_session_user_state(
        &self,
        command: GetSessionUserStateCommand,
    ) -> KernelResult<SessionUserStateResult> {
        validate_agent_id(command.path_agent_id.as_str())?;
        validate_standard_id(command.session_id.as_str(), "sessionId", Some("session."))?;
        self.authorize(
            "agent.business.session.user_state.retrieve",
            command.requested_by,
            format!("agent.business.session.{}.user_state", command.session_id),
            "session.user_state.retrieve",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
            command.path_agent_id.as_str(),
            Some(command.user_id),
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::validation("session not found"));
        }
        self.repository
            .get_resource_user_state(
                command.tenant_id,
                command.organization_id,
                command.user_id,
                AgentResourceType::Session,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("session user state not found"))
    }

    pub fn update_session_user_state(
        &self,
        command: UpdateSessionUserStateCommand,
    ) -> KernelResult<SessionUserStateResult> {
        validate_agent_id(command.path_agent_id.as_str())?;
        validate_standard_id(command.session_id.as_str(), "sessionId", Some("session."))?;
        if command.pinned.is_none()
            && command.hidden.is_none()
            && !command.mark_opened
            && command.last_read_item_sequence.is_none()
            && command.custom_title.is_none()
        {
            return Err(KernelError::validation(
                "session user state update requires a changed field",
            ));
        }
        self.authorize(
            "agent.business.session.user_state.update",
            command.requested_by,
            format!("agent.business.session.{}.user_state", command.session_id),
            "session.user_state.update",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
            command.path_agent_id.as_str(),
            Some(command.user_id),
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::validation("session not found"));
        }

        let existing = self.repository.get_resource_user_state(
            command.tenant_id,
            command.organization_id,
            command.user_id,
            AgentResourceType::Session,
            command.session_id.as_str(),
        )?;
        let mut record = match existing {
            Some(record) => {
                ensure_expected_version(
                    record.version,
                    command.expected_version,
                    "session user state",
                )?;
                record
            }
            None => {
                if command.expected_version.is_some() {
                    return Err(KernelError::conflict("session user state version mismatch"));
                }
                AgentResourceUserStateRecord {
                    id: self.repository.next_id()?,
                    tenant_id: command.tenant_id,
                    organization_id: command.organization_id,
                    user_id: command.user_id,
                    resource_type: AgentResourceType::Session,
                    resource_id: command.session_id.clone(),
                    pinned_at: None,
                    hidden_at: None,
                    last_opened_at: None,
                    last_read_item_sequence: None,
                    custom_title: None,
                    version: 0,
                    created_at: command.requested_at.clone(),
                    updated_at: command.requested_at.clone(),
                }
            }
        };

        if let Some(pinned) = command.pinned {
            record.pinned_at = pinned.then(|| command.requested_at.clone());
        }
        if let Some(hidden) = command.hidden {
            record.hidden_at = hidden.then(|| command.requested_at.clone());
        }
        if command.mark_opened {
            record.last_opened_at = Some(command.requested_at.clone());
        }
        if let Some(sequence) = command.last_read_item_sequence {
            if sequence > session.last_item_sequence {
                return Err(KernelError::validation(
                    "lastReadItemSequence exceeds the session item sequence",
                ));
            }
            if record
                .last_read_item_sequence
                .is_some_and(|current| sequence < current)
            {
                return Err(KernelError::conflict(
                    "lastReadItemSequence cannot move backwards",
                ));
            }
            record.last_read_item_sequence = Some(sequence);
        }
        if let Some(custom_title) = command.custom_title {
            record.custom_title = custom_title
                .map(|title| {
                    require_non_blank(title.as_str(), "customTitle")?;
                    if title.len() > 512 {
                        return Err(KernelError::validation("customTitle exceeds 512 bytes"));
                    }
                    reject_secret_material(title.as_str(), "customTitle")?;
                    Ok(trim(title.as_str()).to_string())
                })
                .transpose()?;
        }
        if command.expected_version.is_some() {
            record.version = record
                .version
                .checked_add(1)
                .ok_or_else(|| KernelError::conflict("session user state version overflow"))?;
        }
        record.updated_at = command.requested_at;
        self.repository
            .upsert_resource_user_state(record, command.expected_version)
    }

    pub fn list_item_feedback(
        &self,
        command: ListItemFeedbackCommand,
    ) -> KernelResult<PaginatedResult<AgentItemFeedbackRecord>> {
        validate_agent_id(command.path_agent_id.as_str())?;
        validate_standard_id(
            command.query.session_id.as_str(),
            "sessionId",
            Some("session."),
        )?;
        self.authorize(
            "agent.business.message.feedback.list",
            command.requested_by,
            format!(
                "agent.business.session.{}.message.feedback",
                command.query.session_id
            ),
            "message.feedback.list",
        )?;
        let session = self.load_session_for_nested_route(
            command.query.tenant_id,
            command.query.organization_id,
            command.query.session_id.as_str(),
            command.path_agent_id.as_str(),
            Some(command.query.user_id),
        )?;
        if session.organization_id != command.query.organization_id {
            return Err(KernelError::validation("session not found"));
        }
        let total_count = self.repository.count_item_feedback(&command.query)?;
        let items = self.repository.list_item_feedback(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn update_item_feedback(
        &self,
        command: UpdateItemFeedbackCommand,
    ) -> KernelResult<ItemFeedbackResult> {
        validate_agent_id(command.path_agent_id.as_str())?;
        validate_standard_id(command.session_id.as_str(), "sessionId", Some("session."))?;
        validate_standard_id(command.item_id.as_str(), "itemId", Some("item."))?;
        self.authorize(
            "agent.business.item_feedback.update",
            command.requested_by.clone(),
            format!("agent.business.session_item.{}.feedback", command.item_id),
            "item_feedback.update",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
            command.path_agent_id.as_str(),
            Some(command.user_id),
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::validation("session item not found"));
        }
        let item = self
            .repository
            .get_session_item(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                command.item_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("session item not found"))?;
        if item.kind != AgentSessionItemKind::AssistantOutput {
            return Err(KernelError::validation(
                "feedback is only supported for assistant items",
            ));
        }
        if let Some(reason_code) = command.reason_code.as_deref() {
            require_non_blank(reason_code, "reasonCode")?;
            if reason_code.len() > 64 {
                return Err(KernelError::validation("reasonCode exceeds 64 bytes"));
            }
            reject_secret_material(reason_code, "reasonCode")?;
        }
        if let Some(comment) = command.comment.as_deref() {
            require_non_blank(comment, "comment")?;
            if comment.len() > 1024 {
                return Err(KernelError::validation("comment exceeds 1024 bytes"));
            }
            reject_secret_material(comment, "comment")?;
        }
        if command.rating.is_none() && (command.reason_code.is_some() || command.comment.is_some())
        {
            return Err(KernelError::validation(
                "clearing feedback cannot include reasonCode or comment",
            ));
        }

        let existing = self.repository.get_item_feedback(
            command.tenant_id,
            command.organization_id,
            command.item_id.as_str(),
            command.user_id,
            true,
        )?;
        let mut record =
            match (existing, command.rating) {
                (Some(mut record), Some(rating)) => {
                    if record.deleted_at.is_none() || command.expected_version.is_some() {
                        ensure_expected_version(
                            record.version,
                            command.expected_version,
                            "message feedback",
                        )?;
                    }
                    record.version = record.version.checked_add(1).ok_or_else(|| {
                        KernelError::conflict("message feedback version overflow")
                    })?;
                    record.rating = rating;
                    record.reason_code = command.reason_code.as_deref().map(trim);
                    record.comment = command.comment.as_deref().map(trim);
                    record.updated_at = command.requested_at.clone();
                    record.deleted_at = None;
                    record
                }
                (None, Some(rating)) => {
                    if command.expected_version.is_some() {
                        return Err(KernelError::conflict("message feedback version mismatch"));
                    }
                    AgentItemFeedbackRecord {
                        id: self.repository.next_id()?,
                        tenant_id: command.tenant_id,
                        organization_id: command.organization_id,
                        item_id: command.item_id.clone(),
                        user_id: command.user_id,
                        rating,
                        reason_code: command.reason_code.as_deref().map(trim),
                        comment: command.comment.as_deref().map(trim),
                        version: 0,
                        created_at: command.requested_at.clone(),
                        updated_at: command.requested_at.clone(),
                        deleted_at: None,
                    }
                }
                (Some(mut record), None) if record.deleted_at.is_none() => {
                    ensure_expected_version(
                        record.version,
                        command.expected_version,
                        "message feedback",
                    )?;
                    record.version = record.version.checked_add(1).ok_or_else(|| {
                        KernelError::conflict("message feedback version overflow")
                    })?;
                    record.updated_at = command.requested_at.clone();
                    record.deleted_at = Some(command.requested_at.clone());
                    record
                }
                (_, None) => {
                    return Err(KernelError::validation("message feedback not found"));
                }
            };
        record.updated_at = command.requested_at.clone();
        let record = self
            .repository
            .upsert_item_feedback(record, command.expected_version)?;
        self.emit_session_item_audit_event(
            AgentAuditAction::ItemFeedbackChanged,
            &item,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    // -----------------------------------------------------------------------
    // Task management
    // -----------------------------------------------------------------------

    pub fn create_task(&self, command: CreateTaskCommand) -> KernelResult<AgentTaskRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let task_id = if is_trimmed_blank(command.task_id.as_str()) {
            format!("task.{}", self.repository.next_id()?)
        } else {
            command.task_id.clone()
        };
        validate_standard_id(task_id.as_str(), "taskId", Some("task."))?;
        self.authorize(
            "agent.business.task.create",
            command.requested_by.clone(),
            format!("agent.business.task.{}", task_id),
            "task.create",
        )?;

        require_non_blank(command.prompt.as_str(), "prompt")?;
        reject_secret_material(command.prompt.as_str(), "prompt")?;
        if command.prompt.len() > MAX_TURN_INPUT_CONTENT_BYTES {
            return Err(KernelError::validation(format!(
                "prompt exceeds maximum size of {MAX_TURN_INPUT_CONTENT_BYTES} bytes"
            )));
        }

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        if self
            .repository
            .get_task(command.tenant_id, task_id.as_str())?
            .is_some()
        {
            return Err(KernelError::conflict("task already exists"));
        }

        if !is_trimmed_blank(command.metadata_json.as_str()) {
            validate_json_payload(command.metadata_json.as_str(), "metadataJson")?;
        }

        let metadata_json = default_json_object_if_blank(command.metadata_json.as_str());

        let record = AgentTaskRecord {
            id: self.repository.next_id()?,
            task_id,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            agent_id: command.agent_id,
            owner_user_id: command.owner_user_id,
            title: command.title,
            prompt: command.prompt,
            status: AgentTaskStatus::Pending,
            external_ref: command.external_ref,
            metadata_json,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            started_at: None,
            completed_at: None,
            cancelled_at: None,
        };

        self.repository.insert_task(record.clone())?;
        self.emit_task_audit_event(
            AgentAuditAction::TaskCreated,
            &record,
            command.requested_by.clone(),
            command.requested_at.clone(),
        )?;

        let record = self.dispatch_task_execution(
            record,
            command.requested_by,
            command.requested_at,
            false,
        )?;
        Ok(record)
    }

    /// HTTP and worker callers defer LLM execution unless `metadataJson.autoExecute` is
    /// `true`. Legacy `deferExecution: true` also defers; `deferExecution: false` opts
    /// into inline execution for backward compatibility.
    fn should_auto_execute_task(metadata_json: &str) -> bool {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(metadata_json) else {
            return false;
        };
        if value
            .get("deferExecution")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        {
            return false;
        }
        if value
            .get("autoExecute")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        {
            return true;
        }
        value
            .get("deferExecution")
            .and_then(serde_json::Value::as_bool)
            == Some(false)
    }

    fn dispatch_task_execution(
        &self,
        mut record: AgentTaskRecord,
        requested_by: PolicySubject,
        requested_at: String,
        force: bool,
    ) -> KernelResult<AgentTaskRecord> {
        if record.status != AgentTaskStatus::Pending {
            return Ok(record);
        }
        if !force && !Self::should_auto_execute_task(record.metadata_json.as_str()) {
            return Ok(record);
        }

        let agent = self
            .repository
            .get(record.tenant_id, record.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        record.mark_running(requested_at.clone());
        self.repository.update_task(record.clone())?;

        let active_binding = self
            .repository
            .get_active_provider_binding(record.tenant_id, record.agent_id.as_str())?;

        let provider_has_model_chat = active_binding
            .as_ref()
            .map(|binding| binding.capabilities.iter().any(|cap| cap == "model.chat"))
            .unwrap_or(false);

        let session_stub = AgentSessionRecord {
            id: record.id,
            session_id: format!("task.session.{}", record.task_id),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            agent_id: record.agent_id.clone(),
            owner_user_id: record.owner_user_id,
            project_id: None,
            session_kind: AgentSessionKind::Automation,
            entry_surface: AgentSessionEntrySurface::Automation,
            source_module: Some("sdkwork-agents".to_string()),
            source_context_kind: Some("agent_task".to_string()),
            source_context_id: Some(record.task_id.clone()),
            parent_session_id: None,
            forked_from_turn_id: None,
            title: record.title.clone(),
            status: AgentSessionStatus::Active,
            item_count: 0,
            last_item_sequence: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            idempotency_key: None,
            payload_hash: None,
            created_by: record.owner_user_id,
            updated_by: record.owner_user_id,
            version: 0,
            created_at: requested_at.clone(),
            updated_at: requested_at.clone(),
            last_item_at: None,
            closed_at: None,
            archived_at: None,
            archived_by: None,
            deleted_at: None,
            deleted_by: None,
            retention_until: None,
        };

        let prompt = record.prompt.clone();
        let completion = complete_with_timeout(
            Arc::clone(&self.turn_executor),
            &TurnExecutionInput {
                agent_display_name: agent.display_name.clone(),
                welcome_message: None,
                session: session_stub,
                history: Vec::new(),
                user_content: prompt,
                model_id: None,
                provider_id: active_binding
                    .as_ref()
                    .map(|binding| binding.provider_id.clone()),
                binding_id: active_binding
                    .as_ref()
                    .map(|binding| binding.binding_id.clone()),
                provider_has_model_chat,
            },
            false,
            TURN_EXECUTION_TIMEOUT,
        );

        if is_capacity_error(completion.runtime_mode) {
            return Err(KernelError::resource_exhausted(completion.content));
        }
        if is_capacity_error(completion.runtime_mode) {
            record.mark_failed(requested_at.clone(), completion.content.as_str());
            self.repository.update_task(record.clone())?;
            return Err(KernelError::resource_exhausted(completion.content));
        }
        if is_inference_error(completion.runtime_mode) {
            record.mark_failed(requested_at.clone(), completion.content.as_str());
            self.repository.update_task(record.clone())?;
            self.emit_task_audit_event(
                AgentAuditAction::TaskFailed,
                &record,
                requested_by,
                requested_at,
            )?;
            return Ok(record);
        }

        record.mark_completed(requested_at.clone(), completion.content.as_str());
        self.repository.update_task(record.clone())?;
        self.emit_task_audit_event(
            AgentAuditAction::TaskCompleted,
            &record,
            requested_by,
            requested_at,
        )?;
        Ok(record)
    }

    pub fn cancel_task(&self, command: CancelTaskCommand) -> KernelResult<AgentTaskRecord> {
        self.authorize(
            "agent.business.task.cancel",
            command.requested_by.clone(),
            format!("agent.business.task.{}", command.task_id),
            "task.cancel",
        )?;
        validate_standard_id(command.task_id.as_str(), "taskId", Some("task."))?;
        validate_agent_id(command.path_agent_id.as_str())?;

        let mut record = self
            .repository
            .get_task(command.tenant_id, command.task_id.as_str())?
            .ok_or_else(|| KernelError::validation("task not found"))?;

        Self::ensure_task_owner_scope(&record, command.owner_scope)?;
        Self::ensure_nested_agent_id(&record.agent_id, command.path_agent_id.as_str(), "task")?;

        if !record.status.is_cancellable() {
            return Err(KernelError::validation("task cannot be cancelled"));
        }

        ensure_expected_version(record.version, command.expected_version, "task")?;

        record.cancel(command.requested_at.clone());
        self.repository.update_task(record.clone())?;
        self.emit_task_audit_event(
            AgentAuditAction::TaskCancelled,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    /// Run LLM execution for a deferred (`pending`) task created without `autoExecute`.
    pub fn execute_task(&self, command: ExecuteTaskCommand) -> KernelResult<AgentTaskRecord> {
        self.authorize(
            "agent.business.task.execute",
            command.requested_by.clone(),
            format!("agent.business.task.{}", command.task_id),
            "task.execute",
        )?;
        validate_standard_id(command.task_id.as_str(), "taskId", Some("task."))?;
        validate_agent_id(command.path_agent_id.as_str())?;

        let record = self
            .repository
            .get_task(command.tenant_id, command.task_id.as_str())?
            .ok_or_else(|| KernelError::validation("task not found"))?;

        Self::ensure_task_owner_scope(&record, command.owner_scope)?;
        Self::ensure_nested_agent_id(&record.agent_id, command.path_agent_id.as_str(), "task")?;

        if record.status != AgentTaskStatus::Pending {
            return Err(KernelError::validation(
                "task is not pending, cannot execute",
            ));
        }

        ensure_expected_version(record.version, command.expected_version, "task")?;

        self.dispatch_task_execution(record, command.requested_by, command.requested_at, true)
    }

    pub fn get_task(&self, command: GetTaskCommand) -> KernelResult<AgentTaskRecord> {
        self.authorize(
            "agent.business.task.retrieve",
            command.requested_by,
            format!("agent.business.task.{}", command.task_id),
            "task.retrieve",
        )?;
        validate_standard_id(command.task_id.as_str(), "taskId", Some("task."))?;
        self.repository
            .get_task(command.tenant_id, command.task_id.as_str())?
            .ok_or_else(|| KernelError::validation("task not found"))
            .and_then(|record| {
                Self::ensure_task_owner_scope(&record, command.owner_scope)?;
                Self::ensure_nested_agent_id(
                    &record.agent_id,
                    command.path_agent_id.as_str(),
                    "task",
                )?;
                Ok(record)
            })
    }

    pub fn list_tasks(
        &self,
        command: ListTasksCommand,
    ) -> KernelResult<PaginatedResult<AgentTaskRecord>> {
        self.authorize(
            "agent.business.task.list",
            command.requested_by,
            format!("agent.business.tenant.{}", command.query.tenant_id),
            "task.list",
        )?;
        let total_count = self.repository.count_tasks(&command.query)?;
        let items = self.repository.list_tasks(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    // -----------------------------------------------------------------------
    // Session runtime bindings
    // -----------------------------------------------------------------------

    pub fn create_session_runtime_binding(
        &self,
        command: CreateSessionRuntimeBindingCommand,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        validate_requested_at(&command.requested_at)?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::validation("session not found"));
        }
        if !session.status.is_active() {
            return Err(KernelError::validation(
                "session is not active, cannot create a runtime binding",
            ));
        }
        self.authorize(
            "agent.business.session_runtime_binding.create",
            command.requested_by.clone(),
            format!(
                "agent.business.session.{}.runtime_binding",
                command.session_id
            ),
            "session_runtime_binding.create",
        )?;
        validate_runtime_token(&command.host_mode, "hostMode", 32)?;
        validate_runtime_token(&command.transport_kind, "transportKind", 64)?;
        require_non_blank(&command.model_id, "modelId")?;
        if command.model_id.len() > 128 {
            return Err(KernelError::validation("modelId exceeds 128 bytes"));
        }
        validate_standard_id(&command.provider_id, "providerId", Some("provider."))?;
        let provider_binding = self
            .repository
            .get_provider_binding(
                command.tenant_id,
                &command.path_agent_id,
                &command.provider_binding_id,
            )?
            .ok_or_else(|| KernelError::validation("agent provider binding not found"))?;
        if provider_binding.provider_id != command.provider_id || !provider_binding.active {
            return Err(KernelError::validation(
                "agent provider binding is not active for providerId",
            ));
        }
        let runtime_binding_id = match command.runtime_binding_id {
            Some(value) => value,
            None => format!("runtime_binding.{}", self.repository.next_id()?),
        };
        validate_standard_id(
            &runtime_binding_id,
            "runtimeBindingId",
            Some("runtime_binding."),
        )?;
        if self
            .repository
            .get_session_runtime_binding(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                &runtime_binding_id,
            )?
            .is_some()
        {
            return Err(KernelError::conflict(
                "session runtime binding already exists",
            ));
        }
        if self
            .repository
            .get_current_session_runtime_binding(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
            )?
            .is_some()
        {
            return Err(KernelError::conflict(
                "session already has a current runtime binding; update or activate an existing binding",
            ));
        }
        validate_optional_bounded(&command.runtime_location_id, "runtimeLocationId", 256)?;
        validate_optional_bounded(&command.native_session_id, "nativeSessionId", 256)?;
        validate_optional_bounded(&command.native_session_tree_id, "nativeSessionTreeId", 256)?;
        validate_optional_bounded(
            &command.native_parent_session_id,
            "nativeParentSessionId",
            256,
        )?;
        validate_optional_bounded(
            &command.native_forked_from_session_id,
            "nativeForkedFromSessionId",
            256,
        )?;
        let record = AgentSessionRuntimeBindingRecord {
            id: self.repository.next_id()?,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            session_id: command.session_id,
            runtime_binding_id,
            runtime_location_id: command.runtime_location_id,
            host_mode: command.host_mode,
            transport_kind: command.transport_kind,
            provider_binding_id: command.provider_binding_id,
            model_id: command.model_id,
            provider_id: command.provider_id,
            native_session_id: command.native_session_id,
            native_session_tree_id: command.native_session_tree_id,
            native_parent_session_id: command.native_parent_session_id,
            native_forked_from_session_id: command.native_forked_from_session_id,
            status: AgentSessionRuntimeBindingStatus::Active,
            is_current: true,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            activated_at: Some(command.requested_at.clone()),
            deactivated_at: None,
        };
        self.repository
            .insert_session_runtime_binding(record.clone())?;
        self.emit_session_resource_audit_event(
            AgentAuditAction::SessionRuntimeBindingCreated,
            "runtime_binding",
            &record.runtime_binding_id,
            &record.session_id,
            record.tenant_id,
            record.organization_id,
            record.version,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn list_session_runtime_bindings(
        &self,
        command: ListSessionRuntimeBindingsCommand,
    ) -> KernelResult<PaginatedResult<SessionRuntimeBindingResult>> {
        self.authorize(
            "agent.business.session_runtime_binding.list",
            command.requested_by,
            format!(
                "agent.business.session.{}.runtime_binding",
                command.query.session_id
            ),
            "session_runtime_binding.list",
        )?;
        let session = self.load_session_for_nested_route(
            command.query.tenant_id,
            command.query.organization_id,
            &command.query.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.query.organization_id {
            return Err(KernelError::validation("session not found"));
        }
        let total_count = self
            .repository
            .count_session_runtime_bindings(&command.query)?;
        let items = self
            .repository
            .list_session_runtime_bindings(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn get_session_runtime_binding(
        &self,
        command: GetSessionRuntimeBindingCommand,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        validate_standard_id(
            &command.runtime_binding_id,
            "runtimeBindingId",
            Some("runtime_binding."),
        )?;
        self.authorize(
            "agent.business.session_runtime_binding.retrieve",
            command.requested_by,
            format!(
                "agent.business.session.{}.runtime_binding.{}",
                command.session_id, command.runtime_binding_id
            ),
            "session_runtime_binding.retrieve",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::validation("session runtime binding not found"));
        }
        self.repository
            .get_session_runtime_binding(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                &command.runtime_binding_id,
            )?
            .ok_or_else(|| KernelError::validation("session runtime binding not found"))
    }

    pub fn update_session_runtime_binding(
        &self,
        command: UpdateSessionRuntimeBindingCommand,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        validate_requested_at(&command.requested_at)?;
        self.authorize(
            "agent.business.session_runtime_binding.update",
            command.requested_by.clone(),
            format!(
                "agent.business.session.{}.runtime_binding.{}",
                command.session_id, command.runtime_binding_id
            ),
            "session_runtime_binding.update",
        )?;
        let mut record = self.get_session_runtime_binding(GetSessionRuntimeBindingCommand {
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            path_agent_id: command.path_agent_id.clone(),
            session_id: command.session_id.clone(),
            runtime_binding_id: command.runtime_binding_id.clone(),
            owner_scope: command.owner_scope,
            requested_by: command.requested_by.clone(),
        })?;
        ensure_expected_version(
            record.version,
            Some(command.expected_version),
            "session runtime binding",
        )?;
        if let Some(runtime_location_id) = command.runtime_location_id {
            validate_optional_bounded(&runtime_location_id, "runtimeLocationId", 256)?;
            record.runtime_location_id = runtime_location_id;
        }
        if let Some(host_mode) = command.host_mode {
            validate_runtime_token(&host_mode, "hostMode", 32)?;
            record.host_mode = host_mode;
        }
        if let Some(transport_kind) = command.transport_kind {
            validate_runtime_token(&transport_kind, "transportKind", 64)?;
            record.transport_kind = transport_kind;
        }
        let provider_binding_id = command
            .provider_binding_id
            .unwrap_or_else(|| record.provider_binding_id.clone());
        let provider_id = command
            .provider_id
            .unwrap_or_else(|| record.provider_id.clone());
        validate_standard_id(&provider_id, "providerId", Some("provider."))?;
        let provider_binding = self
            .repository
            .get_provider_binding(
                command.tenant_id,
                &command.path_agent_id,
                &provider_binding_id,
            )?
            .ok_or_else(|| KernelError::validation("agent provider binding not found"))?;
        if provider_binding.provider_id != provider_id || !provider_binding.active {
            return Err(KernelError::validation(
                "agent provider binding is not active for providerId",
            ));
        }
        record.provider_binding_id = provider_binding_id;
        record.provider_id = provider_id;
        if let Some(model_id) = command.model_id {
            require_non_blank(&model_id, "modelId")?;
            if model_id.len() > 128 {
                return Err(KernelError::validation("modelId exceeds 128 bytes"));
            }
            record.model_id = model_id;
        }
        for (target, value, field) in [
            (
                &mut record.native_session_id,
                command.native_session_id,
                "nativeSessionId",
            ),
            (
                &mut record.native_session_tree_id,
                command.native_session_tree_id,
                "nativeSessionTreeId",
            ),
            (
                &mut record.native_parent_session_id,
                command.native_parent_session_id,
                "nativeParentSessionId",
            ),
            (
                &mut record.native_forked_from_session_id,
                command.native_forked_from_session_id,
                "nativeForkedFromSessionId",
            ),
        ] {
            if let Some(value) = value {
                validate_optional_bounded(&Some(value.clone()), field, 256)?;
                *target = Some(value);
            }
        }
        record.mark_updated(command.requested_at.clone());
        self.repository
            .update_session_runtime_binding(record.clone())?;
        self.emit_session_resource_audit_event(
            AgentAuditAction::SessionRuntimeBindingUpdated,
            "runtime_binding",
            &record.runtime_binding_id,
            &record.session_id,
            record.tenant_id,
            record.organization_id,
            record.version,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn activate_session_runtime_binding(
        &self,
        command: ChangeSessionRuntimeBindingStatusCommand,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        self.change_session_runtime_binding_status(command, true)
    }

    pub fn deactivate_session_runtime_binding(
        &self,
        command: ChangeSessionRuntimeBindingStatusCommand,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        self.change_session_runtime_binding_status(command, false)
    }

    fn change_session_runtime_binding_status(
        &self,
        command: ChangeSessionRuntimeBindingStatusCommand,
        activate: bool,
    ) -> KernelResult<SessionRuntimeBindingResult> {
        validate_requested_at(&command.requested_at)?;
        let action = if activate { "activate" } else { "deactivate" };
        self.authorize(
            format!("agent.business.session_runtime_binding.{action}"),
            command.requested_by.clone(),
            format!(
                "agent.business.session.{}.runtime_binding.{}",
                command.session_id, command.runtime_binding_id
            ),
            format!("session_runtime_binding.{action}"),
        )?;
        let mut record = self.get_session_runtime_binding(GetSessionRuntimeBindingCommand {
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            path_agent_id: command.path_agent_id,
            session_id: command.session_id,
            runtime_binding_id: command.runtime_binding_id,
            owner_scope: command.owner_scope,
            requested_by: command.requested_by.clone(),
        })?;
        ensure_expected_version(
            record.version,
            Some(command.expected_version),
            "session runtime binding",
        )?;
        if activate {
            if record.is_current && record.status == AgentSessionRuntimeBindingStatus::Active {
                return Ok(record);
            }
            record = self.repository.activate_session_runtime_binding_atomic(
                record.tenant_id,
                record.organization_id,
                &record.session_id,
                &record.runtime_binding_id,
                command.expected_version,
                command.requested_at.clone(),
            )?;
        } else {
            if !record.is_current || record.status != AgentSessionRuntimeBindingStatus::Active {
                return Err(KernelError::validation(
                    "session runtime binding is not active",
                ));
            }
            record.deactivate(
                AgentSessionRuntimeBindingStatus::Deactivated,
                command.requested_at.clone(),
            );
            self.repository
                .update_session_runtime_binding(record.clone())?;
        }
        self.emit_session_resource_audit_event(
            if activate {
                AgentAuditAction::SessionRuntimeBindingActivated
            } else {
                AgentAuditAction::SessionRuntimeBindingDeactivated
            },
            "runtime_binding",
            &record.runtime_binding_id,
            &record.session_id,
            record.tenant_id,
            record.organization_id,
            record.version,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    // -----------------------------------------------------------------------
    // Session checkpoints
    // -----------------------------------------------------------------------

    pub fn create_session_checkpoint(
        &self,
        command: CreateSessionCheckpointCommand,
    ) -> KernelResult<SessionCheckpointResult> {
        validate_requested_at(&command.requested_at)?;
        parse_optional_rfc3339_datetime(command.retention_until.as_deref(), "retentionUntil")?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::validation("session not found"));
        }
        if !session.status.is_active() {
            return Err(KernelError::validation(
                "session is not active, cannot create a checkpoint",
            ));
        }
        self.authorize(
            "agent.business.checkpoint.create",
            command.requested_by.clone(),
            format!("agent.business.session.{}.checkpoint", command.session_id),
            "checkpoint.create",
        )?;
        validate_runtime_token(&command.checkpoint_kind, "checkpointKind", 64)?;
        let has_provider =
            command.runtime_binding_id.is_some() || command.provider_checkpoint_ref.is_some();
        let has_drive = command.drive_space_id.is_some() || command.drive_node_id.is_some();
        if has_provider == has_drive {
            return Err(KernelError::validation(
                "checkpoint must use exactly one provider or Drive reference",
            ));
        }
        if has_provider {
            let runtime_binding_id = command.runtime_binding_id.as_deref().ok_or_else(|| {
                KernelError::validation(
                    "runtimeBindingId and providerCheckpointRef must be supplied together",
                )
            })?;
            require_non_blank(
                command.provider_checkpoint_ref.as_deref().ok_or_else(|| {
                    KernelError::validation(
                        "runtimeBindingId and providerCheckpointRef must be supplied together",
                    )
                })?,
                "providerCheckpointRef",
            )?;
            self.repository
                .get_session_runtime_binding(
                    command.tenant_id,
                    command.organization_id,
                    &command.session_id,
                    runtime_binding_id,
                )?
                .ok_or_else(|| KernelError::validation("session runtime binding not found"))?;
        } else {
            require_non_blank(
                command.drive_space_id.as_deref().ok_or_else(|| {
                    KernelError::validation(
                        "driveSpaceId and driveNodeId must be supplied together",
                    )
                })?,
                "driveSpaceId",
            )?;
            require_non_blank(
                command.drive_node_id.as_deref().ok_or_else(|| {
                    KernelError::validation(
                        "driveSpaceId and driveNodeId must be supplied together",
                    )
                })?,
                "driveNodeId",
            )?;
        }
        if let Some(turn_id) = command.turn_id.as_deref() {
            validate_standard_id(turn_id, "turnId", Some("turn."))?;
            let turn = self
                .repository
                .get_turn(command.tenant_id, command.organization_id, turn_id)?
                .ok_or_else(|| KernelError::validation("turn not found"))?;
            if turn.session_id != command.session_id {
                return Err(KernelError::validation("turn not found"));
            }
        }
        let checkpoint_id = match command.checkpoint_id {
            Some(value) => value,
            None => format!("checkpoint.{}", self.repository.next_id()?),
        };
        validate_standard_id(&checkpoint_id, "checkpointId", Some("checkpoint."))?;
        if self
            .repository
            .get_session_checkpoint(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                &checkpoint_id,
            )?
            .is_some()
        {
            return Err(KernelError::conflict("session checkpoint already exists"));
        }
        let record = AgentSessionCheckpointRecord {
            id: self.repository.next_id()?,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            session_id: command.session_id,
            checkpoint_id,
            turn_id: command.turn_id,
            runtime_binding_id: command.runtime_binding_id,
            checkpoint_kind: command.checkpoint_kind,
            provider_checkpoint_ref: command.provider_checkpoint_ref,
            drive_space_id: command.drive_space_id,
            drive_node_id: command.drive_node_id,
            resumable: command.resumable,
            status: AgentSessionCheckpointStatus::Active,
            created_by: session.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            restored_at: None,
            invalidated_at: None,
            retention_until: command.retention_until,
        };
        self.repository.insert_session_checkpoint(record.clone())?;
        self.emit_session_resource_audit_event(
            AgentAuditAction::SessionCheckpointCreated,
            "checkpoint",
            &record.checkpoint_id,
            &record.session_id,
            record.tenant_id,
            record.organization_id,
            record.version,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn list_session_checkpoints(
        &self,
        command: ListSessionCheckpointsCommand,
    ) -> KernelResult<PaginatedResult<SessionCheckpointResult>> {
        self.authorize(
            "agent.business.checkpoint.list",
            command.requested_by,
            format!(
                "agent.business.session.{}.checkpoint",
                command.query.session_id
            ),
            "checkpoint.list",
        )?;
        let session = self.load_session_for_nested_route(
            command.query.tenant_id,
            command.query.organization_id,
            &command.query.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.query.organization_id {
            return Err(KernelError::validation("session not found"));
        }
        let total_count = self.repository.count_session_checkpoints(&command.query)?;
        let items = self.repository.list_session_checkpoints(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn get_session_checkpoint(
        &self,
        command: GetSessionCheckpointCommand,
    ) -> KernelResult<SessionCheckpointResult> {
        validate_standard_id(&command.checkpoint_id, "checkpointId", Some("checkpoint."))?;
        self.authorize(
            "agent.business.checkpoint.retrieve",
            command.requested_by,
            format!(
                "agent.business.session.{}.checkpoint.{}",
                command.session_id, command.checkpoint_id
            ),
            "checkpoint.retrieve",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::validation("session checkpoint not found"));
        }
        self.repository
            .get_session_checkpoint(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                &command.checkpoint_id,
            )?
            .ok_or_else(|| KernelError::validation("session checkpoint not found"))
    }

    pub fn restore_session_checkpoint(
        &self,
        command: ChangeSessionCheckpointStatusCommand,
    ) -> KernelResult<SessionCheckpointResult> {
        self.change_session_checkpoint_status(command, true)
    }

    pub fn invalidate_session_checkpoint(
        &self,
        command: ChangeSessionCheckpointStatusCommand,
    ) -> KernelResult<SessionCheckpointResult> {
        self.change_session_checkpoint_status(command, false)
    }

    fn change_session_checkpoint_status(
        &self,
        command: ChangeSessionCheckpointStatusCommand,
        restore: bool,
    ) -> KernelResult<SessionCheckpointResult> {
        validate_requested_at(&command.requested_at)?;
        let action = if restore { "restore" } else { "invalidate" };
        self.authorize(
            format!("agent.business.checkpoint.{action}"),
            command.requested_by.clone(),
            format!(
                "agent.business.session.{}.checkpoint.{}",
                command.session_id, command.checkpoint_id
            ),
            format!("checkpoint.{action}"),
        )?;
        let mut record = self.get_session_checkpoint(GetSessionCheckpointCommand {
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            path_agent_id: command.path_agent_id,
            session_id: command.session_id,
            checkpoint_id: command.checkpoint_id,
            owner_scope: command.owner_scope,
            requested_by: command.requested_by.clone(),
        })?;
        ensure_expected_version(
            record.version,
            Some(command.expected_version),
            "session checkpoint",
        )?;
        if record.status != AgentSessionCheckpointStatus::Active {
            return Err(KernelError::validation("session checkpoint is not active"));
        }
        if restore {
            if !record.resumable {
                return Err(KernelError::validation(
                    "session checkpoint is not resumable",
                ));
            }
            record.mark_restored(command.requested_at.clone());
        } else {
            record.invalidate(command.requested_at.clone());
        }
        self.repository.update_session_checkpoint(record.clone())?;
        self.emit_session_resource_audit_event(
            if restore {
                AgentAuditAction::SessionCheckpointRestored
            } else {
                AgentAuditAction::SessionCheckpointInvalidated
            },
            "checkpoint",
            &record.checkpoint_id,
            &record.session_id,
            record.tenant_id,
            record.organization_id,
            record.version,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    // -----------------------------------------------------------------------
    // Session item and turn management
    // -----------------------------------------------------------------------

    /// Low-level item insert for tests and internal tooling. HTTP surfaces use
    /// [`Self::execute_turn`] which persists input and output items atomically.
    pub fn create_session_item(
        &self,
        command: CreateSessionItemCommand,
    ) -> KernelResult<AgentSessionItemRecord> {
        validate_standard_id(command.item_id.as_str(), "itemId", Some("item."))?;
        self.authorize(
            "agent.business.session_item.create",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "session_item.create",
        )?;

        // Ensure session exists and is active
        let mut session = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("session not found"))?;

        if !session.status.is_active() {
            return Err(KernelError::validation(
                "session is not active, cannot create an item",
            ));
        }

        // Ensure the item does not already exist.
        if self
            .repository
            .get_session_item(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                command.item_id.as_str(),
            )?
            .is_some()
        {
            return Err(KernelError::conflict("session item already exists"));
        }

        require_non_blank(command.content.as_str(), "content")?;
        if command.content.len() > MAX_TURN_INPUT_CONTENT_BYTES {
            return Err(KernelError::validation(format!(
                "content exceeds maximum size of {MAX_TURN_INPUT_CONTENT_BYTES} bytes"
            )));
        }
        reject_secret_material(command.content.as_str(), "content")?;

        let sequence = self.repository.next_item_sequence(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
        )?;

        let record = AgentSessionItemRecord {
            id: self.repository.next_id()?,
            item_id: command.item_id,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            session_id: command.session_id.clone(),
            kind: command.kind,
            content: Some(command.content),
            content_type: default_plain_text_if_blank(command.content_type.as_str()),
            status: AgentSessionItemStatus::Completed,
            sequence,
            input_tokens: command.input_tokens,
            output_tokens: command.output_tokens,
            model_id: command.model_id,
            provider_id: command.provider_id,
            tool_name: None,
            tool_call_id: None,
            tool_arguments_json: None,
            tool_result_json: None,
            parent_item_id: command.parent_item_id,
            turn_id: None,
            created_by: session.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            completed_at: Some(command.requested_at.clone()),
            redacted_at: None,
            redacted_by: None,
            retention_until: None,
        };

        self.repository.insert_session_item(record.clone())?;

        // Update session counters
        session.record_item(
            command.input_tokens,
            command.output_tokens,
            command.requested_at.clone(),
        );
        self.repository.update_session(session)?;

        self.emit_session_item_audit_event(
            AgentAuditAction::SessionItemCreated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn get_session_item(
        &self,
        command: GetSessionItemCommand,
    ) -> KernelResult<AgentSessionItemRecord> {
        self.authorize(
            "agent.business.session_item.retrieve",
            command.requested_by,
            format!("agent.business.session.{}", command.session_id),
            "session_item.retrieve",
        )?;
        validate_standard_id(command.item_id.as_str(), "itemId", Some("item."))?;
        let session = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("session not found"))?;
        Self::ensure_session_owner_scope(&session, command.owner_scope)?;
        Self::ensure_nested_agent_id(&session.agent_id, command.path_agent_id.as_str(), "session")?;
        self.repository
            .get_session_item(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                command.item_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("session item not found"))
    }

    pub fn list_session_items(
        &self,
        command: ListSessionItemsCommand,
    ) -> KernelResult<PaginatedResult<AgentSessionItemRecord>> {
        self.authorize(
            "agent.business.session_item.list",
            command.requested_by,
            format!("agent.business.session.{}", command.query.session_id),
            "session_item.list",
        )?;
        let session = self
            .repository
            .get_session(
                command.query.tenant_id,
                command.query.organization_id,
                command.query.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("session not found"))?;
        Self::ensure_session_owner_scope(&session, command.owner_scope)?;
        let total_count = self.repository.count_session_items(&command.query)?;
        let items = self.repository.list_session_items(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn list_session_items_with_drive_refs(
        &self,
        command: ListSessionItemsCommand,
    ) -> KernelResult<PaginatedResult<AgentSessionItemWithDriveRefs>> {
        let tenant_id = command.query.tenant_id;
        let organization_id = command.query.organization_id;
        let page = self.list_session_items(command)?;
        let item_ids = page
            .items
            .iter()
            .map(|item| item.item_id.clone())
            .collect::<Vec<_>>();
        let mut refs_by_item = HashMap::<String, Vec<AgentItemDriveRefRecord>>::new();
        for drive_ref in
            self.repository
                .list_item_drive_refs_batch(tenant_id, organization_id, &item_ids)?
        {
            refs_by_item
                .entry(drive_ref.item_id.clone())
                .or_default()
                .push(drive_ref);
        }
        let items = page
            .items
            .into_iter()
            .map(|item| AgentSessionItemWithDriveRefs {
                drive_refs: refs_by_item.remove(&item.item_id).unwrap_or_default(),
                item,
            })
            .collect();
        Ok(PaginatedResult {
            items,
            has_more: page.has_more,
            next_page_token: page.next_page_token,
            total_count: page.total_count,
        })
    }

    pub fn get_session_item_with_drive_refs(
        &self,
        command: GetSessionItemCommand,
    ) -> KernelResult<AgentSessionItemWithDriveRefs> {
        let tenant_id = command.tenant_id;
        let organization_id = command.organization_id;
        let item = self.get_session_item(command)?;
        let drive_refs =
            self.repository
                .list_item_drive_refs(tenant_id, organization_id, &item.item_id)?;
        Ok(AgentSessionItemWithDriveRefs { item, drive_refs })
    }

    pub fn get_turn(&self, command: GetTurnCommand) -> KernelResult<AgentTurnRecord> {
        validate_agent_id(&command.path_agent_id)?;
        validate_standard_id(&command.session_id, "sessionId", Some("session."))?;
        validate_standard_id(&command.turn_id, "turnId", Some("turn."))?;
        self.authorize(
            "agent.business.turn.retrieve",
            command.requested_by,
            format!("agent.business.turn.{}", command.turn_id),
            "turn.retrieve",
        )?;
        let session = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
            )?
            .ok_or_else(|| KernelError::validation("session not found"))?;
        Self::ensure_session_owner_scope(&session, command.owner_scope)?;
        if session.organization_id != command.organization_id
            || session.agent_id != command.path_agent_id
        {
            return Err(KernelError::validation("turn not found"));
        }
        let turn = self
            .repository
            .get_turn(command.tenant_id, command.organization_id, &command.turn_id)?
            .ok_or_else(|| KernelError::validation("turn not found"))?;
        if turn.session_id != command.session_id || turn.agent_id != command.path_agent_id {
            return Err(KernelError::validation("turn not found"));
        }
        Ok(turn)
    }

    pub fn list_turns(
        &self,
        command: ListTurnsCommand,
    ) -> KernelResult<PaginatedResult<AgentTurnRecord>> {
        self.authorize(
            "agent.business.turn.list",
            command.requested_by,
            format!("agent.business.session.{}.turn", command.query.session_id),
            "turn.list",
        )?;
        let session = self.load_session_for_nested_route(
            command.query.tenant_id,
            command.query.organization_id,
            &command.query.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.query.organization_id {
            return Err(KernelError::validation("session not found"));
        }
        let total_count = self.repository.count_turns(&command.query)?;
        let items = self.repository.list_turns(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn get_turn_by_idempotency(
        &self,
        command: GetTurnByIdempotencyCommand,
    ) -> KernelResult<Option<AgentTurnRecord>> {
        validate_agent_id(&command.path_agent_id)?;
        validate_standard_id(&command.session_id, "sessionId", Some("session."))?;
        require_non_blank(&command.idempotency_key, "idempotencyKey")?;
        if command.idempotency_key.len() > 256 {
            return Err(KernelError::validation("idempotencyKey exceeds 256 bytes"));
        }
        self.authorize(
            "agent.business.turn.retrieve",
            command.requested_by,
            format!(
                "agent.business.turn.idempotency.{}",
                command.idempotency_key
            ),
            "turn.retrieve",
        )?;
        let session = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
            )?
            .ok_or_else(|| KernelError::validation("session not found"))?;
        Self::ensure_session_owner_scope(&session, Some(command.owner_user_id))?;
        if session.organization_id != command.organization_id
            || session.agent_id != command.path_agent_id
        {
            return Err(KernelError::validation("turn not found"));
        }
        let turn = self.repository.get_turn_by_idempotency(
            command.tenant_id,
            command.organization_id,
            command.owner_user_id,
            &command.idempotency_key,
        )?;
        if let Some(turn) = turn.as_ref() {
            if turn.session_id != command.session_id || turn.agent_id != command.path_agent_id {
                return Err(KernelError::validation("turn not found"));
            }
        }
        Ok(turn)
    }

    pub fn cancel_turn(&self, command: CancelTurnCommand) -> KernelResult<AgentTurnRecord> {
        let audit_subject = command.requested_by.clone();
        self.authorize(
            "agent.business.turn.cancel",
            command.requested_by.clone(),
            format!("agent.business.turn.{}", command.turn_id),
            "turn.cancel",
        )?;
        let mut turn = self.get_turn(GetTurnCommand {
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            path_agent_id: command.path_agent_id,
            session_id: command.session_id,
            turn_id: command.turn_id,
            owner_scope: command.owner_scope,
            requested_by: command.requested_by,
        })?;
        if let Some(expected_version) = command.expected_version {
            if expected_version != turn.version {
                return Err(KernelError::conflict("turn version mismatch"));
            }
        }
        if !matches!(
            turn.status,
            AgentTurnStatus::Requested | AgentTurnStatus::Running
        ) {
            return Err(KernelError::validation("turn cannot be cancelled"));
        }
        let expected_version = turn.version;
        turn.mark_cancelled(command.requested_at.clone());
        let turn = self.repository.update_turn_state(turn, expected_version)?;
        self.emit_turn_audit_event(
            AgentAuditAction::TurnCancelRequested,
            &turn,
            audit_subject.clone(),
            command.requested_at.clone(),
        )?;
        self.emit_turn_audit_event(
            AgentAuditAction::TurnCancelled,
            &turn,
            audit_subject,
            command.requested_at,
        )?;
        Ok(turn)
    }

    pub fn reconcile_stale_turns(
        &self,
        stale_before: &str,
        occurred_at: &str,
        limit: usize,
    ) -> KernelResult<TurnReconciliationResult> {
        if is_trimmed_blank(stale_before) || is_trimmed_blank(occurred_at) {
            return Err(KernelError::validation(
                "stale_before and occurred_at are required",
            ));
        }
        let turns = self
            .repository
            .list_reconcilable_turns(stale_before, limit.clamp(1, 200))?;
        let examined = turns.len();
        let mut failed = Vec::with_capacity(examined);
        let mut skipped_conflicts = 0usize;
        for mut turn in turns {
            let expected_version = turn.version;
            turn.mark_failed(
                "turn_reconciliation_timeout",
                "turn did not reach a terminal state before the reconciliation deadline",
                occurred_at,
            );
            match self.repository.update_turn_state(turn, expected_version) {
                Ok(record) => {
                    self.emit_turn_audit_event(
                        AgentAuditAction::TurnFailed,
                        &record,
                        PolicySubject::new(
                            "system.agents.reconciliation",
                            record.tenant_id.to_string(),
                        ),
                        occurred_at.to_string(),
                    )?;
                    failed.push(record);
                }
                Err(error) if error.kind() == KernelErrorKind::Conflict => {
                    skipped_conflicts = skipped_conflicts.saturating_add(1);
                }
                Err(error) => return Err(error),
            }
        }
        Ok(TurnReconciliationResult {
            examined,
            failed,
            skipped_conflicts,
        })
    }

    /// Execute one turn and atomically persist its input, output and session counters.
    pub fn execute_turn(&self, command: CreateTurnCommand) -> KernelResult<TurnExecutionResult> {
        validate_agent_id(command.agent_id.as_str())?;
        validate_standard_id(command.session_id.as_str(), "sessionId", Some("session."))?;
        self.authorize(
            "agent.business.turn.create",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "turn.create",
        )?;

        let agent = self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())?
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        let mut session = self
            .repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("session not found"))?;

        Self::ensure_session_owner_scope(&session, command.owner_scope)?;

        if session.agent_id != command.agent_id
            || session.organization_id != command.organization_id
        {
            return Err(KernelError::validation("session not found"));
        }
        if !session.status.is_active() {
            return Err(KernelError::validation(
                "session is not active, cannot execute a turn",
            ));
        }

        require_non_blank(command.content.as_str(), "content")?;
        if command.content.len() > MAX_TURN_INPUT_CONTENT_BYTES {
            return Err(KernelError::validation(format!(
                "content exceeds maximum size of {MAX_TURN_INPUT_CONTENT_BYTES} bytes"
            )));
        }
        reject_secret_material(command.content.as_str(), "content")?;
        let normalized_drive_refs = normalize_item_drive_resources(&command.drive_refs)?;
        let idempotency_key = command.idempotency_key.trim().to_string();
        require_non_blank(&idempotency_key, "idempotencyKey")?;
        if idempotency_key.len() > 256 {
            return Err(KernelError::validation("idempotencyKey exceeds 256 bytes"));
        }
        let payload_hash = command.payload_hash.trim().to_string();
        require_non_blank(&payload_hash, "payloadHash")?;
        if payload_hash.len() > 128 {
            return Err(KernelError::validation("payloadHash exceeds 128 bytes"));
        }
        if let Some(existing_turn) = self.repository.get_turn_by_idempotency(
            command.tenant_id,
            session.organization_id,
            session.owner_user_id,
            &idempotency_key,
        )? {
            if existing_turn.payload_hash != payload_hash {
                return Err(KernelError::conflict(
                    "idempotency key was already used with a different payload",
                ));
            }
            if existing_turn.status != AgentTurnStatus::Completed {
                return Err(KernelError::conflict("turn is not completed"));
            }
            let response_item_id =
                existing_turn
                    .response_item_id
                    .as_deref()
                    .ok_or_else(|| KernelError::Internal {
                        message: "completed turn is missing response_item_id".to_string(),
                    })?;
            let user_input_item = self
                .repository
                .get_session_item(
                    command.tenant_id,
                    command.organization_id,
                    &command.session_id,
                    &existing_turn.request_item_id,
                )?
                .ok_or_else(|| KernelError::Internal {
                    message: "completed turn is missing request item".to_string(),
                })?;
            let assistant_output_item = self
                .repository
                .get_session_item(
                    command.tenant_id,
                    command.organization_id,
                    &command.session_id,
                    response_item_id,
                )?
                .ok_or_else(|| KernelError::Internal {
                    message: "completed turn is missing response item".to_string(),
                })?;
            let user_item_drive_refs = self.repository.list_item_drive_refs(
                command.tenant_id,
                session.organization_id,
                &user_input_item.item_id,
            )?;
            return Ok(TurnExecutionResult {
                session,
                turn: existing_turn,
                user_input_item,
                assistant_output_item,
                user_item_drive_refs,
                stream_deltas: Vec::new(),
            });
        }

        let session_runtime_binding = match command.runtime_binding_id.as_deref() {
            Some(runtime_binding_id) => self.repository.get_session_runtime_binding(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                runtime_binding_id,
            )?,
            None => self.repository.get_current_session_runtime_binding(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
            )?,
        }
        .ok_or_else(|| KernelError::validation("active session runtime binding not found"))?;
        if !session_runtime_binding.is_current
            || session_runtime_binding.status != AgentSessionRuntimeBindingStatus::Active
        {
            return Err(KernelError::validation(
                "session runtime binding is not active",
            ));
        }
        if let Some(requested_model_id) = command.requested_model_id.as_deref() {
            if requested_model_id != session_runtime_binding.model_id {
                return Err(KernelError::validation(
                    "requestedModelId does not match the active session runtime binding",
                ));
            }
        }
        let provider_binding = self
            .repository
            .get_provider_binding(
                command.tenant_id,
                command.agent_id.as_str(),
                &session_runtime_binding.provider_binding_id,
            )?
            .ok_or_else(|| KernelError::validation("agent provider binding not found"))?;
        if !provider_binding.active
            || provider_binding.provider_id != session_runtime_binding.provider_id
        {
            return Err(KernelError::validation(
                "session runtime binding references an inactive provider binding",
            ));
        }
        let turn_id = match command.turn_id.as_deref() {
            Some(turn_id) => {
                validate_standard_id(turn_id, "turnId", Some("turn."))?;
                turn_id.to_string()
            }
            None => format!("turn.{}", self.repository.next_id()?),
        };
        let user_input_item_id = format!("item.{}", self.repository.next_id()?);
        let assistant_output_item_id = format!("item.{}", self.repository.next_id()?);
        let mut turn = AgentTurnRecord {
            id: self.repository.next_id()?,
            turn_id: turn_id.clone(),
            tenant_id: command.tenant_id,
            organization_id: session.organization_id,
            session_id: command.session_id.clone(),
            agent_id: command.agent_id.clone(),
            owner_user_id: session.owner_user_id,
            runtime_binding_id: Some(session_runtime_binding.runtime_binding_id.clone()),
            client_request_id: command.client_request_id.clone(),
            idempotency_key: idempotency_key.clone(),
            payload_hash: payload_hash.clone(),
            request_item_id: user_input_item_id.clone(),
            response_item_id: None,
            turn_mode: command.turn_mode,
            status: AgentTurnStatus::Requested,
            requested_model_id: command.requested_model_id.clone(),
            provider_binding_id: Some(session_runtime_binding.provider_binding_id.clone()),
            model_id: Some(session_runtime_binding.model_id.clone()),
            provider_id: Some(session_runtime_binding.provider_id.clone()),
            input_tokens: 0,
            output_tokens: 0,
            cached_tokens: 0,
            finish_reason: None,
            error_code: None,
            error_detail: None,
            trace_id: command.client_request_id.clone(),
            attempt_count: 0,
            max_attempts: 3,
            next_retry_at: None,
            available_at: command.requested_at.clone(),
            lease_owner: None,
            lease_token: None,
            lease_expires_at: None,
            fencing_token: 0,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            started_at: None,
            completed_at: None,
            cancel_requested_at: None,
            cancelled_at: None,
            retention_until: None,
        };
        self.repository.insert_turn_reservation(turn.clone())?;
        self.emit_turn_audit_event(
            AgentAuditAction::TurnRequested,
            &turn,
            command.requested_by.clone(),
            command.requested_at.clone(),
        )?;
        turn.mark_running(command.requested_at.clone());
        turn = self.repository.update_turn_state(turn, 0)?;

        let history_items =
            self.repository
                .list_session_items(&SessionItemListQuery::for_recent_turn_context(
                    command.tenant_id,
                    command.organization_id,
                    command.session_id.clone(),
                    TURN_CONTEXT_ITEM_LIMIT,
                ))?;
        let history = history_items
            .iter()
            .filter_map(|record| record.content.clone().map(|content| (record.kind, content)))
            .collect::<Vec<_>>();

        let welcome_message = AgentManagementProfileDto::from_default_code_task_intent(
            agent.default_code_task_intent.as_ref(),
        )
        .and_then(|profile| profile.welcome_message);
        let provider_has_model_chat = provider_binding
            .capabilities
            .iter()
            .any(|capability| capability == "model.chat");

        let user_content = command.content.clone();
        let completion = complete_with_timeout(
            Arc::clone(&self.turn_executor),
            &TurnExecutionInput {
                agent_display_name: agent.display_name.clone(),
                welcome_message,
                session: session.clone(),
                history,
                user_content: user_content.clone(),
                model_id: Some(session_runtime_binding.model_id.clone()),
                provider_id: Some(session_runtime_binding.provider_id.clone()),
                binding_id: Some(session_runtime_binding.provider_binding_id.clone()),
                provider_has_model_chat,
            },
            command.prefer_stream,
            TURN_EXECUTION_TIMEOUT,
        );
        if is_inference_error(completion.runtime_mode) {
            turn.mark_failed(
                "turn_inference_failed",
                "managed turn inference failed",
                command.requested_at.clone(),
            );
            let failed_turn = self.repository.update_turn_state(turn, 1)?;
            self.emit_turn_audit_event(
                AgentAuditAction::TurnFailed,
                &failed_turn,
                command.requested_by,
                command.requested_at,
            )?;
            return Err(KernelError::provider_error(
                "turn_inference_failed",
                completion.content,
            ));
        }

        let user_input_item = AgentSessionItemRecord {
            id: self.repository.next_id()?,
            item_id: user_input_item_id,
            tenant_id: command.tenant_id,
            organization_id: session.organization_id,
            session_id: command.session_id.clone(),
            kind: AgentSessionItemKind::UserInput,
            content: Some(user_content),
            content_type: default_plain_text_if_blank(command.content_type.as_str()),
            status: AgentSessionItemStatus::Completed,
            sequence: 0,
            input_tokens: 0,
            output_tokens: 0,
            model_id: None,
            provider_id: None,
            tool_name: None,
            tool_call_id: None,
            tool_arguments_json: None,
            tool_result_json: None,
            parent_item_id: None,
            turn_id: Some(turn_id.clone()),
            created_by: session.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            completed_at: Some(command.requested_at.clone()),
            redacted_at: None,
            redacted_by: None,
            retention_until: None,
        };
        let user_item_drive_refs = normalized_drive_refs
            .into_iter()
            .map(|resource| {
                Ok(AgentItemDriveRefRecord {
                    id: self.repository.next_id()?,
                    tenant_id: command.tenant_id,
                    organization_id: session.organization_id,
                    item_id: user_input_item.item_id.clone(),
                    resource_role: resource.resource_role,
                    drive_space_id: resource.drive_space_id,
                    drive_node_id: resource.drive_node_id,
                    media_resource_id: None,
                    object_blob_id: None,
                    resource_hash: None,
                    alt_text: None,
                    sort_order: resource.sort_order,
                    status: 0,
                    created_by: session.owner_user_id,
                    created_at: command.requested_at.clone(),
                    updated_at: command.requested_at.clone(),
                    deleted_at: None,
                    retention_until: None,
                })
            })
            .collect::<KernelResult<Vec<_>>>()?;

        let assistant_output_item = AgentSessionItemRecord {
            id: self.repository.next_id()?,
            item_id: assistant_output_item_id,
            tenant_id: command.tenant_id,
            organization_id: session.organization_id,
            session_id: command.session_id.clone(),
            kind: AgentSessionItemKind::AssistantOutput,
            content: Some(completion.content),
            content_type: "text/plain".to_string(),
            status: AgentSessionItemStatus::Completed,
            sequence: 0,
            input_tokens: completion.input_tokens,
            output_tokens: completion.output_tokens,
            model_id: Some(
                completion
                    .model_id
                    .clone()
                    .unwrap_or_else(|| session_runtime_binding.model_id.clone()),
            ),
            provider_id: Some(
                completion
                    .provider_id
                    .clone()
                    .unwrap_or_else(|| session_runtime_binding.provider_id.clone()),
            ),
            tool_name: None,
            tool_call_id: None,
            tool_arguments_json: None,
            tool_result_json: None,
            parent_item_id: Some(user_input_item.item_id.clone()),
            turn_id: Some(turn_id.clone()),
            created_by: session.owner_user_id,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            completed_at: Some(command.requested_at.clone()),
            redacted_at: None,
            redacted_by: None,
            retention_until: None,
        };

        session.record_turn(
            completion.input_tokens,
            completion.output_tokens,
            command.requested_at.clone(),
        );

        turn.response_item_id = Some(assistant_output_item.item_id.clone());
        turn.model_id = assistant_output_item.model_id.clone();
        turn.provider_id = assistant_output_item.provider_id.clone();
        turn.input_tokens = completion.input_tokens;
        turn.output_tokens = completion.output_tokens;
        turn.mark_completed(command.requested_at.clone());
        let completed_turn = turn.clone();

        let (session, user_input_item, assistant_output_item) =
            self.repository.insert_turn_with_drive_refs(
                turn,
                session,
                user_input_item,
                assistant_output_item,
                user_item_drive_refs.clone(),
            )?;

        self.emit_turn_audit_event(
            AgentAuditAction::TurnCompleted,
            &completed_turn,
            command.requested_by.clone(),
            command.requested_at.clone(),
        )?;

        self.emit_session_item_audit_event(
            AgentAuditAction::SessionItemCreated,
            &user_input_item,
            command.requested_by.clone(),
            command.requested_at.clone(),
        )?;
        self.emit_session_item_audit_event(
            AgentAuditAction::SessionItemCreated,
            &assistant_output_item,
            command.requested_by,
            command.requested_at,
        )?;

        Ok(TurnExecutionResult {
            session,
            turn: completed_turn,
            user_input_item,
            assistant_output_item,
            user_item_drive_refs,
            stream_deltas: completion.stream_deltas,
        })
    }

    fn authorize(
        &self,
        request_id: impl Into<String>,
        subject: PolicySubject,
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> KernelResult<()> {
        let policy_request = PolicyRequest::new(
            request_id,
            DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY,
            resource,
        )
        .with_category(PolicyCategory::ProductSpecific(
            DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY.to_string(),
        ))
        .with_subject(subject)
        .with_action(action)
        .with_redaction(KernelEventRedaction::TenantSensitive);

        let decision = self.policy_provider.evaluate(policy_request)?;
        if decision.decision != PolicyDecisionValue::Allow {
            return Err(KernelError::permission_required(
                decision
                    .safe_reason
                    .unwrap_or_else(|| "agent management denied".to_string()),
            ));
        }
        Ok(())
    }

    fn emit_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentBusinessRecord,
        previous_status: Option<AgentBusinessStatus>,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let mut audit_payload = AgentAuditPayload::new(action, record);
        if let Some(previous_status) = previous_status {
            audit_payload = audit_payload.with_previous_status(previous_status);
        }
        let payload_json = audit_payload.to_json().map_err(|error| {
            KernelError::validation(format!("audit payload serialization: {error}"))
        })?;

        let event = KernelEvent::new(
            format!("agent_audit_{}_{}", record.agent_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", AgentAuditPayload::SCHEMA_VERSION)
        .with_context("audit_action", action.action_code())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .with_context("agent_internal_id", record.id.to_string().as_str())
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.audit.v1");

        self.audit_sink.record(event)
    }

    fn deactivate_provider_bindings(
        &self,
        tenant_id: u64,
        agent_id: &str,
        updated_at: String,
    ) -> KernelResult<()> {
        for mut binding in self.all_provider_bindings_for_agent(tenant_id, agent_id)? {
            if binding.active {
                binding.active = false;
                binding.mark_updated(updated_at.clone());
                self.repository.update_provider_binding(binding)?;
            }
        }
        Ok(())
    }

    fn emit_project_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentProjectRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let payload_json = serde_json::json!({
            "schemaVersion": "v1",
            "projectId": record.project_id,
            "tenantId": record.tenant_id.to_string(),
            "organizationId": record.organization_id.to_string(),
            "ownerUserId": record.owner_user_id.to_string(),
            "status": record.status.as_str(),
            "visibility": record.visibility.as_str(),
            "version": record.version.to_string(),
        })
        .to_string();
        let event = KernelEvent::new(
            format!("agent_project_{}_{}", record.project_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", "v1")
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "project")
        .with_context("aggregate_id", record.project_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.project.audit.v1");
        self.audit_sink.record(event)
    }

    fn emit_project_composition_slot_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentProjectCompositionSlotRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let payload_json = serde_json::json!({
            "schemaVersion": "v1",
            "projectId": record.project_id,
            "slotId": record.slot_id,
            "slotKind": record.slot_kind.as_str(),
            "targetModule": record.target_module.as_str(),
            "targetRef": record.target_ref,
            "enabled": record.enabled,
            "priority": record.priority,
            "version": record.version.to_string(),
        })
        .to_string();
        let event = KernelEvent::new(
            format!(
                "agent_project_slot_{}_{}_{}",
                record.project_id, record.slot_id, record.version
            ),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", "v1")
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "project_composition_slot")
        .with_context("aggregate_id", record.slot_id.as_str())
        .with_context("project_id", record.project_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.project-composition-slot.audit.v1");
        self.audit_sink.record(event)
    }

    fn emit_turn_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentTurnRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let payload_json = serde_json::json!({
            "schemaVersion": "v1",
            "turnId": record.turn_id,
            "sessionId": record.session_id,
            "agentId": record.agent_id,
            "status": record.status.as_str(),
            "errorCode": record.error_code,
            "version": record.version.to_string(),
        })
        .to_string();
        let event = KernelEvent::new(
            format!(
                "agent_turn_{}_{}_{}",
                record.turn_id,
                action.action_code(),
                record.version
            ),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", "v1")
        .with_context("audit_action", action.action_code())
        .with_context("aggregate_type", "turn")
        .with_context("aggregate_id", record.turn_id.as_str())
        .with_context("session_id", record.session_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.chat-turn.audit.v1");
        self.audit_sink.record(event)
    }

    fn emit_binding_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentProviderBindingRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        // Use structured JSON payload for provider binding events
        let audit_payload = ProviderBindingAuditPayload::new(action, record);
        let payload_json = audit_payload.to_json().map_err(|error| {
            KernelError::validation(format!("binding audit payload serialization: {error}"))
        })?;

        let event = KernelEvent::new(
            format!("agent_binding_{}_{}", record.binding_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context(
            "schema_version",
            ProviderBindingAuditPayload::SCHEMA_VERSION,
        )
        .with_context("audit_action", action.action_code())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context("binding_id", record.binding_id.as_str())
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.provider_binding.v1");

        self.audit_sink.record(event)
    }

    fn emit_runtime_execution_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentRuntimeExecutionRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        // Use structured JSON payload for runtime execution events
        let audit_payload = RuntimeExecutionAuditPayload::new(action, record);
        let payload_json = audit_payload.to_json().map_err(|error| {
            KernelError::validation(format!(
                "runtime execution audit payload serialization: {error}"
            ))
        })?;

        let event = KernelEvent::new(
            format!("agent_runtime_execution_{}", record.execution_id),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context(
            "schema_version",
            RuntimeExecutionAuditPayload::SCHEMA_VERSION,
        )
        .with_context("audit_action", action.action_code())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.runtime_execution.v1");

        self.audit_sink.record(event)
    }

    fn emit_marketplace_audit_event(
        &self,
        input: AgentBusinessAuditEventInput<'_>,
    ) -> KernelResult<()> {
        // Use structured JSON payload for marketplace events
        let audit_payload =
            MarketplaceAuditPayload::new(crate::domain::MarketplaceAuditPayloadInput {
                action: input.action,
                item_kind: input.item_kind,
                item_id: input.item_id,
                tenant_id: input.tenant_id,
                organization_id: input.organization_id,
                status: input.status,
                visibility: input.visibility,
                version: input.version,
            });
        let payload_json = audit_payload.to_json().map_err(|error| {
            KernelError::validation(format!("marketplace audit payload serialization: {error}"))
        })?;

        let event = KernelEvent::new(
            format!(
                "agent_marketplace_{}_{}_{}",
                input.item_kind, input.item_id, input.version
            ),
            input.action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", MarketplaceAuditPayload::SCHEMA_VERSION)
        .with_context("audit_action", input.action.action_code())
        .with_context("aggregate_type", "marketplace")
        .with_context("aggregate_id", input.item_id)
        .with_context("subject_id", input.subject.subject_id.as_str())
        .with_context("subject_tenant_id", input.subject.tenant_id.as_str())
        .with_context("tenant_id", input.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            input.organization_id.to_string().as_str(),
        )
        .occurred_at(input.occurred_at)
        .with_payload_schema("sdkwork.agent.business.marketplace.v1");

        self.audit_sink.record(event)
    }

    fn emit_session_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentSessionRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let audit_payload = SessionAuditPayload::new(action, record);
        let payload_json = audit_payload.to_json().map_err(|error| {
            KernelError::validation(format!("session audit payload serialization: {error}"))
        })?;

        let event = KernelEvent::new(
            format!("agent_session_{}_{}", record.session_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", SessionAuditPayload::SCHEMA_VERSION)
        .with_context("audit_action", action.action_code())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("session_id", record.session_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.session.v1");

        self.audit_sink.record(event)
    }

    fn emit_session_item_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentSessionItemRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let audit_payload = SessionItemAuditPayload::new(action, record);
        let payload_json = audit_payload.to_json().map_err(|error| {
            KernelError::validation(format!("session-item audit payload serialization: {error}"))
        })?;

        let event = KernelEvent::new(
            format!("agent_session_item_{}_{}", record.item_id, record.sequence),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", SessionItemAuditPayload::SCHEMA_VERSION)
        .with_context("audit_action", action.action_code())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("item_id", record.item_id.as_str())
        .with_context("session_id", record.session_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.session_item.v1");

        self.audit_sink.record(event)
    }

    fn emit_task_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentTaskRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let audit_payload = TaskAuditPayload::new(action, record);
        let payload_json = audit_payload.to_json().map_err(|error| {
            KernelError::validation(format!("task audit payload serialization: {error}"))
        })?;

        let event = KernelEvent::new(
            format!("agent_task_{}_{}", record.task_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", TaskAuditPayload::SCHEMA_VERSION)
        .with_context("audit_action", action.action_code())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("task_id", record.task_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.task.v1");

        self.audit_sink.record(event)
    }

    // -----------------------------------------------------------------------
    // Live interaction operations
    // -----------------------------------------------------------------------

    pub fn create_interaction(
        &self,
        command: CreateInteractionCommand,
    ) -> KernelResult<AgentInteractionRecord> {
        validate_requested_at(&command.requested_at)?;
        parse_optional_rfc3339_datetime(command.retention_until.as_deref(), "retentionUntil")?;
        validate_standard_id(command.session_id.as_str(), "sessionId", Some("session."))?;
        validate_agent_id(command.path_agent_id.as_str())?;
        let interaction_id = if is_trimmed_blank(command.interaction_id.as_str()) {
            format!("interaction.{}", self.repository.next_id()?)
        } else {
            command.interaction_id.clone()
        };
        validate_standard_id(
            interaction_id.as_str(),
            "interactionId",
            Some("interaction."),
        )?;
        self.authorize(
            "agent.business.interaction.create",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "interaction.create",
        )?;

        require_non_blank(command.prompt.as_str(), "prompt")?;
        reject_secret_material(command.prompt.as_str(), "prompt")?;
        if command.prompt.len() > MAX_TURN_INPUT_CONTENT_BYTES {
            return Err(KernelError::validation(format!(
                "prompt exceeds maximum size of {MAX_TURN_INPUT_CONTENT_BYTES} bytes"
            )));
        }
        let options_json = default_json_array_if_blank(command.options_json.as_str());
        let options_value: serde_json::Value = serde_json::from_str(&options_json)
            .map_err(|error| KernelError::validation(format!("options is invalid: {error}")))?;
        validate_interaction_options(&options_value)?;
        if command.provider_interaction_id.is_some() && command.runtime_binding_id.is_none() {
            return Err(KernelError::validation(
                "runtimeBindingId is required with providerInteractionId",
            ));
        }
        if let Some(runtime_binding_id) = command.runtime_binding_id.as_deref() {
            self.repository
                .get_session_runtime_binding(
                    command.tenant_id,
                    command.organization_id,
                    &command.session_id,
                    runtime_binding_id,
                )?
                .ok_or_else(|| KernelError::validation("session runtime binding not found"))?;
        }

        self.repository
            .get_session(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("session not found"))
            .and_then(|session| {
                Self::ensure_session_owner_scope(&session, command.owner_scope)?;
                Self::ensure_nested_agent_id(
                    &session.agent_id,
                    command.path_agent_id.as_str(),
                    "session",
                )?;
                if session.organization_id != command.organization_id {
                    return Err(KernelError::validation("session not found"));
                }
                if !session.status.is_active() {
                    return Err(KernelError::validation(
                        "session is not active, cannot create interaction",
                    ))?;
                }
                Ok(session)
            })?;

        if let Some(turn_id) = command.turn_id.as_deref() {
            validate_standard_id(turn_id, "turnId", Some("turn."))?;
            let turn = self
                .repository
                .get_turn(command.tenant_id, command.organization_id, turn_id)?
                .ok_or_else(|| KernelError::validation("turn not found"))?;
            if turn.session_id != command.session_id {
                return Err(KernelError::validation("turn not found"));
            }
        }

        if self
            .repository
            .get_interaction(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                interaction_id.as_str(),
            )?
            .is_some()
        {
            return Err(KernelError::conflict("interaction already exists"));
        }

        let record = AgentInteractionRecord {
            id: self.repository.next_id()?,
            interaction_id,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            session_id: command.session_id,
            turn_id: command.turn_id,
            runtime_binding_id: command.runtime_binding_id,
            provider_interaction_id: command.provider_interaction_id,
            kind: command.kind,
            status: AgentInteractionStatus::Pending,
            prompt: command.prompt,
            options_json,
            resolution_json: None,
            claim_owner: None,
            claim_token_hash: None,
            claim_expires_at: None,
            fencing_token: 0,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            resolved_at: None,
            retention_until: command.retention_until,
        };

        self.repository.insert_interaction(record.clone())?;
        self.emit_interaction_audit_event(
            AgentAuditAction::InteractionCreated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn list_interactions(
        &self,
        command: ListInteractionsCommand,
    ) -> KernelResult<PaginatedResult<AgentInteractionRecord>> {
        self.authorize(
            "agent.business.interaction.list",
            command.requested_by,
            format!("agent.business.session.{}", command.query.session_id),
            "interaction.list",
        )?;
        let session = self.load_session_for_nested_route(
            command.query.tenant_id,
            command.query.organization_id,
            command.query.session_id.as_str(),
            command.path_agent_id.as_str(),
            command.owner_scope,
        )?;
        if session.organization_id != command.query.organization_id {
            return Err(KernelError::validation("session not found"));
        }
        let total_count = self.repository.count_interactions(&command.query)?;
        let items = self.repository.list_interactions(&command.query)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn get_interaction(
        &self,
        command: GetInteractionCommand,
    ) -> KernelResult<AgentInteractionRecord> {
        self.authorize(
            "agent.business.interaction.retrieve",
            command.requested_by,
            format!("agent.business.session.{}", command.session_id),
            "interaction.retrieve",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
            command.path_agent_id.as_str(),
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::validation("interaction not found"));
        }
        self.repository
            .get_interaction(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                command.interaction_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("interaction not found"))
    }

    pub fn claim_interaction(
        &self,
        command: ClaimInteractionCommand,
    ) -> KernelResult<InteractionClaimResult> {
        validate_requested_at(&command.requested_at)?;
        validate_standard_id(
            &command.interaction_id,
            "interactionId",
            Some("interaction."),
        )?;
        require_non_blank(&command.claim_owner, "claimOwner")?;
        if command.claim_owner.len() > 128 {
            return Err(KernelError::validation("claimOwner exceeds 128 bytes"));
        }
        if !(1..=MAX_INTERACTION_LEASE_SECONDS).contains(&command.lease_seconds) {
            return Err(KernelError::validation(
                "leaseSeconds must be between 1 and 300",
            ));
        }
        self.authorize(
            "agent.business.interaction.claim",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "interaction.claim",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::validation("interaction not found"));
        }
        let mut record = self
            .repository
            .get_interaction(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                &command.interaction_id,
            )?
            .ok_or_else(|| KernelError::validation("interaction not found"))?;
        if !record.is_pending() {
            return Err(KernelError::validation(
                "interaction is no longer pending and cannot be claimed",
            ));
        }
        ensure_expected_version(
            record.version,
            Some(command.expected_version),
            "interaction",
        )?;
        let now = OffsetDateTime::now_utc();
        if let (Some(existing_owner), Some(expires_at)) = (
            record.claim_owner.as_deref(),
            record.claim_expires_at.as_deref(),
        ) {
            if parse_rfc3339_datetime(expires_at, "claimExpiresAt")? > now
                && existing_owner != command.claim_owner
            {
                return Err(KernelError::conflict(
                    "interaction is already claimed by another owner",
                ));
            }
        }
        let raw_claim_token = sdkwork_utils_rust::id::random_string(48);
        let claim_token_hash = sha256_hash(raw_claim_token.as_bytes());
        let claim_expires_at =
            format_utc_seconds(now + time::Duration::seconds(i64::from(command.lease_seconds)));
        record.claim(
            command.claim_owner,
            claim_token_hash,
            claim_expires_at.clone(),
            format_utc_seconds(now),
        );
        self.repository.update_interaction(record.clone())?;
        self.emit_interaction_audit_event(
            AgentAuditAction::InteractionClaimed,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(InteractionClaimResult {
            fencing_token: record.fencing_token,
            interaction: record,
            claim_token: raw_claim_token,
            claim_expires_at,
        })
    }

    pub fn approve_interaction(
        &self,
        command: ApproveInteractionCommand,
    ) -> KernelResult<AgentInteractionRecord> {
        validate_standard_id(
            command.interaction_id.as_str(),
            "interactionId",
            Some("interaction."),
        )?;
        self.authorize(
            "agent.business.interaction.approve",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "interaction.approve",
        )?;

        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
            command.path_agent_id.as_str(),
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::validation("interaction not found"));
        }
        let mut record = self
            .repository
            .get_interaction(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                command.interaction_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("interaction not found"))?;

        if !record.is_pending() {
            return Err(KernelError::validation(
                "interaction is no longer pending and cannot be approved",
            ));
        }

        if record.kind != AgentInteractionKind::Approval {
            return Err(KernelError::validation(
                "interaction is not an approval type; use answer instead",
            ));
        }

        ensure_expected_version(
            record.version,
            Some(command.expected_version),
            "interaction",
        )?;
        validate_interaction_claim(&record, &command.claim_token, command.fencing_token)?;

        let resolution = serde_json::json!({
            "outcome": if command.approved { "approved" } else { "rejected" },
            "reason": command.reason,
        });

        let new_status = if command.approved {
            AgentInteractionStatus::Resolved
        } else {
            AgentInteractionStatus::Rejected
        };

        record.resolve(
            new_status,
            resolution.to_string(),
            command.requested_at.as_str(),
        );

        self.repository.update_interaction(record.clone())?;

        let audit_action = if command.approved {
            AgentAuditAction::InteractionResolved
        } else {
            AgentAuditAction::InteractionRejected
        };
        self.emit_interaction_audit_event(
            audit_action,
            &record,
            command.requested_by,
            command.requested_at,
        )?;

        Ok(record)
    }

    pub fn answer_interaction(
        &self,
        command: AnswerInteractionCommand,
    ) -> KernelResult<AgentInteractionRecord> {
        validate_standard_id(
            command.interaction_id.as_str(),
            "interactionId",
            Some("interaction."),
        )?;
        if !command.rejected {
            require_non_blank(command.answer.as_str(), "answer")?;
        }
        self.authorize(
            "agent.business.interaction.answer",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "interaction.answer",
        )?;

        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            command.session_id.as_str(),
            command.path_agent_id.as_str(),
            command.owner_scope,
        )?;
        if session.organization_id != command.organization_id {
            return Err(KernelError::validation("interaction not found"));
        }
        let mut record = self
            .repository
            .get_interaction(
                command.tenant_id,
                command.organization_id,
                command.session_id.as_str(),
                command.interaction_id.as_str(),
            )?
            .ok_or_else(|| KernelError::validation("interaction not found"))?;

        if !record.is_pending() {
            return Err(KernelError::validation(
                "interaction is no longer pending and cannot be answered",
            ));
        }

        if record.kind != AgentInteractionKind::UserQuestion {
            return Err(KernelError::validation(
                "interaction is not a user-question type; use approve instead",
            ));
        }

        ensure_expected_version(
            record.version,
            Some(command.expected_version),
            "interaction",
        )?;
        validate_interaction_claim(&record, &command.claim_token, command.fencing_token)?;

        let resolution = serde_json::json!({
            "outcome": if command.rejected { "rejected" } else { "answered" },
            "answer": command.answer,
            "selectedOptionValue": command.selected_option_value,
        });

        let new_status = if command.rejected {
            AgentInteractionStatus::Rejected
        } else {
            AgentInteractionStatus::Resolved
        };

        record.resolve(
            new_status,
            resolution.to_string(),
            command.requested_at.as_str(),
        );

        self.repository.update_interaction(record.clone())?;

        let audit_action = if command.rejected {
            AgentAuditAction::InteractionRejected
        } else {
            AgentAuditAction::InteractionResolved
        };
        self.emit_interaction_audit_event(
            audit_action,
            &record,
            command.requested_by,
            command.requested_at,
        )?;

        Ok(record)
    }

    fn emit_interaction_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentInteractionRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let payload_json = serde_json::json!({
            "schema_version": "v1",
            "action": action.action_code(),
            "interaction_id": record.interaction_id,
            "session_id": record.session_id,
            "tenant_id": record.tenant_id,
            "organization_id": record.organization_id,
            "kind": record.kind.as_str(),
            "status": record.status.as_str(),
            "version": record.version,
        })
        .to_string();

        let event = KernelEvent::new(
            format!(
                "agent_interaction_{}_{}",
                record.interaction_id, record.version
            ),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", "v1")
        .with_context("audit_action", action.action_code())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("interaction_id", record.interaction_id.as_str())
        .with_context("session_id", record.session_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context(
            "organization_id",
            record.organization_id.to_string().as_str(),
        )
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.interaction.v1");

        self.audit_sink.record(event)
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_session_resource_audit_event(
        &self,
        action: AgentAuditAction,
        resource_kind: &str,
        resource_id: &str,
        session_id: &str,
        tenant_id: u64,
        organization_id: u64,
        version: u64,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let payload_json = serde_json::json!({
            "schema_version": "v1",
            "action": action.action_code(),
            "resource_kind": resource_kind,
            "resource_id": resource_id,
            "session_id": session_id,
            "tenant_id": tenant_id,
            "organization_id": organization_id,
            "version": version,
        })
        .to_string();
        let event = KernelEvent::new(
            format!("agent_{resource_kind}_{resource_id}_{version}"),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", "v1")
        .with_context("audit_action", action.action_code())
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("resource_kind", resource_kind)
        .with_context("resource_id", resource_id)
        .with_context("session_id", session_id)
        .with_context("tenant_id", tenant_id.to_string().as_str())
        .with_context("organization_id", organization_id.to_string().as_str())
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.session-resource.v1");
        self.audit_sink.record(event)
    }
}

/// JSON field name under which structured context metadata is embedded
/// within the `KernelEvent` payload by [`KernelEventExt::with_context`].
///
/// The value is a flat JSON object of `key → string` pairs that
/// supplements the audit payload with routing metadata (tenant_id,
/// agent_id, subject_id, etc.).  Keeping the context in a dedicated
/// sub-object preserves the integrity of the outer JSON payload.
const AUDIT_CONTEXT_FIELD: &str = "_context";

/// Extension trait that attaches structured context metadata to a
/// [`KernelEvent`] without corrupting its JSON payload.
///
/// Each call to `with_context` parses the event payload as a JSON
/// object, obtains (or creates) the [`AUDIT_CONTEXT_FIELD`] sub-object,
/// inserts the `key → value` pair, and re-serialises.  The payload
/// therefore remains valid JSON at all times, in contrast to the
/// previous string-concatenation approach that produced malformed
/// `{"json":...};key=value` strings.
trait KernelEventExt {
    fn with_context(self, key: impl Into<String>, value: impl Into<String>) -> Self;
}

impl KernelEventExt for KernelEvent {
    fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();
        let value = value.into();

        // All audit event payloads produced by `emit_*_audit_event`
        // are serialised JSON objects, so parsing should succeed.  If
        // the payload is unexpectedly not a JSON object, we silently
        // skip the context addition rather than corrupting it.
        if let Ok(mut payload) = serde_json::from_str::<serde_json::Value>(self.payload.as_str()) {
            if let Some(obj) = payload.as_object_mut() {
                let context = obj
                    .entry(AUDIT_CONTEXT_FIELD)
                    .or_insert_with(|| serde_json::json!({}));
                if let Some(ctx) = context.as_object_mut() {
                    ctx.insert(key, serde_json::Value::String(value));
                    if let Ok(serialised) = serde_json::to_string(&payload) {
                        self.payload = serialised;
                    }
                }
            }
        }
        self
    }
}

fn is_valid_status_transition(from: AgentBusinessStatus, to: AgentBusinessStatus) -> bool {
    matches!(
        (from, to),
        (AgentBusinessStatus::Draft, AgentBusinessStatus::Active)
            | (AgentBusinessStatus::Draft, AgentBusinessStatus::Archived)
            | (AgentBusinessStatus::Active, AgentBusinessStatus::Disabled)
            | (AgentBusinessStatus::Active, AgentBusinessStatus::Archived)
            | (AgentBusinessStatus::Disabled, AgentBusinessStatus::Active)
            | (AgentBusinessStatus::Disabled, AgentBusinessStatus::Archived)
            | (AgentBusinessStatus::Archived, AgentBusinessStatus::Active)
            | (AgentBusinessStatus::Archived, AgentBusinessStatus::Disabled)
            | (AgentBusinessStatus::Deleted, AgentBusinessStatus::Active)
            | (_, AgentBusinessStatus::Deleted)
    ) || from == to
}

fn validate_agent_id(value: &str) -> KernelResult<()> {
    validate_standard_id(value, "agentId", Some("agent."))
}

fn validate_project_drive_access(
    visibility: AgentProjectVisibility,
    drive_access_mode: AgentProjectDriveAccessMode,
) -> KernelResult<()> {
    if visibility == AgentProjectVisibility::Shared
        && drive_access_mode == AgentProjectDriveAccessMode::OwnerLibrary
    {
        return Err(KernelError::validation(
            "shared projects cannot access the owner's private Drive library",
        ));
    }
    Ok(())
}

fn validate_project_composition_slot_fields(
    slot_kind: AgentCompositionSlotKind,
    target_module: AgentCompositionTargetModule,
    target_ref: &str,
    target_version_ref: Option<&str>,
    priority: i32,
    policy_json: &str,
) -> KernelResult<()> {
    let module_matches_kind = matches!(
        (slot_kind, target_module),
        (
            AgentCompositionSlotKind::Prompt,
            AgentCompositionTargetModule::Prompts
        ) | (
            AgentCompositionSlotKind::Memory,
            AgentCompositionTargetModule::Memory
        ) | (
            AgentCompositionSlotKind::Knowledge,
            AgentCompositionTargetModule::Knowledgebase
        ) | (
            AgentCompositionSlotKind::Skill,
            AgentCompositionTargetModule::Skills
        ) | (
            AgentCompositionSlotKind::Mcp,
            AgentCompositionTargetModule::Mcp
        ) | (
            AgentCompositionSlotKind::Drive,
            AgentCompositionTargetModule::Drive
        ) | (
            AgentCompositionSlotKind::Tool,
            AgentCompositionTargetModule::Tools
        )
    );
    if !module_matches_kind {
        return Err(KernelError::validation(
            "slotKind does not match targetModule",
        ));
    }
    require_non_blank(target_ref, "targetRef")?;
    if target_ref != trim(target_ref) {
        return Err(KernelError::validation(
            "targetRef must not contain leading or trailing whitespace",
        ));
    }
    if target_ref.chars().count() > 256 {
        return Err(KernelError::validation(
            "targetRef must be at most 256 characters",
        ));
    }
    reject_secret_material(target_ref, "targetRef")?;
    validate_optional_plain_ref(target_version_ref, "targetVersionRef")?;
    if !(-10_000..=10_000).contains(&priority) {
        return Err(KernelError::validation(
            "priority must be between -10000 and 10000",
        ));
    }
    if policy_json.len() > 16 * 1024 {
        return Err(KernelError::validation("policyJson exceeds 16384 bytes"));
    }
    let policy: serde_json::Value = serde_json::from_str(policy_json).map_err(|error| {
        KernelError::validation(format!("policyJson must be valid JSON: {error}"))
    })?;
    if !policy.is_object() {
        return Err(KernelError::validation("policyJson must be a JSON object"));
    }
    Ok(())
}

fn validate_optional_plain_ref(value: Option<&str>, field_name: &str) -> KernelResult<()> {
    if let Some(value) = value {
        validate_non_empty(value, field_name)?;
        reject_secret_material(value, field_name)?;
        if value.trim() != value {
            return Err(KernelError::validation(format!(
                "{field_name} must not contain leading or trailing whitespace"
            )));
        }
        if value.chars().count() > 128 {
            return Err(KernelError::validation(format!(
                "{field_name} must be at most 128 characters"
            )));
        }
    }
    Ok(())
}

fn ensure_expected_version(
    actual_version: u64,
    expected_version: Option<u64>,
    entity_name: &str,
) -> KernelResult<()> {
    let expected_version = expected_version.ok_or_else(|| {
        KernelError::validation(format!("{entity_name} mutation requires expectedVersion"))
    })?;
    if actual_version != expected_version {
        return Err(KernelError::conflict(format!(
            "{entity_name} version mismatch: expected={expected_version}, actual={actual_version}"
        )));
    }
    Ok(())
}

fn validate_non_empty(value: &str, field_name: &str) -> KernelResult<()> {
    require_non_blank(value, field_name)
}

fn validate_json_payload(value: &str, field_name: &str) -> KernelResult<()> {
    let _: serde_json::Value = serde_json::from_str(value).map_err(|error| {
        KernelError::validation(format!("{field_name} must be valid JSON: {error}"))
    })?;
    Ok(())
}

fn reject_secret_material(value: &str, field_name: &str) -> KernelResult<()> {
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

#[cfg(test)]
mod task_tests {
    use super::*;
    use crate::application::{
        CancelTaskCommand, CreateAgentCommand, CreateTaskCommand, ExecuteTaskCommand,
        GetTaskCommand, ListTasksCommand,
    };
    use crate::domain::AgentBusinessStatus;
    use crate::infrastructure::{
        IamGatedPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
    };
    use crate::ports::TaskListQuery;
    use sdkwork_agent_kernel::{AgentManifest, PolicySubject};

    fn sample_subject() -> PolicySubject {
        PolicySubject {
            subject_id: "user.100".to_string(),
            tenant_id: "100001".to_string(),
            roles: vec!["ai.agents.manage".to_string()],
        }
    }

    fn test_policy_provider() -> IamGatedPolicyProvider {
        IamGatedPolicyProvider::new("policy.agents.test.iam-gated")
    }

    fn sample_manifest(agent_id: &str) -> AgentManifest {
        AgentManifest {
            schema_version: "1.0.0".to_string(),
            manifest_type: "agent".to_string(),
            agent_id: agent_id.to_string(),
            name: "tasks-demo".to_string(),
            display_name: "Tasks Demo".to_string(),
            description: "sample".to_string(),
            version: "0.1.0".to_string(),
            domain: "intelligence".to_string(),
            required_capabilities: vec!["model.chat".to_string()],
            optional_capabilities: vec![],
            required_capability_requirements: vec![],
            optional_capability_requirements: vec![],
            event_families: vec!["agent.lifecycle".to_string()],
            owner_name: "sdkwork".to_string(),
            status: "active".to_string(),
        }
    }

    fn create_agent_cmd(
        agent_id: &str,
        tenant_id: u64,
        organization_id: u64,
        owner_user_id: u64,
        code: &str,
        display_name: &str,
        requested_at: &str,
    ) -> CreateAgentCommand {
        CreateAgentCommand {
            agent_id: agent_id.to_string(),
            tenant_id,
            organization_id,
            owner_user_id,
            code: code.to_string(),
            display_name: display_name.to_string(),
            description: None,
            manifest: sample_manifest(agent_id),
            visibility: AgentVisibility::Private,
            tags: Vec::new(),
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: None,
            requested_by: sample_subject(),
            requested_at: requested_at.to_string(),
        }
    }

    #[test]
    fn create_and_list_tasks_roundtrip() {
        let repository = InMemoryAgentRepository::new();
        let audit_sink = InMemoryAgentAuditSink::default();
        let policy_provider = test_policy_provider();
        let service = AgentsService::new(repository, audit_sink, policy_provider);

        let created = service
            .create_agent(create_agent_cmd(
                "agent.tasks.demo",
                100_001,
                0,
                100,
                "tasks-demo",
                "Tasks Demo",
                "2026-06-01T05:00:00Z",
            ))
            .expect("create agent");

        service
            .change_status(ChangeAgentStatusCommand {
                tenant_id: 100_001,
                agent_id: created.agent_id.clone(),
                expected_version: Some(created.version),
                target_status: AgentBusinessStatus::Active,
                requested_by: sample_subject(),
                requested_at: "2026-06-01T05:00:30Z".to_string(),
            })
            .expect("activate agent");

        let task = service
            .create_task(CreateTaskCommand {
                tenant_id: 100_001,
                organization_id: 0,
                agent_id: created.agent_id.clone(),
                owner_user_id: 100,
                task_id: String::new(),
                title: Some("Nightly sync".to_string()),
                prompt: "Run nightly data sync".to_string(),
                external_ref: None,
                metadata_json: r#"{"autoExecute":true}"#.to_string(),
                requested_by: sample_subject(),
                requested_at: "2026-06-01T05:01:00Z".to_string(),
            })
            .expect("create task");

        assert!(task.task_id.starts_with("task."));
        assert_eq!(task.status, AgentTaskStatus::Completed);
        assert!(task.completed_at.is_some());

        let listed = service
            .list_tasks(ListTasksCommand {
                query: TaskListQuery::for_tenant(100_001).for_agent(created.agent_id),
                requested_by: sample_subject(),
            })
            .expect("list tasks");
        assert_eq!(listed.items.len(), 1);
        assert_eq!(listed.items[0].task_id, task.task_id);
    }

    #[test]
    fn cancel_task_rejects_terminal_status() {
        let repository = InMemoryAgentRepository::new();
        let audit_sink = InMemoryAgentAuditSink::default();
        let policy_provider = test_policy_provider();
        let service = AgentsService::new(repository, audit_sink, policy_provider);

        let created = service
            .create_agent(create_agent_cmd(
                "agent.tasks.cancel",
                100_001,
                0,
                100,
                "tasks-cancel",
                "Tasks Cancel",
                "2026-06-01T05:00:00Z",
            ))
            .expect("create agent");

        let task = service
            .create_task(CreateTaskCommand {
                tenant_id: 100_001,
                organization_id: 0,
                agent_id: created.agent_id,
                owner_user_id: 100,
                task_id: String::new(),
                title: None,
                prompt: "Do work".to_string(),
                external_ref: None,
                metadata_json: r#"{"deferExecution":true}"#.to_string(),
                requested_by: sample_subject(),
                requested_at: "2026-06-01T05:01:00Z".to_string(),
            })
            .expect("create task");

        let cancelled = service
            .cancel_task(CancelTaskCommand {
                tenant_id: 100_001,
                path_agent_id: task.agent_id.clone(),
                task_id: task.task_id.clone(),
                expected_version: Some(task.version),
                owner_scope: None,
                requested_by: sample_subject(),
                requested_at: "2026-06-01T05:02:00Z".to_string(),
            })
            .expect("cancel task");
        assert_eq!(cancelled.status, AgentTaskStatus::Cancelled);

        let error = service
            .cancel_task(CancelTaskCommand {
                tenant_id: 100_001,
                path_agent_id: cancelled.agent_id.clone(),
                task_id: task.task_id,
                expected_version: Some(cancelled.version),
                owner_scope: None,
                requested_by: sample_subject(),
                requested_at: "2026-06-01T05:03:00Z".to_string(),
            })
            .expect_err("second cancel must fail");
        assert!(error.to_string().contains("cannot be cancelled"));
    }

    #[test]
    fn execute_task_completes_deferred_pending_task() {
        let repository = InMemoryAgentRepository::new();
        let audit_sink = InMemoryAgentAuditSink::default();
        let policy_provider = test_policy_provider();
        let service = AgentsService::new(repository, audit_sink, policy_provider);

        let created = service
            .create_agent(create_agent_cmd(
                "agent.tasks.execute",
                100_001,
                0,
                100,
                "tasks-execute",
                "Tasks Execute",
                "2026-06-01T05:00:00Z",
            ))
            .expect("create agent");

        let task = service
            .create_task(CreateTaskCommand {
                tenant_id: 100_001,
                organization_id: 0,
                agent_id: created.agent_id.clone(),
                owner_user_id: 100,
                task_id: String::new(),
                title: None,
                prompt: "Run deferred work".to_string(),
                external_ref: None,
                metadata_json: r#"{"deferExecution":true}"#.to_string(),
                requested_by: sample_subject(),
                requested_at: "2026-06-01T05:01:00Z".to_string(),
            })
            .expect("create task");
        assert_eq!(task.status, AgentTaskStatus::Pending);

        let executed = service
            .execute_task(ExecuteTaskCommand {
                tenant_id: 100_001,
                path_agent_id: created.agent_id.clone(),
                task_id: task.task_id.clone(),
                expected_version: Some(task.version),
                owner_scope: None,
                requested_by: sample_subject(),
                requested_at: "2026-06-01T05:02:00Z".to_string(),
            })
            .expect("execute task");
        assert_eq!(executed.status, AgentTaskStatus::Completed);
    }

    #[test]
    fn get_task_rejects_foreign_owner_scope() {
        let repository = InMemoryAgentRepository::new();
        let audit_sink = InMemoryAgentAuditSink::default();
        let policy_provider = test_policy_provider();
        let service = AgentsService::new(repository, audit_sink, policy_provider);

        let created = service
            .create_agent(create_agent_cmd(
                "agent.tasks.owner",
                100_001,
                0,
                100,
                "tasks-owner",
                "Tasks Owner",
                "2026-06-01T05:00:00Z",
            ))
            .expect("create agent");

        let path_agent_id = created.agent_id.clone();
        let task = service
            .create_task(CreateTaskCommand {
                tenant_id: 100_001,
                organization_id: 0,
                agent_id: path_agent_id.clone(),
                owner_user_id: 100,
                task_id: String::new(),
                title: None,
                prompt: "Private task".to_string(),
                external_ref: None,
                metadata_json: "{}".to_string(),
                requested_by: sample_subject(),
                requested_at: "2026-06-01T05:01:00Z".to_string(),
            })
            .expect("create task");

        let error = service
            .get_task(GetTaskCommand {
                tenant_id: 100_001,
                path_agent_id,
                task_id: task.task_id,
                owner_scope: Some(999),
                requested_by: sample_subject(),
            })
            .expect_err("foreign owner must not read task");
        assert!(error.to_string().contains("task not found"));
    }
}
