use crate::application::{
    ActivateAgentProviderBindingCommand, AgentPreviewResponseCommand,
    AgentPromptOptimizationCommand, AgentProviderBindingCommand, AnswerInteractionCommand,
    ApproveInteractionCommand, ArchiveSessionCommand, CancelTaskCommand, ChangeAgentStatusCommand,
    CloseSessionCommand, CreateAgentCommand, CreateInteractionCommand, CreateSessionItemCommand,
    CreateSessionCommand, CreateTaskCommand, DeleteAgentCommand, ExecuteTaskCommand,
    GetAgentCommand, ListAgentsCommand, ListInteractionsCommand, ListSessionItemsCommand,
    ListSessionsCommand, ListTasksCommand, RestoreAgentCommand, UpdateAgentCommand,
};
use crate::domain::{
    AgentBusinessRecord, AgentBusinessStatus, AgentCompositionSlotRecord, AgentImplementationKind,
    AgentImplementationType, AgentInteractionKind, AgentInteractionRecord, AgentInteractionStatus,
    AgentItemDriveRefRecord, AgentItemFeedbackRecord, AgentProviderBindingRecord,
    AgentResourceUserStateRecord, AgentRuntimeExecutionRecord, AgentSessionEntrySurface,
    AgentSessionItemKind, AgentSessionItemRecord, AgentSessionItemStatus, AgentSessionKind,
    AgentSessionRecord, AgentSessionStatus, AgentTaskRecord, AgentTaskStatus, AgentVisibility,
};
use crate::ports::{
    AgentListQuery, InteractionListQuery, SessionItemListQuery, PaginationParams, SessionListQuery,
    TaskListQuery,
};
use crate::validation::{
    is_trimmed_blank, parse_expected_version, parse_organization_id, parse_owner_user_id,
    parse_tenant_id, validate_requested_at,
};
use sdkwork_agent_kernel::{AgentManifest, KernelError, KernelResult, PolicySubject};
use sdkwork_code_kernel::CodeTaskIntent;
use serde_json::{json, Map, Value};

const AGENT_MANAGEMENT_PROFILE_CONSTRAINT_PREFIX: &str = "sdkwork.agent.pc.config:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListAgentsRequestDto {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: Option<String>,
    pub include_deleted: bool,
    pub search_query: Option<String>,
    pub visibility: Option<String>,
    pub pagination: PaginationParams,
}

