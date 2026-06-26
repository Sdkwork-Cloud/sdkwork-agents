use crate::application::{
    ActivateAgentProviderBindingCommand, AgentKnowledgeBaseCreateCommand,
    AgentKnowledgeBaseUpdateCommand, AgentKnowledgeBindingCreateCommand,
    AgentKnowledgeChunkCreateCommand, AgentKnowledgeDocumentCreateCommand,
    AgentKnowledgeDocumentUpdateCommand, AgentKnowledgeIndexUpsertCommand,
    AgentKnowledgeSearchCommand, AgentKnowledgeSourceCreateCommand,
    AgentKnowledgeSourceUpdateCommand, AgentKnowledgeSyncJobCreateCommand,
    AgentMemoryBindingCreateCommand, AgentMemoryNamespaceCreateCommand,
    AgentMemoryProfileCreateCommand, AgentMemoryRecordCreateCommand,
    AgentMemoryRelationCreateCommand, AgentMemoryRetrievalIndexUpsertCommand,
    AgentMemorySourceCreateCommand, AgentMemoryStoreCreateCommand, AgentMemoryStoreUpdateCommand,
    AgentPreviewResponseCommand, AgentPromptOptimizationCommand, AgentProviderBindingCommand,
    AgentProviderDeploymentCommand, ChangeAgentStatusCommand, CreateAgentCommand,
    DeleteAgentCommand, GetAgentCommand, ListAgentsCommand, RestoreAgentCommand,
    UpdateAgentCommand,
};
use crate::domain::{
    AgentBusinessRecord, AgentBusinessStatus, AgentDeploymentRecord, AgentImplementationKind,
    AgentImplementationType, AgentKnowledgeBaseKind, AgentKnowledgeBaseRecord,
    AgentKnowledgeBindingRecord, AgentKnowledgeBindingScopeKind, AgentKnowledgeChunkRecord,
    AgentKnowledgeDocumentKind, AgentKnowledgeDocumentRecord, AgentKnowledgeIndexKind,
    AgentKnowledgeIndexRecord, AgentKnowledgeSearchResult, AgentKnowledgeSourceKind,
    AgentKnowledgeSourceRecord, AgentKnowledgeSyncJobKind, AgentKnowledgeSyncJobRecord,
    AgentMemoryBindingRecord, AgentMemoryBindingScopeKind, AgentMemoryIndexKind,
    AgentMemoryNamespaceKind, AgentMemoryNamespaceRecord, AgentMemoryProfileRecord,
    AgentMemoryRecord, AgentMemoryRecordKind, AgentMemoryRelationKind, AgentMemoryRelationRecord,
    AgentMemoryRetrievalIndexRecord, AgentMemorySourceKind, AgentMemorySourceRecord,
    AgentMemoryStoreKind, AgentMemoryStoreRecord, AgentProviderBindingRecord,
    AgentRuntimeExecutionRecord, AgentVisibility,
};
use crate::ports::{AgentListQuery, AgentMarketplaceListQuery};
use crate::validation::{
    parse_expected_version, parse_organization_id, parse_owner_user_id, parse_tenant_id,
    validate_requested_at,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderDeploymentRequestDto {
    pub tenant_id: String,
    pub agent_id: String,
    pub deployment_id: String,
    pub binding_id: String,
    pub requested_at: String,
}

impl AgentProviderDeploymentRequestDto {
    pub fn into_command(
        self,
        requested_by: PolicySubject,
    ) -> KernelResult<AgentProviderDeploymentCommand> {
        validate_requested_at(&self.requested_at)?;
        Ok(AgentProviderDeploymentCommand {
            tenant_id: parse_tenant_id(&self.tenant_id)?,
            agent_id: self.agent_id,
            deployment_id: self.deployment_id,
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
    pub memory_enabled: bool,
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
            memory_enabled: self.memory_enabled,
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
            if intent.prompt.trim().is_empty() || intent.prompt == "Agent management profile" {
                intent.prompt = system_prompt.clone();
            }
        }

        for knowledge_base_id in self.knowledge_base_ids.iter() {
            if !intent
                .context_paths
                .iter()
                .any(|context_path| context_path == knowledge_base_id)
            {
                intent.context_paths.push(knowledge_base_id.clone());
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDeploymentRecordDto {
    pub tenant_id: String,
    pub agent_id: String,
    pub deployment_id: String,
    pub binding_id: String,
    pub provider_id_snapshot: String,
    pub implementation_kind_snapshot: String,
    pub configuration_profile_id_snapshot: String,
    pub capabilities_snapshot: Vec<String>,
    pub status: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

impl AgentDeploymentRecordDto {
    pub fn from_record(record: &AgentDeploymentRecord) -> Self {
        Self {
            tenant_id: record.tenant_id.to_string(),
            agent_id: record.agent_id.clone(),
            deployment_id: record.deployment_id.clone(),
            binding_id: record.binding_id.clone(),
            provider_id_snapshot: record.provider_id_snapshot.clone(),
            implementation_kind_snapshot: record.implementation_kind_snapshot.as_str().to_string(),
            configuration_profile_id_snapshot: record.configuration_profile_id_snapshot.clone(),
            capabilities_snapshot: record.capabilities_snapshot.clone(),
            status: record.status.as_str().to_string(),
            version: record.version.to_string(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDeploymentResponseDto {
    pub data: AgentDeploymentRecordDto,
}

impl AgentDeploymentResponseDto {
    pub fn from_record(record: &AgentDeploymentRecord) -> Self {
        Self {
            data: AgentDeploymentRecordDto::from_record(record),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDeploymentListDataDto {
    pub items: Vec<AgentDeploymentRecordDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDeploymentListResponseDto {
    pub data: AgentDeploymentListDataDto,
}

impl AgentDeploymentListResponseDto {
    pub fn from_records(records: &[AgentDeploymentRecord]) -> Self {
        Self {
            data: AgentDeploymentListDataDto {
                items: records
                    .iter()
                    .map(AgentDeploymentRecordDto::from_record)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListAgentKnowledgeBasesRequestDto {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: Option<String>,
    pub include_deleted: bool,
    pub search_query: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub category: Option<String>,
    pub tag: Option<String>,
}

impl ListAgentKnowledgeBasesRequestDto {
    pub fn into_query(self) -> KernelResult<AgentMarketplaceListQuery> {
        let mut query = AgentMarketplaceListQuery::for_tenant(parse_tenant_id(&self.tenant_id)?);
        if let Some(organization_id) = self.organization_id {
            query = query.for_organization(parse_organization_id(&organization_id)?);
        }
        if let Some(owner_user_id) = self.owner_user_id {
            query = query.for_owner(parse_owner_user_id(&owner_user_id)?);
        }
        if let Some(status) = self.status {
            query = query.with_status(parse_status(&status)?);
        }
        if let Some(visibility) = self.visibility {
            query = query.with_visibility(parse_visibility(&visibility)?);
        }
        if self.include_deleted {
            query = query.with_deleted();
        }
        if let Some(search_query) = self.search_query {
            query = query.with_search(search_query);
        }
        if let Some(category) = self.category {
            query = query.with_category(category);
        }
        if let Some(tag) = self.tag {
            query = query.with_tag(tag);
        }
        Ok(query)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub priority: i32,
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
            priority: record.priority,
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
    pub priority: Option<i32>,
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
    pub priority: Option<i32>,
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
    pub request_id: Option<String>,
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
    pub request_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AgentDeploymentRecord, AgentDeploymentStatus, AgentProviderBindingRecord};
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
        let management_profile_json = r##"{"author":"SDKWork","avatar":"robot","categoryId":"assistant","color":"#3b82f6","iconName":"bot","knowledgeBaseIds":["knowledge.base.product","knowledge.base.runbook"],"systemPrompt":"Answer from approved knowledge only.","type":"independent","users":"12 users","welcomeMessage":"How can I help?"}"##;
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
            management_profile.knowledge_base_ids,
            vec![
                "knowledge.base.product".to_string(),
                "knowledge.base.runbook".to_string()
            ]
        );
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

    #[test]
    fn knowledge_document_dto_exposes_pc_document_profile_from_existing_metadata() {
        let record = AgentKnowledgeDocumentRecord {
            id: 17,
            tenant_id: 100_001,
            organization_id: 0,
            knowledge_document_id: "knowledge.document.product.manual".to_string(),
            knowledge_base_id: "knowledge.base.product".to_string(),
            knowledge_source_id: None,
            document_kind: AgentKnowledgeDocumentKind::WikiPage,
            title: "Product Manual".to_string(),
            content_ref: "knowledge://pc/documents/knowledge.document.product.manual".to_string(),
            content_hash: "sha256-pc-12345678".to_string(),
            summary: Some("Manual summary".to_string()),
            metadata_json: r#"{"pcAuthor":"SDKWork Docs","pcContent":"Full manual content","pcParentId":"knowledge.document.product.root","pcType":"file","fileName":"manual.pdf","fileSize":"42 KB","mimeType":"application/pdf","driveUri":"drive://knowledge/manual.pdf"}"#.to_string(),
            tags: vec!["product".to_string()],
            categories: vec![],
            trust_level: 4,
            redaction_classification: "internal".to_string(),
            chunk_count: 0,
            status: AgentBusinessStatus::Draft,
            visibility: AgentVisibility::Private,
            version: 3,
            created_at: "2026-06-01T00:00:00Z".to_string(),
            updated_at: "2026-06-01T00:00:00Z".to_string(),
            deleted_at: None,
        };

        let dto = AgentKnowledgeDocumentRecordDto::from_record(&record);
        let document_profile = dto
            .document_profile
            .expect("document profile should parse from compatible metadata");

        assert_eq!(document_profile.author.as_deref(), Some("SDKWork Docs"));
        assert_eq!(
            document_profile.content.as_deref(),
            Some("Full manual content")
        );
        assert_eq!(
            document_profile.parent_id.as_deref(),
            Some("knowledge.document.product.root")
        );
        assert_eq!(document_profile.document_type.as_deref(), Some("file"));
        assert_eq!(document_profile.file_name.as_deref(), Some("manual.pdf"));
        assert_eq!(document_profile.file_size.as_deref(), Some("42 KB"));
        assert_eq!(
            document_profile.mime_type.as_deref(),
            Some("application/pdf")
        );
        assert_eq!(
            document_profile.drive_uri.as_deref(),
            Some("drive://knowledge/manual.pdf")
        );
    }

    #[test]
    fn provider_binding_and_deployment_records_map_to_standard_dtos() {
        let binding = AgentProviderBindingRecord {
            id: 10,
            tenant_id: 100_001,
            agent_id: "agent.alpha".to_string(),
            binding_id: "binding.rig.default".to_string(),
            provider_id: "provider.model.rig-rust".to_string(),
            implementation_kind: crate::domain::AgentImplementationKind::TypedLocalProvider,
            configuration_profile_id: "profile.rig.local".to_string(),
            capabilities: vec!["model.chat".to_string(), "tool.invoke".to_string()],
            active: true,
            version: 1,
            created_at: "2026-06-01T00:00:00Z".to_string(),
            updated_at: "2026-06-01T00:00:00Z".to_string(),
        };
        let binding_dto = AgentProviderBindingRecordDto::from_record(&binding);

        assert_eq!(binding_dto.tenant_id, "100001");
        assert_eq!(binding_dto.binding_id, "binding.rig.default");
        assert_eq!(binding_dto.implementation_kind, "typed-local-provider");
        assert!(binding_dto.active);

        let deployment = AgentDeploymentRecord {
            id: 11,
            tenant_id: 100_001,
            agent_id: "agent.alpha".to_string(),
            deployment_id: "deployment.rig.1".to_string(),
            binding_id: "binding.rig.default".to_string(),
            provider_id_snapshot: "provider.model.rig-rust".to_string(),
            implementation_kind_snapshot:
                crate::domain::AgentImplementationKind::TypedLocalProvider,
            configuration_profile_id_snapshot: "profile.rig.local".to_string(),
            capabilities_snapshot: vec!["model.chat".to_string()],
            status: AgentDeploymentStatus::Created,
            version: 1,
            created_at: "2026-06-01T00:01:00Z".to_string(),
            updated_at: "2026-06-01T00:01:00Z".to_string(),
        };
        let deployment_dto = AgentDeploymentRecordDto::from_record(&deployment);

        assert_eq!(deployment_dto.deployment_id, "deployment.rig.1");
        assert_eq!(
            deployment_dto.provider_id_snapshot,
            "provider.model.rig-rust"
        );
        assert_eq!(
            deployment_dto.implementation_kind_snapshot,
            "typed-local-provider"
        );
        assert_eq!(deployment_dto.status, "created");
    }
}
