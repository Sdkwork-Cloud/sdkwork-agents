use sdkwork_agent_kernel::{
    KernelResult, ModelDescriptor, ModelProvider, ModelRequest, ModelResponse, ModelStreamChunk,
};
use sdkwork_agent_provider_claude_code::ClaudeCodeSdkIntegration;
use sdkwork_agent_provider_codex::CodexSdkIntegration;
use sdkwork_agent_provider_gemini_cli::GeminiCliSdkIntegration;
use sdkwork_agent_provider_hermes::HermesSdkIntegration;
use sdkwork_agent_provider_openclaw::OpenClawSdkIntegration;
use sdkwork_agent_provider_opencode::OpenCodeSdkIntegration;
use sdkwork_agent_provider_spi::{
    CLAUDE_CODE_BINDING_ID, CODEX_BINDING_ID, GEMINI_CLI_BINDING_ID, HERMES_BINDING_ID,
    OPENCLAW_BINDING_ID, OPENCODE_BINDING_ID,
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

    pub fn invoke_model(&self, request: ModelRequest) -> KernelResult<ModelResponse> {
        self.model_provider().invoke(request)
    }

    pub fn stream_model(&self, request: ModelRequest) -> KernelResult<Vec<ModelStreamChunk>> {
        self.model_provider().stream(request)
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
}
