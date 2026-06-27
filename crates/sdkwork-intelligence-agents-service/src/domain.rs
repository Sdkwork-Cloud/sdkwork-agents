use sdkwork_agent_kernel::AgentManifest;
use sdkwork_code_kernel::CodeTaskIntent;

pub const DEFAULT_AGENT_MANAGEMENT_POLICY_CATEGORY: &str = "agent.business.manage";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentBusinessStatus {
    Draft,
    Active,
    Disabled,
    Archived,
    Deleted,
}

impl AgentBusinessStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Archived => "archived",
            Self::Deleted => "deleted",
        }
    }

    pub(crate) fn from_code(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(Self::Draft),
            "active" => Some(Self::Active),
            "disabled" => Some(Self::Disabled),
            "archived" => Some(Self::Archived),
            "deleted" => Some(Self::Deleted),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::Draft => 0,
            Self::Active => 1,
            Self::Disabled => 2,
            Self::Archived => 3,
            Self::Deleted => 4,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Draft),
            1 => Some(Self::Active),
            2 => Some(Self::Disabled),
            3 => Some(Self::Archived),
            4 => Some(Self::Deleted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentVisibility {
    Private,
    Organization,
    Tenant,
    Public,
}

impl AgentVisibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Organization => "organization",
            Self::Tenant => "tenant",
            Self::Public => "public",
        }
    }

    pub(crate) fn from_code(value: &str) -> Option<Self> {
        match value {
            "private" => Some(Self::Private),
            "organization" => Some(Self::Organization),
            "tenant" => Some(Self::Tenant),
            "public" => Some(Self::Public),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::Private => 0,
            Self::Organization => 1,
            Self::Tenant => 2,
            Self::Public => 3,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Private),
            1 => Some(Self::Organization),
            2 => Some(Self::Tenant),
            3 => Some(Self::Public),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentAuditAction {
    Create,
    Update,
    Delete,
    Restore,
    ChangeStatus,
    RuntimeExecutionCompleted,
    ProviderBindingChanged,
    SkillPackageCreated,
    SkillPackageUpdated,
    SkillPackageDeleted,
    SkillPackageRestored,
    CompositionSlotCreated,
    CompositionSlotUpdated,
    CompositionSlotDeleted,
}

impl AgentAuditAction {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::Create => "agent.business.created",
            Self::Update => "agent.business.updated",
            Self::Delete => "agent.business.deleted",
            Self::Restore => "agent.business.restored",
            Self::ChangeStatus => "agent.business.status_changed",
            Self::RuntimeExecutionCompleted => "agent.business.runtime.executed",
            Self::ProviderBindingChanged => "agent.business.provider_binding_changed",
            Self::SkillPackageCreated => "agent.business.skill.created",
            Self::SkillPackageUpdated => "agent.business.skill.updated",
            Self::SkillPackageDeleted => "agent.business.skill.deleted",
            Self::SkillPackageRestored => "agent.business.skill.restored",
            Self::CompositionSlotCreated => "agent.business.composition_slot.created",
            Self::CompositionSlotUpdated => "agent.business.composition_slot.updated",
            Self::CompositionSlotDeleted => "agent.business.composition_slot.deleted",
        }
    }

    pub fn action_code(&self) -> &'static str {
        match self {
            Self::Create => "created",
            Self::Update => "updated",
            Self::Delete => "deleted",
            Self::Restore => "restored",
            Self::ChangeStatus => "status_changed",
            Self::RuntimeExecutionCompleted => "runtime_executed",
            Self::ProviderBindingChanged => "provider_binding_changed",
            Self::SkillPackageCreated => "skill_created",
            Self::SkillPackageUpdated => "skill_updated",
            Self::SkillPackageDeleted => "skill_deleted",
            Self::SkillPackageRestored => "skill_restored",
            Self::CompositionSlotCreated => "composition_slot_created",
            Self::CompositionSlotUpdated => "composition_slot_updated",
            Self::CompositionSlotDeleted => "composition_slot_deleted",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRuntimeExecutionOperation {
    PreviewResponse,
    PromptOptimization,
}

impl AgentRuntimeExecutionOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PreviewResponse => "preview_response",
            Self::PromptOptimization => "prompt_optimization",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRuntimeExecutionStatus {
    Completed,
}

impl AgentRuntimeExecutionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Completed => "completed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRuntimeExecutionRecord {
    pub tenant_id: u64,
    pub agent_id: String,
    pub execution_id: String,
    pub operation: AgentRuntimeExecutionOperation,
    pub status: AgentRuntimeExecutionStatus,
    pub input_payload_json: String,
    pub output_payload_json: String,
    pub requested_at: String,
    pub completed_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentImplementationKind {
    ManifestOnly,
    TypedLocalProvider,
    ProcessAdapter,
    ProtocolAdapter,
}

impl AgentImplementationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ManifestOnly => "manifest-only",
            Self::TypedLocalProvider => "typed-local-provider",
            Self::ProcessAdapter => "process-adapter",
            Self::ProtocolAdapter => "protocol-adapter",
        }
    }

    pub(crate) fn from_code(value: &str) -> Option<Self> {
        match value {
            "manifest-only" => Some(Self::ManifestOnly),
            "typed-local-provider" => Some(Self::TypedLocalProvider),
            "process-adapter" => Some(Self::ProcessAdapter),
            "protocol-adapter" => Some(Self::ProtocolAdapter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentImplementationType {
    SdkworkNative,
    RigRust,
    OpenAiAgents,
    LangChain,
    LangGraph,
    CrewAi,
    AutoGen,
    SemanticKernel,
    Custom,
}

impl AgentImplementationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SdkworkNative => "sdkwork-native",
            Self::RigRust => "rig-rust",
            Self::OpenAiAgents => "openai-agents",
            Self::LangChain => "langchain",
            Self::LangGraph => "langgraph",
            Self::CrewAi => "crewai",
            Self::AutoGen => "autogen",
            Self::SemanticKernel => "semantic-kernel",
            Self::Custom => "custom",
        }
    }

    pub(crate) fn from_code(value: &str) -> Option<Self> {
        match value {
            "sdkwork-native" => Some(Self::SdkworkNative),
            "rig-rust" => Some(Self::RigRust),
            "openai-agents" => Some(Self::OpenAiAgents),
            "langchain" => Some(Self::LangChain),
            "langgraph" => Some(Self::LangGraph),
            "crewai" => Some(Self::CrewAi),
            "autogen" => Some(Self::AutoGen),
            "semantic-kernel" => Some(Self::SemanticKernel),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

impl Default for AgentImplementationType {
    fn default() -> Self {
        Self::SdkworkNative
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentBusinessRecord {
    pub id: u64,
    pub agent_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub manifest: AgentManifest,
    pub default_code_task_intent: Option<CodeTaskIntent>,
    pub implementation_provider_id: Option<String>,
    pub implementation_kind: Option<AgentImplementationKind>,
    pub implementation_type: AgentImplementationType,
    pub status: AgentBusinessStatus,
    pub visibility: AgentVisibility,
    pub tags: Vec<String>,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProviderBindingRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub agent_id: String,
    pub binding_id: String,
    pub provider_id: String,
    pub implementation_kind: AgentImplementationKind,
    pub configuration_profile_id: String,
    pub capabilities: Vec<String>,
    pub active: bool,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
}

impl AgentProviderBindingRecord {
    pub fn mark_updated(&mut self, updated_at: impl Into<String>) {
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCompositionSlotKind {
    Memory,
    Knowledge,
    Skill,
    Prompt,
    Drive,
    Tool,
    Mcp,
}

impl AgentCompositionSlotKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Knowledge => "knowledge",
            Self::Skill => "skill",
            Self::Prompt => "prompt",
            Self::Drive => "drive",
            Self::Tool => "tool",
            Self::Mcp => "mcp",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "memory" => Some(Self::Memory),
            "knowledge" => Some(Self::Knowledge),
            "skill" => Some(Self::Skill),
            "prompt" => Some(Self::Prompt),
            "drive" => Some(Self::Drive),
            "tool" => Some(Self::Tool),
            "mcp" => Some(Self::Mcp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCompositionTargetModule {
    Memory,
    Knowledgebase,
    Skills,
    Prompts,
    Drive,
    Mcp,
}

impl AgentCompositionTargetModule {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Knowledgebase => "knowledgebase",
            Self::Skills => "skills",
            Self::Prompts => "prompts",
            Self::Drive => "drive",
            Self::Mcp => "mcp",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "memory" => Some(Self::Memory),
            "knowledgebase" => Some(Self::Knowledgebase),
            "skills" => Some(Self::Skills),
            "prompts" => Some(Self::Prompts),
            "drive" => Some(Self::Drive),
            "mcp" => Some(Self::Mcp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCompositionSlotRecord {
    pub id: u64,
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
    pub status: AgentBusinessStatus,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

impl AgentCompositionSlotRecord {
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn mark_updated(&mut self, updated_at: impl Into<String>) {
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }

    pub fn mark_deleted(&mut self, deleted_at: impl Into<String>) {
        self.deleted_at = Some(deleted_at.into());
        self.status = AgentBusinessStatus::Deleted;
        self.mark_updated(self.deleted_at.clone().unwrap_or_default());
    }
}

impl AgentBusinessRecord {
    pub fn is_deleted(&self) -> bool {
        self.status == AgentBusinessStatus::Deleted || self.deleted_at.is_some()
    }

    pub fn mark_updated(&mut self, updated_at: impl Into<String>) {
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }

    pub fn mark_deleted(&mut self, deleted_at: impl Into<String>) {
        self.status = AgentBusinessStatus::Deleted;
        self.deleted_at = Some(deleted_at.into());
        self.version = self.version.saturating_add(1);
    }

    pub fn mark_restored(&mut self, restored_at: impl Into<String>) {
        self.status = AgentBusinessStatus::Active;
        self.deleted_at = None;
        self.updated_at = restored_at.into();
        self.version = self.version.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_slot_kind_roundtrips_all_variants_including_mcp() {
        // 确保所有 slot_kind 变体（包括新增的 Mcp）能正确 roundtrip
        for kind in [
            AgentCompositionSlotKind::Memory,
            AgentCompositionSlotKind::Knowledge,
            AgentCompositionSlotKind::Skill,
            AgentCompositionSlotKind::Prompt,
            AgentCompositionSlotKind::Drive,
            AgentCompositionSlotKind::Tool,
            AgentCompositionSlotKind::Mcp,
        ] {
            let s = kind.as_str();
            assert_eq!(AgentCompositionSlotKind::from_str(s), Some(kind));
        }
    }

    #[test]
    fn composition_target_module_roundtrips_all_variants_including_mcp() {
        // 确保所有 target_module 变体（包括新增的 Mcp）能正确 roundtrip
        for module in [
            AgentCompositionTargetModule::Memory,
            AgentCompositionTargetModule::Knowledgebase,
            AgentCompositionTargetModule::Skills,
            AgentCompositionTargetModule::Prompts,
            AgentCompositionTargetModule::Drive,
            AgentCompositionTargetModule::Mcp,
        ] {
            let s = module.as_str();
            assert_eq!(AgentCompositionTargetModule::from_str(s), Some(module));
        }
    }

    #[test]
    fn composition_slot_kind_rejects_unknown_value() {
        assert!(AgentCompositionSlotKind::from_str("unknown").is_none());
    }
}

