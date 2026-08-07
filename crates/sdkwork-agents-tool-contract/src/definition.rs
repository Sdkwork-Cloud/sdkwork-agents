//! Media tool definition shapes: descriptors, schemas, and availability.

use serde::{Deserialize, Serialize};

use crate::category::ToolCategory;

/// Whether a tool is callable today or reserved for an upstream capability
/// that is not yet available on the cloudrouter open-api surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolAvailability {
    /// The tool is fully implemented and callable.
    Available,
    /// The tool definition is reserved; invocation returns a capability error
    /// with `reason` until the upstream surface is opened.
    PendingCapability { reason: String },
}

/// Static, machine-readable definition of one media tool.
///
/// Mirrors the kernel `ToolDescriptor` contract (JSON Schema draft 2020-12
/// documents carried as JSON values) so a category crate can project it onto
/// `sdkwork_agent_kernel::ToolDescriptor` without loss.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaToolDefinition {
    /// Stable tool id across provider restarts, e.g. `audio.speech.create`.
    pub tool_id: String,
    /// Owning category.
    pub category: ToolCategory,
    /// Machine-friendly name, e.g. `speech.create`.
    pub name: String,
    /// Human-readable name, e.g. `Text to Speech`.
    pub display_name: String,
    /// Tool version.
    pub version: String,
    /// Safe summary describing behaviour and side effects.
    pub description: String,
    /// JSON Schema draft 2020-12 input schema.
    pub input_schema: serde_json::Value,
    /// JSON Schema draft 2020-12 output schema.
    pub output_schema: serde_json::Value,
    /// Side-effect classification projected onto the kernel enum.
    pub side_effect_level: String,
    /// Policy categories driving authorization, e.g. `media.audio.generate`.
    pub policy_categories: Vec<String>,
    /// Default timeout for a synchronous invocation, in milliseconds.
    pub timeout_ms: u64,
    /// Whether the tool is callable today.
    pub availability: ToolAvailability,
}

impl MediaToolDefinition {
    /// Builds an input schema for tools that submit an async generation task
    /// (`task_id` returned; a poll/retrieve tool follows up).
    pub fn async_task_output_schema(task_id_description: &str) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "taskId": { "type": "string", "description": task_id_description }
            },
            "required": ["taskId"]
        })
    }
}

/// Normalized media output shape aligned with `MEDIA_RESOURCE_SPEC`:
/// business surfaces carry `kind`/`source`/`url` instead of raw vendor wire
/// fields such as `image_url`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaResource {
    /// Resource kind, e.g. `image`, `video`, `audio`, `music`.
    pub kind: String,
    /// Source origin: `provider_asset` (cloudrouter gateway) or
    /// `external_url`.
    pub source: String,
    /// Delivery URL of the generated asset.
    pub url: String,
    /// Asynchronous task id when the asset was produced by a task API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// MIME type of the generated asset when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Original file name of the generated asset when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

impl MediaResource {
    pub fn provider_asset(kind: &str, url: impl Into<String>) -> Self {
        Self {
            kind: kind.to_string(),
            source: "provider_asset".to_string(),
            url: url.into(),
            task_id: None,
            mime_type: None,
            file_name: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_asset_uses_expected_normalization() {
        let resource = MediaResource::provider_asset("image", "https://cdn.example/a.png");
        assert_eq!(resource.kind, "image");
        assert_eq!(resource.source, "provider_asset");
        assert_eq!(resource.url, "https://cdn.example/a.png");
        assert_eq!(resource.task_id, None);
    }

    #[test]
    fn resource_serializes_camel_case_without_empty_task_id() {
        let resource = MediaResource::provider_asset("audio", "https://cdn.example/a.mp3");
        let json = serde_json::to_value(&resource).expect("serializable");
        assert_eq!(
            json,
            serde_json::json!({
                "kind": "audio",
                "source": "provider_asset",
                "url": "https://cdn.example/a.mp3"
            })
        );
    }
}
