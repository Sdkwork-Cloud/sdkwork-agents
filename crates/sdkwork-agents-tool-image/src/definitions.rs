//! Tool definitions for the image category.

use sdkwork_agents_tool_contract::{MediaToolDefinition, ToolAvailability, ToolCategory};

/// Stable ids for the image category tools.
pub mod tool_ids {
    pub const GENERATIONS_CREATE: &str = "image.generations.create";
    pub const EDITS_CREATE: &str = "image.edits.create";
    pub const VARIATIONS_CREATE: &str = "image.variations.create";
    pub const MIDJOURNEY_GENERATIONS_CREATE: &str = "image.midjourney.generations.create";
    pub const MIDJOURNEY_GENERATIONS_LIST: &str = "image.midjourney.generations.list";
    pub const NANO_BANANA_GENERATIONS_CREATE: &str = "image.nano-banana.generations.create";
    pub const NANO_BANANA_GENERATIONS_RETRIEVE: &str = "image.nano-banana.generations.retrieve";
    pub const VIDU_REFERENCE2IMAGE: &str = "image.vidu.reference2image";
}

const VERSION: &str = "0.1.0";

/// Static definitions for every image tool.
pub fn image_tool_definitions() -> Vec<MediaToolDefinition> {
    vec![
        generations_create_definition(),
        edits_create_definition(),
        variations_create_definition(),
        midjourney_generations_create_definition(),
        midjourney_generations_list_definition(),
        nano_banana_generations_create_definition(),
        nano_banana_generations_retrieve_definition(),
        vidu_reference2image_definition(),
    ]
}

/// Shared output schema: one or more normalized image resources.
fn images_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "images": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "kind": { "type": "string", "const": "image" },
                        "source": { "type": "string", "const": "provider_asset" },
                        "url": { "type": "string", "description": "Image asset delivery URL." },
                        "revisedPrompt": { "type": "string" }
                    },
                    "required": ["kind", "source", "url"]
                }
            }
        },
        "required": ["images"]
    })
}

fn generations_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::GENERATIONS_CREATE.to_string(),
        category: ToolCategory::Image,
        name: "generations.create".to_string(),
        display_name: "Generate Image".to_string(),
        version: VERSION.to_string(),
        description: "Generates one or more images from a text prompt through the \
                      cloudrouter image generation gateway."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Text prompt describing the image to generate."
                },
                "model": {
                    "type": "string",
                    "description": "Image model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "n": {
                    "type": "integer",
                    "description": "Number of images to generate when supported."
                },
                "size": {
                    "type": "string",
                    "description": "Requested image size, e.g. 1024x1024."
                },
                "quality": {
                    "type": "string",
                    "description": "Requested quality when supported."
                },
                "response_format": {
                    "type": "string",
                    "description": "Desired response format, such as url or b64_json."
                }
            },
            "required": ["prompt"]
        }),
        output_schema: images_output_schema(),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Image.policy_category("generate")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

fn edits_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::EDITS_CREATE.to_string(),
        category: ToolCategory::Image,
        name: "edits.create".to_string(),
        display_name: "Edit Image".to_string(),
        version: VERSION.to_string(),
        description: "Edits a source image according to a text prompt through the \
                      cloudrouter image edit gateway."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Text prompt describing the edit."
                },
                "image": {
                    "type": "object",
                    "description": "Source image reference (url or asset reference)."
                },
                "mask": {
                    "type": "object",
                    "description": "Optional mask image reference (url or asset reference)."
                },
                "model": {
                    "type": "string",
                    "description": "Image model id or Cloud Router catalog key.",
                    "default": "default"
                }
            },
            "required": ["prompt", "image"]
        }),
        output_schema: images_output_schema(),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Image.policy_category("edit")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

fn variations_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VARIATIONS_CREATE.to_string(),
        category: ToolCategory::Image,
        name: "variations.create".to_string(),
        display_name: "Create Image Variation".to_string(),
        version: VERSION.to_string(),
        description: "Creates variations of a source image through the cloudrouter \
                      image variation gateway."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "image": {
                    "type": "object",
                    "description": "Source image reference (url or asset reference)."
                },
                "model": {
                    "type": "string",
                    "description": "Image model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "size": {
                    "type": "string",
                    "description": "Requested image size."
                }
            },
            "required": ["image"]
        }),
        output_schema: images_output_schema(),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Image.policy_category("generate")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

