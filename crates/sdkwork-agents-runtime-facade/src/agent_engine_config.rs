//! Provider-native configuration file reading for the agent-engine catalog.
//!
//! Settings surfaces need to show the real configuration file each provider
//! materializes (Codex `config.toml`, Claude Code `settings.json`, Gemini CLI
//! `.env`, OpenCode/OpenClaw JSON, ...) so operators can verify what is
//! applied. The file is served read-only with credential values masked: raw
//! secrets must never leave the host through the catalog API.

use sdkwork_agent_provider_claude_code::claude_code_settings_path;
use sdkwork_agent_provider_codex::codex_config_path;
use sdkwork_agent_provider_core::read_provider_config;
use sdkwork_agent_provider_gemini_cli::gemini_env_path;
use sdkwork_agent_provider_hermes::hermes_config_path;
use sdkwork_agent_provider_mimo_code::mimo_code_settings_path;
use sdkwork_agent_provider_openclaw::openclaw_config_path;
use sdkwork_agent_provider_opencode::opencode_config_path;
use serde::{Deserialize, Serialize};

/// Format hints for the settings code editor language selection.
pub const CONFIG_FORMAT_TOML: &str = "toml";
pub const CONFIG_FORMAT_JSON: &str = "json";
pub const CONFIG_FORMAT_ENV: &str = "env";
pub const CONFIG_FORMAT_TEXT: &str = "text";

