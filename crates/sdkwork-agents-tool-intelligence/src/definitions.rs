//! Tool definitions for the intelligence category.

use sdkwork_agents_tool_contract::{MediaToolDefinition, ToolAvailability, ToolCategory};

/// Stable ids for the intelligence category tools.
pub mod tool_ids {
    pub const MODEL_LIST: &str = "model.list";
    pub const EMBEDDING_CREATE: &str = "embedding.create";
    pub const MODERATION_CREATE: &str = "moderation.create";
}

const VERSION: &str = "0.1.0";

/// Static definitions for every intelligence tool.
pub fn intelligence_tool_definitions() -> Vec<MediaToolDefinition> {
    vec![
        model_list_definition(),
        embedding_create_definition(),
        moderation_create_definition(),
    ]
}

fn model_list_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::MODEL_LIST.to_string(),
        category: ToolCategory::Intelligence,
        name: "model.list".to_string(),
        display_name: "List Models".to_string(),
        version: VERSION.to_string(),
        description: "Lists models available to the caller's cloudrouter account pool, \
                      letting agents discover usable model ids instead of hard-coding \
                      `default`."
            .to_string(),
        input_schema: serde_json::json!({ "type": "object" }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "models": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string", "description": "Model id usable in media tool `model` arguments." },
                            "ownedBy": { "type": "string" },
                            "created": { "type": "integer" }
                        },
                        "required": ["id"]
                    }
                }
            },
            "required": ["models"]
        }),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Intelligence.policy_category("list")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn embedding_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::EMBEDDING_CREATE.to_string(),
        category: ToolCategory::Intelligence,
        name: "embedding.create".to_string(),
        display_name: "Create Embedding".to_string(),
        version: VERSION.to_string(),
        description: "Vectorizes input text through the cloudrouter embedding gateway.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Text to embed."
                },
                "model": {
                    "type": "string",
                    "description": "Embedding model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "dimensions": {
                    "type": "integer",
                    "description": "Requested embedding dimensions when supported."
                },
                "encoding_format": {
                    "type": "string",
                    "description": "Desired encoding format (float, base64, ...)."
                }
            },
            "required": ["input"]
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "embeddings": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "index": { "type": "integer" },
                            "embedding": { "type": "array", "items": { "type": "number" } }
                        },
                        "required": ["index", "embedding"]
                    }
                },
                "model": { "type": "string" }
            },
            "required": ["embeddings"]
        }),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Intelligence.policy_category("embed")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

fn moderation_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::MODERATION_CREATE.to_string(),
        category: ToolCategory::Intelligence,
        name: "moderation.create".to_string(),
        display_name: "Moderate Content".to_string(),
        version: VERSION.to_string(),
        description: "Scores content against safety categories through the cloudrouter \
                      moderation gateway (flagged / category scores)."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Text to moderate."
                },
                "model": {
                    "type": "string",
                    "description": "Moderation model id or Cloud Router catalog key.",
                    "default": "default"
                }
            },
            "required": ["input"]
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "model": { "type": "string" },
                "results": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "flagged": { "type": "boolean" },
                            "categories": { "type": "object" },
                            "categoryScores": { "type": "object" }
                        },
                        "required": ["flagged"]
                    }
                }
            },
            "required": ["results"]
        }),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Intelligence.policy_category("moderate")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_intelligence_tool_has_stable_ids_and_required_fields() {
        for definition in intelligence_tool_definitions() {
            assert!(
                definition.tool_id.starts_with("model.")
                    || definition.tool_id.starts_with("embedding.")
                    || definition.tool_id.starts_with("moderation."),
                "{}",
                definition.tool_id
            );
            assert_eq!(definition.category, ToolCategory::Intelligence);
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
    fn all_intelligence_tools_are_read_only() {
        for definition in intelligence_tool_definitions() {
            assert_eq!(
                definition.side_effect_level, "read_only",
                "{}",
                definition.tool_id
            );
        }
    }

    #[test]
    fn embedding_and_moderation_require_input() {
        let definitions = intelligence_tool_definitions();
        let embedding = definitions
            .iter()
            .find(|definition| definition.tool_id == tool_ids::EMBEDDING_CREATE)
            .expect("embedding tool present");
        assert_eq!(
            embedding.input_schema["required"],
            serde_json::json!(["input"])
        );

        let moderation = definitions
            .iter()
            .find(|definition| definition.tool_id == tool_ids::MODERATION_CREATE)
            .expect("moderation tool present");
        assert_eq!(
            moderation.input_schema["required"],
            serde_json::json!(["input"])
        );
    }
}
