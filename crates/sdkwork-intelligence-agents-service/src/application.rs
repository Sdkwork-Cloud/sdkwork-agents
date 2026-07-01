use std::sync::Arc;

mod commands;
pub use commands::*;

use crate::chat_runtime::{
    ChatCompleter, ChatCompletionInput, ContractChatCompleter,
};
use crate::runtime_facade_bridge::{execute_preview_response, execute_prompt_optimization};
use crate::domain::{
    AgentAuditAction, AgentAuditPayload, AgentBusinessRecord, AgentBusinessStatus,
    AgentCompositionSlotRecord, AgentInteractionKind, AgentInteractionRecord,
    AgentInteractionStatus, AgentMessageRecord, AgentMessageRole, AgentMessageStatus,
    AgentProviderBindingRecord, AgentRuntimeExecutionOperation, AgentRuntimeExecutionRecord,
    AgentRuntimeExecutionStatus, AgentSessionRecord, AgentSessionStatus, AgentVisibility,
    MarketplaceAuditPayload, MessageAuditPayload, ProviderBindingAuditPayload,
    RuntimeExecutionAuditPayload, SessionAuditPayload, DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY,
};
use crate::dto::AgentManagementProfileDto;
use crate::ports::{AgentAuditSink, AgentRepository, MessageListQuery};
use crate::validation::{
    default_json_array_if_blank, default_json_object_if_blank, default_plain_text_if_blank,
    is_trimmed_blank, require_non_blank, validate_capabilities, validate_standard_id,
};
use sdkwork_agent_kernel::{
    KernelError, KernelEvent, KernelEventRedaction, KernelEventSeverity,
    KernelEventSource, KernelResult, PolicyCategory, PolicyDecisionValue, PolicyProvider,
    PolicyRequest, PolicySubject,
};

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
    chat_completer: Arc<dyn ChatCompleter>,
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
            chat_completer: Arc::new(ContractChatCompleter),
        }
    }

    /// Replace the default contract completer with a kernel-backed implementation
    /// at gateway bootstrap (production) without changing HTTP handlers.
    pub fn with_chat_completer(mut self, chat_completer: Arc<dyn ChatCompleter>) -> Self {
        self.chat_completer = chat_completer;
        self
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
            .get(command.tenant_id, command.agent_id.as_str())
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
            .get(command.tenant_id, command.agent_id.as_str())
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
            )
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
            .get(command.tenant_id, command.agent_id.as_str())
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        validate_standard_id(command.binding_id.as_str(), "bindingId", Some("binding."))?;

        let mut record = self
            .repository
            .get_provider_binding(
                command.tenant_id,
                command.agent_id.as_str(),
                command.binding_id.as_str(),
            )
            .ok_or_else(|| KernelError::validation("agent provider binding not found"))?;

        if record.active {
            return Ok(record);
        }

        self.deactivate_provider_bindings(
            command.tenant_id,
            command.agent_id.as_str(),
            command.requested_at.clone(),
        )?;
        record.active = true;
        record.mark_updated(command.requested_at.clone());
        self.repository.update_provider_binding(record.clone())?;
        self.emit_binding_audit_event(
            AgentAuditAction::ProviderBindingChanged,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn list_provider_bindings(
        &self,
        tenant_id: u64,
        agent_id: &str,
        requested_by: PolicySubject,
    ) -> KernelResult<Vec<AgentProviderBindingRecord>> {
        validate_agent_id(agent_id)?;
        self.authorize(
            "agent.business.provider_binding.list",
            requested_by,
            format!("agent.business.{}", agent_id),
            "provider_binding.list",
        )?;
        self.repository
            .get(tenant_id, agent_id)
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        Ok(self.repository.list_provider_bindings(tenant_id, agent_id))
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
            .get(tenant_id, agent_id)
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        self.repository
            .get_provider_binding(tenant_id, agent_id, binding_id)
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
            .get_provider_binding(tenant_id, agent_id, binding_id)
            .ok_or_else(|| KernelError::validation("provider binding not found"))?;
        if !record.active {
            return Err(KernelError::validation(
                "provider binding is already inactive",
            ));
        }
        record.active = false;
        record.mark_updated(requested_at.as_str());
        self.repository.update_provider_binding(record.clone())?;
        let _ = self.emit_binding_audit_event(
            AgentAuditAction::ProviderBindingChanged,
            &record,
            requested_by,
            record.updated_at.clone(),
        );
        Ok(record)
    }

    pub fn update_session_metadata(
        &self,
        tenant_id: u64,
        session_id: &str,
        title: Option<String>,
        model_id: Option<String>,
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
            .get_session(tenant_id, session_id)
            .ok_or_else(|| KernelError::validation("session not found"))?;
        if let Some(title) = title {
            record.title = Some(title);
        }
        if let Some(model_id) = model_id {
            record.model_id = Some(model_id);
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
        tenant_id: u64,
        requested_by: PolicySubject,
    ) -> KernelResult<Vec<crate::mcp_marketplace::McpServerMarketplaceRecord>> {
        self.authorize(
            "agent.business.mcp_server.list",
            requested_by,
            "agent.business".to_string(),
            "mcp_server.list",
        )?;
        Ok(crate::mcp_marketplace::list_mcp_marketplace_records(
            &self.repository,
            tenant_id,
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
            .get(command.tenant_id, command.agent_id.as_str())
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
            .list_provider_bindings(command.tenant_id, command.agent_id.as_str())
            .into_iter()
            .find(|binding| binding.active);

        let preview = execute_preview_response(
            active_binding.as_ref(),
            command.content.as_str(),
            command.model.as_deref(),
        );

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
            .get(command.tenant_id, command.agent_id.as_str())
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
            .list_provider_bindings(command.tenant_id, command.agent_id.as_str())
            .into_iter()
            .find(|binding| binding.active);

        let optimization = execute_prompt_optimization(
            active_binding.as_ref(),
            command.prompt.as_str(),
        );

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
            .get(command.tenant_id, command.agent_id.as_str())
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        if self
            .repository
            .get_composition_slot(
                command.tenant_id,
                command.agent_id.as_str(),
                command.slot_id.as_str(),
            )
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
    ) -> KernelResult<Vec<AgentCompositionSlotRecord>> {
        self.authorize(
            "agent.business.composition_slot.list",
            command.requested_by,
            format!("agent.business.{}", command.agent_id),
            "composition_slot.list",
        )?;
        validate_agent_id(command.agent_id.as_str())?;
        self.repository
            .get(command.tenant_id, command.agent_id.as_str())
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        Ok(self
            .repository
            .list_composition_slots(command.tenant_id, command.agent_id.as_str()))
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
            )
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
            )
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
            )
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
            .get(command.tenant_id, command.agent_id.as_str())
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
            .get(command.tenant_id, command.agent_id.as_str())
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
            .get(command.tenant_id, command.agent_id.as_str())
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        if record.is_deleted() {
            return Err(KernelError::validation("agent already deleted"));
        }
        ensure_expected_version(record.version, command.expected_version, "agent")?;

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
            .get(command.tenant_id, command.agent_id.as_str())
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
            .get(command.tenant_id, command.agent_id.as_str())
            .ok_or_else(|| KernelError::validation("agent not found"))
    }

    pub fn list_agents(
        &self,
        command: ListAgentsCommand,
    ) -> KernelResult<Vec<AgentBusinessRecord>> {
        self.authorize(
            "agent.business.list",
            command.requested_by,
            format!("agent.business.tenant.{}", command.query.tenant_id),
            "list",
        )?;
        Ok(self.repository.list(&command.query))
    }

    pub fn list_agent_audit_events(
        &self,
        tenant_id: u64,
        agent_id: &str,
        requested_by: PolicySubject,
    ) -> KernelResult<Vec<KernelEvent>> {
        validate_agent_id(agent_id)?;
        self.authorize(
            "agent.business.audit.read",
            requested_by,
            format!("agent.business.{}", agent_id),
            "audit.read",
        )?;
        self.audit_sink.list_events(tenant_id, agent_id)
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
            format!("agent.business.session.{}", command.session_id),
            "session.create",
        )?;

        // Ensure the agent exists
        self.repository
            .get(command.tenant_id, command.agent_id.as_str())
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        // Ensure session does not already exist
        if self
            .repository
            .get_session(command.tenant_id, session_id.as_str())
            .is_some()
        {
            return Err(KernelError::conflict("session already exists"));
        }

        // Validate metadata_json if non-empty
        if !is_trimmed_blank(command.metadata_json.as_str()) {
            validate_json_payload(command.metadata_json.as_str(), "metadataJson")?;
        } else {
            // default to empty object
        }

        let metadata_json = default_json_object_if_blank(command.metadata_json.as_str());

        let record = AgentSessionRecord {
            id: self.repository.next_id()?,
            session_id,
            tenant_id: command.tenant_id,
            organization_id: command.organization_id,
            agent_id: command.agent_id,
            owner_user_id: command.owner_user_id,
            title: command.title,
            status: AgentSessionStatus::Active,
            provider_binding_id: command.provider_binding_id,
            model_id: command.model_id,
            message_count: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            metadata_json,
            version: 0,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
            last_message_at: None,
            closed_at: None,
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

    pub fn close_session(
        &self,
        command: CloseSessionCommand,
    ) -> KernelResult<AgentSessionRecord> {
        self.authorize(
            "agent.business.session.close",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "session.close",
        )?;
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some("session."),
        )?;

        let mut record = self
            .repository
            .get_session(command.tenant_id, command.session_id.as_str())
            .ok_or_else(|| KernelError::validation("session not found"))?;

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
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some("session."),
        )?;

        let mut record = self
            .repository
            .get_session(command.tenant_id, command.session_id.as_str())
            .ok_or_else(|| KernelError::validation("session not found"))?;

        ensure_expected_version(record.version, command.expected_version, "session")?;

        record.status = AgentSessionStatus::Archived;
        record.mark_updated(command.requested_at.clone());
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
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some("session."),
        )?;
        self.repository
            .get_session(command.tenant_id, command.session_id.as_str())
            .ok_or_else(|| KernelError::validation("session not found"))
    }

    pub fn list_sessions(
        &self,
        command: ListSessionsCommand,
    ) -> KernelResult<Vec<AgentSessionRecord>> {
        self.authorize(
            "agent.business.session.list",
            command.requested_by,
            format!("agent.business.tenant.{}", command.query.tenant_id),
            "session.list",
        )?;
        Ok(self.repository.list_sessions(&command.query))
    }

    // -----------------------------------------------------------------------
    // Message management
    // -----------------------------------------------------------------------

    pub fn create_message(
        &self,
        command: CreateMessageCommand,
    ) -> KernelResult<AgentMessageRecord> {
        validate_standard_id(
            command.message_id.as_str(),
            "messageId",
            Some("message."),
        )?;
        self.authorize(
            "agent.business.message.create",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "message.create",
        )?;

        // Ensure session exists and is active
        let mut session = self
            .repository
            .get_session(command.tenant_id, command.session_id.as_str())
            .ok_or_else(|| KernelError::validation("session not found"))?;

        if !session.status.is_active() {
            return Err(KernelError::validation(
                "session is not active, cannot create message",
            ));
        }

        // Ensure message does not already exist
        if self
            .repository
            .get_message(
                command.tenant_id,
                command.session_id.as_str(),
                command.message_id.as_str(),
            )
            .is_some()
        {
            return Err(KernelError::conflict("message already exists"));
        }

        require_non_blank(command.content.as_str(), "content")?;
        if !is_trimmed_blank(command.content_type.as_str()) {
            reject_secret_material(command.content.as_str(), "content")?;
        }

        // Validate JSON fields
        if !is_trimmed_blank(command.artifacts_json.as_str()) {
            validate_json_payload(command.artifacts_json.as_str(), "artifactsJson")?;
        }
        if !is_trimmed_blank(command.metadata_json.as_str()) {
            validate_json_payload(command.metadata_json.as_str(), "metadataJson")?;
        }

        let sequence = self
            .repository
            .next_message_sequence(command.tenant_id, command.session_id.as_str())?;

        let artifacts_json = default_json_array_if_blank(command.artifacts_json.as_str());
        let metadata_json = default_json_object_if_blank(command.metadata_json.as_str());

        let record = AgentMessageRecord {
            id: self.repository.next_id()?,
            message_id: command.message_id,
            tenant_id: command.tenant_id,
            session_id: command.session_id.clone(),
            agent_id: session.agent_id.clone(),
            role: command.role,
            content: command.content,
            content_type: default_plain_text_if_blank(command.content_type.as_str()),
            status: AgentMessageStatus::Sent,
            sequence,
            input_tokens: command.input_tokens,
            output_tokens: command.output_tokens,
            model_id: command.model_id,
            provider_id: command.provider_id,
            artifacts_json,
            metadata_json,
            parent_message_id: command.parent_message_id,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
        };

        self.repository.insert_message(record.clone())?;

        // Update session counters
        session.record_message(
            command.input_tokens,
            command.output_tokens,
            command.requested_at.clone(),
        );
        self.repository.update_session(session)?;

        self.emit_message_audit_event(
            AgentAuditAction::MessageCreated,
            &record,
            command.requested_by,
            command.requested_at,
        )?;
        Ok(record)
    }

    pub fn get_message(&self, command: GetMessageCommand) -> KernelResult<AgentMessageRecord> {
        self.authorize(
            "agent.business.message.retrieve",
            command.requested_by,
            format!("agent.business.session.{}", command.session_id),
            "message.retrieve",
        )?;
        validate_standard_id(
            command.message_id.as_str(),
            "messageId",
            Some("message."),
        )?;
        self.repository
            .get_message(
                command.tenant_id,
                command.session_id.as_str(),
                command.message_id.as_str(),
            )
            .ok_or_else(|| KernelError::validation("message not found"))
    }

    pub fn list_messages(
        &self,
        command: ListMessagesCommand,
    ) -> KernelResult<Vec<AgentMessageRecord>> {
        self.authorize(
            "agent.business.message.list",
            command.requested_by,
            format!("agent.business.session.{}", command.query.session_id),
            "message.list",
        )?;
        Ok(self.repository.list_messages(&command.query))
    }

    /// Send one user message, persist it, run managed chat completion, and persist
    /// the assistant reply. This is the canonical chat turn for app/open/backend APIs.
    pub fn send_chat_message(
        &self,
        command: SendChatMessageCommand,
    ) -> KernelResult<ChatCompletionResult> {
        validate_agent_id(command.agent_id.as_str())?;
        validate_standard_id(
            command.session_id.as_str(),
            "sessionId",
            Some("session."),
        )?;
        self.authorize(
            "agent.business.message.create",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "message.create",
        )?;

        let agent = self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        let mut session = self
            .repository
            .get_session(command.tenant_id, command.session_id.as_str())
            .ok_or_else(|| KernelError::validation("session not found"))?;

        if session.agent_id != command.agent_id {
            return Err(KernelError::validation(
                "session does not belong to the requested agent",
            ));
        }
        if !session.status.is_active() {
            return Err(KernelError::validation(
                "session is not active, cannot send chat message",
            ));
        }

        require_non_blank(command.content.as_str(), "content")?;
        if !is_trimmed_blank(command.content_type.as_str()) {
            reject_secret_material(command.content.as_str(), "content")?;
        }
        if !is_trimmed_blank(command.metadata_json.as_str()) {
            validate_json_payload(command.metadata_json.as_str(), "metadataJson")?;
        }

        let history_messages = self.repository.list_messages(
            &MessageListQuery::for_session(command.tenant_id, command.session_id.clone()),
        );
        let history = history_messages
            .iter()
            .map(|record| (record.role, record.content.clone()))
            .collect::<Vec<_>>();

        let user_message_id = format!("msg.{}", self.repository.next_id()?);
        let user_metadata_json = default_json_object_if_blank(command.metadata_json.as_str());
        let user_sequence = self
            .repository
            .next_message_sequence(command.tenant_id, command.session_id.as_str())?;
        let user_message = AgentMessageRecord {
            id: self.repository.next_id()?,
            message_id: user_message_id,
            tenant_id: command.tenant_id,
            session_id: command.session_id.clone(),
            agent_id: command.agent_id.clone(),
            role: AgentMessageRole::User,
            content: command.content.clone(),
            content_type: default_plain_text_if_blank(command.content_type.as_str()),
            status: AgentMessageStatus::Sent,
            sequence: user_sequence,
            input_tokens: 0,
            output_tokens: 0,
            model_id: command.model_id.clone(),
            provider_id: None,
            artifacts_json: "[]".to_string(),
            metadata_json: user_metadata_json,
            parent_message_id: None,
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
        };
        self.repository.insert_message(user_message.clone())?;
        session.record_message(0, 0, command.requested_at.clone());
        self.repository.update_session(session.clone())?;

        let active_binding = self
            .repository
            .list_provider_bindings(command.tenant_id, command.agent_id.as_str())
            .into_iter()
            .find(|binding| binding.active);

        let welcome_message = AgentManagementProfileDto::from_default_code_task_intent(
            agent.default_code_task_intent.as_ref(),
        )
        .and_then(|profile| profile.welcome_message);
        let provider_has_model_chat = active_binding
            .as_ref()
            .map(|binding| binding.capabilities.iter().any(|cap| cap == "model.chat"))
            .unwrap_or(false);

        let completion = self.chat_completer.complete(&ChatCompletionInput {
            agent_display_name: agent.display_name.clone(),
            welcome_message,
            session: session.clone(),
            history,
            user_content: command.content,
            model_id: command.model_id,
            provider_id: active_binding.as_ref().map(|binding| binding.provider_id.clone()),
            provider_has_model_chat,
        });

        let assistant_message_id = format!("msg.{}", self.repository.next_id()?);
        let assistant_sequence = self
            .repository
            .next_message_sequence(command.tenant_id, command.session_id.as_str())?;
        let assistant_metadata_json = serde_json::json!({
            "runtimeMode": completion.runtime_mode,
        })
        .to_string();
        let assistant_message = AgentMessageRecord {
            id: self.repository.next_id()?,
            message_id: assistant_message_id,
            tenant_id: command.tenant_id,
            session_id: command.session_id.clone(),
            agent_id: command.agent_id.clone(),
            role: AgentMessageRole::Assistant,
            content: completion.content,
            content_type: "text/plain".to_string(),
            status: AgentMessageStatus::Delivered,
            sequence: assistant_sequence,
            input_tokens: completion.input_tokens,
            output_tokens: completion.output_tokens,
            model_id: completion.model_id,
            provider_id: completion.provider_id,
            artifacts_json: "[]".to_string(),
            metadata_json: assistant_metadata_json,
            parent_message_id: Some(user_message.message_id.clone()),
            created_at: command.requested_at.clone(),
            updated_at: command.requested_at.clone(),
        };
        self.repository.insert_message(assistant_message.clone())?;
        session.record_message(
            completion.input_tokens,
            completion.output_tokens,
            command.requested_at.clone(),
        );
        self.repository.update_session(session.clone())?;

        self.emit_message_audit_event(
            AgentAuditAction::MessageCreated,
            &user_message,
            command.requested_by.clone(),
            command.requested_at.clone(),
        )?;
        self.emit_message_audit_event(
            AgentAuditAction::MessageCreated,
            &assistant_message,
            command.requested_by,
            command.requested_at,
        )?;

        Ok(ChatCompletionResult {
            session,
            user_message,
            assistant_message,
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
        let payload_json = audit_payload
            .to_json()
            .map_err(|error| KernelError::validation(format!("audit payload serialization: {error}")))?;

        let event = KernelEvent::new(
            format!("agent_audit_{}_{}", record.agent_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", AgentAuditPayload::SCHEMA_VERSION)
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
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
        for mut binding in self.repository.list_provider_bindings(tenant_id, agent_id) {
            if binding.active {
                binding.active = false;
                binding.mark_updated(updated_at.clone());
                self.repository.update_provider_binding(binding)?;
            }
        }
        Ok(())
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
        let payload_json = audit_payload
            .to_json()
            .map_err(|error| KernelError::validation(format!("binding audit payload serialization: {error}")))?;

        let event = KernelEvent::new(
            format!("agent_binding_{}_{}", record.binding_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", ProviderBindingAuditPayload::SCHEMA_VERSION)
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
        let payload_json = audit_payload
            .to_json()
            .map_err(|error| KernelError::validation(format!("runtime execution audit payload serialization: {error}")))?;

        let event = KernelEvent::new(
            format!("agent_runtime_execution_{}", record.execution_id),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", RuntimeExecutionAuditPayload::SCHEMA_VERSION)
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
        let audit_payload = MarketplaceAuditPayload::new(
            input.action,
            input.item_kind,
            input.item_id,
            input.tenant_id,
            input.organization_id,
            input.status,
            input.visibility,
            input.version,
        );
        let payload_json = audit_payload
            .to_json()
            .map_err(|error| KernelError::validation(format!("marketplace audit payload serialization: {error}")))?;

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
        let payload_json = audit_payload
            .to_json()
            .map_err(|error| KernelError::validation(format!("session audit payload serialization: {error}")))?;

        let event = KernelEvent::new(
            format!("agent_session_{}_{}", record.session_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", SessionAuditPayload::SCHEMA_VERSION)
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

    fn emit_message_audit_event(
        &self,
        action: AgentAuditAction,
        record: &AgentMessageRecord,
        subject: PolicySubject,
        occurred_at: String,
    ) -> KernelResult<()> {
        let audit_payload = MessageAuditPayload::new(action, record);
        let payload_json = audit_payload
            .to_json()
            .map_err(|error| KernelError::validation(format!("message audit payload serialization: {error}")))?;

        let event = KernelEvent::new(
            format!("agent_message_{}_{}", record.message_id, record.sequence),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", MessageAuditPayload::SCHEMA_VERSION)
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("message_id", record.message_id.as_str())
        .with_context("session_id", record.session_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.message.v1");

        self.audit_sink.record(event)
    }

    // -----------------------------------------------------------------------
    // Live interaction operations
    // -----------------------------------------------------------------------

    pub fn list_interactions(
        &self,
        command: ListInteractionsCommand,
    ) -> KernelResult<Vec<AgentInteractionRecord>> {
        self.authorize(
            "agent.business.interaction.list",
            command.requested_by,
            format!("agent.business.session.{}", command.query.session_id),
            "interaction.list",
        )?;
        Ok(self.repository.list_interactions(&command.query))
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
        self.repository
            .get_interaction(command.tenant_id, command.session_id.as_str(), command.interaction_id.as_str())
            .ok_or_else(|| KernelError::validation("interaction not found"))
    }

    pub fn approve_interaction(
        &self,
        command: ApproveInteractionCommand,
    ) -> KernelResult<AgentInteractionRecord> {
        validate_standard_id(command.interaction_id.as_str(), "interactionId", Some("interaction."))?;
        self.authorize(
            "agent.business.interaction.approve",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "interaction.approve",
        )?;

        let mut record = self
            .repository
            .get_interaction(command.tenant_id, command.session_id.as_str(), command.interaction_id.as_str())
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

        let resolution = serde_json::json!({
            "approved": command.approved,
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
        let _ = self.emit_interaction_audit_event(
            audit_action,
            &record,
            command.requested_by,
            command.requested_at,
        );

        Ok(record)
    }

    pub fn answer_interaction(
        &self,
        command: AnswerInteractionCommand,
    ) -> KernelResult<AgentInteractionRecord> {
        validate_standard_id(command.interaction_id.as_str(), "interactionId", Some("interaction."))?;
        require_non_blank(command.answer.as_str(), "answer")?;
        self.authorize(
            "agent.business.interaction.answer",
            command.requested_by.clone(),
            format!("agent.business.session.{}", command.session_id),
            "interaction.answer",
        )?;

        let mut record = self
            .repository
            .get_interaction(command.tenant_id, command.session_id.as_str(), command.interaction_id.as_str())
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

        let resolution = serde_json::json!({
            "answer": command.answer,
            "option_label": command.option_label,
            "rejected": command.rejected,
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
        let _ = self.emit_interaction_audit_event(
            audit_action,
            &record,
            command.requested_by,
            command.requested_at,
        );

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
            "agent_id": record.agent_id,
            "tenant_id": record.tenant_id,
            "kind": record.kind.as_str(),
            "status": record.status.as_str(),
            "version": record.version,
        })
        .to_string();

        let event = KernelEvent::new(
            format!("agent_interaction_{}_{}", record.interaction_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload_json,
        )
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("schema_version", "v1")
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("interaction_id", record.interaction_id.as_str())
        .with_context("session_id", record.session_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.interaction.v1");

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
