//! Tool definitions for the unified generations provider.
//!
//! The generations provider wraps the sdkwork-generations service and exposes
//! image/video/music/sfx/voice generation tools under the `generations.*`
//! namespace. Unlike single-category providers, this provider spans several
//! categories; each tool definition carries its own category for dispatch and
//! authorization.

use sdkwork_agents_tool_contract::{MediaToolDefinition, ToolAvailability, ToolCategory};

/// Stable ids for every generations tool.
pub mod tool_ids {
    // Image tools
    pub const IMAGE_TEXT_TO_IMAGE: &str = "generations.image.text_to_image";
    pub const IMAGE_EDIT: &str = "generations.image.image_edit";

    // Video tools
    pub const VIDEO_TEXT_TO_VIDEO: &str = "generations.video.text_to_video";
    pub const VIDEO_IMAGE_TO_VIDEO: &str = "generations.video.image_to_video";
    pub const VIDEO_EXTEND: &str = "generations.video.video_extend";

    // Music tools
    pub const MUSIC_TEXT_TO_MUSIC: &str = "generations.music.text_to_music";
    pub const MUSIC_LYRICS_TO_MUSIC: &str = "generations.music.lyrics_to_music";

    // SFX tools
    pub const SFX_CREATE: &str = "generations.sfx.create";

    // Voice tools
    pub const VOICE_SPEECH: &str = "generations.voice.speech";
    pub const VOICE_TRANSCRIPTION: &str = "generations.voice.transcription";
    pub const VOICE_TRANSLATION: &str = "generations.voice.translation";
}

const VERSION: &str = "0.1.0";

/// Static definitions for every generations tool.
pub fn generations_tool_definitions() -> Vec<MediaToolDefinition> {
    vec![
        image_text_to_image_definition(),
        image_edit_definition(),
        video_text_to_video_definition(),
        video_image_to_video_definition(),
        video_extend_definition(),
        music_text_to_music_definition(),
        music_lyrics_to_music_definition(),
        sfx_create_definition(),
        voice_speech_definition(),
        voice_transcription_definition(),
        voice_translation_definition(),
    ]
}

// ---------------------------------------------------------------------------
// Shared output schemas
// ---------------------------------------------------------------------------

/// Synchronous generation output: one or more normalized media resources.
fn media_items_output_schema(kind: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
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
            }
        },
        "required": ["items"]
    })
}

/// Async task submission output: a `taskId` for polling with the retrieve tool.
fn task_output_schema(poll_hint: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "taskId": {
                "type": "string",
                "description": format!("Generation task id; {poll_hint}.")
            }
        },
        "required": ["taskId"]
    })
}

/// Text-only output for transcription/translation tools.
fn text_output_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "text": { "type": "string", "description": "Result text." }
        },
        "required": ["text"]
    })
}

// ---------------------------------------------------------------------------
// Image tool definitions
// ---------------------------------------------------------------------------

