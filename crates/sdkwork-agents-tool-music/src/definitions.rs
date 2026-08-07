//! Tool definitions for the music category.

use sdkwork_agents_tool_contract::{MediaToolDefinition, ToolAvailability, ToolCategory};

/// Stable ids for the music category tools.
pub mod tool_ids {
    pub const GENERATIONS_CREATE: &str = "music.generations.create";
    pub const GENERATIONS_LIST: &str = "music.generations.list";
}

const VERSION: &str = "0.1.0";

/// Static definitions for every music tool.
pub fn music_tool_definitions() -> Vec<MediaToolDefinition> {
    vec![
        generations_create_definition(),
        generations_list_definition(),
    ]
}

fn generations_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::GENERATIONS_CREATE.to_string(),
        category: ToolCategory::Music,
        name: "generations.create".to_string(),
        display_name: "Generate Music".to_string(),
        version: VERSION.to_string(),
        description: "Submits a music generation task (Suno-compatible) through the \
                      cloudrouter gateway and returns the task id for polling."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Music style/lyrics prompt for the generation task."
                },
                "model": {
                    "type": "string",
                    "description": "Music model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "title": {
                    "type": "string",
                    "description": "Optional track title."
                },
                "tags": {
                    "type": "string",
                    "description": "Optional genre/style tags (comma separated)."
                },
                "duration": {
                    "type": "number",
                    "description": "Requested duration in seconds."
                },
                "negativeTags": {
                    "type": "string",
                    "description": "Optional negative style tags."
                }
            },
            "required": ["prompt"]
        }),
        output_schema: MediaToolDefinition::async_task_output_schema(
            "Music generation task id; poll with music.generations.list.",
        ),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Music.policy_category("generate")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn generations_list_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::GENERATIONS_LIST.to_string(),
        category: ToolCategory::Music,
        name: "generations.list".to_string(),
        display_name: "Retrieve Music Task".to_string(),
        version: VERSION.to_string(),
        description: "Polls a music generation task and returns its status and audio \
                      track URLs when completed."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "taskId": {
                    "type": "string",
                    "description": "Task id returned by music.generations.create."
                }
            },
            "required": ["taskId"]
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "taskId": { "type": "string" },
                "status": {
                    "type": "string",
                    "enum": ["queued", "processing", "completed", "failed"]
                },
                "tracks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "kind": { "type": "string", "const": "music" },
                            "source": { "type": "string", "const": "provider_asset" },
                            "url": { "type": "string", "description": "Audio track URL." },
                            "title": { "type": "string" },
                            "duration": { "type": "number" }
                        }
                    }
                },
                "error": { "type": "string" }
            },
            "required": ["taskId", "status"]
        }),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Music.policy_category("retrieve")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_music_tool_has_stable_ids_and_required_fields() {
        for definition in music_tool_definitions() {
            assert!(
                definition.tool_id.starts_with("music."),
                "{}",
                definition.tool_id
            );
            assert_eq!(definition.category, ToolCategory::Music);
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
    fn create_is_generative_and_list_is_read_only() {
        let definitions = music_tool_definitions();
        let create = definitions
            .iter()
            .find(|definition| definition.tool_id == tool_ids::GENERATIONS_CREATE)
            .expect("create tool present");
        let list = definitions
            .iter()
            .find(|definition| definition.tool_id == tool_ids::GENERATIONS_LIST)
            .expect("list tool present");

        assert_eq!(create.side_effect_level, "side_effectful");
        assert_eq!(
            create.input_schema["required"],
            serde_json::json!(["prompt"])
        );
        assert_eq!(list.side_effect_level, "read_only");
        assert_eq!(list.input_schema["required"], serde_json::json!(["taskId"]));
    }
}
