//! The image category media tool provider.

use sdkwork_agent_kernel::{
    KernelResult, ProviderHealth, ProviderManifest, ToolCall, ToolDescriptor, ToolProvider,
    ToolResult,
};
use sdkwork_agents_tool_contract::{
    media_tool_call, project_invoke_result, project_kernel_error, project_tool_descriptor,
    MediaAuthTokenResolver, MediaToolCall, MediaToolDefinition, MediaToolError, MediaToolProvider,
    MediaToolResult, StaticMediaAuthTokenResolver, ToolCategory,
};

use crate::definitions::image_tool_definitions;
use crate::invoke::invoke_image_tool;

/// Provider id for the cloudrouter-backed image category.
pub const IMAGE_PROVIDER_ID: &str = "cloudrouter.media.image";

/// Image category tool provider (cloudrouter open-api backed).
#[derive(Debug)]
pub struct ImageMediaToolProvider {
    token_resolver: Box<dyn MediaAuthTokenResolver>,
}

impl ImageMediaToolProvider {
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

impl MediaToolProvider for ImageMediaToolProvider {
    fn category(&self) -> ToolCategory {
        ToolCategory::Image
    }

    fn definitions(&self) -> Vec<MediaToolDefinition> {
        image_tool_definitions()
    }

    fn invoke(
        &self,
        call: &MediaToolCall,
        auth_token: Option<&str>,
    ) -> Result<MediaToolResult, MediaToolError> {
        invoke_image_tool(call, auth_token)
    }
}

impl ToolProvider for ImageMediaToolProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            IMAGE_PROVIDER_ID,
            "tool",
            "Image media tools (cloudrouter open-api)",
            "0.1.0",
            vec![
                "tool.invoke".to_string(),
                "media.image.generate".to_string(),
            ],
        )
    }

    fn list_tools(&self) -> Vec<ToolDescriptor> {
        image_tool_definitions()
            .iter()
            .map(|definition| project_tool_descriptor(definition, IMAGE_PROVIDER_ID))
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
        image_tool_definitions()
            .into_iter()
            .find(|definition| definition.tool_id == tool_id)
            .map(|definition| project_tool_descriptor(&definition, IMAGE_PROVIDER_ID))
            .ok_or_else(|| {
                project_kernel_error(MediaToolError::CapabilityMissing(format!(
                    "image provider has no tool `{tool_id}`"
                )))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definitions::tool_ids;

    #[test]
    fn provider_exposes_all_image_tools() {
        let provider = ImageMediaToolProvider::without_auth_resolver();
        let descriptors = provider.list_tools();
        let ids: Vec<&str> = descriptors
            .iter()
            .map(|descriptor| descriptor.tool_id.as_str())
            .collect();
        assert_eq!(
            ids,
            vec![
                tool_ids::GENERATIONS_CREATE,
                tool_ids::EDITS_CREATE,
                tool_ids::VARIATIONS_CREATE,
                tool_ids::MIDJOURNEY_GENERATIONS_CREATE,
                tool_ids::MIDJOURNEY_GENERATIONS_LIST,
                tool_ids::NANO_BANANA_GENERATIONS_CREATE,
                tool_ids::NANO_BANANA_GENERATIONS_RETRIEVE,
                tool_ids::VIDU_REFERENCE2IMAGE,
            ]
        );
        assert!(descriptors
            .iter()
            .all(|d| d.provider_id == IMAGE_PROVIDER_ID));
    }

    #[test]
    fn kernel_invoke_without_auth_token_fails_closed() {
        let provider = ImageMediaToolProvider::without_auth_resolver();
        let call = ToolCall::new(
            "call.1",
            tool_ids::GENERATIONS_CREATE,
            r#"{"prompt":"a red fox"}"#,
        );
        let error = provider.invoke_tool(call).expect_err("auth required");
        assert_eq!(
            error.kind(),
            sdkwork_agent_kernel::KernelErrorKind::PermissionRequired
        );
    }

    #[test]
    fn describe_known_and_unknown_tools() {
        let provider = ImageMediaToolProvider::without_auth_resolver();
        assert!(provider.describe_tool(tool_ids::GENERATIONS_CREATE).is_ok());
        assert!(provider.describe_tool("image.missing").is_err());
    }
}
