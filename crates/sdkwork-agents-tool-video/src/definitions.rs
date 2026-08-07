//! Tool definitions for the video category.

use sdkwork_agents_tool_contract::{MediaToolDefinition, ToolAvailability, ToolCategory};

/// Stable ids for the video category tools.
pub mod tool_ids {
    pub const CREATE: &str = "video.create";
    pub const RETRIEVE: &str = "video.retrieve";
    pub const LIST: &str = "video.list";
    pub const EDITS_CREATE: &str = "video.edits.create";
    pub const EXTENSIONS_CREATE: &str = "video.extensions.create";
    pub const REMIX_CREATE: &str = "video.remix.create";
    pub const CHARACTERS_CREATE: &str = "video.characters.create";
    pub const CHARACTERS_LIST: &str = "video.characters.list";
    pub const KLING_GENERATIONS_CREATE: &str = "video.kling.generations.create";
    pub const KLING_GENERATIONS_RETRIEVE: &str = "video.kling.generations.retrieve";
    pub const VIDU_TEXT2VIDEO: &str = "video.vidu.text2video";
    pub const VIDU_IMG2VIDEO: &str = "video.vidu.img2video";
    pub const VIDU_REFERENCE2VIDEO: &str = "video.vidu.reference2video";
    pub const VIDU_START_END2VIDEO: &str = "video.vidu.start-end2video";
    pub const VIDU_TASKS_CREATIONS: &str = "video.vidu.tasks.creations";
    pub const VOLCENGINE_GENERATIONS_CREATE: &str = "video.volcengine.generations.create";
    pub const VOLCENGINE_GENERATIONS_RETRIEVE: &str = "video.volcengine.generations.retrieve";
}

const VERSION: &str = "0.1.0";

/// Static definitions for every video tool.
pub fn video_tool_definitions() -> Vec<MediaToolDefinition> {
    vec![
        create_definition(),
        retrieve_definition(),
        list_definition(),
        edits_create_definition(),
        extensions_create_definition(),
        remix_create_definition(),
        characters_create_definition(),
        characters_list_definition(),
        kling_generations_create_definition(),
        kling_generations_retrieve_definition(),
        vidu_text2video_definition(),
        vidu_img2video_definition(),
        vidu_reference2video_definition(),
        vidu_start_end2video_definition(),
        vidu_tasks_creations_definition(),
        volcengine_generations_create_definition(),
        volcengine_generations_retrieve_definition(),
    ]
}

/// Shared generation-style input schema (prompt + optional image/video seed).
fn generation_input_schema(required: &[&str]) -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "prompt": {
                "type": "string",
                "description": "Instruction for the video generation task."
            },
            "model": {
                "type": "string",
                "description": "Video model id or Cloud Router catalog key.",
                "default": "default"
            },
            "image": {
                "type": "string",
                "description": "Source image URL or asset reference for image-to-video."
            },
            "video": {
                "type": "string",
                "description": "Source video URL or asset reference for video-to-video."
            },
            "seconds": {
                "type": "integer",
                "description": "Target video duration in seconds."
            },
            "size": {
                "type": "string",
                "description": "Requested resolution, e.g. 1920x1080."
            }
        },
        "required": required
    })
}