fn image_text_to_image_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::IMAGE_TEXT_TO_IMAGE.to_string(),
        category: ToolCategory::Image,
        name: "image.text_to_image".to_string(),
        display_name: "Generate Image".to_string(),
        version: VERSION.to_string(),
        description: "Generates one or more images from a text prompt through the \
                      sdkwork-generations image gateway."
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
        output_schema: media_items_output_schema("image"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Image.policy_category("generate")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

fn image_edit_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::IMAGE_EDIT.to_string(),
        category: ToolCategory::Image,
        name: "image.image_edit".to_string(),
        display_name: "Edit Image".to_string(),
        version: VERSION.to_string(),
        description: "Edits a source image according to a text prompt through the \
                      sdkwork-generations image edit gateway."
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
        output_schema: media_items_output_schema("image"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Image.policy_category("edit")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

// ---------------------------------------------------------------------------
// Video tool definitions
// ---------------------------------------------------------------------------

fn video_text_to_video_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VIDEO_TEXT_TO_VIDEO.to_string(),
        category: ToolCategory::Video,
        name: "video.text_to_video".to_string(),
        display_name: "Generate Video from Text".to_string(),
        version: VERSION.to_string(),
        description: "Submits a text-to-video generation task through the \
                      sdkwork-generations video gateway and returns the task id."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Text prompt describing the video to generate."
                },
                "model": {
                    "type": "string",
                    "description": "Video model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "seconds": {
                    "type": "integer",
                    "description": "Requested duration in seconds when supported."
                },
                "size": {
                    "type": "string",
                    "description": "Requested video size or resolution, e.g. 1280x720."
                }
            },
            "required": ["prompt"]
        }),
        output_schema: task_output_schema("poll with video retrieve tool"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("generate")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn video_image_to_video_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VIDEO_IMAGE_TO_VIDEO.to_string(),
        category: ToolCategory::Video,
        name: "video.image_to_video".to_string(),
        display_name: "Animate Image to Video".to_string(),
        version: VERSION.to_string(),
        description: "Submits an image-to-video generation task through the \
                      sdkwork-generations video gateway and returns the task id."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Text prompt describing the motion or scene."
                },
                "image": {
                    "type": "string",
                    "description": "Source image URL or asset reference to animate."
                },
                "model": {
                    "type": "string",
                    "description": "Video model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "seconds": {
                    "type": "integer",
                    "description": "Requested duration in seconds when supported."
                },
                "size": {
                    "type": "string",
                    "description": "Requested video size or resolution."
                }
            },
            "required": ["prompt", "image"]
        }),
        output_schema: task_output_schema("poll with video retrieve tool"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("generate")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn video_extend_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VIDEO_EXTEND.to_string(),
        category: ToolCategory::Video,
        name: "video.video_extend".to_string(),
        display_name: "Extend Video".to_string(),
        version: VERSION.to_string(),
        description: "Submits a video extension task through the sdkwork-generations \
                      video gateway and returns the task id."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Text prompt describing the continuation."
                },
                "video": {
                    "type": "string",
                    "description": "Source video URL or asset reference to extend."
                },
                "model": {
                    "type": "string",
                    "description": "Video model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "seconds": {
                    "type": "integer",
                    "description": "Requested additional duration in seconds."
                },
                "size": {
                    "type": "string",
                    "description": "Requested video size or resolution."
                }
            },
            "required": ["prompt", "video"]
        }),
        output_schema: task_output_schema("poll with video retrieve tool"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Video.policy_category("extend")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

// ---------------------------------------------------------------------------
// Music tool definitions
// ---------------------------------------------------------------------------

fn music_text_to_music_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::MUSIC_TEXT_TO_MUSIC.to_string(),
        category: ToolCategory::Music,
        name: "music.text_to_music".to_string(),
        display_name: "Generate Music from Text".to_string(),
        version: VERSION.to_string(),
        description: "Submits a text-to-music generation task through the \
                      sdkwork-generations music gateway and returns the task id."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Text description of the music to generate."
                },
                "model": {
                    "type": "string",
                    "description": "Music model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "title": {
                    "type": "string",
                    "description": "Optional song title."
                },
                "duration": {
                    "type": "number",
                    "description": "Requested duration in seconds."
                },
                "tags": {
                    "type": "string",
                    "description": "Musical style tags, comma-separated."
                },
                "negative_tags": {
                    "type": "string",
                    "description": "Musical styles to avoid."
                }
            },
            "required": ["prompt"]
        }),
        output_schema: task_output_schema("poll with music retrieve tool"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Music.policy_category("generate")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn music_lyrics_to_music_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::MUSIC_LYRICS_TO_MUSIC.to_string(),
        category: ToolCategory::Music,
        name: "music.lyrics_to_music".to_string(),
        display_name: "Generate Music from Lyrics".to_string(),
        version: VERSION.to_string(),
        description: "Submits a lyrics-to-music generation task through the \
                      sdkwork-generations music gateway and returns the task id."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "lyrics": {
                    "type": "string",
                    "description": "Lyrics to set to music."
                },
                "model": {
                    "type": "string",
                    "description": "Music model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "title": {
                    "type": "string",
                    "description": "Optional song title."
                },
                "duration": {
                    "type": "number",
                    "description": "Requested duration in seconds."
                },
                "tags": {
                    "type": "string",
                    "description": "Musical style tags, comma-separated."
                }
            },
            "required": ["lyrics"]
        }),
        output_schema: task_output_schema("poll with music retrieve tool"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Music.policy_category("create")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

// ---------------------------------------------------------------------------
// SFX tool definition
// ---------------------------------------------------------------------------

fn sfx_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::SFX_CREATE.to_string(),
        category: ToolCategory::SoundEffect,
        name: "sfx.create".to_string(),
        display_name: "Create Sound Effect".to_string(),
        version: VERSION.to_string(),
        description: "Generates a sound effect from a text prompt through the \
                      sdkwork-generations sound-effect gateway."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Text description of the sound effect to generate."
                },
                "model": {
                    "type": "string",
                    "description": "Sound-effect model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "duration": {
                    "type": "number",
                    "description": "Requested duration in seconds."
                }
            },
            "required": ["prompt"]
        }),
        output_schema: media_items_output_schema("audio"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::SoundEffect.policy_category("create")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

// ---------------------------------------------------------------------------
// Voice tool definitions
// ---------------------------------------------------------------------------

fn voice_speech_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VOICE_SPEECH.to_string(),
        category: ToolCategory::Audio,
        name: "voice.speech".to_string(),
        display_name: "Text to Speech".to_string(),
        version: VERSION.to_string(),
        description: "Synthesizes speech audio from text through the \
                      sdkwork-generations speech gateway."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Text to synthesize into speech."
                },
                "voice": {
                    "type": "string",
                    "description": "Voice identifier."
                },
                "model": {
                    "type": "string",
                    "description": "Speech model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "response_format": {
                    "type": "string",
                    "description": "Audio response format, e.g. mp3, wav, opus."
                },
                "speed": {
                    "type": "number",
                    "description": "Speech speed multiplier when supported."
                }
            },
            "required": ["input", "voice"]
        }),
        output_schema: media_items_output_schema("audio"),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Audio.policy_category("generate")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