/// Shared vendor task submission output: `{ "taskId": "..." }` for polling
/// with the vendor's retrieve/list tool.
fn vendor_task_output_schema(poll_tool: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "taskId": {
                "type": "string",
                "description": format!("Vendor generation task id; poll with {poll_tool}.")
            }
        },
        "required": ["taskId"]
    })
}

/// Shared vendor generated-media output: normalized resources.
fn vendor_media_output_schema(kind: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "taskId": { "type": "string" },
            "status": { "type": "string" },
            "items": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "kind": { "type": "string", "const": kind },
                        "source": { "type": "string", "const": "provider_asset" },
                        "url": { "type": "string", "description": "Asset delivery URL." },
                        "uri": { "type": "string", "description": "Asset URI when provided." }
                    },
                    "required": ["kind", "source", "url"]
                }
            },
            "error": { "type": "string" }
        },
        "required": ["taskId", "status", "items"]
    })
}

fn midjourney_generations_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::MIDJOURNEY_GENERATIONS_CREATE.to_string(),
        category: ToolCategory::Image,
        name: "midjourney.generations.create".to_string(),
        display_name: "Midjourney Generate Image".to_string(),
        version: VERSION.to_string(),
        description: "Submits a Midjourney image generation task through the \
                      cloudrouter vendor-direct surface and returns the task id."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": { "type": "string", "description": "Midjourney prompt." },
                "model": { "type": "string", "description": "Model id or catalog key.", "default": "default" },
                "aspectRatio": { "type": "string", "description": "e.g. 16:9, 1:1, 9:16." },
                "style": { "type": "string", "description": "Style reference." },
                "seed": { "type": "integer", "description": "Reproducibility seed." }
            },
            "required": ["prompt"]
        }),
        output_schema: vendor_task_output_schema("image.midjourney.generations.list"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Image.policy_category("generate")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn midjourney_generations_list_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::MIDJOURNEY_GENERATIONS_LIST.to_string(),
        category: ToolCategory::Image,
        name: "midjourney.generations.list".to_string(),
        display_name: "Retrieve Midjourney Task".to_string(),
        version: VERSION.to_string(),
        description: "Polls a Midjourney image generation task and returns generated \
                      image URLs when completed."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "taskId": { "type": "string", "description": "Task id from image.midjourney.generations.create." }
            },
            "required": ["taskId"]
        }),
        output_schema: vendor_media_output_schema("image"),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Image.policy_category("retrieve")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn nano_banana_generations_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::NANO_BANANA_GENERATIONS_CREATE.to_string(),
        category: ToolCategory::Image,
        name: "nano-banana.generations.create".to_string(),
        display_name: "Nano Banana Generate Image".to_string(),
        version: VERSION.to_string(),
        description: "Submits a Nano Banana image generation/editing task (Gemini \
                      image model) through the cloudrouter vendor-direct surface."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": { "type": "string", "description": "Image prompt or edit instruction." },
                "model": { "type": "string", "description": "Model id or catalog key.", "default": "default" },
                "images": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Source image URLs or asset references for editing."
                },
                "aspectRatio": { "type": "string" },
                "size": { "type": "string" },
                "seed": { "type": "integer" }
            },
            "required": ["prompt"]
        }),
        output_schema: vendor_task_output_schema("image.nano-banana.generations.retrieve"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Image.policy_category("generate")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn nano_banana_generations_retrieve_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::NANO_BANANA_GENERATIONS_RETRIEVE.to_string(),
        category: ToolCategory::Image,
        name: "nano-banana.generations.retrieve".to_string(),
        display_name: "Retrieve Nano Banana Task".to_string(),
        version: VERSION.to_string(),
        description: "Polls a Nano Banana image generation task and returns generated \
                      image URLs when completed."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "taskId": { "type": "string", "description": "Task id from image.nano-banana.generations.create." }
            },
            "required": ["taskId"]
        }),
        output_schema: vendor_media_output_schema("image"),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Image.policy_category("retrieve")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn vidu_reference2image_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VIDU_REFERENCE2IMAGE.to_string(),
        category: ToolCategory::Image,
        name: "vidu.reference2image".to_string(),
        display_name: "Vidu Reference to Image".to_string(),
        version: VERSION.to_string(),
        description: "Submits a Vidu reference-to-image task through the cloudrouter \
                      vendor-direct surface. Poll with video.vidu.tasks.creations."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": { "type": "string", "description": "Image description." },
                "model": { "type": "string", "description": "Model id or catalog key.", "default": "default" },
                "images": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Reference image URLs or asset references."
                },
                "aspectRatio": { "type": "string" },
                "style": { "type": "string" },
                "seed": { "type": "integer" }
            },
            "required": ["prompt", "images"]
        }),
        output_schema: vendor_task_output_schema("video.vidu.tasks.creations"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Image.policy_category("generate")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_image_tool_has_stable_ids_and_required_fields() {
        for definition in image_tool_definitions() {
            assert!(
                definition.tool_id.starts_with("image."),
                "{}",
                definition.tool_id
            );
            assert_eq!(definition.category, ToolCategory::Image);
            assert!(!definition.name.is_empty());
            assert!(!definition.display_name.is_empty());
            assert!(!definition.description.is_empty());
            assert_eq!(definition.input_schema["type"], "object");
            assert_eq!(definition.output_schema["type"], "object");
            assert!(!definition.policy_categories.is_empty());
            assert_eq!(definition.availability, ToolAvailability::Available);
        }
    }

    #[test]
    fn generation_requires_prompt_and_output_is_normalized_resources() {
        let generation = image_tool_definitions()
            .into_iter()
            .find(|definition| definition.tool_id == tool_ids::GENERATIONS_CREATE)
            .expect("generation tool present");
        assert_eq!(
            generation.input_schema["required"],
            serde_json::json!(["prompt"])
        );
        assert_eq!(
            generation.output_schema["properties"]["images"]["items"]["properties"]["kind"]
                ["const"],
            "image"
        );
        assert_eq!(generation.side_effect_level, "side_effectful");
    }

    #[test]
    fn edits_and_variations_require_image_reference() {
        let definitions = image_tool_definitions();
        let edits = definitions
            .iter()
            .find(|definition| definition.tool_id == tool_ids::EDITS_CREATE)
            .expect("edits tool present");
        assert_eq!(
            edits.input_schema["required"],
            serde_json::json!(["prompt", "image"])
        );

        let variations = definitions
            .iter()
            .find(|definition| definition.tool_id == tool_ids::VARIATIONS_CREATE)
            .expect("variations tool present");
        assert_eq!(
            variations.input_schema["required"],
            serde_json::json!(["image"])
        );
    }

    #[test]
    fn vendor_direct_tools_are_registered_with_task_outputs() {
        let definitions = image_tool_definitions();
        for tool_id in [
            tool_ids::MIDJOURNEY_GENERATIONS_CREATE,
            tool_ids::MIDJOURNEY_GENERATIONS_LIST,
            tool_ids::NANO_BANANA_GENERATIONS_CREATE,
            tool_ids::NANO_BANANA_GENERATIONS_RETRIEVE,
            tool_ids::VIDU_REFERENCE2IMAGE,
        ] {
            assert!(
                definitions
                    .iter()
                    .any(|definition| definition.tool_id == tool_id),
                "{tool_id} registered"
            );
        }

        let create = definitions
            .iter()
            .find(|definition| definition.tool_id == tool_ids::MIDJOURNEY_GENERATIONS_CREATE)
            .expect("midjourney create present");
        assert_eq!(create.side_effect_level, "side_effectful");
        assert_eq!(
            create.output_schema["properties"]["taskId"]["type"],
            "string"
        );

        let list = definitions
            .iter()
            .find(|definition| definition.tool_id == tool_ids::MIDJOURNEY_GENERATIONS_LIST)
            .expect("midjourney list present");
        assert_eq!(list.side_effect_level, "read_only");

        let vidu = definitions
            .iter()
            .find(|definition| definition.tool_id == tool_ids::VIDU_REFERENCE2IMAGE)
            .expect("vidu reference2image present");
        assert_eq!(
            vidu.input_schema["required"],
            serde_json::json!(["prompt", "images"])
        );
    }
}
