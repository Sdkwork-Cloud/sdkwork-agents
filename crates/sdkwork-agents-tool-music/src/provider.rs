//! The music category media tool provider.

use sdkwork_agent_kernel::{
    KernelResult, ProviderHealth, ProviderManifest, ToolCall, ToolDescriptor, ToolProvider,
    ToolResult,
};
use sdkwork_agents_tool_contract::{
    media_tool_call, project_invoke_result, project_kernel_error, project_tool_descriptor,
    MediaAuthTokenResolver, MediaToolCall, MediaToolDefinition, MediaToolError, MediaToolProvider,
    MediaToolResult, StaticMediaAuthTokenResolver, ToolCategory,
};

use crate::definitions::music_tool_definitions;
use crate::invoke::invoke_music_tool;

/// Provider id for the cloudrouter-backed music category.
pub const MUSIC_PROVIDER_ID: &str = "cloudrouter.media.music";

/// Music category tool provider (cloudrouter open-api backed).
#[derive(Debug)]
pub struct MusicMediaToolProvider {
    token_resolver: Box<dyn MediaAuthTokenResolver>,
}

impl MusicMediaToolProvider {
    pub fn new(token_resolver: Box<dyn MediaAuthTokenResolver>) -> Self {
        Self { token_resolver }
    }

    /// Provider with no auth token resolution (kernel projections fail with
    /// an auth-required error unless the application wires a resolver).
    pub fn without_auth_resolver() -> Self {
        Self {
            token_resolver: Box::new(StaticMediaAuthTokenResolver(None)),
        }
    }
}

impl MediaToolProvider for MusicMediaToolProvider {
    fn category(&self) -> ToolCategory {
        ToolCategory::Music
    }

    fn definitions(&self) -> Vec<MediaToolDefinition> {
        music_tool_definitions()
    }

    fn invoke(
        &self,
        call: &MediaToolCall,
        auth_token: Option<&str>,
    ) -> Result<MediaToolResult, MediaToolError> {
        invoke_music_tool(call, auth_token)
    }
}

impl ToolProvider for MusicMediaToolProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            MUSIC_PROVIDER_ID,
            "tool",
            "Music media tools (cloudrouter open-api)",
            "0.1.0",
            vec![
                "tool.invoke".to_string(),
                "media.music.generate".to_string(),
            ],
        )
    }

    fn list_tools(&self) -> Vec<ToolDescriptor> {
        music_tool_definitions()
            .iter()
            .map(|definition| project_tool_descriptor(definition, MUSIC_PROVIDER_ID))
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
        music_tool_definitions()
            .into_iter()
            .find(|definition| definition.tool_id == tool_id)
            .map(|definition| project_tool_descriptor(&definition, MUSIC_PROVIDER_ID))
            .ok_or_else(|| {
                project_kernel_error(MediaToolError::CapabilityMissing(format!(
                    "music provider has no tool `{tool_id}`"
                )))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definitions::tool_ids;

    #[test]
    fn provider_exposes_all_music_tools() {
        let provider = MusicMediaToolProvider::without_auth_resolver();
        let descriptors = provider.list_tools();
        let ids: Vec<&str> = descriptors
            .iter()
            .map(|descriptor| descriptor.tool_id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec![tool_ids::GENERATIONS_CREATE, tool_ids::GENERATIONS_LIST,]
        );
        assert!(descriptors
            .iter()
            .all(|d| d.provider_id == MUSIC_PROVIDER_ID));
    }

    #[test]
    fn kernel_invoke_without_auth_token_fails_closed() {
        let provider = MusicMediaToolProvider::without_auth_resolver();
        let call = ToolCall::new(
            "call.1",
            tool_ids::GENERATIONS_CREATE,
            r#"{"prompt":"lofi"}"#,
        );
        let error = provider.invoke_tool(call).expect_err("auth required");
        assert_eq!(
            error.kind(),
            sdkwork_agent_kernel::KernelErrorKind::PermissionRequired
        );
    }

    #[test]
    fn describe_known_and_unknown_tools() {
        let provider = MusicMediaToolProvider::without_auth_resolver();
        assert!(provider.describe_tool(tool_ids::GENERATIONS_CREATE).is_ok());
        assert!(provider.describe_tool("music.missing").is_err());
    }
}
