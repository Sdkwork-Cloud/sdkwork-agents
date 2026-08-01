use sdkwork_agent_kernel::{
    AgentConfigurationProvider, AgentExecutionSettingsRequest, AgentExecutionSettingsResolution,
    AgentExecutionSettingsSpec, AgentMessage, AgentModelConfigurationApplication,
    AgentModelConfigurationRequest, AgentModelSelectionRequest, AgentSession, KernelResult,
    ModelDescriptor, ModelProvider, ModelRequest, ModelResponse, ModelStreamChunk, ModelStreamSink,
    ProviderSessionActivityProvider, SessionActivitySnapshot,
};
use sdkwork_agent_provider_claude_code::{
    ClaudeCodeConfigurationProvider, ClaudeCodeSdkIntegration,
};
use sdkwork_agent_provider_codex::{CodexConfigurationProvider, CodexSdkIntegration};
use sdkwork_agent_provider_core::{SessionLifecycleProvider, SessionListQuery};
use sdkwork_agent_provider_gemini_cli::{GeminiCliConfigurationProvider, GeminiCliSdkIntegration};
use sdkwork_agent_provider_hermes::{HermesConfigurationProvider, HermesSdkIntegration};
use sdkwork_agent_provider_openclaw::{OpenClawConfigurationProvider, OpenClawSdkIntegration};
use sdkwork_agent_provider_opencode::{OpenCodeConfigurationProvider, OpenCodeSdkIntegration};
use sdkwork_agent_provider_spi::{
    SdkRuntimeInteractionResolution, SdkRuntimeStreamCompletion, CLAUDE_CODE_BINDING_ID,
    CODEX_BINDING_ID, GEMINI_CLI_BINDING_ID, HERMES_BINDING_ID, OPENCLAW_BINDING_ID,
    OPENCODE_BINDING_ID,
};

/// Canonical T1 code-engine keys bootstrapped by default in production hosts.
pub const CANONICAL_CODE_ENGINE_KEYS: [&str; 4] = ["codex", "claude-code", "gemini", "opencode"];

/// T2 autonomous agent engines (bootstrap on demand; included in full catalog).
pub const EXTENDED_AUTONOMOUS_ENGINE_KEYS: [&str; 2] = ["openclaw", "hermes"];

pub fn canonical_code_engine_keys() -> &'static [&'static str] {
    &CANONICAL_CODE_ENGINE_KEYS
}

pub fn bootstrappable_engine_keys() -> [&'static str; 6] {
    [
        CANONICAL_CODE_ENGINE_KEYS[0],
        CANONICAL_CODE_ENGINE_KEYS[1],
        CANONICAL_CODE_ENGINE_KEYS[2],
        CANONICAL_CODE_ENGINE_KEYS[3],
        EXTENDED_AUTONOMOUS_ENGINE_KEYS[0],
        EXTENDED_AUTONOMOUS_ENGINE_KEYS[1],
    ]
}

pub fn engine_catalog_tier(engine_key: &str) -> Option<&'static str> {
    match engine_key {
        "codex" | "claude-code" | "gemini" | "opencode" => Some("t1-code"),
        "openclaw" | "hermes" => Some("t2-autonomous"),
        _ => None,
    }
}

pub fn is_canonical_code_engine(engine_key: &str) -> bool {
    CANONICAL_CODE_ENGINE_KEYS.contains(&engine_key)
}

pub fn apply_code_engine_model_configuration(
    engine_key: &str,
    request: &AgentModelConfigurationRequest,
) -> crate::RuntimeFacadeResult<AgentModelConfigurationApplication> {
    let expected_agent_id = code_engine_agent_id(engine_key).ok_or_else(|| {
        crate::RuntimeFacadeError::UnsupportedEngine {
            engine_key: engine_key.to_string(),
        }
    })?;
    if request.agent_id != expected_agent_id {
        return Err(crate::RuntimeFacadeError::InvalidInput(format!(
            "model configuration agentId does not match engineId {engine_key}"
        )));
    }

    let result = match engine_key {
        "codex" => CodexConfigurationProvider::new().apply_model_configuration(request),
        "claude-code" => ClaudeCodeConfigurationProvider::new().apply_model_configuration(request),
        "gemini" => GeminiCliConfigurationProvider::new().apply_model_configuration(request),
        "opencode" => OpenCodeConfigurationProvider::new().apply_model_configuration(request),
        "openclaw" => OpenClawConfigurationProvider::new().apply_model_configuration(request),
        "hermes" => HermesConfigurationProvider::new().apply_model_configuration(request),
        _ => unreachable!("validated code engine"),
    };
    result.map_err(|error| crate::RuntimeFacadeError::Kernel(error.to_string()))
}

