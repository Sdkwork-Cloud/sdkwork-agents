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
    SessionCreated,
    SessionRenamed,
    SessionMoved,
    SessionClosed,
    SessionArchived,
    SessionDeleted,
    MessageCreated,
    MessageFailed,
    MessageFeedbackChanged,
    InteractionCreated,
    InteractionResolved,
    InteractionRejected,
    InteractionExpired,
    InteractionCancelled,
    TaskCreated,
    TaskCompleted,
    TaskFailed,
    TaskCancelled,
    ProjectCreated,
    ProjectUpdated,
    ProjectArchived,
    ProjectDeleted,
    ProjectCompositionSlotCreated,
    ProjectCompositionSlotUpdated,
    ProjectCompositionSlotDeleted,
    TurnRequested,
    TurnCompleted,
    TurnFailed,
    TurnCancelRequested,
    TurnCancelled,
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
            Self::SessionCreated => "agent.business.session.created",
            Self::SessionRenamed => "agent.business.session.renamed",
            Self::SessionMoved => "agent.business.session.moved",
            Self::SessionClosed => "agent.business.session.closed",
            Self::SessionArchived => "agent.business.session.archived",
            Self::SessionDeleted => "agent.business.session.deleted",
            Self::MessageCreated => "agent.business.message.created",
            Self::MessageFailed => "agent.business.message.failed",
            Self::MessageFeedbackChanged => "agent.business.message.feedback_changed",
            Self::InteractionCreated => "agent.business.interaction.created",
            Self::InteractionResolved => "agent.business.interaction.resolved",
            Self::InteractionRejected => "agent.business.interaction.rejected",
            Self::InteractionExpired => "agent.business.interaction.expired",
            Self::InteractionCancelled => "agent.business.interaction.cancelled",
            Self::TaskCreated => "agent.business.task.created",
            Self::TaskCompleted => "agent.business.task.completed",
            Self::TaskFailed => "agent.business.task.failed",
            Self::TaskCancelled => "agent.business.task.cancelled",
            Self::ProjectCreated => "agent.business.project.created",
            Self::ProjectUpdated => "agent.business.project.updated",
            Self::ProjectArchived => "agent.business.project.archived",
            Self::ProjectDeleted => "agent.business.project.deleted",
            Self::ProjectCompositionSlotCreated => {
                "agent.business.project.composition_slot.created"
            }
            Self::ProjectCompositionSlotUpdated => {
                "agent.business.project.composition_slot.updated"
            }
            Self::ProjectCompositionSlotDeleted => {
                "agent.business.project.composition_slot.deleted"
            }
            Self::TurnRequested => "agent.business.chat_turn.requested",
            Self::TurnCompleted => "agent.business.chat_turn.completed",
            Self::TurnFailed => "agent.business.chat_turn.failed",
            Self::TurnCancelRequested => "agent.business.chat_turn.cancel_requested",
            Self::TurnCancelled => "agent.business.chat_turn.cancelled",
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
            Self::SessionCreated => "session_created",
            Self::SessionRenamed => "session_renamed",
            Self::SessionMoved => "session_moved",
            Self::SessionClosed => "session_closed",
            Self::SessionArchived => "session_archived",
            Self::SessionDeleted => "session_deleted",
            Self::MessageCreated => "message_created",
            Self::MessageFailed => "message_failed",
            Self::MessageFeedbackChanged => "message_feedback_changed",
            Self::InteractionCreated => "interaction_created",
            Self::InteractionResolved => "interaction_resolved",
            Self::InteractionRejected => "interaction_rejected",
            Self::InteractionExpired => "interaction_expired",
            Self::InteractionCancelled => "interaction_cancelled",
            Self::TaskCreated => "task_created",
            Self::TaskCompleted => "task_completed",
            Self::TaskFailed => "task_failed",
            Self::TaskCancelled => "task_cancelled",
            Self::ProjectCreated => "project_created",
            Self::ProjectUpdated => "project_updated",
            Self::ProjectArchived => "project_archived",
            Self::ProjectDeleted => "project_deleted",
            Self::ProjectCompositionSlotCreated => "project_composition_slot_created",
            Self::ProjectCompositionSlotUpdated => "project_composition_slot_updated",
            Self::ProjectCompositionSlotDeleted => "project_composition_slot_deleted",
            Self::TurnRequested => "turn_requested",
            Self::TurnCompleted => "turn_completed",
            Self::TurnFailed => "turn_failed",
            Self::TurnCancelRequested => "turn_cancel_requested",
            Self::TurnCancelled => "turn_cancelled",
        }
    }
}