/// Sensitive key fragments matched case-insensitively against the key part of
/// a configuration line. Values under these keys are masked before serving.
const SENSITIVE_KEY_FRAGMENTS: [&str; 6] = [
    "api_key", "apikey", "token", "secret", "credential", "password",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEngineConfigFile {
    pub engine_key: String,
    pub config_file_path: String,
    pub format: String,
    /// File content with credential values masked (`****`). Empty when the
    /// provider has no native configuration file or the file does not exist.
    pub content: String,
    pub exists: bool,
}

impl AgentEngineConfigFile {
    fn missing(engine_key: &str, format: &str) -> Self {
        Self {
            engine_key: engine_key.to_string(),
            config_file_path: String::new(),
            format: format.to_string(),
            content: String::new(),
            exists: false,
        }
    }
}

/// Resolves the provider-native configuration file path and format for an
/// engine key. Engines without a native file (e.g. the in-process Rig runtime)
/// return `(None, text)`.
fn provider_config_location(engine_key: &str) -> (Option<std::path::PathBuf>, &'static str) {
    match engine_key {
        "codex" => (codex_config_path(), CONFIG_FORMAT_TOML),
        "claude-code" => (claude_code_settings_path(), CONFIG_FORMAT_JSON),
        "gemini" => (gemini_env_path(), CONFIG_FORMAT_ENV),
        "opencode" => (opencode_config_path(), CONFIG_FORMAT_JSON),
        "hermes" => (hermes_config_path(), CONFIG_FORMAT_TEXT),
        "mimo-code" => (mimo_code_settings_path(), CONFIG_FORMAT_JSON),
        "openclaw" => (openclaw_config_path(), CONFIG_FORMAT_JSON),
        _ => (None, CONFIG_FORMAT_TEXT),
    }
}

/// Reads the provider-native configuration file for an engine, masking
/// credential values. Missing files are reported as `exists: false` rather
/// than an error so settings surfaces can render a helpful empty state.
pub fn read_agent_engine_config_file(
    engine_key: &str,
) -> crate::RuntimeFacadeResult<AgentEngineConfigFile> {
    if !crate::agent_engines::bootstrappable_engine_keys().contains(&engine_key) {
        return Err(crate::RuntimeFacadeError::UnsupportedEngine {
            engine_key: engine_key.to_string(),
        });
    }
    let (path, format) = provider_config_location(engine_key);
    let Some(path) = path else {
        return Ok(AgentEngineConfigFile::missing(engine_key, format));
    };
    if !path.is_file() {
        return Ok(AgentEngineConfigFile::missing(engine_key, format));
    }
    let raw = read_provider_config(&path)
        .map_err(|error| crate::RuntimeFacadeError::Kernel(error.to_string()))?
        .unwrap_or_default();
    Ok(AgentEngineConfigFile {
        engine_key: engine_key.to_string(),
        config_file_path: path.to_string_lossy().into_owned(),
        format: format.to_string(),
        content: mask_sensitive_config(&raw),
        exists: true,
    })
}

/// Masks credential values in configuration content by line: any key that
/// contains a sensitive fragment (api key, token, secret, credential,
/// password) has its value replaced with `****`. Works for `key = value`,
/// `"key": value`, and `KEY=VALUE` line shapes.
fn mask_sensitive_config(content: &str) -> String {
    content
        .lines()
        .map(mask_sensitive_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn mask_sensitive_line(line: &str) -> String {
    let trimmed = line.trim_start();
    let Some(separator_index) = trimmed
        .find('=')
        .or_else(|| trimmed.find(':'))
    else {
        return line.to_string();
    };
    let key_part = &trimmed[..separator_index];
    let key = key_part
        .trim()
        .trim_matches(['"', '\'', ' ', '\t'])
        .to_ascii_lowercase();
    let is_sensitive = SENSITIVE_KEY_FRAGMENTS
        .iter()
        .any(|fragment| key.contains(fragment));
    if !is_sensitive {
        return line.to_string();
    }
    let prefix = &trimmed[..=separator_index];
    // Preserve surrounding whitespace of the original line.
    let leading = &line[..line.len() - trimmed.len()];
    format!("{leading}{prefix} \"****\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_toml_api_key_lines() {
        let content = "model = \"codex-1\"\n[model_providers.sdkwork]\napi_key = \"sk-live-secret\"\nbase_url = \"https://models.example.test/v1\"\n";
        let masked = mask_sensitive_config(content);
        assert!(masked.contains("api_key = \"****\""));
        assert!(!masked.contains("sk-live-secret"));
        assert!(masked.contains("model = \"codex-1\""));
        assert!(masked.contains("base_url = \"https://models.example.test/v1\""));
    }

    #[test]
    fn masks_json_api_key_lines() {
        let content = "{\n  \"apiKey\": \"sk-json-secret\",\n  \"defaultModel\": \"gpt-5\",\n  \"env\": {\n    \"ANTHROPIC_AUTH_TOKEN\": \"token-value\"\n  }\n}\n";
        let masked = mask_sensitive_config(content);
        assert!(masked.contains("\"apiKey\": \"****\""));
        assert!(masked.contains("\"ANTHROPIC_AUTH_TOKEN\": \"****\""));
        assert!(!masked.contains("sk-json-secret"));
        assert!(!masked.contains("token-value"));
        assert!(masked.contains("\"defaultModel\": \"gpt-5\""));
    }

    #[test]
    fn masks_env_var_lines() {
        let content = "GEMINI_API_KEY=sk-env-secret\nGEMINI_MODEL=gemini-2.5-pro\n";
        let masked = mask_sensitive_config(content);
        assert!(masked.contains("GEMINI_API_KEY= \"****\""));
        assert!(!masked.contains("sk-env-secret"));
        assert!(masked.contains("GEMINI_MODEL=gemini-2.5-pro"));
    }

    #[test]
    fn leaves_unsensitive_lines_untouched() {
        let line = "model_provider = \"sdkwork\"";
        assert_eq!(mask_sensitive_line(line), line);
    }

    #[test]
    fn missing_engine_is_rejected() {
        let result = read_agent_engine_config_file("not-an-engine");
        assert!(result.is_err());
    }

    #[test]
    fn rig_reports_missing_without_error() {
        let file = read_agent_engine_config_file("rig").expect("rig config file read");
        assert!(!file.exists);
        assert!(file.config_file_path.is_empty());
    }
}