pub fn apply_code_engine_model_selection(
    engine_key: &str,
    request: &AgentModelSelectionRequest,
) -> crate::RuntimeFacadeResult<AgentModelConfigurationApplication> {
    let expected_agent_id = code_engine_agent_id(engine_key).ok_or_else(|| {
        crate::RuntimeFacadeError::UnsupportedEngine {
            engine_key: engine_key.to_string(),
        }
    })?;
    if request.agent_id != expected_agent_id {
        return Err(crate::RuntimeFacadeError::InvalidInput(format!(
            "model selection agentId does not match engineId {engine_key}"
        )));
    }

    let result = match engine_key {
        "codex" => CodexConfigurationProvider::new().apply_model_selection(request),
        "claude-code" => ClaudeCodeConfigurationProvider::new().apply_model_selection(request),
        "gemini" => GeminiCliConfigurationProvider::new().apply_model_selection(request),
        "opencode" => OpenCodeConfigurationProvider::new().apply_model_selection(request),
        "openclaw" => OpenClawConfigurationProvider::new().apply_model_selection(request),
        "hermes" => HermesConfigurationProvider::new().apply_model_selection(request),
        _ => unreachable!("validated code engine"),
    };
    result.map_err(|error| crate::RuntimeFacadeError::Kernel(error.to_string()))
}

pub fn code_engine_agent_id(engine_key: &str) -> Option<&'static str> {
    match engine_key {
        "codex" => Some("agent.intelligence.codex"),
        "claude-code" => Some("agent.intelligence.claude-code"),
        "gemini" => Some("agent.intelligence.gemini"),
        "opencode" => Some("agent.intelligence.opencode"),
        "openclaw" => Some("agent.intelligence.openclaw"),
        "hermes" => Some("agent.intelligence.hermes"),
        _ => None,
    }
}

pub fn code_engine_binding_id(engine_key: &str) -> Option<&'static str> {
    match engine_key {
        "codex" => Some(CODEX_BINDING_ID),
        "claude-code" => Some(CLAUDE_CODE_BINDING_ID),
        "gemini" => Some(GEMINI_CLI_BINDING_ID),
        "opencode" => Some(OPENCODE_BINDING_ID),
        "openclaw" => Some(OPENCLAW_BINDING_ID),
        "hermes" => Some(HERMES_BINDING_ID),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeEngineRuntimeIdentity {
    pub engine_key: String,
    pub agent_id: String,
    pub binding_id: String,
    pub provider_id: String,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CodeEngineInteractionResolution {
    pub model_request_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub provider_session_id: String,
    pub provider_turn_id: String,
    pub provider_request_id: serde_json::Value,
    pub resolution: serde_json::Value,
}

pub fn resolve_code_engine_runtime_identity(
    agent_id: &str,
) -> Result<Option<CodeEngineRuntimeIdentity>, CodeEngineBootstrapError> {
    let Some(engine_key) = bootstrappable_engine_keys()
        .into_iter()
        .find(|engine_key| code_engine_agent_id(engine_key) == Some(agent_id))
    else {
        return Ok(None);
    };
    let slot = bootstrap_code_engine(engine_key)?;
    let provider_id = slot
        .list_model_descriptors()
        .into_iter()
        .next()
        .map(|descriptor| descriptor.provider_id)
        .ok_or_else(|| {
            CodeEngineBootstrapError::Bootstrap(format!(
                "code engine {engine_key} did not publish a model provider"
            ))
        })?;
    Ok(Some(CodeEngineRuntimeIdentity {
        engine_key: engine_key.to_string(),
        agent_id: agent_id.to_string(),
        binding_id: slot.binding_id().to_string(),
        provider_id,
    }))
}

#[derive(Debug)]
pub enum CodeEngineBootstrapError {
    UnsupportedEngine(String),
    Bootstrap(String),
}

impl std::fmt::Display for CodeEngineBootstrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedEngine(engine) => {
                write!(f, "unsupported code engine for bootstrap: {engine}")
            }
            Self::Bootstrap(message) => write!(f, "code engine bootstrap failed: {message}"),
        }
    }
}

impl std::error::Error for CodeEngineBootstrapError {}

/// Bootstrapped kernel provider slot for one canonical code engine.
pub enum CodeEngineSlot {
    Codex(CodexSdkIntegration),
    ClaudeCode(ClaudeCodeSdkIntegration),
    Gemini(GeminiCliSdkIntegration),
    OpenCode(OpenCodeSdkIntegration),
    OpenClaw(OpenClawSdkIntegration),
    Hermes(HermesSdkIntegration),
}