/// Structured audit event payload for agent business operations.
///
/// This payload format replaces the legacy string concatenation approach,
/// providing a versioned, parseable JSON structure for audit events.
/// The `schema_version` field enables future schema evolution without
/// breaking existing consumers.
///
/// # Version History
/// - v1: Initial structured payload format
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgentAuditPayload {
    /// Schema version for forward/backward compatibility
    pub schema_version: String,
    /// The action that triggered this audit event
    pub action: String,
    /// Agent identifier
    pub agent_id: String,
    /// Tenant identifier for multi-tenant isolation
    pub tenant_id: u64,
    /// Organization identifier within tenant
    pub organization_id: u64,
    /// User or service that owns this agent
    pub owner_user_id: u64,
    /// Agent business code (human-readable identifier)
    pub code: String,
    /// Current status after the action
    pub status: String,
    /// Visibility level
    pub visibility: String,
    /// Entity version after the action
    pub version: u64,
    /// Previous status (for status change events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_status: Option<String>,
}

impl AgentAuditPayload {
    /// Current schema version constant
    pub const SCHEMA_VERSION: &'static str = "v1";

    /// Create a new audit payload for agent business events
    pub fn new(action: AgentAuditAction, record: &AgentBusinessRecord) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            action: action.action_code().to_string(),
            agent_id: record.agent_id.clone(),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            owner_user_id: record.owner_user_id,
            code: record.code.clone(),
            status: record.status.as_str().to_string(),
            visibility: record.visibility.as_str().to_string(),
            version: record.version,
            previous_status: None,
        }
    }

    /// Add previous status for status change events
    pub fn with_previous_status(mut self, previous_status: AgentBusinessStatus) -> Self {
        self.previous_status = Some(previous_status.as_str().to_string());
        self
    }

    /// Convert to JSON string for storage
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Structured audit payload for provider binding operations
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProviderBindingAuditPayload {
    pub schema_version: String,
    pub action: String,
    pub agent_id: String,
    pub tenant_id: u64,
    pub binding_id: String,
    pub provider_id: String,
    pub implementation_kind: String,
    pub configuration_profile_id: String,
    pub capabilities: Vec<String>,
    pub active: bool,
    pub version: u64,
}

impl ProviderBindingAuditPayload {
    pub const SCHEMA_VERSION: &'static str = "v1";

    pub fn new(action: AgentAuditAction, record: &AgentProviderBindingRecord) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            action: action.action_code().to_string(),
            agent_id: record.agent_id.clone(),
            tenant_id: record.tenant_id,
            binding_id: record.binding_id.clone(),
            provider_id: record.provider_id.clone(),
            implementation_kind: record.implementation_kind.as_str().to_string(),
            configuration_profile_id: record.configuration_profile_id.clone(),
            capabilities: record.capabilities.clone(),
            active: record.active,
            version: record.version,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Structured audit payload for runtime execution operations
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeExecutionAuditPayload {
    pub schema_version: String,
    pub action: String,
    pub agent_id: String,
    pub tenant_id: u64,
    pub execution_id: String,
    pub operation: String,
    pub status: String,
}

impl RuntimeExecutionAuditPayload {
    pub const SCHEMA_VERSION: &'static str = "v1";