fn voice_transcription_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VOICE_TRANSCRIPTION.to_string(),
        category: ToolCategory::Audio,
        name: "voice.transcription".to_string(),
        display_name: "Transcribe Audio".to_string(),
        version: VERSION.to_string(),
        description: "Transcribes audio to text through the sdkwork-generations \
                      audio transcription gateway."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file": {
                    "type": "object",
                    "description": "Audio file reference (url or asset reference)."
                },
                "model": {
                    "type": "string",
                    "description": "Transcription model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "language": {
                    "type": "string",
                    "description": "Optional source language hint, e.g. en, zh."
                },
                "prompt": {
                    "type": "string",
                    "description": "Optional text prompt to guide transcription."
                },
                "response_format": {
                    "type": "string",
                    "description": "Desired response format."
                }
            },
            "required": ["file"]
        }),
        output_schema: text_output_schema(),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Audio.policy_category("transcribe")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

fn voice_translation_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VOICE_TRANSLATION.to_string(),
        category: ToolCategory::Audio,
        name: "voice.translation".to_string(),
        display_name: "Translate Audio".to_string(),
        version: VERSION.to_string(),
        description: "Translates audio to text (optionally in another language) \
                      through the sdkwork-generations audio translation gateway."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file": {
                    "type": "object",
                    "description": "Audio file reference (url or asset reference)."
                },
                "model": {
                    "type": "string",
                    "description": "Translation model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "prompt": {
                    "type": "string",
                    "description": "Optional text prompt to guide translation."
                },
                "response_format": {
                    "type": "string",
                    "description": "Desired response format."
                }
            },
            "required": ["file"]
        }),
        output_schema: text_output_schema(),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Audio.policy_category("translate")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tool_has_stable_generations_id() {
        for definition in generations_tool_definitions() {
            assert!(
                definition.tool_id.starts_with("generations."),
                "{} does not start with `generations.`",
                definition.tool_id
            );
            assert!(!definition.name.is_empty());
            assert!(!definition.display_name.is_empty());
            assert!(!definition.description.is_empty());
            assert_eq!(definition.input_schema["type"], "object");
            assert_eq!(definition.output_schema["type"], "object");
            assert!(!definition.policy_categories.is_empty());
            assert!(definition.timeout_ms > 0);
            assert_eq!(definition.availability, ToolAvailability::Available);
        }
    }

    #[test]
    fn image_tools_are_registered_with_image_category() {
        let definitions = generations_tool_definitions();
        for tool_id in [tool_ids::IMAGE_TEXT_TO_IMAGE, tool_ids::IMAGE_EDIT] {
            let definition = definitions
                .iter()
                .find(|d| d.tool_id == tool_id)
                .unwrap_or_else(|| panic!("{tool_id} registered"));
            assert_eq!(definition.category, ToolCategory::Image);
            assert_eq!(definition.side_effect_level, "side_effectful");
        }
    }

    #[test]
    fn video_tools_are_registered_with_task_outputs() {
        let definitions = generations_tool_definitions();
        for tool_id in [
            tool_ids::VIDEO_TEXT_TO_VIDEO,
            tool_ids::VIDEO_IMAGE_TO_VIDEO,
            tool_ids::VIDEO_EXTEND,
        ] {
            let definition = definitions
                .iter()
                .find(|d| d.tool_id == tool_id)
                .unwrap_or_else(|| panic!("{tool_id} registered"));
            assert_eq!(definition.category, ToolCategory::Video);
            assert_eq!(definition.side_effect_level, "side_effectful");
            assert_eq!(
                definition.output_schema["properties"]["taskId"]["type"],
                "string"
            );
        }
    }

    #[test]
    fn music_tools_require_prompt_and_return_task_id() {
        let definitions = generations_tool_definitions();
        let text_to_music = definitions
            .iter()
            .find(|d| d.tool_id == tool_ids::MUSIC_TEXT_TO_MUSIC)
            .unwrap();
        assert_eq!(text_to_music.input_schema["required"], serde_json::json!(["prompt"]));

        let lyrics_to_music = definitions
            .iter()
            .find(|d| d.tool_id == tool_ids::MUSIC_LYRICS_TO_MUSIC)
            .unwrap();
        assert_eq!(lyrics_to_music.input_schema["required"], serde_json::json!(["lyrics"]));
    }

    #[test]
    fn sfx_and_voice_tools_registered_with_media_output() {
        let definitions = generations_tool_definitions();
        let sfx = definitions
            .iter()
            .find(|d| d.tool_id == tool_ids::SFX_CREATE)
            .unwrap();
        assert_eq!(sfx.category, ToolCategory::SoundEffect);
        assert_eq!(sfx.input_schema["required"], serde_json::json!(["prompt"]));

        let speech = definitions
            .iter()
            .find(|d| d.tool_id == tool_ids::VOICE_SPEECH)
            .unwrap();
        assert_eq!(speech.category, ToolCategory::Audio);
        assert_eq!(
            speech.input_schema["required"],
            serde_json::json!(["input", "voice"])
        );

        let transcription = definitions
            .iter()
            .find(|d| d.tool_id == tool_ids::VOICE_TRANSCRIPTION)
            .unwrap();
        assert_eq!(transcription.side_effect_level, "read_only");
        assert_eq!(transcription.input_schema["required"], serde_json::json!(["file"]));
        assert_eq!(transcription.output_schema["properties"]["text"]["type"], "string");

        let translation = definitions
            .iter()
            .find(|d| d.tool_id == tool_ids::VOICE_TRANSLATION)
            .unwrap();
        assert_eq!(translation.side_effect_level, "read_only");
        assert_eq!(translation.input_schema["required"], serde_json::json!(["file"]));
    }

    #[test]
    fn expected_tool_count_and_id_uniqueness() {
        let definitions = generations_tool_definitions();
        let mut ids: Vec<&str> = definitions.iter().map(|d| d.tool_id.as_str()).collect();
        let original_len = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "all tool ids must be unique");
        assert_eq!(original_len, 11);
    }
}