impl CodeEngineSlot {
    pub fn engine_key(&self) -> &'static str {
        match self {
            Self::Codex(_) => "codex",
            Self::ClaudeCode(_) => "claude-code",
            Self::Gemini(_) => "gemini",
            Self::OpenCode(_) => "opencode",
            Self::OpenClaw(_) => "openclaw",
            Self::Hermes(_) => "hermes",
        }
    }

    pub fn binding_id(&self) -> &str {
        match self {
            Self::Codex(integration) => integration.binding_id(),
            Self::ClaudeCode(integration) => integration.binding_id(),
            Self::Gemini(integration) => integration.binding_id(),
            Self::OpenCode(integration) => integration.binding_id(),
            Self::OpenClaw(integration) => integration.binding_id(),
            Self::Hermes(integration) => integration.binding_id(),
        }
    }

    pub fn list_model_ids(&self) -> Vec<String> {
        self.list_model_descriptors()
            .into_iter()
            .map(|descriptor| descriptor.model_id)
            .collect()
    }

    pub fn list_model_descriptors(&self) -> Vec<ModelDescriptor> {
        self.model_provider().list_models()
    }

    pub fn execution_settings_spec(&self) -> KernelResult<AgentExecutionSettingsSpec> {
        let agent_id = code_engine_agent_id(self.engine_key()).ok_or_else(|| {
            sdkwork_agent_kernel::KernelError::CapabilityMissing {
                capability_id: format!("agent.configure.execution.{}", self.engine_key()),
            }
        })?;
        match self {
            Self::Codex(_) => CodexConfigurationProvider::new().execution_settings_spec(agent_id),
            Self::ClaudeCode(_) => {
                ClaudeCodeConfigurationProvider::new().execution_settings_spec(agent_id)
            }
            Self::Gemini(_) => {
                GeminiCliConfigurationProvider::new().execution_settings_spec(agent_id)
            }
            Self::OpenCode(_) => {
                OpenCodeConfigurationProvider::new().execution_settings_spec(agent_id)
            }
            Self::OpenClaw(_) | Self::Hermes(_) => {
                Err(sdkwork_agent_kernel::KernelError::CapabilityMissing {
                    capability_id: format!("agent.configure.execution.{}", self.engine_key()),
                })
            }
        }
    }

    pub fn resolve_execution_settings(
        &self,
        access_mode_id: &str,
    ) -> KernelResult<AgentExecutionSettingsResolution> {
        let agent_id = code_engine_agent_id(self.engine_key()).ok_or_else(|| {
            sdkwork_agent_kernel::KernelError::CapabilityMissing {
                capability_id: format!("agent.configure.execution.{}", self.engine_key()),
            }
        })?;
        let request = AgentExecutionSettingsRequest::new(agent_id).with_access_mode(access_mode_id);
        match self {
            Self::Codex(_) => {
                CodexConfigurationProvider::new().resolve_execution_settings(&request)
            }
            Self::ClaudeCode(_) => {
                ClaudeCodeConfigurationProvider::new().resolve_execution_settings(&request)
            }
            Self::Gemini(_) => {
                GeminiCliConfigurationProvider::new().resolve_execution_settings(&request)
            }
            Self::OpenCode(_) => {
                OpenCodeConfigurationProvider::new().resolve_execution_settings(&request)
            }
            Self::OpenClaw(_) | Self::Hermes(_) => {
                Err(sdkwork_agent_kernel::KernelError::CapabilityMissing {
                    capability_id: format!("agent.configure.execution.{}", self.engine_key()),
                })
            }
        }
    }

    pub fn invoke_model(&self, request: ModelRequest) -> KernelResult<ModelResponse> {
        self.model_provider().invoke(request)
    }

    pub fn stream_model(&self, request: ModelRequest) -> KernelResult<Vec<ModelStreamChunk>> {
        self.model_provider().stream(request)
    }

    /// Streams provider-neutral model chunks through the kernel SPI.
    ///
    /// Product callers must consume this method through the runtime facade
    /// instead of importing individual provider SDKs or transport crates.
    pub fn stream_model_into(
        &self,
        request: ModelRequest,
        sink: &mut dyn ModelStreamSink,
    ) -> KernelResult<()> {
        self.model_provider().stream_into(request, sink)
    }

    pub fn resolve_interaction(
        &self,
        resolution: &CodeEngineInteractionResolution,
    ) -> crate::RuntimeFacadeResult<serde_json::Value> {
        let runtime_resolution = SdkRuntimeInteractionResolution {
            model_request_id: resolution.model_request_id.clone(),
            session_id: resolution.session_id.clone(),
            turn_id: resolution.turn_id.clone(),
            provider_session_id: resolution.provider_session_id.clone(),
            provider_turn_id: resolution.provider_turn_id.clone(),
            provider_request_id: resolution.provider_request_id.clone(),
            resolution: resolution.resolution.clone(),
        };
        match self {
            Self::Codex(integration) => integration
                .resolve_interaction(&runtime_resolution)
                .map_err(|error| crate::RuntimeFacadeError::Kernel(error.to_string())),
            _ => Err(crate::RuntimeFacadeError::InvalidInput(format!(
                "code engine {} does not support typed interaction resolution",
                self.engine_key()
            ))),
        }
    }

    pub fn get_provider_session_activity(
        &self,
        provider_session_id: &str,
    ) -> KernelResult<SessionActivitySnapshot> {
        match self {
            Self::Codex(integration) => {
                integration.get_provider_session_activity(provider_session_id)
            }
            Self::ClaudeCode(integration) => {
                integration.get_provider_session_activity(provider_session_id)
            }
            Self::Gemini(integration) => {
                integration.get_provider_session_activity(provider_session_id)
            }
            Self::OpenCode(integration) => {
                integration.get_provider_session_activity(provider_session_id)
            }
            Self::OpenClaw(_) | Self::Hermes(_) => {
                Ok(SessionActivitySnapshot::unsupported(provider_session_id))
            }
        }
    }

    pub fn list_provider_sessions(&self) -> KernelResult<Vec<AgentSession>> {
        match self {
            Self::Codex(integration) => integration.list_provider_sessions(),
            Self::ClaudeCode(integration) => integration.list_provider_sessions(),
            Self::Gemini(integration) => integration.list_provider_sessions(),
            Self::OpenCode(integration) => integration.list_provider_sessions(),
            Self::OpenClaw(integration) => integration
                .lifecycle
                .list_sessions(&SessionListQuery::default()),
            Self::Hermes(integration) => integration
                .lifecycle
                .list_sessions(&SessionListQuery::default()),
        }
    }

    pub fn get_provider_session_history(
        &self,
        provider_session_id: &str,
    ) -> KernelResult<Vec<AgentMessage>> {
        match self {
            Self::Codex(integration) => {
                integration.get_provider_session_history(provider_session_id)
            }
            Self::ClaudeCode(integration) => {
                integration.get_provider_session_history(provider_session_id)
            }
            Self::Gemini(integration) => {
                integration.get_provider_session_history(provider_session_id)
            }
            Self::OpenCode(integration) => {
                integration.get_provider_session_history(provider_session_id)
            }
            Self::OpenClaw(integration) => integration
                .lifecycle
                .get_conversation_history(provider_session_id),
            Self::Hermes(integration) => integration
                .lifecycle
                .get_conversation_history(provider_session_id),
        }
    }

    /// Whether this engine can establish a new provider session from a verified
    /// runtime stream completion. Codex is the first provider with that
    /// end-to-end contract; other providers remain invoke-only for initial
    /// turns until their runtime can prove the same identity.
    pub(crate) fn supports_first_turn_streaming_completion(&self) -> bool {
        matches!(self, Self::Codex(_))
    }

    /// Streams an initial turn through the runtime-backed completion boundary.
    ///
    /// This intentionally remains crate-private: callers consume the
    /// provider-neutral facade completion rather than transport metadata.
    pub(crate) fn stream_first_turn_model_into(
        &self,
        request: ModelRequest,
        sink: &mut dyn ModelStreamSink,
    ) -> KernelResult<SdkRuntimeStreamCompletion> {
        match self {
            Self::Codex(integration) => {
                integration.model.stream_into_with_completion(request, sink)
            }
            _ => Err(sdkwork_agent_kernel::KernelError::CapabilityMissing {
                capability_id: "model.streaming.initial_session_completion".to_string(),
            }),
        }
    }

    pub(crate) fn model_provider(&self) -> &dyn ModelProvider {
        match self {
            Self::Codex(integration) => &integration.model,
            Self::ClaudeCode(integration) => &integration.model,
            Self::Gemini(integration) => &integration.model,
            Self::OpenCode(integration) => &integration.model,
            Self::OpenClaw(integration) => &integration.model,
            Self::Hermes(integration) => &integration.model,
        }
    }
}