    pub fn new(action: AgentAuditAction, record: &AgentRuntimeExecutionRecord) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            action: action.action_code().to_string(),
            agent_id: record.agent_id.clone(),
            tenant_id: record.tenant_id,
            execution_id: record.execution_id.clone(),
            operation: record.operation.as_str().to_string(),
            status: record.status.as_str().to_string(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Structured audit payload for marketplace/composition slot operations
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MarketplaceAuditPayload {
    pub schema_version: String,
    pub action: String,
    pub item_kind: String,
    pub item_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub status: String,
    pub visibility: String,
    pub version: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarketplaceAuditPayloadInput<'a> {
    pub action: AgentAuditAction,
    pub item_kind: &'a str,
    pub item_id: &'a str,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub status: AgentBusinessStatus,
    pub visibility: AgentVisibility,
    pub version: u64,
}

impl MarketplaceAuditPayload {
    pub const SCHEMA_VERSION: &'static str = "v1";

    pub fn new(input: MarketplaceAuditPayloadInput<'_>) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            action: input.action.action_code().to_string(),
            item_kind: input.item_kind.to_string(),
            item_id: input.item_id.to_string(),
            tenant_id: input.tenant_id,
            organization_id: input.organization_id,
            status: input.status.as_str().to_string(),
            visibility: input.visibility.as_str().to_string(),
            version: input.version,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Structured audit payload for agent session operations.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SessionAuditPayload {
    pub schema_version: String,
    pub action: String,
    pub session_id: String,
    pub agent_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub status: String,
    pub message_count: u64,
    pub version: u64,
}

impl SessionAuditPayload {
    pub const SCHEMA_VERSION: &'static str = "v1";

    pub fn new(action: AgentAuditAction, record: &AgentSessionRecord) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            action: action.action_code().to_string(),
            session_id: record.session_id.clone(),
            agent_id: record.agent_id.clone(),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            owner_user_id: record.owner_user_id,
            status: record.status.as_str().to_string(),
            message_count: record.message_count,
            version: record.version,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Structured audit payload for agent message operations.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MessageAuditPayload {
    pub schema_version: String,
    pub action: String,
    pub message_id: String,
    pub session_id: String,
    pub agent_id: String,
    pub tenant_id: u64,
    pub role: String,
    pub status: String,
    pub sequence: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl MessageAuditPayload {
    pub const SCHEMA_VERSION: &'static str = "v1";

    pub fn new(action: AgentAuditAction, record: &AgentMessageRecord) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            action: action.action_code().to_string(),
            message_id: record.message_id.clone(),
            session_id: record.session_id.clone(),
            agent_id: record.agent_id.clone(),
            tenant_id: record.tenant_id,
            role: record.role.as_str().to_string(),
            status: record.status.as_str().to_string(),
            sequence: record.sequence,
            input_tokens: record.input_tokens,
            output_tokens: record.output_tokens,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentImplementationType {
    #[default]
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

    pub fn try_from_str(value: &str) -> Option<Self> {
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
    Tools,
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
            Self::Tools => "tools",
        }
    }

    pub fn try_from_str(value: &str) -> Option<Self> {
        match value {
            "memory" => Some(Self::Memory),
            "knowledgebase" => Some(Self::Knowledgebase),
            "skills" => Some(Self::Skills),
            "prompts" => Some(Self::Prompts),
            "drive" => Some(Self::Drive),
            "mcp" => Some(Self::Mcp),
            "tools" => Some(Self::Tools),
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

// ============================================================================
// Agent Session Management — aligns with kernel lifecycle SPI (AgentSession)
// ============================================================================

/// Lifecycle status of an agent chat session.
///
/// This mirrors the kernel's `SessionState` but uses business-friendly
/// values suitable for database storage and API exposure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSessionStatus {
    /// Session is active and accepting messages
    Active,
    /// Session is idle (no recent activity, still resumable)
    Idle,
    /// Session has been explicitly closed by the user or system
    Closed,
    /// Session has been archived (read-only, retained for history)
    Archived,
}

impl AgentSessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Idle => "idle",
            Self::Closed => "closed",
            Self::Archived => "archived",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "idle" => Some(Self::Idle),
            "closed" => Some(Self::Closed),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::Active => 0,
            Self::Idle => 1,
            Self::Closed => 2,
            Self::Archived => 3,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Active),
            1 => Some(Self::Idle),
            2 => Some(Self::Closed),
            3 => Some(Self::Archived),
            _ => None,
        }
    }

    /// Whether the session can accept new messages.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active | Self::Idle)
    }
}

/// A chat session between a user and an agent.
///
/// This record maps to the kernel's `AgentSession` lifecycle SPI and
/// provides the persistence boundary for conversation state. Each session
/// belongs to a tenant and is scoped to a specific agent. Sessions track
/// token usage, message count, and lifecycle timestamps for observability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSessionRecord {
    pub id: u64,
    pub session_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub project_id: Option<String>,
    pub title: Option<String>,
    pub status: AgentSessionStatus,
    pub provider_binding_id: Option<String>,
    pub model_id: Option<String>,
    pub message_count: u64,
    pub last_message_sequence: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub metadata_json: String,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub last_message_at: Option<String>,
    pub closed_at: Option<String>,
    pub archived_at: Option<String>,
    pub deleted_at: Option<String>,
}

impl AgentSessionRecord {
    pub fn mark_updated(&mut self, updated_at: impl Into<String>) {
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }

    pub fn record_message(
        &mut self,
        input_tokens: u64,
        output_tokens: u64,
        occurred_at: impl Into<String>,
    ) {
        self.message_count = self.message_count.saturating_add(1);
        self.last_message_sequence = self.last_message_sequence.saturating_add(1);
        self.total_input_tokens = self.total_input_tokens.saturating_add(input_tokens);
        self.total_output_tokens = self.total_output_tokens.saturating_add(output_tokens);
        self.last_message_at = Some(occurred_at.into());
        self.updated_at = self
            .last_message_at
            .clone()
            .unwrap_or_else(|| self.updated_at.clone());
        self.version = self.version.saturating_add(1);
    }

    /// Record one persisted user + assistant chat turn (single optimistic version bump).
    pub fn record_chat_turn(
        &mut self,
        input_tokens: u64,
        output_tokens: u64,
        occurred_at: impl Into<String>,
    ) {
        self.message_count = self.message_count.saturating_add(2);
        self.last_message_sequence = self.last_message_sequence.saturating_add(2);
        self.total_input_tokens = self.total_input_tokens.saturating_add(input_tokens);
        self.total_output_tokens = self.total_output_tokens.saturating_add(output_tokens);
        self.last_message_at = Some(occurred_at.into());
        self.updated_at = self
            .last_message_at
            .clone()
            .unwrap_or_else(|| self.updated_at.clone());
        self.version = self.version.saturating_add(1);
    }

    pub fn close(&mut self, closed_at: impl Into<String>) {
        let ts = closed_at.into();
        self.status = AgentSessionStatus::Closed;
        self.closed_at = Some(ts.clone());
        self.updated_at = ts;
        self.version = self.version.saturating_add(1);
    }

    pub fn archive(&mut self, archived_at: impl Into<String>) {
        let ts = archived_at.into();
        self.status = AgentSessionStatus::Archived;
        self.archived_at = Some(ts.clone());
        self.updated_at = ts;
        self.version = self.version.saturating_add(1);
    }

    pub fn soft_delete(&mut self, deleted_at: impl Into<String>) {
        let ts = deleted_at.into();
        self.deleted_at = Some(ts.clone());
        self.updated_at = ts;
        self.version = self.version.saturating_add(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentResourceType {
    Session,
    Project,
}

impl AgentResourceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Project => "project",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Session => 0,
            Self::Project => 1,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Session),
            1 => Some(Self::Project),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentResourceUserStateRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub resource_type: AgentResourceType,
    pub resource_id: String,
    pub pinned_at: Option<String>,
    pub hidden_at: Option<String>,
    pub last_opened_at: Option<String>,
    pub last_read_message_sequence: Option<u64>,
    pub custom_title: Option<String>,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
}

// ============================================================================
// Agent Message Management — aligns with kernel message SPI (AgentMessage)
// ============================================================================

/// Role of a message participant in a session.
///
/// Mirrors the kernel's `AgentMessageRole` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMessageRole {
    User,
    Assistant,
    System,
    Tool,
}

impl AgentMessageRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
            Self::Tool => "tool",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "user" => Some(Self::User),
            "assistant" => Some(Self::Assistant),
            "system" => Some(Self::System),
            "tool" => Some(Self::Tool),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::User => 0,
            Self::Assistant => 1,
            Self::System => 2,
            Self::Tool => 3,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::User),
            1 => Some(Self::Assistant),
            2 => Some(Self::System),
            3 => Some(Self::Tool),
            _ => None,
        }
    }
}

