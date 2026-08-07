//! The audio category media tool provider.

use sdkwork_agent_kernel::{
    KernelResult, ProviderHealth, ProviderManifest, ToolCall, ToolDescriptor, ToolProvider,
    ToolResult,
};
use sdkwork_agents_tool_contract::{
    media_tool_call, project_invoke_result, project_kernel_error, project_tool_descriptor,
    MediaAuthTokenResolver, MediaToolCall, MediaToolDefinition, MediaToolError, MediaToolProvider,
    MediaToolResult, ToolCategory,
};

use crate::definitions::audio_tool_definitions;
use crate::invoke::invoke_audio_tool;

/// Provider id for the cloudrouter-backed audio category.
pub const AUDIO_PROVIDER_ID: &str = "cloudrouter.media.audio";

/// Audio category tool provider (cloudrouter open-api backed).
///
/// Implements both the media tool contract (`MediaToolProvider`) and the
/// kernel `ToolProvider` SPI; the kernel projection resolves the caller auth
/// token through the injected resolver at invocation time.
#[derive(Debug)]
pub struct AudioMediaToolProvider {
    token_resolver: Box<dyn MediaAuthTokenResolver>,
}

impl AudioMediaToolProvider {
    pub fn new(token_resolver: Box<dyn MediaAuthTokenResolver>) -> Self {
        Self { token_resolver }
    }

    /// Provider with no auth token resolution (kernel projections will fail
    /// with an auth-required error unless the application wires a resolver).
    pub fn without_auth_resolver() -> Self {
        Self {
            token_resolver: Box::new(sdkwork_agents_tool_contract::StaticMediaAuthTokenResolver(
                None,
            )),
        }
    }
}

impl MediaToolProvider for AudioMediaToolProvider {
    fn category(&self) -> ToolCategory {
        ToolCategory::Audio
    }

    fn definitions(&self) -> Vec<MediaToolDefinition> {
        audio_tool_definitions()
    }

    fn invoke(
        &self,
        call: &MediaToolCall,
        auth_token: Option<&str>,
    ) -> Result<MediaToolResult, MediaToolError> {
        invoke_audio_tool(call, auth_token)
    }
}

impl ToolProvider for AudioMediaToolProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            AUDIO_PROVIDER_ID,
            "tool",
            "Audio media tools (cloudrouter open-api)",
            "0.1.0",
            vec![
                "tool.invoke".to_string(),
                "media.audio.generate".to_string(),
            ],
        )
    }

    fn list_tools(&self) -> Vec<ToolDescriptor> {
        audio_tool_definitions()
            .iter()
            .map(|definition| project_tool_descriptor(definition, AUDIO_PROVIDER_ID))
            .collect()
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }

    fn invoke_tool(&self, call: ToolCall) -> KernelResult<ToolResult> {
        let media_call: MediaToolCall = media_tool_call(&call)?;
        let auth_token = self
            .token_resolver
            .resolve(media_call.session_id.as_deref());
        let outcome = self.invoke(&media_call, auth_token.as_deref());
        project_invoke_result(&call.tool_call_id, outcome)
    }

    fn describe_tool(&self, tool_id: &str) -> KernelResult<ToolDescriptor> {
        audio_tool_definitions()
            .into_iter()
            .find(|definition| definition.tool_id == tool_id)
            .map(|definition| project_tool_descriptor(&definition, AUDIO_PROVIDER_ID))
            .ok_or_else(|| {
                project_kernel_error(MediaToolError::CapabilityMissing(format!(
                    "audio provider has no tool `{tool_id}`"
                )))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definitions::tool_ids;

    #[test]
    fn provider_exposes_all_audio_tools() {
        let provider = AudioMediaToolProvider::without_auth_resolver();
        let descriptors = provider.list_tools();
        let ids: Vec<&str> = descriptors
            .iter()
            .map(|descriptor| descriptor.tool_id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec![
                tool_ids::SPEECH_CREATE,
                tool_ids::TRANSCRIPTIONS_CREATE,
                tool_ids::TRANSLATIONS_CREATE,
                tool_ids::VOICES_LIST,
                tool_ids::VOICES_CREATE,
                tool_ids::VOICE_CONSENTS_CREATE,
                tool_ids::VOICE_CONSENTS_LIST,
            ]
        );
        assert!(descriptors
            .iter()
            .all(|d| d.provider_id == AUDIO_PROVIDER_ID));
    }

    #[test]
    fn kernel_invoke_without_auth_token_fails_closed() {
        let provider = AudioMediaToolProvider::without_auth_resolver();
        let call = ToolCall::new("call.1", tool_ids::SPEECH_CREATE, r#"{"input":"hello"}"#);
        let error = provider.invoke_tool(call).expect_err("auth required");
        assert_eq!(
            error.kind(),
            sdkwork_agent_kernel::KernelErrorKind::PermissionRequired
        );
    }

    #[test]
    fn kernel_invoke_rejects_unknown_tool() {
        let provider = AudioMediaToolProvider::without_auth_resolver();
        let call = ToolCall::new("call.2", "audio.not.a.tool", "{}");
        let error = provider.invoke_tool(call).expect_err("unknown tool");
        assert_eq!(
            error.kind(),
            sdkwork_agent_kernel::KernelErrorKind::CapabilityMissing
        );
    }

    #[test]
    fn describe_known_and_unknown_tools() {
        let provider = AudioMediaToolProvider::without_auth_resolver();
        assert!(provider.describe_tool(tool_ids::SPEECH_CREATE).is_ok());
        assert!(provider.describe_tool("audio.missing").is_err());
    }
}
