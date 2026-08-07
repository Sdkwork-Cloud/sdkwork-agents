//! The video category media tool provider.

use sdkwork_agent_kernel::{
    KernelResult, ProviderHealth, ProviderManifest, ToolCall, ToolDescriptor, ToolProvider,
    ToolResult,
};
use sdkwork_agents_tool_contract::{
    media_tool_call, project_invoke_result, project_kernel_error, project_tool_descriptor,
    MediaAuthTokenResolver, MediaToolCall, MediaToolDefinition, MediaToolError, MediaToolProvider,
    MediaToolResult, StaticMediaAuthTokenResolver, ToolCategory,
};

use crate::definitions::video_tool_definitions;
use crate::invoke::invoke_video_tool;

/// Provider id for the cloudrouter-backed video category.
pub const VIDEO_PROVIDER_ID: &str = "cloudrouter.media.video";

/// Video category tool provider (cloudrouter open-api backed).
#[derive(Debug)]
pub struct VideoMediaToolProvider {
    token_resolver: Box<dyn MediaAuthTokenResolver>,
}

impl VideoMediaToolProvider {
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

impl MediaToolProvider for VideoMediaToolProvider {
    fn category(&self) -> ToolCategory {
        ToolCategory::Video
    }

    fn definitions(&self) -> Vec<MediaToolDefinition> {
        video_tool_definitions()
    }

    fn invoke(
        &self,
        call: &MediaToolCall,
        auth_token: Option<&str>,
    ) -> Result<MediaToolResult, MediaToolError> {
        invoke_video_tool(call, auth_token)
    }
}

impl ToolProvider for VideoMediaToolProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            VIDEO_PROVIDER_ID,
            "tool",
            "Video media tools (cloudrouter open-api)",
            "0.1.0",
            vec![
                "tool.invoke".to_string(),
                "media.video.generate".to_string(),
            ],
        )
    }

    fn list_tools(&self) -> Vec<ToolDescriptor> {
        video_tool_definitions()
            .iter()
            .map(|definition| project_tool_descriptor(definition, VIDEO_PROVIDER_ID))
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
        video_tool_definitions()
            .into_iter()
            .find(|definition| definition.tool_id == tool_id)
            .map(|definition| project_tool_descriptor(&definition, VIDEO_PROVIDER_ID))
            .ok_or_else(|| {
                project_kernel_error(MediaToolError::CapabilityMissing(format!(
                    "video provider has no tool `{tool_id}`"
                )))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definitions::tool_ids;

    #[test]
    fn provider_exposes_all_video_tools() {
        let provider = VideoMediaToolProvider::without_auth_resolver();
        let descriptors = provider.list_tools();
        let ids: Vec<&str> = descriptors
            .iter()
            .map(|descriptor| descriptor.tool_id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec![
                tool_ids::CREATE,
                tool_ids::RETRIEVE,
                tool_ids::LIST,
                tool_ids::EDITS_CREATE,
                tool_ids::EXTENSIONS_CREATE,
                tool_ids::REMIX_CREATE,
                tool_ids::CHARACTERS_CREATE,
                tool_ids::CHARACTERS_LIST,
                tool_ids::KLING_GENERATIONS_CREATE,
                tool_ids::KLING_GENERATIONS_RETRIEVE,
                tool_ids::VIDU_TEXT2VIDEO,
                tool_ids::VIDU_IMG2VIDEO,
                tool_ids::VIDU_REFERENCE2VIDEO,
                tool_ids::VIDU_START_END2VIDEO,
                tool_ids::VIDU_TASKS_CREATIONS,
                tool_ids::VOLCENGINE_GENERATIONS_CREATE,
                tool_ids::VOLCENGINE_GENERATIONS_RETRIEVE,
            ]
        );
        assert!(descriptors
            .iter()
            .all(|d| d.provider_id == VIDEO_PROVIDER_ID));
    }

    #[test]
    fn kernel_invoke_without_auth_token_fails_closed() {
        let provider = VideoMediaToolProvider::without_auth_resolver();
        let call = ToolCall::new("call.1", tool_ids::CREATE, r#"{"prompt":"a cat"}"#);
        let error = provider.invoke_tool(call).expect_err("auth required");
        assert_eq!(
            error.kind(),
            sdkwork_agent_kernel::KernelErrorKind::PermissionRequired
        );
    }

    #[test]
    fn describe_known_and_unknown_tools() {
        let provider = VideoMediaToolProvider::without_auth_resolver();
        assert!(provider.describe_tool(tool_ids::CREATE).is_ok());
        assert!(provider.describe_tool("video.missing").is_err());
    }
}