/// Delivery status of a message in a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMessageStatus {
    /// Message has been sent but not yet processed
    Sent,
    /// Message has been delivered to the recipient
    Delivered,
    /// Message has been read by the recipient
    Read,
    /// Message processing failed
    Failed,
    /// Message was cancelled before completion
    Cancelled,
}

impl AgentMessageStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sent => "sent",
            Self::Delivered => "delivered",
            Self::Read => "read",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "sent" => Some(Self::Sent),
            "delivered" => Some(Self::Delivered),
            "read" => Some(Self::Read),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::Sent => 0,
            Self::Delivered => 1,
            Self::Read => 2,
            Self::Failed => 3,
            Self::Cancelled => 4,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Sent),
            1 => Some(Self::Delivered),
            2 => Some(Self::Read),
            3 => Some(Self::Failed),
            4 => Some(Self::Cancelled),
            _ => None,
        }
    }
}

/// A single message in an agent chat session.
///
/// This record maps to the kernel's `AgentMessage` SPI and provides
/// the persistence boundary for individual messages. Messages track
/// their role, content, status, token usage, and optional artifacts
/// (file references, tool outputs, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentMessageRecord {
    pub id: u64,
    pub message_id: String,
    pub tenant_id: u64,
    pub session_id: String,
    pub agent_id: String,
    pub role: AgentMessageRole,
    pub content: String,
    pub content_type: String,
    pub status: AgentMessageStatus,
    pub sequence: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model_id: Option<String>,
    pub provider_id: Option<String>,
    pub artifacts_json: String,
    pub metadata_json: String,
    pub parent_message_id: Option<String>,
    pub turn_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl AgentMessageRecord {
    pub fn mark_updated(&mut self, updated_at: impl Into<String>) {
        self.updated_at = updated_at.into();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMessageFeedbackRating {
    Up,
    Down,
}

impl AgentMessageFeedbackRating {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Up => 1,
            Self::Down => -1,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::Up),
            -1 => Some(Self::Down),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentMessageFeedbackRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub message_id: String,
    pub user_id: u64,
    pub rating: AgentMessageFeedbackRating,
    pub reason_code: Option<String>,
    pub comment: Option<String>,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMessageMediaRole {
    Attachment,
    Image,
    Voice,
    GeneratedOutput,
    Artifact,
}

impl AgentMessageMediaRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Attachment => "attachment",
            Self::Image => "image",
            Self::Voice => "voice",
            Self::GeneratedOutput => "generated_output",
            Self::Artifact => "artifact",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentMessageDriveRefRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub message_id: String,
    pub media_role: AgentMessageMediaRole,
    pub drive_space_id: String,
    pub drive_node_id: String,
    pub drive_uri: String,
    pub media_resource_id: Option<String>,
    pub object_blob_id: Option<String>,
    pub resource_snapshot_json: String,
    pub resource_hash: String,
    pub alt_text: Option<String>,
    pub sort_order: u32,
    pub status: i16,
    pub created_by: u64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub retention_until: Option<String>,
}

// ============================================================================
// Agent Live Interaction Management — code-engine pause/resume lifecycle
// ============================================================================

/// Lifecycle status of a live interaction during a code-engine turn.
///
/// Code engines (codex, claude-code, opencode, gemini) may pause execution
/// to request user input. This enum tracks the interaction from creation
/// through resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentInteractionStatus {
    /// Interaction is pending user response.
    Pending,
    /// User has approved or answered the interaction.
    Resolved,
    /// User has rejected the interaction.
    Rejected,
    /// Interaction expired without response.
    Expired,
    /// Interaction was cancelled by the engine.
    Cancelled,
}

