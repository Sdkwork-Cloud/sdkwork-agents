//! Tool definitions for the file category.

use sdkwork_agents_tool_contract::{MediaToolDefinition, ToolAvailability, ToolCategory};

/// Stable ids for the file category tools.
pub mod tool_ids {
    pub const UPLOAD: &str = "file.upload";
    pub const LIST: &str = "file.list";
    pub const RETRIEVE: &str = "file.retrieve";
    pub const DELETE: &str = "file.delete";
    pub const CONTENT: &str = "file.content";
}

const VERSION: &str = "0.1.0";

/// Static definitions for every file tool.
pub fn file_tool_definitions() -> Vec<MediaToolDefinition> {
    vec![
        upload_definition(),
        list_definition(),
        retrieve_definition(),
        delete_definition(),
        content_definition(),
    ]
}

/// Shared file object schema used by list/retrieve outputs.
fn file_object_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "id": { "type": "string", "description": "Gateway file id usable as a media tool file reference." },
            "filename": { "type": "string" },
            "bytes": { "type": "integer" },
            "purpose": { "type": "string" },
            "status": { "type": "string" }
        },
        "required": ["id"]
    })
}

fn upload_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::UPLOAD.to_string(),
        category: ToolCategory::File,
        name: "upload".to_string(),
        display_name: "Upload File".to_string(),
        version: VERSION.to_string(),
        description: "Registers a file (URL or asset reference) on the cloudrouter \
                      gateway and returns a file id usable by the media tools \
                      (transcription, image edit, video generation)."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file": {
                    "type": "string",
                    "description": "Source URL or asset reference to register."
                },
                "purpose": {
                    "type": "string",
                    "description": "Intended purpose, e.g. fine-tune, assistants, audio.",
                    "default": "assistants"
                }
            },
            "required": ["file", "purpose"]
        }),
        output_schema: file_object_schema(),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::File.policy_category("upload")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

fn list_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::LIST.to_string(),
        category: ToolCategory::File,
        name: "list".to_string(),
        display_name: "List Files".to_string(),
        version: VERSION.to_string(),
        description: "Lists files registered on the cloudrouter gateway for the caller."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "limit": { "type": "integer", "description": "Maximum files to return." }
            }
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "files": {
                    "type": "array",
                    "items": file_object_schema()
                }
            },
            "required": ["files"]
        }),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::File.policy_category("list")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn retrieve_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::RETRIEVE.to_string(),
        category: ToolCategory::File,
        name: "retrieve".to_string(),
        display_name: "Retrieve File".to_string(),
        version: VERSION.to_string(),
        description: "Retrieves the metadata of one gateway file.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "fileId": {
                    "type": "string",
                    "description": "Gateway file id."
                }
            },
            "required": ["fileId"]
        }),
        output_schema: file_object_schema(),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::File.policy_category("retrieve")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn delete_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::DELETE.to_string(),
        category: ToolCategory::File,
        name: "delete".to_string(),
        display_name: "Delete File".to_string(),
        version: VERSION.to_string(),
        description: "Deletes one gateway file. Destructive and irreversible.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "fileId": {
                    "type": "string",
                    "description": "Gateway file id to delete."
                }
            },
            "required": ["fileId"]
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "deleted": { "type": "boolean" },
                "fileId": { "type": "string" }
            },
            "required": ["deleted", "fileId"]
        }),
        side_effect_level: "destructive".to_string(),
        policy_categories: vec![
            ToolCategory::File.policy_category("delete"),
            "host.filesystem.write".to_string(),
        ],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn content_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::CONTENT.to_string(),
        category: ToolCategory::File,
        name: "content".to_string(),
        display_name: "Get File Content".to_string(),
        version: VERSION.to_string(),
        description: "Fetches the content of one gateway file (text or encoded payload)."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "fileId": {
                    "type": "string",
                    "description": "Gateway file id."
                }
            },
            "required": ["fileId"]
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "fileId": { "type": "string" },
                "content": { "type": "string", "description": "File content payload." }
            },
            "required": ["fileId", "content"]
        }),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::File.policy_category("content")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_file_tool_has_stable_ids_and_required_fields() {
        for definition in file_tool_definitions() {
            assert!(
                definition.tool_id.starts_with("file."),
                "{}",
                definition.tool_id
            );
            assert_eq!(definition.category, ToolCategory::File);
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
    fn upload_requires_file_and_purpose() {
        let upload = file_tool_definitions()
            .into_iter()
            .find(|definition| definition.tool_id == tool_ids::UPLOAD)
            .expect("upload tool present");
        assert_eq!(upload.side_effect_level, "side_effectful");
        assert_eq!(
            upload.input_schema["required"],
            serde_json::json!(["file", "purpose"])
        );
    }

    #[test]
    fn delete_is_destructive_and_requires_file_id() {
        let delete = file_tool_definitions()
            .into_iter()
            .find(|definition| definition.tool_id == tool_ids::DELETE)
            .expect("delete tool present");
        assert_eq!(delete.side_effect_level, "destructive");
        assert_eq!(
            delete.input_schema["required"],
            serde_json::json!(["fileId"])
        );
        assert!(delete
            .policy_categories
            .contains(&"host.filesystem.write".to_string()));
    }
}