fn create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::CREATE.to_string(),
        category: ToolCategory::Video,
        name: "create".to_string(),
        display_name: "Generate Video".to_string(),
        version: VERSION.to_string(),
        description: "Submits a text-to-video or image-to-video generation task through \
                      the cloudrouter gateway and returns the task id for polling."
            .to_string(),
        input_schema: generation_input_schema(&["prompt"]),
        output_schema: MediaToolDefinition::async_task_output_schema(
            "Video generation task id; poll with video.retrieve.",
        ),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("generate")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn retrieve_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::RETRIEVE.to_string(),
        category: ToolCategory::Video,
        name: "retrieve".to_string(),
        display_name: "Retrieve Video Task".to_string(),
        version: VERSION.to_string(),
        description: "Polls a video generation task and returns its status and asset URL \
                      when completed."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "videoId": {
                    "type": "string",
                    "description": "Task id returned by video.create or video.edits.create."
                }
            },
            "required": ["videoId"]
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "taskId": { "type": "string" },
                "status": {
                    "type": "string",
                    "enum": ["queued", "processing", "completed", "failed"]
                },
                "kind": { "type": "string", "const": "video" },
                "source": { "type": "string", "const": "provider_asset" },
                "url": { "type": "string", "description": "Video asset URL when completed." },
                "contentUrl": { "type": "string", "description": "Playback content URL." },
                "error": { "type": "string" }
            },
            "required": ["taskId", "status"]
        }),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("retrieve")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn list_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::LIST.to_string(),
        category: ToolCategory::Video,
        name: "list".to_string(),
        display_name: "List Videos".to_string(),
        version: VERSION.to_string(),
        description: "Lists videos available on the cloudrouter gateway for the caller."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "limit": { "type": "integer", "description": "Maximum videos to return." }
            }
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "videos": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "status": { "type": "string" },
                            "url": { "type": "string" }
                        }
                    }
                }
            },
            "required": ["videos"]
        }),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("list")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn edits_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::EDITS_CREATE.to_string(),
        category: ToolCategory::Video,
        name: "edits.create".to_string(),
        display_name: "Edit Video".to_string(),
        version: VERSION.to_string(),
        description: "Submits a video editing task (prompt-driven changes to a source \
                      video) and returns the task id for polling."
            .to_string(),
        input_schema: generation_input_schema(&["prompt", "video"]),
        output_schema: MediaToolDefinition::async_task_output_schema(
            "Video edit task id; poll with video.retrieve.",
        ),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("edit")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn extensions_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::EXTENSIONS_CREATE.to_string(),
        category: ToolCategory::Video,
        name: "extensions.create".to_string(),
        display_name: "Extend Video".to_string(),
        version: VERSION.to_string(),
        description: "Submits a video extension task (lengthens a source video) and \
                      returns the task id for polling."
            .to_string(),
        input_schema: generation_input_schema(&["prompt", "video"]),
        output_schema: MediaToolDefinition::async_task_output_schema(
            "Video extension task id; poll with video.retrieve.",
        ),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("extend")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn remix_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::REMIX_CREATE.to_string(),
        category: ToolCategory::Video,
        name: "remix.create".to_string(),
        display_name: "Remix Video".to_string(),
        version: VERSION.to_string(),
        description: "Submits a video remix task for an existing video and returns the \
                      task id for polling."
            .to_string(),
        input_schema: generation_input_schema(&["prompt"]),
        output_schema: MediaToolDefinition::async_task_output_schema(
            "Video remix task id; poll with video.retrieve.",
        ),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("remix")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn characters_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::CHARACTERS_CREATE.to_string(),
        category: ToolCategory::Video,
        name: "characters.create".to_string(),
        display_name: "Create Video Character".to_string(),
        version: VERSION.to_string(),
        description: "Registers a reusable character (name, description, reference \
                      image) for video generation."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Character name." },
                "description": { "type": "string", "description": "Character description." },
                "image": {
                    "type": "string",
                    "description": "Reference image URL or asset reference."
                }
            },
            "required": ["name"]
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "characterId": { "type": "string", "description": "Created character id." },
                "status": { "type": "string" }
            },
            "required": ["characterId"]
        }),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("character")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn characters_list_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::CHARACTERS_LIST.to_string(),
        category: ToolCategory::Video,
        name: "characters.list".to_string(),
        display_name: "Get Video Character".to_string(),
        version: VERSION.to_string(),
        description: "Returns the details of a registered video character.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "characterId": {
                    "type": "string",
                    "description": "Character id from video.characters.create."
                }
            },
            "required": ["characterId"]
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "characterId": { "type": "string" },
                "name": { "type": "string" },
                "description": { "type": "string" },
                "status": { "type": "string" }
            },
            "required": ["characterId"]
        }),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("character")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

/// Shared vendor task submission output: `{ "taskId": "..." }` for polling
/// with the vendor's retrieve tool.
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