impl AgentInteractionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Resolved => "resolved",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "resolved" => Some(Self::Resolved),
            "rejected" => Some(Self::Rejected),
            "expired" => Some(Self::Expired),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::Pending => 0,
            Self::Resolved => 1,
            Self::Rejected => 2,
            Self::Expired => 3,
            Self::Cancelled => 4,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Pending),
            1 => Some(Self::Resolved),
            2 => Some(Self::Rejected),
            3 => Some(Self::Expired),
            4 => Some(Self::Cancelled),
            _ => None,
        }
    }

    /// Whether the interaction still awaits a user response.
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }
}

/// Kind of live interaction requested by a code engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentInteractionKind {
    /// Permission approval request (e.g. file write, command execution).
    Approval,
    /// User question with selectable options or free-text answer.
    UserQuestion,
}

impl AgentInteractionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Approval => "approval",
            Self::UserQuestion => "user_question",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "approval" => Some(Self::Approval),
            "user_question" => Some(Self::UserQuestion),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::Approval => 0,
            Self::UserQuestion => 1,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Approval),
            1 => Some(Self::UserQuestion),
            _ => None,
        }
    }
}

/// A live interaction record representing a code-engine pause point.
///
/// When a code engine (codex, claude-code, opencode) pauses execution to
/// request user input, an interaction record is created. The record tracks
/// the prompt, optional selectable options, and the user's resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentInteractionRecord {
    pub id: u64,
    pub interaction_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub session_id: String,
    pub agent_id: String,
    pub engine_key: String,
    pub kind: AgentInteractionKind,
    pub status: AgentInteractionStatus,
    /// The prompt text shown to the user (e.g. "Allow file write to /src/main.rs?").
    pub prompt: String,
    /// Selectable options for user-question interactions (JSON array of strings).
    pub options_json: String,
    /// The user's resolution payload (JSON: approved/answer/reason).
    pub resolution_json: String,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub resolved_at: Option<String>,
}

