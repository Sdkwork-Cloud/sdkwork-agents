use crate::domain::{
    AgentAuditAction, AgentBusinessRecord, AgentBusinessStatus,
    AgentImplementationKind, AgentImplementationType,
    AgentCompositionSlotRecord,
    AgentCompositionSlotKind, AgentCompositionTargetModule, AgentProviderBindingRecord,
    AgentRuntimeExecutionOperation, AgentRuntimeExecutionRecord,
    AgentRuntimeExecutionStatus, AgentVisibility, DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY,
};
use crate::ports::{AgentAuditSink, AgentListQuery, AgentRepository};
use crate::validation::{require_non_blank, validate_capabilities, validate_standard_id};
use sdkwork_agent_kernel::{
    AgentManifest, KernelError, KernelEvent, KernelEventRedaction, KernelEventSeverity,
    KernelEventSource, KernelResult, PolicyCategory, PolicyDecisionValue, PolicyProvider,
    PolicyRequest, PolicySubject,
};
use sdkwork_code_kernel::CodeTaskIntent;


struct AgentBusinessAuditEventInput<'a> {
    action: AgentAuditAction,
    item_kind: &'a str,
    tenant_id: u64,
    organization_id: u64,
    item_id: &'a str,
    status: AgentBusinessStatus,
    visibility: AgentVisibility,
    version: u64,
    subject: PolicySubject,
    occurred_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAgentCommand {
    pub agent_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub manifest: AgentManifest,
    pub visibility: AgentVisibility,
    pub tags: Vec<String>,
    pub default_code_task_intent: Option<CodeTaskIntent>,
    pub implementation_provider_id: Option<String>,
    pub implementation_kind: Option<AgentImplementationKind>,
    pub implementation_type: Option<AgentImplementationType>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAgentCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub expected_version: Option<u64>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub manifest: Option<AgentManifest>,
    pub visibility: Option<AgentVisibility>,
    pub tags: Option<Vec<String>>,
    pub default_code_task_intent: Option<CodeTaskIntent>,
    pub implementation_provider_id: Option<Option<String>>,
    pub implementation_kind: Option<Option<AgentImplementationKind>>,
    pub implementation_type: Option<AgentImplementationType>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeAgentStatusCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub expected_version: Option<u64>,
    pub target_status: AgentBusinessStatus,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteAgentCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub expected_version: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreAgentCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub expected_version: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetAgentCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListAgentsCommand {
    pub query: AgentListQuery,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderBindingCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub binding_id: String,
    pub provider_id: String,
    pub implementation_kind: AgentImplementationKind,
    pub configuration_profile_id: String,
    pub capabilities: Vec<String>,
    pub make_default: bool,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivateAgentProviderBindingCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub binding_id: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentPreviewResponseCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub execution_id: String,
    pub content: String,
    pub debug_mode: bool,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub input_payload_json: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentPromptOptimizationCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub execution_id: String,
    pub prompt: String,
    pub input_payload_json: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotCreateCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub slot_id: String,
    pub slot_kind: AgentCompositionSlotKind,
    pub target_module: AgentCompositionTargetModule,
    pub target_ref: String,
    pub target_version_ref: Option<String>,
    pub priority: i32,
    pub enabled: bool,
    pub policy_json: String,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotUpdateCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub slot_id: String,
    pub expected_version: Option<u64>,
    pub slot_kind: Option<AgentCompositionSlotKind>,
    pub target_module: Option<AgentCompositionTargetModule>,
    pub target_ref: Option<String>,
    pub target_version_ref: Option<Option<String>>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
    pub policy_json: Option<String>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotDeleteCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub slot_id: String,
    pub expected_version: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotListCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotGetCommand {
    pub tenant_id: u64,
    pub agent_id: String,
    pub slot_id: String,
    pub requested_by: PolicySubject,
}
pub struct AgentsService<R, A, P>
where
    R: AgentRepository,
    A: AgentAuditSink,
    P: PolicyProvider,
{
    repository: R,
    audit_sink: A,
    policy_provider: P,
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
        }
    }

    pub fn create_agent(
        &mut self,
        command: CreateAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;

        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.create",
            command.requested_by.clone(),
            policy_resource,
            "create")?;

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
            command.requested_by,
            command.requested_at)?;
        Ok(record)
    }

    pub fn add_provider_binding(
        &mut self,
        command: AgentProviderBindingCommand) -> KernelResult<AgentProviderBindingRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        self.authorize(
            "agent.business.provider_binding.add",
            command.requested_by.clone(),
            format!("agent.business.{}", command.agent_id),
            "provider_binding.add")?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        validate_standard_id(command.binding_id.as_str(), "bindingId", Some("binding."))?;
        validate_standard_id(
            command.provider_id.as_str(),
            "providerId",
            Some("provider."))?;
        validate_standard_id(
            command.configuration_profile_id.as_str(),
            "configurationProfileId",
            Some("profile."))?;
        validate_capabilities(command.capabilities.as_slice(), "capabilities")?;

        if self
            .repository
            .get_provider_binding(
                command.tenant_id,
                command.agent_id.as_str(),
                command.binding_id.as_str())
            .is_some()
        {
            return Err(KernelError::conflict(
                "agent provider binding already exists"));
        }

        if command.make_default {
            self.deactivate_provider_bindings(
                command.tenant_id,
                command.agent_id.as_str(),
                command.requested_at.clone())?;
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
            command.requested_at)?;
        Ok(record)
    }

    pub fn activate_provider_binding(
        &mut self,
        command: ActivateAgentProviderBindingCommand) -> KernelResult<AgentProviderBindingRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        self.authorize(
            "agent.business.provider_binding.activate",
            command.requested_by.clone(),
            format!("agent.business.{}", command.agent_id),
            "provider_binding.activate")?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        validate_standard_id(command.binding_id.as_str(), "bindingId", Some("binding."))?;

        let mut record = self
            .repository
            .get_provider_binding(
                command.tenant_id,
                command.agent_id.as_str(),
                command.binding_id.as_str())
            .ok_or_else(|| KernelError::validation("agent provider binding not found"))?;

        if record.active {
            return Ok(record);
        }

        self.deactivate_provider_bindings(
            command.tenant_id,
            command.agent_id.as_str(),
            command.requested_at.clone())?;
        record.active = true;
        record.mark_updated(command.requested_at.clone());
        self.repository.update_provider_binding(record.clone())?;
        self.emit_binding_audit_event(
            AgentAuditAction::ProviderBindingChanged,
            &record,
            command.requested_by,
            command.requested_at)?;
        Ok(record)
    }

    pub fn list_provider_bindings(
        &mut self,
        tenant_id: u64,
        agent_id: &str,
        requested_by: PolicySubject) -> KernelResult<Vec<AgentProviderBindingRecord>> {
        validate_agent_id(agent_id)?;
        self.authorize(
            "agent.business.provider_binding.list",
            requested_by,
            format!("agent.business.{}", agent_id),
            "provider_binding.list")?;
        self.repository
            .get(tenant_id, agent_id)
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        Ok(self.repository.list_provider_bindings(tenant_id, agent_id))
    }

    pub fn create_preview_response(
        &mut self,
        command: AgentPreviewResponseCommand) -> KernelResult<AgentRuntimeExecutionRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        self.authorize(
            "agent.business.runtime.preview_response",
            command.requested_by.clone(),
            format!("agent.business.{}", command.agent_id),
            "runtime.preview_response")?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        validate_standard_id(
            command.execution_id.as_str(),
            "executionId",
            Some("execution."))?;
        validate_non_empty(command.content.as_str(), "content")?;
        validate_json_payload(command.input_payload_json.as_str(), "inputPayload")?;
        if let Some(model) = command.model.as_deref() {
            validate_optional_plain_ref(Some(model), "model")?;
        }
        if let Some(temperature) = command.temperature {
            if !(0.0..=2.0).contains(&temperature) || !temperature.is_finite() {
                return Err(KernelError::validation(
                    "temperature must be between 0 and 2"));
            }
        }

        let output_payload_json = serde_json::json!({
            "content": command.content,
            "debugMode": command.debug_mode,
            "model": command.model,
            "temperature": command.temperature,
            "runtimeMode": "deterministic-local-contract"
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
            command.requested_at)?;
        Ok(record)
    }

    pub fn create_prompt_optimization(
        &mut self,
        command: AgentPromptOptimizationCommand) -> KernelResult<AgentRuntimeExecutionRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        self.authorize(
            "agent.business.runtime.prompt_optimization",
            command.requested_by.clone(),
            format!("agent.business.{}", command.agent_id),
            "runtime.prompt_optimization")?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        validate_standard_id(
            command.execution_id.as_str(),
            "executionId",
            Some("execution."))?;
        validate_non_empty(command.prompt.as_str(), "prompt")?;
        validate_json_payload(command.input_payload_json.as_str(), "inputPayload")?;

        let optimized_prompt = normalize_prompt_text(command.prompt.as_str());
        let output_payload_json = serde_json::json!({
            "optimizedPrompt": optimized_prompt,
            "runtimeMode": "deterministic-local-contract"
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
            command.requested_at)?;
        Ok(record)
    }

    pub fn create_composition_slot(
        &mut self,
        command: AgentCompositionSlotCreateCommand) -> KernelResult<AgentCompositionSlotRecord> {
        self.authorize(
            "agent.business.composition_slot.create",
            command.requested_by.clone(),
            format!(
                "agent.business.composition_slot.{}.{}",
                command.agent_id, command.slot_id
            ),
            "composition_slot.create")?;
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
                command.slot_id.as_str())
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
        &mut self,
        command: AgentCompositionSlotListCommand) -> KernelResult<Vec<AgentCompositionSlotRecord>> {
        self.authorize(
            "agent.business.composition_slot.list",
            command.requested_by,
            format!("agent.business.{}", command.agent_id),
            "composition_slot.list")?;
        validate_agent_id(command.agent_id.as_str())?;
        self.repository
            .get(command.tenant_id, command.agent_id.as_str())
            .ok_or_else(|| KernelError::validation("agent not found"))?;
        Ok(self
            .repository
            .list_composition_slots(command.tenant_id, command.agent_id.as_str()))
    }

    pub fn get_composition_slot(
        &mut self,
        command: AgentCompositionSlotGetCommand) -> KernelResult<AgentCompositionSlotRecord> {
        self.authorize(
            "agent.business.composition_slot.retrieve",
            command.requested_by,
            format!(
                "agent.business.composition_slot.{}.{}",
                command.agent_id, command.slot_id
            ),
            "composition_slot.retrieve")?;
        validate_agent_id(command.agent_id.as_str())?;
        validate_standard_id(command.slot_id.as_str(), "slotId", Some("slot."))?;
        self.repository
            .get_composition_slot(
                command.tenant_id,
                command.agent_id.as_str(),
                command.slot_id.as_str())
            .ok_or_else(|| KernelError::validation("composition slot not found"))
    }

    pub fn update_composition_slot(
        &mut self,
        command: AgentCompositionSlotUpdateCommand) -> KernelResult<AgentCompositionSlotRecord> {
        self.authorize(
            "agent.business.composition_slot.update",
            command.requested_by.clone(),
            format!(
                "agent.business.composition_slot.{}.{}",
                command.agent_id, command.slot_id
            ),
            "composition_slot.update")?;
        validate_agent_id(command.agent_id.as_str())?;
        validate_standard_id(command.slot_id.as_str(), "slotId", Some("slot."))?;
        let mut record = self
            .repository
            .get_composition_slot(
                command.tenant_id,
                command.agent_id.as_str(),
                command.slot_id.as_str())
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
        &mut self,
        command: AgentCompositionSlotDeleteCommand) -> KernelResult<AgentCompositionSlotRecord> {
        self.authorize(
            "agent.business.composition_slot.delete",
            command.requested_by.clone(),
            format!(
                "agent.business.composition_slot.{}.{}",
                command.agent_id, command.slot_id
            ),
            "composition_slot.delete")?;
        validate_agent_id(command.agent_id.as_str())?;
        validate_standard_id(command.slot_id.as_str(), "slotId", Some("slot."))?;
        let mut record = self
            .repository
            .get_composition_slot(
                command.tenant_id,
                command.agent_id.as_str(),
                command.slot_id.as_str())
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
    pub fn update_agent(
        &mut self,
        command: UpdateAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.update",
            command.requested_by.clone(),
            policy_resource,
            "update")?;

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
            command.requested_by,
            command.requested_at)?;
        Ok(record)
    }

    pub fn change_status(
        &mut self,
        command: ChangeAgentStatusCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.status.update",
            command.requested_by.clone(),
            policy_resource,
            "change_status")?;

        let mut record = self
            .repository
            .get(command.tenant_id, command.agent_id.as_str())
            .ok_or_else(|| KernelError::validation("agent not found"))?;

        if record.is_deleted() {
            return Err(KernelError::validation(
                "deleted agent status cannot be changed"));
        }
        ensure_expected_version(record.version, command.expected_version, "agent")?;

        if !is_valid_status_transition(record.status, command.target_status) {
            return Err(KernelError::validation("invalid agent status transition"));
        }

        record.status = command.target_status;
        record.mark_updated(command.requested_at.clone());
        self.repository.update(record.clone())?;
        self.emit_audit_event(
            AgentAuditAction::ChangeStatus,
            &record,
            command.requested_by,
            command.requested_at)?;
        Ok(record)
    }

    pub fn delete_agent(
        &mut self,
        command: DeleteAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.delete",
            command.requested_by.clone(),
            policy_resource,
            "delete")?;

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
            command.requested_by,
            command.requested_at)?;
        Ok(record)
    }

    pub fn restore_agent(
        &mut self,
        command: RestoreAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.restore",
            command.requested_by.clone(),
            policy_resource,
            "restore")?;

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
            command.requested_by,
            command.requested_at)?;
        Ok(record)
    }

    pub fn get_agent(&mut self, command: GetAgentCommand) -> KernelResult<AgentBusinessRecord> {
        validate_agent_id(command.agent_id.as_str())?;
        let policy_resource = format!("agent.business.{}", command.agent_id);
        self.authorize(
            "agent.business.retrieve",
            command.requested_by,
            policy_resource,
            "retrieve")?;

        self.repository
            .get(command.tenant_id, command.agent_id.as_str())
            .ok_or_else(|| KernelError::validation("agent not found"))
    }

    pub fn list_agents(
        &mut self,
        command: ListAgentsCommand) -> KernelResult<Vec<AgentBusinessRecord>> {
        self.authorize(
            "agent.business.list",
            command.requested_by,
            format!("agent.business.tenant.{}", command.query.tenant_id),
            "list")?;
        Ok(self.repository.list(&command.query))
    }

    pub fn list_agent_audit_events(
        &mut self,
        tenant_id: u64,
        agent_id: &str,
        requested_by: PolicySubject) -> KernelResult<Vec<KernelEvent>> {
        validate_agent_id(agent_id)?;
        self.authorize(
            "agent.business.audit.read",
            requested_by,
            format!("agent.business.{}", agent_id),
            "audit.read")?;
        self.audit_sink.list_events(tenant_id, agent_id)
    }

    fn authorize(
        &mut self,
        request_id: impl Into<String>,
        subject: PolicySubject,
        resource: impl Into<String>,
        action: impl Into<String>) -> KernelResult<()> {
        let policy_request = PolicyRequest::new(
            request_id,
            DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY,
            resource)
        .with_category(PolicyCategory::ProductSpecific(
            DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY.to_string()))
        .with_subject(subject)
        .with_action(action)
        .with_redaction(KernelEventRedaction::TenantSensitive);

        let decision = self.policy_provider.evaluate(policy_request)?;
        if decision.decision != PolicyDecisionValue::Allow {
            return Err(KernelError::permission_required(
                decision
                    .safe_reason
                    .unwrap_or_else(|| "agent management denied".to_string())));
        }
        Ok(())
    }

    fn emit_audit_event(
        &mut self,
        action: AgentAuditAction,
        record: &AgentBusinessRecord,
        subject: PolicySubject,
        occurred_at: String) -> KernelResult<()> {
        let payload = format!(
            "action={};agent_id={};tenant_id={};organization_id={};owner_user_id={};status={};visibility={}",
            action.event_type(),
            record.agent_id,
            record.tenant_id,
            record.organization_id,
            record.owner_user_id,
            record.status.as_str(),
            record.visibility.as_str()
        );
        let event = KernelEvent::new(
            format!("agent_audit_{}_{}", record.agent_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload)
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .with_context("organization_id", record.organization_id.to_string().as_str())
        .with_context("agent_internal_id", record.id.to_string().as_str())
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.audit.v1");

        self.audit_sink.record(event)
    }

    fn deactivate_provider_bindings(
        &mut self,
        tenant_id: u64,
        agent_id: &str,
        updated_at: String) -> KernelResult<()> {
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
        &mut self,
        action: AgentAuditAction,
        record: &AgentProviderBindingRecord,
        subject: PolicySubject,
        occurred_at: String) -> KernelResult<()> {
        let payload = format!(
            "action={};agent_id={};tenant_id={};binding_id={};provider_id={};implementation_kind={};active={}",
            action.event_type(),
            record.agent_id,
            record.tenant_id,
            record.binding_id,
            record.provider_id,
            record.implementation_kind.as_str(),
            record.active
        );
        let event = KernelEvent::new(
            format!("agent_binding_{}_{}", record.binding_id, record.version),
            action.event_type(),
            KernelEventSeverity::Info,
            payload)
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.provider_binding.v1");

        self.audit_sink.record(event)
    }

    fn emit_runtime_execution_audit_event(
        &mut self,
        action: AgentAuditAction,
        record: &AgentRuntimeExecutionRecord,
        subject: PolicySubject,
        occurred_at: String) -> KernelResult<()> {
        let payload = format!(
            "action={};agent_id={};tenant_id={};execution_id={};operation={};status={}",
            action.event_type(),
            record.agent_id,
            record.tenant_id,
            record.execution_id,
            record.operation.as_str(),
            record.status.as_str()
        );
        let event = KernelEvent::new(
            format!("agent_runtime_execution_{}", record.execution_id),
            action.event_type(),
            KernelEventSeverity::Info,
            payload)
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("subject_id", subject.subject_id.as_str())
        .with_context("subject_tenant_id", subject.tenant_id.as_str())
        .with_context("agent_id", record.agent_id.as_str())
        .with_context("tenant_id", record.tenant_id.to_string().as_str())
        .occurred_at(occurred_at)
        .with_payload_schema("sdkwork.agent.business.runtime_execution.v1");

        self.audit_sink.record(event)
    }

    fn emit_marketplace_audit_event(
        &mut self,
        input: AgentBusinessAuditEventInput<'_>) -> KernelResult<()> {
        let payload = format!(
            "action={};item_kind={};item_id={};tenant_id={};organization_id={};status={};visibility={}",
            input.action.event_type(),
            input.item_kind,
            input.item_id,
            input.tenant_id,
            input.organization_id,
            input.status.as_str(),
            input.visibility.as_str()
        );
        let event = KernelEvent::new(
            format!(
                "agent_marketplace_{}_{}_{}",
                input.item_kind, input.item_id, input.version
            ),
            input.action.event_type(),
            KernelEventSeverity::Info,
            payload)
        .from_source(KernelEventSource::Runtime)
        .with_redaction(KernelEventRedaction::TenantSensitive)
        .with_context("subject_id", input.subject.subject_id.as_str())
        .with_context("subject_tenant_id", input.subject.tenant_id.as_str())
        .with_context("tenant_id", input.tenant_id.to_string().as_str())
        .with_context("organization_id", input.organization_id.to_string().as_str())
        .occurred_at(input.occurred_at)
        .with_payload_schema("sdkwork.agent.business.marketplace.v1");

        self.audit_sink.record(event)
    }




}

trait KernelEventExt {
    fn with_context(self, key: impl Into<String>, value: impl Into<String>) -> Self;
}

impl KernelEventExt for KernelEvent {
    fn with_context(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let payload = format!("{};{}={}", self.payload, key.into(), value.into());
        KernelEvent { payload, ..self }
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
    entity_name: &str) -> KernelResult<()> {
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

fn normalize_prompt_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
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