impl ListAgentsRequestDto {
    pub fn into_command(self, requested_by: PolicySubject) -> KernelResult<ListAgentsCommand> {
        let mut query = AgentListQuery::for_tenant(parse_tenant_id(&self.tenant_id)?);
        if let Some(organization_id) = self.organization_id {
            query = query.for_organization(parse_organization_id(&organization_id)?);
        }
        if let Some(owner_user_id) = self.owner_user_id {
            query = query.for_owner(parse_owner_user_id(&owner_user_id)?);
        }
        if self.include_deleted {
            query = query.with_deleted();
        }
        if let Some(search_query) = self.search_query {
            query = query.with_search(search_query);
        }
        if let Some(visibility) = self.visibility {
            query = query.with_visibility(parse_visibility(&visibility)?);
        }
        query = query.with_pagination(self.pagination);
        Ok(ListAgentsCommand {
            query,
            requested_by,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAgentRequestDto {
    pub agent_id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub owner_user_id: String,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub manifest: AgentManifest,
    pub visibility: String,
    pub tags: Vec<String>,
    pub default_code_task_intent: Option<CodeTaskIntent>,
    pub implementation_provider_id: Option<String>,
    pub implementation_kind: Option<String>,
    pub implementation_type: Option<String>,
    pub requested_at: String,
}

impl CreateAgentRequestDto {
    pub fn into_command(self, requested_by: PolicySubject) -> KernelResult<CreateAgentCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(CreateAgentCommand {
            agent_id: self.agent_id,
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            organization_id: parse_organization_id(&self.organization_id)?,
            owner_user_id: parse_owner_user_id(&self.owner_user_id)?,
            code: self.code,
            display_name: self.display_name,
            description: self.description,
            manifest: self.manifest,
            visibility: parse_visibility(&self.visibility)?,
            tags: self.tags,
            default_code_task_intent: self.default_code_task_intent,
            implementation_provider_id: self.implementation_provider_id,
            implementation_kind: self
                .implementation_kind
                .as_deref()
                .map(parse_implementation_kind)
                .transpose()?,
            implementation_type: self
                .implementation_type
                .as_deref()
                .map(parse_implementation_type)
                .transpose()?,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderBindingRequestDto {
    pub tenant_id: String,
    pub agent_id: String,
    pub binding_id: String,
    pub provider_id: String,
    pub implementation_kind: String,
    pub configuration_profile_id: String,
    pub capabilities: Vec<String>,
    pub make_default: bool,
    pub requested_at: String,
}

impl AgentProviderBindingRequestDto {
    pub fn into_command(
        self,
        requested_by: PolicySubject,
    ) -> KernelResult<AgentProviderBindingCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(AgentProviderBindingCommand {
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            agent_id: self.agent_id,
            binding_id: self.binding_id,
            provider_id: self.provider_id,
            implementation_kind: parse_implementation_kind(&self.implementation_kind)?,
            configuration_profile_id: self.configuration_profile_id,
            capabilities: self.capabilities,
            make_default: self.make_default,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivateAgentProviderBindingRequestDto {
    pub tenant_id: String,
    pub agent_id: String,
    pub binding_id: String,
    pub requested_at: String,
}

impl ActivateAgentProviderBindingRequestDto {
    pub fn into_command(
        self,
        requested_by: PolicySubject,
    ) -> KernelResult<ActivateAgentProviderBindingCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(ActivateAgentProviderBindingCommand {
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            agent_id: self.agent_id,
            binding_id: self.binding_id,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentPreviewResponseRequestDto {
    pub tenant_id: String,
    pub agent_id: String,
    pub execution_id: String,
    pub content: String,
    pub debug_mode: bool,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub input_payload_json: String,
    pub requested_at: String,
}

impl AgentPreviewResponseRequestDto {
    pub fn into_command(
        self,
        requested_by: PolicySubject,
    ) -> KernelResult<AgentPreviewResponseCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(AgentPreviewResponseCommand {
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            agent_id: self.agent_id,
            execution_id: self.execution_id,
            content: self.content,
            debug_mode: self.debug_mode,
            model: self.model,
            temperature: self.temperature,
            input_payload_json: self.input_payload_json,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentPromptOptimizationRequestDto {
    pub tenant_id: String,
    pub agent_id: String,
    pub execution_id: String,
    pub prompt: String,
    pub input_payload_json: String,
    pub requested_at: String,
}

impl AgentPromptOptimizationRequestDto {
    pub fn into_command(
        self,
        requested_by: PolicySubject,
    ) -> KernelResult<AgentPromptOptimizationCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(AgentPromptOptimizationCommand {
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            agent_id: self.agent_id,
            execution_id: self.execution_id,
            prompt: self.prompt,
            input_payload_json: self.input_payload_json,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAgentRequestDto {
    pub tenant_id: String,
    pub agent_id: String,
    pub expected_version: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub manifest: Option<AgentManifest>,
    pub visibility: Option<String>,
    pub tags: Option<Vec<String>>,
    pub default_code_task_intent: Option<CodeTaskIntent>,
    pub implementation_provider_id: Option<Option<String>>,
    pub implementation_kind: Option<Option<String>>,
    pub implementation_type: Option<String>,
    pub requested_at: String,
}

impl UpdateAgentRequestDto {
    pub fn into_command(self, requested_by: PolicySubject) -> KernelResult<UpdateAgentCommand> {
        validate_requested_at(&self.requested_at)?;
        let visibility = self
            .visibility
            .as_ref()
            .map(|value| parse_visibility(value))
            .transpose()?;
        let expected_version = self
            .expected_version
            .as_deref()
            .map(parse_expected_version)
            .transpose()?;
        Ok(UpdateAgentCommand {
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            agent_id: self.agent_id,
            expected_version,
            display_name: self.display_name,
            description: self.description,
            manifest: self.manifest,
            visibility,
            tags: self.tags,
            default_code_task_intent: self.default_code_task_intent,
            implementation_provider_id: self.implementation_provider_id,
            implementation_kind: self
                .implementation_kind
                .map(|value| value.as_deref().map(parse_implementation_kind).transpose())
                .transpose()?,
            implementation_type: self
                .implementation_type
                .as_deref()
                .map(parse_implementation_type)
                .transpose()?,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAgentStatusRequestDto {
    pub tenant_id: String,
    pub agent_id: String,
    pub expected_version: Option<String>,
    pub target_status: String,
    pub requested_at: String,
}

impl UpdateAgentStatusRequestDto {
    pub fn into_command(
        self,
        requested_by: PolicySubject,
    ) -> KernelResult<ChangeAgentStatusCommand> {
        validate_requested_at(&self.requested_at)?;
        let expected_version = self
            .expected_version
            .as_deref()
            .map(parse_expected_version)
            .transpose()?;
        Ok(ChangeAgentStatusCommand {
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            agent_id: self.agent_id,
            expected_version,
            target_status: parse_status(&self.target_status)?,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteAgentRequestDto {
    pub tenant_id: String,
    pub agent_id: String,
    pub expected_version: Option<String>,
    pub requested_at: String,
}

impl DeleteAgentRequestDto {
    pub fn into_command(self, requested_by: PolicySubject) -> KernelResult<DeleteAgentCommand> {
        validate_requested_at(&self.requested_at)?;
        let expected_version = self
            .expected_version
            .as_deref()
            .map(parse_expected_version)
            .transpose()?;
        Ok(DeleteAgentCommand {
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            agent_id: self.agent_id,
            expected_version,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreAgentRequestDto {
    pub tenant_id: String,
    pub agent_id: String,
    pub expected_version: Option<String>,
    pub requested_at: String,
}

impl RestoreAgentRequestDto {
    pub fn into_command(self, requested_by: PolicySubject) -> KernelResult<RestoreAgentCommand> {
        validate_requested_at(&self.requested_at)?;
        let expected_version = self
            .expected_version
            .as_deref()
            .map(parse_expected_version)
            .transpose()?;
        Ok(RestoreAgentCommand {
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            agent_id: self.agent_id,
            expected_version,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetAgentRequestDto {
    pub tenant_id: String,
    pub agent_id: String,
}

impl GetAgentRequestDto {
    pub fn into_command(self, requested_by: PolicySubject) -> KernelResult<GetAgentCommand> {
        Ok(GetAgentCommand {
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            agent_id: self.agent_id,
            requested_by,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentRecordDto {
    pub id: String,
    pub agent_id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub owner_user_id: String,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub manifest: AgentManifest,
    pub default_code_task_intent: Option<CodeTaskIntent>,
    pub management_profile: Option<AgentManagementProfileDto>,
    pub implementation_provider_id: Option<String>,
    pub implementation_kind: Option<String>,
    pub implementation_type: String,
    pub status: String,
    pub visibility: String,
    pub tags: Vec<String>,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

impl AgentRecordDto {
    pub fn from_record(record: &AgentBusinessRecord) -> Self {
        Self {
            id: record.id.to_string(),
            agent_id: record.agent_id.clone(),
            tenant_id: record.tenant_id.to_string(),
            organization_id: record.organization_id.to_string(),
            owner_user_id: record.owner_user_id.to_string(),
            code: record.code.clone(),
            display_name: record.display_name.clone(),
            description: record.description.clone(),
            manifest: record.manifest.clone(),
            default_code_task_intent: record.default_code_task_intent.clone(),
            management_profile: AgentManagementProfileDto::from_default_code_task_intent(
                record.default_code_task_intent.as_ref(),
            ),
            implementation_provider_id: record.implementation_provider_id.clone(),
            implementation_kind: record
                .implementation_kind
                .map(|kind| kind.as_str().to_string()),
            implementation_type: record.implementation_type.as_str().to_string(),
            status: record.status.as_str().to_string(),
            visibility: record.visibility.as_str().to_string(),
            tags: record.tags.clone(),
            version: record.version.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            deleted_at: record.deleted_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentManagementProfileDto {
    pub author: Option<String>,
    pub avatar: Option<String>,
    pub category_id: Option<String>,
    pub color: Option<String>,
    pub debug_mode: Option<bool>,
    pub icon_name: Option<String>,
    pub json_mode: Option<bool>,
    pub knowledge_base_ids: Vec<String>,
    pub memory_enabled: Option<bool>,
    pub model: Option<String>,
    pub skill_ids: Vec<String>,
    pub suggested_prompts: Vec<String>,
    pub system_prompt: Option<String>,
    pub temperature: Option<f64>,
    pub tool_ids: Vec<String>,
    pub agent_type: Option<String>,
    pub users: Option<String>,
    pub voice_ids: Vec<String>,
    pub welcome_message: Option<String>,
}

impl AgentManagementProfileDto {
    pub fn merge_into_default_code_task_intent(
        self,
        intent: Option<CodeTaskIntent>,
    ) -> KernelResult<Option<CodeTaskIntent>> {
        if self.is_empty() {
            return Ok(intent);
        }

        let mut intent = intent.unwrap_or_else(|| {
            CodeTaskIntent::new(
                self.system_prompt
                    .clone()
                    .or_else(|| self.welcome_message.clone())
                    .unwrap_or_else(|| "Agent management profile".to_string()),
            )
        });

        if let Some(system_prompt) = self.system_prompt.as_ref() {
            if is_trimmed_blank(&intent.prompt) || intent.prompt == "Agent management profile" {
                intent.prompt = system_prompt.clone();
            }
        }

        intent.constraints.retain(|constraint| {
            !constraint.starts_with(AGENT_MANAGEMENT_PROFILE_CONSTRAINT_PREFIX)
                && !constraint.starts_with("agent.type=")
        });
        if let Some(agent_type) = self.agent_type.as_ref().and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        }) {
            intent.constraints.push(format!("agent.type={agent_type}"));
        }
        for knowledge_base_id in &self.knowledge_base_ids {
            if !intent
                .context_paths
                .iter()
                .any(|path| path == knowledge_base_id)
            {
                intent.context_paths.push(knowledge_base_id.clone());
            }
        }
        let encoded = serde_json::to_string(&self.to_pc_config_value()).map_err(|error| {
            KernelError::validation(format!("managementProfile json encode failed: {error}"))
        })?;
        intent.constraints.push(format!(
            "{AGENT_MANAGEMENT_PROFILE_CONSTRAINT_PREFIX}{encoded}"
        ));

        Ok(Some(intent))
    }

    pub fn from_default_code_task_intent(intent: Option<&CodeTaskIntent>) -> Option<Self> {
        let intent = intent?;
        let mut profile = intent
            .constraints
            .iter()
            .find_map(|constraint| Self::from_compatible_constraint(constraint.as_str()))?;

        if profile.agent_type.is_none() {
            profile.agent_type = intent
                .constraints
                .iter()
                .find_map(|constraint| constraint.strip_prefix("agent.type="))
                .and_then(normalize_optional_string);
        }

        Some(profile)
    }

    fn from_compatible_constraint(constraint: &str) -> Option<Self> {
        let encoded = constraint.strip_prefix(AGENT_MANAGEMENT_PROFILE_CONSTRAINT_PREFIX)?;
        let value: Value = serde_json::from_str(encoded).ok()?;
        let object = value.as_object()?;
        let profile = Self {
            author: optional_object_string(object.get("author")),
            avatar: optional_object_string(object.get("avatar")),
            category_id: optional_object_string(object.get("categoryId")),
            color: optional_object_string(object.get("color")),
            debug_mode: optional_object_bool(object.get("debugMode")),
            icon_name: optional_object_string(object.get("iconName")),
            json_mode: optional_object_bool(object.get("jsonMode")),
            knowledge_base_ids: object_string_array(object.get("knowledgeBaseIds")),
            memory_enabled: optional_object_bool(object.get("memoryEnabled")),
            model: optional_object_string(object.get("model")),
            skill_ids: object_string_array(object.get("skillIds")),
            suggested_prompts: object_string_array(object.get("suggestedPrompts")),
            system_prompt: optional_object_string(object.get("systemPrompt")),
            temperature: optional_object_f64(object.get("temperature")),
            tool_ids: object_string_array(object.get("toolIds")),
            agent_type: optional_object_string(object.get("type")),
            users: optional_object_string(object.get("users")),
            voice_ids: object_string_array(object.get("voiceIds")),
            welcome_message: optional_object_string(object.get("welcomeMessage")),
        };

        if profile.is_empty() {
            None
        } else {
            Some(profile)
        }
    }

    fn to_pc_config_value(&self) -> Value {
        let mut object = Map::new();
        insert_optional_string(&mut object, "author", self.author.as_ref());
        insert_optional_string(&mut object, "avatar", self.avatar.as_ref());
        insert_optional_string(&mut object, "categoryId", self.category_id.as_ref());
        insert_optional_string(&mut object, "color", self.color.as_ref());
        insert_optional_bool(&mut object, "debugMode", self.debug_mode);
        insert_optional_string(&mut object, "iconName", self.icon_name.as_ref());
        insert_optional_bool(&mut object, "jsonMode", self.json_mode);
        insert_string_array(&mut object, "knowledgeBaseIds", &self.knowledge_base_ids);
        insert_optional_bool(&mut object, "memoryEnabled", self.memory_enabled);
        insert_optional_string(&mut object, "model", self.model.as_ref());
        insert_string_array(&mut object, "skillIds", &self.skill_ids);
        insert_string_array(&mut object, "suggestedPrompts", &self.suggested_prompts);
        insert_optional_string(&mut object, "systemPrompt", self.system_prompt.as_ref());
        insert_optional_f64(&mut object, "temperature", self.temperature);
        insert_string_array(&mut object, "toolIds", &self.tool_ids);
        insert_optional_string(&mut object, "type", self.agent_type.as_ref());
        insert_optional_string(&mut object, "users", self.users.as_ref());
        insert_string_array(&mut object, "voiceIds", &self.voice_ids);
        insert_optional_string(&mut object, "welcomeMessage", self.welcome_message.as_ref());
        Value::Object(object)
    }

    fn is_empty(&self) -> bool {
        self.author.is_none()
            && self.avatar.is_none()
            && self.category_id.is_none()
            && self.color.is_none()
            && self.debug_mode.is_none()
            && self.icon_name.is_none()
            && self.json_mode.is_none()
            && self.knowledge_base_ids.is_empty()
            && self.memory_enabled.is_none()
            && self.model.is_none()
            && self.skill_ids.is_empty()
            && self.suggested_prompts.is_empty()
            && self.system_prompt.is_none()
            && self.temperature.is_none()
            && self.tool_ids.is_empty()
            && self.agent_type.is_none()
            && self.users.is_none()
            && self.voice_ids.is_empty()
            && self.welcome_message.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderBindingRecordDto {
    pub tenant_id: String,
    pub agent_id: String,
    pub binding_id: String,
    pub provider_id: String,
    pub implementation_kind: String,
    pub configuration_profile_id: String,
    pub capabilities: Vec<String>,
    pub active: bool,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

impl AgentProviderBindingRecordDto {
    pub fn from_record(record: &AgentProviderBindingRecord) -> Self {
        Self {
            tenant_id: record.tenant_id.to_string(),
            agent_id: record.agent_id.clone(),
            binding_id: record.binding_id.clone(),
            provider_id: record.provider_id.clone(),
            implementation_kind: record.implementation_kind.as_str().to_string(),
            configuration_profile_id: record.configuration_profile_id.clone(),
            capabilities: record.capabilities.clone(),
            active: record.active,
            version: record.version.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderBindingResponseDto {
    pub data: AgentProviderBindingRecordDto,
}

impl AgentProviderBindingResponseDto {
    pub fn from_record(record: &AgentProviderBindingRecord) -> Self {
        Self {
            data: AgentProviderBindingRecordDto::from_record(record),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderBindingListDataDto {
    pub items: Vec<AgentProviderBindingRecordDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderBindingListResponseDto {
    pub data: AgentProviderBindingListDataDto,
}

impl AgentProviderBindingListResponseDto {
    pub fn from_records(records: &[AgentProviderBindingRecord]) -> Self {
        Self {
            data: AgentProviderBindingListDataDto {
                items: records
                    .iter()
                    .map(AgentProviderBindingRecordDto::from_record)
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentResponseDto {
    pub data: AgentRecordDto,
}

impl AgentResponseDto {
    pub fn from_record(record: &AgentBusinessRecord) -> Self {
        Self {
            data: AgentRecordDto::from_record(record),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentListDataDto {
    pub items: Vec<AgentRecordDto>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentListResponseDto {
    pub data: AgentListDataDto,
}

impl AgentListResponseDto {
    pub fn from_records(records: &[AgentBusinessRecord]) -> Self {
        Self {
            data: AgentListDataDto {
                items: records.iter().map(AgentRecordDto::from_record).collect(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRuntimeExecutionRecordDto {
    pub tenant_id: String,
    pub agent_id: String,
    pub execution_id: String,
    pub operation: String,
    pub status: String,
    pub input_payload_json: String,
    pub output_payload_json: String,
    pub requested_at: String,
    pub completed_at: String,
}

impl AgentRuntimeExecutionRecordDto {
    pub fn from_record(record: &AgentRuntimeExecutionRecord) -> Self {
        Self {
            tenant_id: record.tenant_id.to_string(),
            agent_id: record.agent_id.clone(),
            execution_id: record.execution_id.clone(),
            operation: record.operation.as_str().to_string(),
            status: record.status.as_str().to_string(),
            input_payload_json: record.input_payload_json.clone(),
            output_payload_json: record.output_payload_json.clone(),
            requested_at: record.requested_at.clone(),
            completed_at: record.completed_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRuntimeExecutionResponseDto {
    pub data: AgentRuntimeExecutionRecordDto,
}

impl AgentRuntimeExecutionResponseDto {
    pub fn from_record(record: &AgentRuntimeExecutionRecord) -> Self {
        Self {
            data: AgentRuntimeExecutionRecordDto::from_record(record),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCompositionSlotRecordDto {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub agent_id: String,
    pub slot_id: String,
    pub slot_kind: String,
    pub target_module: String,
    pub target_ref: String,
    pub target_version_ref: Option<String>,
    pub priority: String,
    pub enabled: bool,
    pub policy_json: String,
    pub status: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

impl AgentCompositionSlotRecordDto {
    pub fn from_record(record: &AgentCompositionSlotRecord) -> Self {
        Self {
            id: record.id.to_string(),
            tenant_id: record.tenant_id.to_string(),
            organization_id: record.organization_id.to_string(),
            agent_id: record.agent_id.clone(),
            slot_id: record.slot_id.clone(),
            slot_kind: record.slot_kind.as_str().to_string(),
            target_module: record.target_module.as_str().to_string(),
            target_ref: record.target_ref.clone(),
            target_version_ref: record.target_version_ref.clone(),
            priority: record.priority.to_string(),
            enabled: record.enabled,
            policy_json: record.policy_json.clone(),
            status: record.status.as_str().to_string(),
            version: record.version.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            deleted_at: record.deleted_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCompositionSlotCreateDataDto {
    pub tenant_id: String,
    pub organization_id: String,
    pub slot_id: String,
    pub slot_kind: String,
    pub target_module: String,
    pub target_ref: String,
    pub target_version_ref: Option<String>,
    pub priority: Option<String>,
    pub enabled: Option<bool>,
    pub policy_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCompositionSlotCreateRequestDto {
    pub data: AgentCompositionSlotCreateDataDto,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCompositionSlotUpdateDataDto {
    pub tenant_id: String,
    pub expected_version: Option<String>,
    pub slot_kind: Option<String>,
    pub target_module: Option<String>,
    pub target_ref: Option<String>,
    pub target_version_ref: Option<Option<String>>,
    pub priority: Option<String>,
    pub enabled: Option<bool>,
    pub policy_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCompositionSlotUpdateRequestDto {
    pub data: AgentCompositionSlotUpdateDataDto,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCompositionSlotDeleteRequestDto {
    pub tenant_id: String,
    pub expected_version: Option<String>,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCompositionSlotResponseDto {
    pub data: AgentCompositionSlotRecordDto,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCompositionSlotListDataDto {
    pub items: Vec<AgentCompositionSlotRecordDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCompositionSlotListResponseDto {
    pub data: AgentCompositionSlotListDataDto,
}

// ===========================================================================
// Session DTOs
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionDataDto {
    pub tenant_id: String,
    pub organization_id: String,
    pub owner_user_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    pub project_id: Option<String>,
    pub session_kind: Option<String>,
    pub entry_surface: Option<String>,
    pub source_module: Option<String>,
    pub source_context_kind: Option<String>,
    pub source_context_id: Option<String>,
    pub parent_session_id: Option<String>,
    pub forked_from_turn_id: Option<String>,
    pub title: Option<String>,
    pub idempotency_key: Option<String>,
    pub payload_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionRequestDto {
    pub data: CreateSessionDataDto,
    pub requested_at: String,
}

impl CreateSessionRequestDto {
    pub fn into_command(
        self,
        agent_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<CreateSessionCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(CreateSessionCommand {
            tenant_id: parse_tenant_id(&self.data.tenant_id)?,
            organization_id: parse_organization_id(&self.data.organization_id)?,
            agent_id,
            owner_user_id: parse_owner_user_id(&self.data.owner_user_id)?,
            session_id: self.data.session_id.unwrap_or_default(),
            project_id: self.data.project_id,
            session_kind: parse_session_kind(
                self.data.session_kind.as_deref().unwrap_or("assistant"),
            )?,
            entry_surface: parse_session_entry_surface(
                self.data.entry_surface.as_deref().unwrap_or("api"),
            )?,
            source_module: self.data.source_module,
            source_context_kind: self.data.source_context_kind,
            source_context_id: self.data.source_context_id,
            parent_session_id: self.data.parent_session_id,
            forked_from_turn_id: self.data.forked_from_turn_id,
            title: self.data.title,
            idempotency_key: self.data.idempotency_key,
            payload_hash: self.data.payload_hash,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CloseSessionRequestDto {
    #[serde(default)]
    pub tenant_id: String,
    #[serde(default)]
    pub organization_id: String,
    pub expected_version: Option<String>,
    pub requested_at: String,
}

impl CloseSessionRequestDto {
    pub fn into_command(
        self,
        session_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<CloseSessionCommand> {
        validate_requested_at(&self.requested_at)?;
        let expected_version = self
            .expected_version
            .as_deref()
            .map(parse_expected_version)
            .transpose()?;
        Ok(CloseSessionCommand {
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            organization_id: parse_organization_id(&self.organization_id)?,
            session_id,
            expected_version,
            owner_scope: None,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArchiveSessionRequestDto {
    #[serde(default)]
    pub tenant_id: String,
    #[serde(default)]
    pub organization_id: String,
    pub expected_version: Option<String>,
    pub requested_at: String,
}

impl ArchiveSessionRequestDto {
    pub fn into_command(
        self,
        session_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<ArchiveSessionCommand> {
        validate_requested_at(&self.requested_at)?;
        let expected_version = self
            .expected_version
            .as_deref()
            .map(parse_expected_version)
            .transpose()?;
        Ok(ArchiveSessionCommand {
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            organization_id: parse_organization_id(&self.organization_id)?,
            session_id,
            expected_version,
            owner_scope: None,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSessionsRequestDto {
    pub tenant_id: String,
    pub owner_user_id: Option<String>,
    pub status: Option<String>,
    pub include_archived: bool,
}

impl ListSessionsRequestDto {
    pub fn into_command(self, requested_by: PolicySubject) -> KernelResult<ListSessionsCommand> {
        let mut query = SessionListQuery::for_tenant(parse_tenant_id(&self.tenant_id)?);
        if let Some(owner_user_id) = self.owner_user_id {
            query = query.for_owner(parse_owner_user_id(&owner_user_id)?);
        }
        if let Some(status) = self.status {
            query = query.with_status(parse_session_status(&status)?);
        }
        if self.include_archived {
            query = query.include_archived();
        }
        Ok(ListSessionsCommand {
            query,
            requested_by,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionRecordDto {
    pub id: String,
    pub session_id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub agent_id: String,
    pub owner_user_id: String,
    pub project_id: Option<String>,
    pub session_kind: String,
    pub entry_surface: String,
    pub source_module: Option<String>,
    pub source_context_kind: Option<String>,
    pub source_context_id: Option<String>,
    pub parent_session_id: Option<String>,
    pub forked_from_turn_id: Option<String>,
    pub title: Option<String>,
    pub status: String,
    pub item_count: String,
    pub last_item_sequence: String,
    pub total_input_tokens: String,
    pub total_output_tokens: String,
    pub created_by: String,
    pub updated_by: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_item_at: Option<String>,
    pub closed_at: Option<String>,
    pub archived_at: Option<String>,
    pub archived_by: Option<String>,
    pub retention_until: Option<String>,
}

impl AgentSessionRecordDto {
    pub fn from_record(record: &AgentSessionRecord) -> Self {
        Self {
            id: record.id.to_string(),
            session_id: record.session_id.clone(),
            tenant_id: record.tenant_id.to_string(),
            organization_id: record.organization_id.to_string(),
            agent_id: record.agent_id.clone(),
            owner_user_id: record.owner_user_id.to_string(),
            project_id: record.project_id.clone(),
            session_kind: record.session_kind.as_str().to_string(),
            entry_surface: record.entry_surface.as_str().to_string(),
            source_module: record.source_module.clone(),
            source_context_kind: record.source_context_kind.clone(),
            source_context_id: record.source_context_id.clone(),
            parent_session_id: record.parent_session_id.clone(),
            forked_from_turn_id: record.forked_from_turn_id.clone(),
            title: record.title.clone(),
            status: record.status.as_str().to_string(),
            item_count: record.item_count.to_string(),
            last_item_sequence: record.last_item_sequence.to_string(),
            total_input_tokens: record.total_input_tokens.to_string(),
            total_output_tokens: record.total_output_tokens.to_string(),
            created_by: record.created_by.to_string(),
            updated_by: record.updated_by.to_string(),
            version: record.version.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            last_item_at: record.last_item_at.clone(),
            closed_at: record.closed_at.clone(),
            archived_at: record.archived_at.clone(),
            archived_by: record.archived_by.map(|value| value.to_string()),
            retention_until: record.retention_until.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionResponseDto {
    pub data: AgentSessionRecordDto,
}

impl AgentSessionResponseDto {
    pub fn from_record(record: &AgentSessionRecord) -> Self {
        Self {
            data: AgentSessionRecordDto::from_record(record),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionListDataDto {
    pub items: Vec<AgentSessionRecordDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionListResponseDto {
    pub data: AgentSessionListDataDto,
}

impl AgentSessionListResponseDto {
    pub fn from_records(records: &[AgentSessionRecord]) -> Self {
        Self {
            data: AgentSessionListDataDto {
                items: records
                    .iter()
                    .map(AgentSessionRecordDto::from_record)
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResourceUserStateRecordDto {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub user_id: String,
    pub resource_type: String,
    pub resource_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_opened_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_read_item_sequence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_title: Option<String>,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

impl AgentResourceUserStateRecordDto {
    pub fn from_record(record: &AgentResourceUserStateRecord) -> Self {
        Self {
            id: record.id.to_string(),
            tenant_id: record.tenant_id.to_string(),
            organization_id: record.organization_id.to_string(),
            user_id: record.user_id.to_string(),
            resource_type: record.resource_type.as_str().to_string(),
            resource_id: record.resource_id.clone(),
            pinned_at: record.pinned_at.clone(),
            hidden_at: record.hidden_at.clone(),
            last_opened_at: record.last_opened_at.clone(),
            last_read_item_sequence: record
                .last_read_item_sequence
                .map(|value| value.to_string()),
            custom_title: record.custom_title.clone(),
            version: record.version.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentItemFeedbackRecordDto {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub item_id: String,
    pub user_id: String,
    pub rating: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
}

impl AgentItemFeedbackRecordDto {
    pub fn from_record(record: &AgentItemFeedbackRecord) -> Self {
        Self {
            id: record.id.to_string(),
            tenant_id: record.tenant_id.to_string(),
            organization_id: record.organization_id.to_string(),
            item_id: record.item_id.clone(),
            user_id: record.user_id.to_string(),
            rating: record.rating.as_str().to_string(),
            reason_code: record.reason_code.clone(),
            comment: record.comment.clone(),
            version: record.version.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            deleted_at: record.deleted_at.clone(),
        }
    }
}

// ===========================================================================
// Task DTOs
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskDataDto {
    pub tenant_id: String,
    pub organization_id: String,
    pub owner_user_id: String,
    #[serde(default)]
    pub task_id: Option<String>,
    pub title: Option<String>,
    pub prompt: String,
    pub external_ref: Option<String>,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequestDto {
    pub data: CreateTaskDataDto,
    pub requested_at: String,
}

impl CreateTaskRequestDto {
    pub fn into_command(
        self,
        agent_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<CreateTaskCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(CreateTaskCommand {
            tenant_id: parse_tenant_id(&self.data.tenant_id)?,
            organization_id: parse_organization_id(&self.data.organization_id)?,
            agent_id,
            owner_user_id: parse_owner_user_id(&self.data.owner_user_id)?,
            task_id: self.data.task_id.unwrap_or_default(),
            title: self.data.title,
            prompt: self.data.prompt,
            external_ref: self.data.external_ref,
            metadata_json: self.data.metadata_json.unwrap_or_else(|| "{}".to_string()),
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelTaskRequestDto {
    pub tenant_id: String,
    pub expected_version: Option<String>,
    pub requested_at: String,
}

impl CancelTaskRequestDto {
    pub fn into_command(
        self,
        path_agent_id: String,
        task_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<CancelTaskCommand> {
        validate_requested_at(&self.requested_at)?;
        let expected_version = self
            .expected_version
            .as_deref()
            .map(parse_expected_version)
            .transpose()?;
        Ok(CancelTaskCommand {
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            path_agent_id,
            task_id,
            expected_version,
            owner_scope: None,
            requested_by,
            requested_at: self.requested_at,
        })
    }

    pub fn into_execute_command(
        self,
        path_agent_id: String,
        task_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<ExecuteTaskCommand> {
        validate_requested_at(&self.requested_at)?;
        let expected_version = self
            .expected_version
            .as_deref()
            .map(parse_expected_version)
            .transpose()?;
        Ok(ExecuteTaskCommand {
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            path_agent_id,
            task_id,
            expected_version,
            owner_scope: None,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListTasksRequestDto {
    pub tenant_id: String,
    pub owner_user_id: Option<String>,
    pub status: Option<String>,
}

impl ListTasksRequestDto {
    pub fn into_command(self, requested_by: PolicySubject) -> KernelResult<ListTasksCommand> {
        let mut query = TaskListQuery::for_tenant(parse_tenant_id(&self.tenant_id)?);
        if let Some(owner_user_id) = self.owner_user_id {
            query = query.for_owner(parse_owner_user_id(&owner_user_id)?);
        }
        if let Some(status) = self.status {
            query = query.with_status(parse_task_status(&status)?);
        }
        Ok(ListTasksCommand {
            query,
            requested_by,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTaskRecordDto {
    pub task_id: String,
    pub agent_id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub owner_user_id: String,
    pub title: Option<String>,
    pub prompt: String,
    pub status: String,
    pub external_ref: Option<String>,
    pub metadata_json: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub cancelled_at: Option<String>,
}

impl AgentTaskRecordDto {
    pub fn from_record(record: &AgentTaskRecord) -> Self {
        Self {
            task_id: record.task_id.clone(),
            agent_id: record.agent_id.clone(),
            tenant_id: record.tenant_id.to_string(),
            organization_id: record.organization_id.to_string(),
            owner_user_id: record.owner_user_id.to_string(),
            title: record.title.clone(),
            prompt: record.prompt.clone(),
            status: record.status.as_str().to_string(),
            external_ref: record.external_ref.clone(),
            metadata_json: record.metadata_json.clone(),
            version: record.version.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            started_at: record.started_at.clone(),
            completed_at: record.completed_at.clone(),
            cancelled_at: record.cancelled_at.clone(),
        }
    }
}

// ===========================================================================
// Interaction DTOs
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentInteractionOptionDto {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentInteractionResolutionDto {
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_option_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInteractionRecordDto {
    pub interaction_id: String,
    pub session_id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub turn_id: Option<String>,
    pub runtime_binding_id: Option<String>,
    pub provider_interaction_id: Option<String>,
    pub kind: String,
    pub status: String,
    pub prompt: String,
    pub options: Vec<AgentInteractionOptionDto>,
    pub resolution: Option<AgentInteractionResolutionDto>,
    pub claim_owner: Option<String>,
    pub claim_expires_at: Option<String>,
    pub fencing_token: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub resolved_at: Option<String>,
    pub retention_until: Option<String>,
}

impl AgentInteractionRecordDto {
    pub fn from_record(record: &AgentInteractionRecord) -> KernelResult<Self> {
        let options = serde_json::from_str(&record.options_json).map_err(|error| {
            KernelError::Internal {
                message: format!("stored interaction options are invalid: {error}"),
            }
        })?;
        let resolution = record
            .resolution_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|error| KernelError::Internal {
                message: format!("stored interaction resolution is invalid: {error}"),
            })?;
        Ok(Self {
            interaction_id: record.interaction_id.clone(),
            session_id: record.session_id.clone(),
            tenant_id: record.tenant_id.to_string(),
            organization_id: record.organization_id.to_string(),
            turn_id: record.turn_id.clone(),
            runtime_binding_id: record.runtime_binding_id.clone(),
            provider_interaction_id: record.provider_interaction_id.clone(),
            kind: record.kind.as_str().to_string(),
            status: record.status.as_str().to_string(),
            prompt: record.prompt.clone(),
            options,
            resolution,
            claim_owner: record.claim_owner.clone(),
            claim_expires_at: record.claim_expires_at.clone(),
            fencing_token: record.fencing_token.to_string(),
            version: record.version.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            resolved_at: record.resolved_at.clone(),
            retention_until: record.retention_until.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentInteractionOptionInputDto {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateInteractionRequestDto {
    pub interaction_id: Option<String>,
    pub turn_id: Option<String>,
    pub runtime_binding_id: Option<String>,
    pub provider_interaction_id: Option<String>,
    pub kind: String,
    pub prompt: String,
    #[serde(default)]
    pub options: Vec<AgentInteractionOptionInputDto>,
    pub retention_until: Option<String>,
    pub requested_at: String,
}

impl CreateInteractionRequestDto {
    pub fn into_command(
        self,
        tenant_id: u64,
        organization_id: u64,
        agent_id: String,
        session_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<CreateInteractionCommand> {
        validate_requested_at(&self.requested_at)?;
        let kind = AgentInteractionKind::from_code(self.kind.as_str())
            .ok_or_else(|| KernelError::validation("invalid interaction kind"))?;
        let options_json = serde_json::to_string(&self.options).map_err(|error| {
            KernelError::validation(format!("options serialization failed: {error}"))
        })?;
        Ok(CreateInteractionCommand {
            tenant_id,
            organization_id,
            session_id,
            path_agent_id: agent_id,
            interaction_id: self.interaction_id.unwrap_or_default(),
            turn_id: self.turn_id,
            runtime_binding_id: self.runtime_binding_id,
            provider_interaction_id: self.provider_interaction_id,
            kind,
            prompt: self.prompt,
            options_json,
            retention_until: self.retention_until,
            owner_scope: None,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListInteractionsRequestDto {
    pub tenant_id: String,
    pub organization_id: String,
    pub status: Option<String>,
}

impl ListInteractionsRequestDto {
    pub fn into_command(
        self,
        session_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<ListInteractionsCommand> {
        let mut query = InteractionListQuery::for_session(
            parse_tenant_id(&self.tenant_id)?,
            parse_organization_id(&self.organization_id)?,
            session_id,
        );
        if let Some(status) = self.status {
            query = query.with_status(parse_interaction_status(&status)?);
        }
        Ok(ListInteractionsCommand {
            query,
            path_agent_id: String::new(),
            owner_scope: None,
            requested_by,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApproveInteractionRequestDto {
    pub approved: bool,
    pub reason: Option<String>,
    pub claim_token: String,
    pub fencing_token: String,
    pub expected_version: String,
    pub requested_at: String,
}

impl ApproveInteractionRequestDto {
    pub fn into_command(
        self,
        tenant_id: u64,
        organization_id: u64,
        path_agent_id: String,
        session_id: String,
        interaction_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<ApproveInteractionCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(ApproveInteractionCommand {
            tenant_id,
            organization_id,
            path_agent_id,
            session_id,
            interaction_id,
            approved: self.approved,
            reason: self.reason,
            claim_token: self.claim_token,
            fencing_token: parse_u64(&self.fencing_token, "fencingToken")?,
            expected_version: parse_expected_version(&self.expected_version)?,
            owner_scope: None,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnswerInteractionRequestDto {
    pub answer: String,
    pub selected_option_value: Option<String>,
    pub rejected: bool,
    pub claim_token: String,
    pub fencing_token: String,
    pub expected_version: String,
    pub requested_at: String,
}

impl AnswerInteractionRequestDto {
    pub fn into_command(
        self,
        tenant_id: u64,
        organization_id: u64,
        path_agent_id: String,
        session_id: String,
        interaction_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<AnswerInteractionCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(AnswerInteractionCommand {
            tenant_id,
            organization_id,
            path_agent_id,
            session_id,
            interaction_id,
            answer: self.answer,
            selected_option_value: self.selected_option_value,
            rejected: self.rejected,
            claim_token: self.claim_token,
            fencing_token: parse_u64(&self.fencing_token, "fencingToken")?,
            expected_version: parse_expected_version(&self.expected_version)?,
            owner_scope: None,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClaimInteractionRequestDto {
    pub claim_owner: String,
    #[serde(default = "default_interaction_lease_seconds")]
    pub lease_seconds: u32,
    pub expected_version: String,
    pub requested_at: String,
}

impl ClaimInteractionRequestDto {
    pub fn into_command(
        self,
        tenant_id: u64,
        organization_id: u64,
        path_agent_id: String,
        session_id: String,
        interaction_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<crate::application::ClaimInteractionCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(crate::application::ClaimInteractionCommand {
            tenant_id,
            organization_id,
            path_agent_id,
            session_id,
            interaction_id,
            claim_owner: self.claim_owner,
            lease_seconds: self.lease_seconds,
            expected_version: parse_expected_version(&self.expected_version)?,
            owner_scope: None,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

const fn default_interaction_lease_seconds() -> u32 {
    60
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionClaimResultDto {
    pub interaction: AgentInteractionRecordDto,
    pub claim_token: String,
    pub claim_expires_at: String,
    pub fencing_token: String,
}

impl InteractionClaimResultDto {
    pub fn from_result(result: &crate::application::InteractionClaimResult) -> KernelResult<Self> {
        Ok(Self {
            interaction: AgentInteractionRecordDto::from_record(&result.interaction)?,
            claim_token: result.claim_token.clone(),
            claim_expires_at: result.claim_expires_at.clone(),
            fencing_token: result.fencing_token.to_string(),
        })
    }
}

// ===========================================================================
// Session item DTOs
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionItemDataDto {
    pub tenant_id: String,
    pub organization_id: String,
    pub item_id: String,
    pub kind: String,
    pub content: String,
    pub content_type: Option<String>,
    pub input_tokens: Option<String>,
    pub output_tokens: Option<String>,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub parent_item_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionItemRequestDto {
    pub data: CreateSessionItemDataDto,
    pub requested_at: String,
}

impl CreateSessionItemRequestDto {
    pub fn into_command(
        self,
        session_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<CreateSessionItemCommand> {
        validate_requested_at(&self.requested_at)?;
        let input_tokens = self
            .data
            .input_tokens
            .as_deref()
            .map(|v| parse_u64(v, "inputTokens"))
            .transpose()?
            .unwrap_or(0);
        let output_tokens = self
            .data
            .output_tokens
            .as_deref()
            .map(|v| parse_u64(v, "outputTokens"))
            .transpose()?
            .unwrap_or(0);
        Ok(CreateSessionItemCommand {
            tenant_id: parse_tenant_id(&self.data.tenant_id)?,
            organization_id: parse_organization_id(&self.data.organization_id)?,
            session_id,
            item_id: self.data.item_id,
            kind: parse_session_item_kind(&self.data.kind)?,
            content: self.data.content,
            content_type: self
                .data
                .content_type
                .unwrap_or_else(|| "text/plain".to_string()),
            input_tokens,
            output_tokens,
            model_id: self.data.model_id,
            provider_id: self.data.provider_id,
            parent_item_id: self.data.parent_item_id,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSessionItemsRequestDto {
    pub tenant_id: String,
    pub organization_id: String,
    pub kind: Option<String>,
    pub status: Option<String>,
}

impl ListSessionItemsRequestDto {
    pub fn into_command(
        self,
        session_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<ListSessionItemsCommand> {
        let mut query = SessionItemListQuery::for_session(
            parse_tenant_id(&self.tenant_id)?,
            parse_organization_id(&self.organization_id)?,
            session_id,
        );
        if let Some(kind) = self.kind {
            query = query.with_kind(parse_session_item_kind(&kind)?.as_str());
        }
        if let Some(status) = self.status {
            query = query.with_status(parse_session_item_status(&status)?);
        }
        Ok(ListSessionItemsCommand {
            query,
            owner_scope: None,
            requested_by,
        })
    }
}

fn parse_optional_json_object(
    raw: Option<&str>,
    field_name: &str,
) -> KernelResult<Option<Map<String, Value>>> {
    raw.map(|value| {
        serde_json::from_str::<Map<String, Value>>(value).map_err(|_| KernelError::Internal {
            message: format!("stored {field_name} is not a valid JSON object"),
        })
    })
    .transpose()
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionItemRecordDto {
    pub item_id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub session_id: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub content_type: String,
    pub status: String,
    pub sequence: String,
    pub input_tokens: String,
    pub output_tokens: String,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_arguments: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_result: Option<Map<String, Value>>,
    pub drive_refs: Vec<AgentItemDriveRefRecordDto>,
    pub parent_item_id: Option<String>,
    pub turn_id: Option<String>,
    pub created_by: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub redacted_at: Option<String>,
    pub redacted_by: Option<String>,
    pub retention_until: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentItemDriveRefRecordDto {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub item_id: String,
    pub resource_role: String,
    pub drive_space_id: String,
    pub drive_node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_resource_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_blob_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    pub sort_order: u32,
    pub status: i16,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention_until: Option<String>,
}

impl AgentItemDriveRefRecordDto {
    fn from_record(record: &AgentItemDriveRefRecord) -> Self {
        Self {
            id: record.id.to_string(),
            tenant_id: record.tenant_id.to_string(),
            organization_id: record.organization_id.to_string(),
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
            created_by: record.created_by.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            deleted_at: record.deleted_at.clone(),
            retention_until: record.retention_until.clone(),
        }
    }
}

impl AgentSessionItemRecordDto {
    pub fn from_record(record: &AgentSessionItemRecord) -> KernelResult<Self> {
        Ok(Self {
            item_id: record.item_id.clone(),
            tenant_id: record.tenant_id.to_string(),
            organization_id: record.organization_id.to_string(),
            session_id: record.session_id.clone(),
            kind: record.kind.as_str().to_string(),
            content: record.content.clone(),
            content_type: record.content_type.clone(),
            status: record.status.as_str().to_string(),
            sequence: record.sequence.to_string(),
            input_tokens: record.input_tokens.to_string(),
            output_tokens: record.output_tokens.to_string(),
            model_id: record.model_id.clone(),
            provider_id: record.provider_id.clone(),
            tool_name: record.tool_name.clone(),
            tool_call_id: record.tool_call_id.clone(),
            tool_arguments: parse_optional_json_object(
                record.tool_arguments_json.as_deref(),
                "toolArguments",
            )?,
            tool_result: parse_optional_json_object(
                record.tool_result_json.as_deref(),
                "toolResult",
            )?,
            drive_refs: Vec::new(),
            parent_item_id: record.parent_item_id.clone(),
            turn_id: record.turn_id.clone(),
            created_by: record.created_by.to_string(),
            version: record.version.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            completed_at: record.completed_at.clone(),
            redacted_at: record.redacted_at.clone(),
            redacted_by: record.redacted_by.map(|value| value.to_string()),
            retention_until: record.retention_until.clone(),
        })
    }

    pub fn from_record_with_drive_refs(
        record: &AgentSessionItemRecord,
        drive_refs: &[AgentItemDriveRefRecord],
    ) -> KernelResult<Self> {
        let mut dto = Self::from_record(record)?;
        dto.drive_refs = drive_refs
            .iter()
            .map(AgentItemDriveRefRecordDto::from_record)
            .collect();
        Ok(dto)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionItemResponseDto {
    pub data: AgentSessionItemRecordDto,
}

impl AgentSessionItemResponseDto {
    pub fn from_record(record: &AgentSessionItemRecord) -> KernelResult<Self> {
        Ok(Self {
            data: AgentSessionItemRecordDto::from_record(record)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionItemListDataDto {
    pub items: Vec<AgentSessionItemRecordDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionItemListResponseDto {
    pub data: AgentSessionItemListDataDto,
}

impl AgentSessionItemListResponseDto {
    pub fn from_records(records: &[AgentSessionItemRecord]) -> KernelResult<Self> {
        Ok(Self {
            data: AgentSessionItemListDataDto {
                items: records
                    .iter()
                    .map(AgentSessionItemRecordDto::from_record)
                    .collect::<KernelResult<Vec<_>>>()?,
            },
        })
    }
}

// ===========================================================================
// Turn DTOs
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTurnRecordDto {
    pub turn_id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub session_id: String,
    pub agent_id: String,
    pub owner_user_id: String,
    pub runtime_binding_id: Option<String>,
    pub client_request_id: Option<String>,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub request_item_id: String,
    pub response_item_id: Option<String>,
    pub turn_mode: String,
    pub status: String,
    pub requested_model_id: Option<String>,
    pub provider_binding_id: Option<String>,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub input_tokens: String,
    pub output_tokens: String,
    pub cached_tokens: String,
    pub finish_reason: Option<String>,
    pub error_code: Option<String>,
    pub error_detail: Option<String>,
    pub trace_id: Option<String>,
    pub attempt_count: u32,
    pub max_attempts: u32,
    pub next_retry_at: Option<String>,
    pub available_at: String,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<String>,
    pub fencing_token: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub cancel_requested_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub retention_until: Option<String>,
}

impl AgentTurnRecordDto {
    pub fn from_record(record: &crate::agent_turn::AgentTurnRecord) -> Self {
        Self {
            turn_id: record.turn_id.clone(),
            tenant_id: record.tenant_id.to_string(),
            organization_id: record.organization_id.to_string(),
            session_id: record.session_id.clone(),
            agent_id: record.agent_id.clone(),
            owner_user_id: record.owner_user_id.to_string(),
            runtime_binding_id: record.runtime_binding_id.clone(),
            client_request_id: record.client_request_id.clone(),
            idempotency_key: record.idempotency_key.clone(),
            payload_hash: record.payload_hash.clone(),
            request_item_id: record.request_item_id.clone(),
            response_item_id: record.response_item_id.clone(),
            turn_mode: record.turn_mode.as_str().to_string(),
            status: record.status.as_str().to_string(),
            requested_model_id: record.requested_model_id.clone(),
            provider_binding_id: record.provider_binding_id.clone(),
            model_id: record.model_id.clone(),
            provider_id: record.provider_id.clone(),
            input_tokens: record.input_tokens.to_string(),
            output_tokens: record.output_tokens.to_string(),
            cached_tokens: record.cached_tokens.to_string(),
            finish_reason: record.finish_reason.clone(),
            error_code: record.error_code.clone(),
            error_detail: record.error_detail.clone(),
            trace_id: record.trace_id.clone(),
            attempt_count: record.attempt_count,
            max_attempts: record.max_attempts,
            next_retry_at: record.next_retry_at.clone(),
            available_at: record.available_at.clone(),
            lease_owner: record.lease_owner.clone(),
            lease_expires_at: record.lease_expires_at.clone(),
            fencing_token: record.fencing_token.to_string(),
            version: record.version.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            started_at: record.started_at.clone(),
            completed_at: record.completed_at.clone(),
            cancel_requested_at: record.cancel_requested_at.clone(),
            cancelled_at: record.cancelled_at.clone(),
            retention_until: record.retention_until.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTurnExecutionDto {
    pub session: AgentSessionRecordDto,
    pub turn: AgentTurnRecordDto,
    pub items: Vec<AgentSessionItemRecordDto>,
}

impl AgentTurnExecutionDto {
    pub fn from_result(result: &crate::application::TurnExecutionResult) -> KernelResult<Self> {
        Ok(Self {
            session: AgentSessionRecordDto::from_record(&result.session),
            turn: AgentTurnRecordDto::from_record(&result.turn),
            items: vec![
                AgentSessionItemRecordDto::from_record_with_drive_refs(
                    &result.user_input_item,
                    &result.user_item_drive_refs,
                )?,
                AgentSessionItemRecordDto::from_record(&result.assistant_output_item)?,
            ],
        })
    }
}

// ===========================================================================
// Checkpoint DTOs
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionCheckpointRecordDto {
    pub checkpoint_id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub session_id: String,
    pub turn_id: Option<String>,
    pub runtime_binding_id: Option<String>,
    pub checkpoint_kind: String,
    pub provider_checkpoint_ref: Option<String>,
    pub drive_space_id: Option<String>,
    pub drive_node_id: Option<String>,
    pub resumable: bool,
    pub status: String,
    pub created_by: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub restored_at: Option<String>,
    pub invalidated_at: Option<String>,
    pub retention_until: Option<String>,
}

impl AgentSessionCheckpointRecordDto {
    pub fn from_record(record: &crate::domain::AgentSessionCheckpointRecord) -> Self {
        Self {
            checkpoint_id: record.checkpoint_id.clone(),
            tenant_id: record.tenant_id.to_string(),
            organization_id: record.organization_id.to_string(),
            session_id: record.session_id.clone(),
            turn_id: record.turn_id.clone(),
            runtime_binding_id: record.runtime_binding_id.clone(),
            checkpoint_kind: record.checkpoint_kind.clone(),
            provider_checkpoint_ref: record.provider_checkpoint_ref.clone(),
            drive_space_id: record.drive_space_id.clone(),
            drive_node_id: record.drive_node_id.clone(),
            resumable: record.resumable,
            status: record.status.as_str().to_string(),
            created_by: record.created_by.to_string(),
            version: record.version.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            restored_at: record.restored_at.clone(),
            invalidated_at: record.invalidated_at.clone(),
            retention_until: record.retention_until.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateSessionCheckpointRequestDto {
    pub checkpoint_id: Option<String>,
    pub turn_id: Option<String>,
    pub runtime_binding_id: Option<String>,
    pub checkpoint_kind: String,
    pub provider_checkpoint_ref: Option<String>,
    pub drive_space_id: Option<String>,
    pub drive_node_id: Option<String>,
    pub resumable: bool,
    pub retention_until: Option<String>,
    pub requested_at: String,
}

impl CreateSessionCheckpointRequestDto {
    pub fn into_command(
        self,
        tenant_id: u64,
        organization_id: u64,
        path_agent_id: String,
        session_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<crate::application::CreateSessionCheckpointCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(crate::application::CreateSessionCheckpointCommand {
            tenant_id,
            organization_id,
            path_agent_id,
            session_id,
            checkpoint_id: self.checkpoint_id,
            turn_id: self.turn_id,
            runtime_binding_id: self.runtime_binding_id,
            checkpoint_kind: self.checkpoint_kind,
            provider_checkpoint_ref: self.provider_checkpoint_ref,
            drive_space_id: self.drive_space_id,
            drive_node_id: self.drive_node_id,
            resumable: self.resumable,
            retention_until: self.retention_until,
            owner_scope: None,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChangeSessionCheckpointStatusRequestDto {
    pub reason: Option<String>,
    pub expected_version: String,
    pub requested_at: String,
}

impl ChangeSessionCheckpointStatusRequestDto {
    pub fn into_command(
        self,
        tenant_id: u64,
        organization_id: u64,
        path_agent_id: String,
        session_id: String,
        checkpoint_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<crate::application::ChangeSessionCheckpointStatusCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(crate::application::ChangeSessionCheckpointStatusCommand {
            tenant_id,
            organization_id,
            path_agent_id,
            session_id,
            checkpoint_id,
            expected_version: parse_expected_version(&self.expected_version)?,
            reason: self.reason,
            owner_scope: None,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

// ===========================================================================
// Session runtime binding DTOs
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionRuntimeBindingRecordDto {
    pub runtime_binding_id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub session_id: String,
    pub runtime_location_id: Option<String>,
    pub host_mode: String,
    pub transport_kind: String,
    pub provider_binding_id: String,
    pub model_id: String,
    pub provider_id: String,
    pub native_session_id: Option<String>,
    pub native_session_tree_id: Option<String>,
    pub native_parent_session_id: Option<String>,
    pub native_forked_from_session_id: Option<String>,
    pub status: String,
    pub is_current: bool,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub activated_at: Option<String>,
    pub deactivated_at: Option<String>,
}

impl AgentSessionRuntimeBindingRecordDto {
    pub fn from_record(record: &crate::domain::AgentSessionRuntimeBindingRecord) -> Self {
        Self {
            runtime_binding_id: record.runtime_binding_id.clone(),
            tenant_id: record.tenant_id.to_string(),
            organization_id: record.organization_id.to_string(),
            session_id: record.session_id.clone(),
            runtime_location_id: record.runtime_location_id.clone(),
            host_mode: record.host_mode.clone(),
            transport_kind: record.transport_kind.clone(),
            provider_binding_id: record.provider_binding_id.clone(),
            model_id: record.model_id.clone(),
            provider_id: record.provider_id.clone(),
            native_session_id: record.native_session_id.clone(),
            native_session_tree_id: record.native_session_tree_id.clone(),
            native_parent_session_id: record.native_parent_session_id.clone(),
            native_forked_from_session_id: record.native_forked_from_session_id.clone(),
            status: record.status.as_str().to_string(),
            is_current: record.is_current,
            version: record.version.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
            activated_at: record.activated_at.clone(),
            deactivated_at: record.deactivated_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateSessionRuntimeBindingRequestDto {
    pub runtime_binding_id: Option<String>,
    pub runtime_location_id: Option<String>,
    pub host_mode: String,
    pub transport_kind: String,
    pub provider_binding_id: String,
    pub model_id: String,
    pub provider_id: String,
    pub native_session_id: Option<String>,
    pub native_session_tree_id: Option<String>,
    pub native_parent_session_id: Option<String>,
    pub native_forked_from_session_id: Option<String>,
    pub requested_at: String,
}

impl CreateSessionRuntimeBindingRequestDto {
    pub fn into_command(
        self,
        tenant_id: u64,
        organization_id: u64,
        path_agent_id: String,
        session_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<crate::application::CreateSessionRuntimeBindingCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(crate::application::CreateSessionRuntimeBindingCommand {
            tenant_id,
            organization_id,
            path_agent_id,
            session_id,
            runtime_binding_id: self.runtime_binding_id,
            runtime_location_id: self.runtime_location_id,
            host_mode: self.host_mode,
            transport_kind: self.transport_kind,
            provider_binding_id: self.provider_binding_id,
            model_id: self.model_id,
            provider_id: self.provider_id,
            native_session_id: self.native_session_id,
            native_session_tree_id: self.native_session_tree_id,
            native_parent_session_id: self.native_parent_session_id,
            native_forked_from_session_id: self.native_forked_from_session_id,
            owner_scope: None,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateSessionRuntimeBindingRequestDto {
    pub runtime_location_id: Option<String>,
    #[serde(default)]
    pub clear_runtime_location: bool,
    pub host_mode: Option<String>,
    pub transport_kind: Option<String>,
    pub provider_binding_id: Option<String>,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub native_session_id: Option<String>,
    pub native_session_tree_id: Option<String>,
    pub native_parent_session_id: Option<String>,
    pub native_forked_from_session_id: Option<String>,
    pub expected_version: String,
    pub requested_at: String,
}

impl UpdateSessionRuntimeBindingRequestDto {
    pub fn into_command(
        self,
        tenant_id: u64,
        organization_id: u64,
        path_agent_id: String,
        session_id: String,
        runtime_binding_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<crate::application::UpdateSessionRuntimeBindingCommand> {
        validate_requested_at(&self.requested_at)?;
        if self.clear_runtime_location && self.runtime_location_id.is_some() {
            return Err(KernelError::validation(
                "runtimeLocationId and clearRuntimeLocation cannot be supplied together",
            ));
        }
        let runtime_location_id = if self.clear_runtime_location {
            Some(None)
        } else {
            self.runtime_location_id.map(Some)
        };
        Ok(crate::application::UpdateSessionRuntimeBindingCommand {
            tenant_id,
            organization_id,
            path_agent_id,
            session_id,
            runtime_binding_id,
            runtime_location_id,
            host_mode: self.host_mode,
            transport_kind: self.transport_kind,
            provider_binding_id: self.provider_binding_id,
            model_id: self.model_id,
            provider_id: self.provider_id,
            native_session_id: self.native_session_id,
            native_session_tree_id: self.native_session_tree_id,
            native_parent_session_id: self.native_parent_session_id,
            native_forked_from_session_id: self.native_forked_from_session_id,
            expected_version: parse_expected_version(&self.expected_version)?,
            owner_scope: None,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChangeSessionRuntimeBindingStatusRequestDto {
    pub reason: Option<String>,
    pub expected_version: String,
    pub requested_at: String,
}

impl ChangeSessionRuntimeBindingStatusRequestDto {
    pub fn into_command(
        self,
        tenant_id: u64,
        organization_id: u64,
        path_agent_id: String,
        session_id: String,
        runtime_binding_id: String,
        requested_by: PolicySubject,
    ) -> KernelResult<crate::application::ChangeSessionRuntimeBindingStatusCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(crate::application::ChangeSessionRuntimeBindingStatusCommand {
            tenant_id,
            organization_id,
            path_agent_id,
            session_id,
            runtime_binding_id,
            expected_version: parse_expected_version(&self.expected_version)?,
            reason: self.reason,
            owner_scope: None,
            requested_by,
            requested_at: self.requested_at,
        })
    }
}

fn parse_visibility(value: &str) -> KernelResult<AgentVisibility> {
    AgentVisibility::from_code(value).ok_or_else(|| {
        KernelError::validation(format!(
            "visibility must be one of private, organization, tenant, public: {value}"
        ))
    })
}

fn parse_status(value: &str) -> KernelResult<AgentBusinessStatus> {
    AgentBusinessStatus::from_code(value).ok_or_else(|| {
        KernelError::validation(format!(
            "target_status must be one of draft, active, disabled, archived, deleted: {value}"
        ))
    })
}

fn parse_session_status(value: &str) -> KernelResult<String> {
    AgentSessionStatus::from_code(value)
        .map(|status| status.as_str().to_string())
        .ok_or_else(|| {
            KernelError::validation(format!(
                "status must be one of active, idle, closed, archived: {value}"
            ))
        })
}

fn parse_task_status(value: &str) -> KernelResult<String> {
    AgentTaskStatus::from_code(value)
        .map(|status| status.as_str().to_string())
        .ok_or_else(|| {
            KernelError::validation(format!(
                "status must be one of pending, running, completed, failed, cancelled: {value}"
            ))
        })
}

fn parse_interaction_status(value: &str) -> KernelResult<String> {
    AgentInteractionStatus::from_code(value)
        .map(|status| status.as_str().to_string())
        .ok_or_else(|| {
            KernelError::validation(format!(
                "status must be one of pending, resolved, rejected, expired, cancelled: {value}"
            ))
        })
}

fn parse_session_item_status(value: &str) -> KernelResult<String> {
    AgentSessionItemStatus::from_code(value)
        .map(|status| status.as_str().to_string())
        .ok_or_else(|| {
            KernelError::validation(format!(
                "status must be one of pending, completed, failed, cancelled, redacted: {value}"
            ))
        })
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

fn parse_session_item_kind(value: &str) -> KernelResult<AgentSessionItemKind> {
    AgentSessionItemKind::from_code(value).ok_or_else(|| {
        KernelError::validation(format!(
            "kind must be one of user_input, system_instruction, assistant_output, reasoning, tool_call, tool_result, artifact_reference, status_notice, error_notice: {value}"
        ))
    })
}

fn parse_session_kind(value: &str) -> KernelResult<AgentSessionKind> {
    AgentSessionKind::from_code(value).ok_or_else(|| {
        KernelError::validation(format!(
            "sessionKind must be one of assistant, coding, automation, im_dispatch: {value}"
        ))
    })
}

fn parse_session_entry_surface(value: &str) -> KernelResult<AgentSessionEntrySurface> {
    AgentSessionEntrySurface::from_code(value).ok_or_else(|| {
        KernelError::validation(format!(
            "entrySurface must be one of pc, h5, flutter, mini_program, api, im_dispatch, automation: {value}"
        ))
    })
}

fn parse_u64(value: &str, field_name: &str) -> KernelResult<u64> {
    value.parse::<u64>().map_err(|_| {
        KernelError::validation(format!(
            "{field_name} must be a valid non-negative integer: {value}"
        ))
    })
}

fn optional_object_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .and_then(normalize_optional_string)
}

fn optional_object_bool(value: Option<&Value>) -> Option<bool> {
    value.and_then(Value::as_bool)
}

fn optional_object_f64(value: Option<&Value>) -> Option<f64> {
    value
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

fn object_string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .filter_map(normalize_optional_string)
                .collect()
        })
        .unwrap_or_default()
}

fn insert_optional_string(object: &mut Map<String, Value>, key: &str, value: Option<&String>) {
    if let Some(value) = value.and_then(|value| normalize_optional_string(value.as_str())) {
        object.insert(key.to_string(), Value::String(value));
    }
}

fn insert_optional_bool(object: &mut Map<String, Value>, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        object.insert(key.to_string(), Value::Bool(value));
    }
}

fn insert_optional_f64(object: &mut Map<String, Value>, key: &str, value: Option<f64>) {
    if let Some(value) = value.filter(|value| value.is_finite()) {
        object.insert(key.to_string(), json!(value));
    }
}

fn insert_string_array(object: &mut Map<String, Value>, key: &str, value: &[String]) {
    let values = value
        .iter()
        .filter_map(|item| normalize_optional_string(item.as_str()))
        .map(Value::String)
        .collect::<Vec<_>>();
    if !values.is_empty() {
        object.insert(key.to_string(), Value::Array(values));
    }
}

fn normalize_optional_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_agent_kernel::PolicySubject;

    fn sample_manifest(agent_id: &str) -> AgentManifest {
        AgentManifest {
            schema_version: "1.0.0".to_string(),
            manifest_type: "agent".to_string(),
            agent_id: agent_id.to_string(),
            name: "sample-agent".to_string(),
            display_name: "Sample Agent".to_string(),
            description: "sample".to_string(),
            version: "0.1.0".to_string(),
            domain: "intelligence".to_string(),
            required_capabilities: vec!["model.chat".to_string()],
            optional_capabilities: vec!["tool.invoke".to_string()],
            required_capability_requirements: vec![],
            optional_capability_requirements: vec![],
            event_families: vec!["agent.lifecycle".to_string()],
            owner_name: "sdkwork".to_string(),
            status: "active".to_string(),
        }
    }

    fn sample_subject() -> PolicySubject {
        PolicySubject::new("u-1", "100001")
    }

    #[test]
    fn create_request_maps_to_command() {
        let command = CreateAgentRequestDto {
            agent_id: "agent.alpha".to_string(),
            tenant_id: "100001".to_string(),
            organization_id: "0".to_string(),
            owner_user_id: "100".to_string(),
            code: "alpha".to_string(),
            display_name: "Alpha".to_string(),
            description: Some("alpha".to_string()),
            manifest: sample_manifest("agent.alpha"),
            visibility: "organization".to_string(),
            tags: vec!["starter".to_string()],
            default_code_task_intent: Some(CodeTaskIntent::new("Refactor runtime")),
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: None,
            requested_at: "2026-06-01T00:00:00Z".to_string(),
        }
        .into_command(sample_subject())
        .expect("mapping should succeed");

        assert_eq!(command.tenant_id, 100_001);
        assert_eq!(command.organization_id, 0);
        assert_eq!(command.owner_user_id, 100);
        assert_eq!(command.visibility, AgentVisibility::Organization);
    }

    #[test]
    fn list_request_maps_search_query() {
        let command = ListAgentsRequestDto {
            tenant_id: "100001".to_string(),
            organization_id: None,
            owner_user_id: None,
            include_deleted: false,
            search_query: Some("beta".to_string()),
            visibility: None,
            pagination: PaginationParams::default(),
        }
        .into_command(sample_subject())
        .expect("mapping should succeed");

        assert_eq!(command.query.search_query.as_deref(), Some("beta"));
    }

    #[test]
    fn invalid_status_is_rejected() {
        let result = UpdateAgentStatusRequestDto {
            tenant_id: "100001".to_string(),
            agent_id: "agent.alpha".to_string(),
            expected_version: None,
            target_status: "ready".to_string(),
            requested_at: "2026-06-01T00:00:00Z".to_string(),
        }
        .into_command(sample_subject());

        let error = result.expect_err("invalid status should fail");
        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("target_status"));
            }
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn invalid_requested_at_is_rejected_for_mutation_commands() {
        let create_error = CreateAgentRequestDto {
            agent_id: "agent.alpha".to_string(),
            tenant_id: "100001".to_string(),
            organization_id: "0".to_string(),
            owner_user_id: "100".to_string(),
            code: "alpha".to_string(),
            display_name: "Alpha".to_string(),
            description: Some("alpha".to_string()),
            manifest: sample_manifest("agent.alpha"),
            visibility: "organization".to_string(),
            tags: vec!["starter".to_string()],
            default_code_task_intent: Some(CodeTaskIntent::new("Refactor runtime")),
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: None,
            requested_at: "2026-06-01".to_string(),
        }
        .into_command(sample_subject())
        .expect_err("invalid requestedAt should fail");
        match create_error {
            KernelError::Validation { message } => {
                assert!(message.contains("requestedAt"));
            }
            _ => panic!("expected validation error"),
        }

        let restore_error = RestoreAgentRequestDto {
            tenant_id: "100001".to_string(),
            agent_id: "agent.alpha".to_string(),
            expected_version: None,
            requested_at: "not-a-date".to_string(),
        }
        .into_command(sample_subject())
        .expect_err("invalid requestedAt should fail");
        match restore_error {
            KernelError::Validation { message } => {
                assert!(message.contains("requestedAt"));
            }
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn invalid_expected_version_is_rejected_for_mutation_commands() {
        let update_error = UpdateAgentRequestDto {
            tenant_id: "100001".to_string(),
            agent_id: "agent.alpha".to_string(),
            expected_version: Some("1x".to_string()),
            display_name: None,
            description: None,
            manifest: None,
            visibility: None,
            tags: None,
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: None,
            requested_at: "2026-06-01T00:00:00Z".to_string(),
        }
        .into_command(sample_subject())
        .expect_err("invalid expectedVersion should fail");

        match update_error {
            KernelError::Validation { message } => {
                assert!(message.contains("expectedVersion"));
            }
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn invalid_implementation_type_is_rejected_for_agent_requests() {
        let error = CreateAgentRequestDto {
            agent_id: "agent.alpha".to_string(),
            tenant_id: "100001".to_string(),
            organization_id: "0".to_string(),
            owner_user_id: "100".to_string(),
            code: "alpha".to_string(),
            display_name: "Alpha".to_string(),
            description: Some("alpha".to_string()),
            manifest: sample_manifest("agent.alpha"),
            visibility: "organization".to_string(),
            tags: vec!["starter".to_string()],
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: Some("not-a-framework".to_string()),
            requested_at: "2026-06-01T00:00:00Z".to_string(),
        }
        .into_command(sample_subject())
        .expect_err("invalid implementationType should fail");

        match error {
            KernelError::Validation { message } => {
                assert!(message.contains("implementationType"));
            }
            _ => panic!("expected validation error"),
        }
    }

    #[test]
    fn record_maps_to_dto_with_int64_strings() {
        let record = AgentBusinessRecord {
            id: 7,
            agent_id: "agent.alpha".to_string(),
            tenant_id: 100_001,
            organization_id: 0,
            owner_user_id: 100,
            code: "alpha".to_string(),
            display_name: "Alpha".to_string(),
            description: None,
            manifest: sample_manifest("agent.alpha"),
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: AgentImplementationType::SdkworkNative,
            status: AgentBusinessStatus::Draft,
            visibility: AgentVisibility::Private,
            tags: vec!["starter".to_string()],
            version: 2,
            created_at: "2026-06-01T00:00:00Z".to_string(),
            updated_at: "2026-06-01T00:00:00Z".to_string(),
            deleted_at: None,
        };
        let dto = AgentRecordDto::from_record(&record);

        assert_eq!(dto.id, "7");
        assert_eq!(dto.tenant_id, "100001");
        assert_eq!(dto.organization_id, "0");
        assert_eq!(dto.owner_user_id, "100");
        assert_eq!(dto.version, "2");
        assert_eq!(dto.status, "draft");
        assert_eq!(dto.visibility, "private");
    }

    #[test]
    fn record_dto_exposes_pc_management_profile_from_existing_intent_constraints() {
        let management_profile_json = r##"{"author":"SDKWork","avatar":"robot","categoryId":"assistant","color":"#3b82f6","iconName":"bot","systemPrompt":"Answer from approved knowledge only.","type":"independent","users":"12 users","welcomeMessage":"How can I help?"}"##;
        let record = AgentBusinessRecord {
            id: 7,
            agent_id: "agent.alpha".to_string(),
            tenant_id: 100_001,
            organization_id: 0,
            owner_user_id: 100,
            code: "alpha".to_string(),
            display_name: "Alpha".to_string(),
            description: None,
            manifest: sample_manifest("agent.alpha"),
            default_code_task_intent: Some(
                CodeTaskIntent::new("Answer from approved knowledge only.")
                    .with_context_path("knowledge.base.product")
                    .with_constraint("agent.type=independent")
                    .with_constraint(format!("sdkwork.agent.pc.config:{management_profile_json}")),
            ),
            implementation_provider_id: None,
            implementation_kind: Some(AgentImplementationKind::ManifestOnly),
            implementation_type: AgentImplementationType::SdkworkNative,
            status: AgentBusinessStatus::Draft,
            visibility: AgentVisibility::Private,
            tags: vec!["assistant".to_string()],
            version: 2,
            created_at: "2026-06-01T00:00:00Z".to_string(),
            updated_at: "2026-06-01T00:00:00Z".to_string(),
            deleted_at: None,
        };

        let dto = AgentRecordDto::from_record(&record);
        let management_profile = dto
            .management_profile
            .expect("PC management profile should be parsed from compatible constraints");

        assert_eq!(management_profile.avatar.as_deref(), Some("robot"));
        assert_eq!(management_profile.author.as_deref(), Some("SDKWork"));
        assert_eq!(management_profile.category_id.as_deref(), Some("assistant"));
        assert_eq!(management_profile.color.as_deref(), Some("#3b82f6"));
        assert_eq!(management_profile.icon_name.as_deref(), Some("bot"));
        assert_eq!(
            management_profile.system_prompt.as_deref(),
            Some("Answer from approved knowledge only.")
        );
        assert_eq!(
            management_profile.agent_type.as_deref(),
            Some("independent")
        );
        assert_eq!(management_profile.users.as_deref(), Some("12 users"));
        assert_eq!(
            management_profile.welcome_message.as_deref(),
            Some("How can I help?")
        );
    }

    #[test]
    fn provider_binding_request_maps_to_command_with_implementation_kind() {
        let command = AgentProviderBindingRequestDto {
            tenant_id: "100001".to_string(),
            agent_id: "agent.alpha".to_string(),
            binding_id: "binding.rig.default".to_string(),
            provider_id: "provider.model.rig-rust".to_string(),
            implementation_kind: "typed-local-provider".to_string(),
            configuration_profile_id: "profile.rig.local".to_string(),
            capabilities: vec!["model.chat".to_string()],
            make_default: true,
            requested_at: "2026-06-01T00:00:00Z".to_string(),
        }
        .into_command(sample_subject())
        .expect("binding command should map");

        assert_eq!(command.tenant_id, 100_001);
        assert_eq!(
            command.implementation_kind,
            crate::domain::AgentImplementationKind::TypedLocalProvider
        );
        assert!(command.make_default);
    }
}