/// Structured audit payload for agent task operations.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TaskAuditPayload {
    pub schema_version: String,
    pub action: String,
    pub task_id: String,
    pub agent_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub status: String,
    pub version: u64,
}

impl TaskAuditPayload {
    pub const SCHEMA_VERSION: &'static str = "v1";

    pub fn new(action: AgentAuditAction, record: &AgentTaskRecord) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            action: action.action_code().to_string(),
            task_id: record.task_id.clone(),
            agent_id: record.agent_id.clone(),
            tenant_id: record.tenant_id,
            organization_id: record.organization_id,
            owner_user_id: record.owner_user_id,
            status: record.status.as_str().to_string(),
            version: record.version,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl AgentInteractionRecord {
    pub fn mark_updated(&mut self, updated_at: impl Into<String>) {
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }

    /// Resolve the interaction with the given resolution JSON.
    pub fn resolve(
        &mut self,
        status: AgentInteractionStatus,
        resolution_json: impl Into<String>,
        resolved_at: impl Into<String>,
    ) {
        let ts = resolved_at.into();
        self.status = status;
        self.resolution_json = resolution_json.into();
        self.resolved_at = Some(ts.clone());
        self.updated_at = ts;
        self.version = self.version.saturating_add(1);
    }

    /// Whether the interaction still awaits a user response.
    pub fn is_pending(&self) -> bool {
        self.status.is_pending()
    }
}

// ============================================================================
// Agent Task Management — kernel AgentTask projection for scheduling
// ============================================================================

/// Lifecycle status of a scheduled agent task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl AgentTaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> i16 {
        match self {
            Self::Pending => 0,
            Self::Running => 1,
            Self::Completed => 2,
            Self::Failed => 3,
            Self::Cancelled => 4,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Pending),
            1 => Some(Self::Running),
            2 => Some(Self::Completed),
            3 => Some(Self::Failed),
            4 => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn is_cancellable(&self) -> bool {
        matches!(self, Self::Pending | Self::Running)
    }
}