fn kling_generations_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::KLING_GENERATIONS_CREATE.to_string(),
        category: ToolCategory::Video,
        name: "kling.generations.create".to_string(),
        display_name: "Kling Generate Video".to_string(),
        version: VERSION.to_string(),
        description: "Submits a Kling video generation task through the cloudrouter \
                      vendor-direct surface and returns the task id."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": { "type": "string", "description": "Kling video prompt." },
                "model": { "type": "string", "description": "Model id or catalog key.", "default": "default" },
                "image": { "type": "string", "description": "Source image URL or asset reference (image-to-video)." },
                "imageTail": { "type": "string", "description": "Trailing frame image reference." },
                "duration": { "type": "integer", "description": "Requested duration in seconds." },
                "aspectRatio": { "type": "string" },
                "mode": { "type": "string" },
                "cfgScale": { "type": "number" },
                "negativePrompt": { "type": "string" }
            },
            "required": ["prompt"]
        }),
        output_schema: vendor_task_output_schema("video.kling.generations.retrieve"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("generate")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn kling_generations_retrieve_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::KLING_GENERATIONS_RETRIEVE.to_string(),
        category: ToolCategory::Video,
        name: "kling.generations.retrieve".to_string(),
        display_name: "Retrieve Kling Task".to_string(),
        version: VERSION.to_string(),
        description: "Polls a Kling video generation task and returns video asset URLs \
                      when completed."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "taskId": { "type": "string", "description": "Task id from video.kling.generations.create." }
            },
            "required": ["taskId"]
        }),
        output_schema: vendor_media_output_schema("video"),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("retrieve")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn vidu_text2video_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VIDU_TEXT2VIDEO.to_string(),
        category: ToolCategory::Video,
        name: "vidu.text2video".to_string(),
        display_name: "Vidu Text to Video".to_string(),
        version: VERSION.to_string(),
        description: "Submits a Vidu text-to-video task through the cloudrouter \
                      vendor-direct surface."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": { "type": "string", "description": "Video description." },
                "model": { "type": "string", "description": "Model id or catalog key.", "default": "default" },
                "duration": { "type": "integer" },
                "aspectRatio": { "type": "string" },
                "resolution": { "type": "string" },
                "movementAmplitude": { "type": "string" },
                "seed": { "type": "integer" }
            },
            "required": ["prompt"]
        }),
        output_schema: vendor_task_output_schema("video.vidu.tasks.creations"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("generate")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn vidu_img2video_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VIDU_IMG2VIDEO.to_string(),
        category: ToolCategory::Video,
        name: "vidu.img2video".to_string(),
        display_name: "Vidu Image to Video".to_string(),
        version: VERSION.to_string(),
        description: "Submits a Vidu image-to-video task through the cloudrouter \
                      vendor-direct surface."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "images": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Source image URLs or asset references."
                },
                "prompt": { "type": "string" },
                "model": { "type": "string", "description": "Model id or catalog key.", "default": "default" },
                "duration": { "type": "integer" },
                "aspectRatio": { "type": "string" },
                "resolution": { "type": "string" },
                "movementAmplitude": { "type": "string" },
                "seed": { "type": "integer" }
            },
            "required": ["images"]
        }),
        output_schema: vendor_task_output_schema("video.vidu.tasks.creations"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("generate")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn vidu_reference2video_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VIDU_REFERENCE2VIDEO.to_string(),
        category: ToolCategory::Video,
        name: "vidu.reference2video".to_string(),
        display_name: "Vidu Reference to Video".to_string(),
        version: VERSION.to_string(),
        description: "Submits a Vidu reference-to-video task (reference images + prompt) \
                      through the cloudrouter vendor-direct surface."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "images": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Reference image URLs or asset references."
                },
                "prompt": { "type": "string" },
                "model": { "type": "string", "description": "Model id or catalog key.", "default": "default" },
                "duration": { "type": "integer" },
                "aspectRatio": { "type": "string" },
                "resolution": { "type": "string" },
                "movementAmplitude": { "type": "string" },
                "seed": { "type": "integer" }
            },
            "required": ["images"]
        }),
        output_schema: vendor_task_output_schema("video.vidu.tasks.creations"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("generate")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn vidu_start_end2video_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VIDU_START_END2VIDEO.to_string(),
        category: ToolCategory::Video,
        name: "vidu.start-end2video".to_string(),
        display_name: "Vidu Start-End to Video".to_string(),
        version: VERSION.to_string(),
        description: "Submits a Vidu start-and-end-frame-to-video task through the \
                      cloudrouter vendor-direct surface."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "images": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Start/end frame image URLs (two entries)."
                },
                "prompt": { "type": "string" },
                "model": { "type": "string", "description": "Model id or catalog key.", "default": "default" },
                "duration": { "type": "integer" },
                "aspectRatio": { "type": "string" },
                "resolution": { "type": "string" },
                "movementAmplitude": { "type": "string" },
                "seed": { "type": "integer" }
            },
            "required": ["images"]
        }),
        output_schema: vendor_task_output_schema("video.vidu.tasks.creations"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("generate")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn vidu_tasks_creations_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VIDU_TASKS_CREATIONS.to_string(),
        category: ToolCategory::Video,
        name: "vidu.tasks.creations".to_string(),
        display_name: "Retrieve Vidu Task".to_string(),
        version: VERSION.to_string(),
        description: "Polls a Vidu task and returns its creations (video/image/audio \
                      asset URLs) when completed. Polls every vidu.* submission."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "taskId": { "type": "string", "description": "Task id from any vidu.* tool." }
            },
            "required": ["taskId"]
        }),
        output_schema: vendor_media_output_schema("video"),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("retrieve")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn volcengine_generations_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VOLCENGINE_GENERATIONS_CREATE.to_string(),
        category: ToolCategory::Video,
        name: "volcengine.generations.create".to_string(),
        display_name: "Volcengine Generate Video".to_string(),
        version: VERSION.to_string(),
        description: "Submits a Volcengine Ark content generation task through the \
                      cloudrouter vendor-direct surface and returns the task id."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": { "type": "string", "description": "Content generation prompt." },
                "model": { "type": "string", "description": "Model id or catalog key.", "default": "default" },
                "imageUrl": { "type": "string", "description": "Optional reference image URL." }
            },
            "required": ["prompt"]
        }),
        output_schema: vendor_task_output_schema("video.volcengine.generations.retrieve"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("generate")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn volcengine_generations_retrieve_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VOLCENGINE_GENERATIONS_RETRIEVE.to_string(),
        category: ToolCategory::Video,
        name: "volcengine.generations.retrieve".to_string(),
        display_name: "Retrieve Volcengine Task".to_string(),
        version: VERSION.to_string(),
        description: "Polls a Volcengine content generation task and returns video asset \
                      URLs when completed."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "taskId": { "type": "string", "description": "Task id from video.volcengine.generations.create." }
            },
            "required": ["taskId"]
        }),
        output_schema: vendor_media_output_schema("video"),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("retrieve")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_video_tool_has_stable_ids_and_required_fields() {
        for definition in video_tool_definitions() {
            assert!(
                definition.tool_id.starts_with("video."),
                "{}",
                definition.tool_id
            );
            assert_eq!(definition.category, ToolCategory::Video);
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
    fn async_task_tools_declare_task_id_output() {
        for tool_id in [
            tool_ids::CREATE,
            tool_ids::EDITS_CREATE,
            tool_ids::EXTENSIONS_CREATE,
            tool_ids::REMIX_CREATE,
        ] {
            let definition = video_tool_definitions()
                .into_iter()
                .find(|definition| definition.tool_id == tool_id)
                .expect("tool present");
            assert_eq!(
                definition.output_schema["properties"]["taskId"]["type"], "string",
                "{tool_id}"
            );
        }
    }

    #[test]
    fn create_requires_prompt_and_is_generative() {
        let create = video_tool_definitions()
            .into_iter()
            .find(|definition| definition.tool_id == tool_ids::CREATE)
            .expect("create tool present");
        assert_eq!(create.side_effect_level, "side_effectful");
        assert_eq!(
            create.input_schema["required"],
            serde_json::json!(["prompt"])
        );
    }

    #[test]
    fn vendor_direct_tools_are_registered_with_task_outputs() {
        let definitions = video_tool_definitions();
        let vendor_tools = [
            tool_ids::KLING_GENERATIONS_CREATE,
            tool_ids::KLING_GENERATIONS_RETRIEVE,
            tool_ids::VIDU_TEXT2VIDEO,
            tool_ids::VIDU_IMG2VIDEO,
            tool_ids::VIDU_REFERENCE2VIDEO,
            tool_ids::VIDU_START_END2VIDEO,
            tool_ids::VIDU_TASKS_CREATIONS,
            tool_ids::VOLCENGINE_GENERATIONS_CREATE,
            tool_ids::VOLCENGINE_GENERATIONS_RETRIEVE,
        ];
        for tool_id in vendor_tools {
            assert!(
                definitions
                    .iter()
                    .any(|definition| definition.tool_id == tool_id),
                "{tool_id} registered"
            );
        }

        // Submission tools are side_effectful and return taskId; poll tools
        // are read_only with normalized items output.
        let kling_create = definitions
            .iter()
            .find(|definition| definition.tool_id == tool_ids::KLING_GENERATIONS_CREATE)
            .expect("kling create present");
        assert_eq!(kling_create.side_effect_level, "side_effectful");
        assert_eq!(
            kling_create.output_schema["properties"]["taskId"]["type"],
            "string"
        );

        let kling_retrieve = definitions
            .iter()
            .find(|definition| definition.tool_id == tool_ids::KLING_GENERATIONS_RETRIEVE)
            .expect("kling retrieve present");
        assert_eq!(kling_retrieve.side_effect_level, "read_only");
        assert_eq!(
            kling_retrieve.output_schema["properties"]["items"]["items"]["properties"]["kind"]
                ["const"],
            "video"
        );

        let vidu_img = definitions
            .iter()
            .find(|definition| definition.tool_id == tool_ids::VIDU_IMG2VIDEO)
            .expect("vidu img2video present");
        assert_eq!(
            vidu_img.input_schema["required"],
            serde_json::json!(["images"])
        );
    }
}