pub fn bootstrap_code_engine(engine_key: &str) -> Result<CodeEngineSlot, CodeEngineBootstrapError> {
    match engine_key {
        "codex" => CodexSdkIntegration::bootstrap()
            .map(CodeEngineSlot::Codex)
            .map_err(|error| CodeEngineBootstrapError::Bootstrap(error.to_string())),
        "claude-code" => ClaudeCodeSdkIntegration::bootstrap()
            .map(CodeEngineSlot::ClaudeCode)
            .map_err(|error| CodeEngineBootstrapError::Bootstrap(error.to_string())),
        "gemini" => GeminiCliSdkIntegration::bootstrap()
            .map(CodeEngineSlot::Gemini)
            .map_err(|error| CodeEngineBootstrapError::Bootstrap(error.to_string())),
        "opencode" => OpenCodeSdkIntegration::bootstrap()
            .map(CodeEngineSlot::OpenCode)
            .map_err(|error| CodeEngineBootstrapError::Bootstrap(error.to_string())),
        "openclaw" => OpenClawSdkIntegration::bootstrap()
            .map(CodeEngineSlot::OpenClaw)
            .map_err(|error| CodeEngineBootstrapError::Bootstrap(error.to_string())),
        "hermes" => HermesSdkIntegration::bootstrap()
            .map(CodeEngineSlot::Hermes)
            .map_err(|error| CodeEngineBootstrapError::Bootstrap(error.to_string())),
        other => Err(CodeEngineBootstrapError::UnsupportedEngine(
            other.to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_code_engines_map_to_binding_ids() {
        for engine in canonical_code_engine_keys() {
            assert!(code_engine_agent_id(engine).is_some());
            assert!(code_engine_binding_id(engine).is_some());
        }
    }

    #[test]
    fn all_canonical_code_engines_bootstrap() {
        for engine in canonical_code_engine_keys() {
            let slot = bootstrap_code_engine(engine).unwrap_or_else(|error| {
                panic!("bootstrap failed for {engine}: {error}");
            });
            assert_eq!(slot.engine_key(), *engine);
            assert!(!slot.list_model_ids().is_empty());
        }
    }

    #[test]
    fn only_codex_is_enabled_for_verified_first_turn_streaming() {
        let codex = bootstrap_code_engine("codex").expect("codex bootstrap");
        let gemini = bootstrap_code_engine("gemini").expect("gemini bootstrap");

        assert!(codex.supports_first_turn_streaming_completion());
        assert!(!gemini.supports_first_turn_streaming_completion());
    }

    #[test]
    fn runtime_identity_resolves_every_bootstrappable_agent_id() {
        for engine_key in bootstrappable_engine_keys() {
            let agent_id = code_engine_agent_id(engine_key).expect("agent id");
            let identity = resolve_code_engine_runtime_identity(agent_id)
                .expect("identity resolution")
                .expect("known identity");
            assert_eq!(identity.engine_key, engine_key);
            assert_eq!(identity.agent_id, agent_id);
            assert_eq!(
                identity.binding_id,
                code_engine_binding_id(engine_key).unwrap()
            );
            assert!(!identity.provider_id.is_empty());
        }
        assert!(
            resolve_code_engine_runtime_identity("agent.intelligence.unknown")
                .expect("unknown identity resolution")
                .is_none()
        );
    }

    #[test]
    fn model_configuration_dispatches_to_each_code_engine_config_spi() {
        for engine_key in bootstrappable_engine_keys() {
            let agent_id = code_engine_agent_id(engine_key).expect("agent id");
            let request = AgentModelConfigurationRequest::new(
                format!("request.model.{engine_key}"),
                agent_id,
                format!("profile.model.{engine_key}"),
                "openai-compatible",
                "https://models.example.test/v1",
                format!("secret-model-{engine_key}"),
                "example-chat",
            );
            let application = apply_code_engine_model_configuration(engine_key, &request)
                .expect("provider Config SPI applies model configuration");
            assert_eq!(application.profile.agent_id, agent_id);
            assert_eq!(application.profile.profile_id, request.profile_id);
            assert_ne!(application.provider_scope, "process_adapter");
        }
    }

    #[test]
    fn model_selection_dispatches_to_each_code_engine_config_spi() {
        for engine_key in bootstrappable_engine_keys() {
            let agent_id = code_engine_agent_id(engine_key).expect("agent id");
            let request = AgentModelSelectionRequest::new(
                format!("request.selection.{engine_key}"),
                agent_id,
                format!("profile.selection.{engine_key}"),
                "catalog-model",
            );
            let application = apply_code_engine_model_selection(engine_key, &request)
                .expect("provider Config SPI applies model selection");
            assert_eq!(application.profile.agent_id, agent_id);
            assert_eq!(application.profile.profile_id, request.profile_id);
            assert_ne!(application.provider_scope, "process_adapter");
        }
    }
}