/// A scheduled task for an agent, projected from the kernel `AgentTask` SPI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTaskRecord {
    pub id: u64,
    pub task_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub title: Option<String>,
    pub prompt: String,
    pub status: AgentTaskStatus,
    pub external_ref: Option<String>,
    pub metadata_json: String,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub cancelled_at: Option<String>,
}

impl AgentTaskRecord {
    pub fn mark_updated(&mut self, updated_at: impl Into<String>) {
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }

    pub fn cancel(&mut self, cancelled_at: impl Into<String>) {
        let ts = cancelled_at.into();
        self.status = AgentTaskStatus::Cancelled;
        self.cancelled_at = Some(ts.clone());
        self.updated_at = ts;
        self.version = self.version.saturating_add(1);
    }

    pub fn mark_running(&mut self, started_at: impl Into<String>) {
        let ts = started_at.into();
        self.status = AgentTaskStatus::Running;
        self.started_at = Some(ts.clone());
        self.updated_at = ts;
        self.version = self.version.saturating_add(1);
    }

    pub fn mark_completed(&mut self, completed_at: impl Into<String>, output: &str) {
        let ts = completed_at.into();
        self.status = AgentTaskStatus::Completed;
        self.completed_at = Some(ts.clone());
        self.updated_at = ts;
        self.version = self.version.saturating_add(1);
        if let Ok(mut metadata) = serde_json::from_str::<serde_json::Value>(&self.metadata_json) {
            if let Some(obj) = metadata.as_object_mut() {
                obj.insert(
                    "output".to_string(),
                    serde_json::Value::String(output.to_string()),
                );
                if let Ok(json) = serde_json::to_string(obj) {
                    self.metadata_json = json;
                }
            }
        }
    }

    pub fn mark_failed(&mut self, completed_at: impl Into<String>, error: &str) {
        let ts = completed_at.into();
        self.status = AgentTaskStatus::Failed;
        self.completed_at = Some(ts.clone());
        self.updated_at = ts;
        self.version = self.version.saturating_add(1);
        if let Ok(mut metadata) = serde_json::from_str::<serde_json::Value>(&self.metadata_json) {
            if let Some(obj) = metadata.as_object_mut() {
                obj.insert(
                    "error".to_string(),
                    serde_json::Value::String(error.to_string()),
                );
                if let Ok(json) = serde_json::to_string(obj) {
                    self.metadata_json = json;
                }
            }
        }
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
            assert_eq!(AgentCompositionSlotKind::try_from_str(s), Some(kind));
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
            assert_eq!(AgentCompositionTargetModule::try_from_str(s), Some(module));
        }
    }

    #[test]
    fn composition_slot_kind_rejects_unknown_value() {
        assert!(AgentCompositionSlotKind::try_from_str("unknown").is_none());
    }
}
