//! Tool definitions for the audio category.

use sdkwork_agents_tool_contract::{MediaToolDefinition, ToolAvailability, ToolCategory};

/// Stable ids for the audio category tools.
pub mod tool_ids {
    pub const SPEECH_CREATE: &str = "audio.speech.create";
    pub const TRANSCRIPTIONS_CREATE: &str = "audio.transcriptions.create";
    pub const TRANSLATIONS_CREATE: &str = "audio.translations.create";
    pub const VOICES_LIST: &str = "audio.voices.list";
    pub const VOICES_CREATE: &str = "audio.voices.create";
    pub const VOICE_CONSENTS_CREATE: &str = "audio.voice_consents.create";
    pub const VOICE_CONSENTS_LIST: &str = "audio.voice_consents.list";
}

const VERSION: &str = "0.1.0";

/// Static definitions for every audio tool.
pub fn audio_tool_definitions() -> Vec<MediaToolDefinition> {
    vec![
        speech_create_definition(),
        transcriptions_create_definition(),
        translations_create_definition(),
        voices_list_definition(),
        voices_create_definition(),
        voice_consents_create_definition(),
        voice_consents_list_definition(),
    ]
}

fn speech_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::SPEECH_CREATE.to_string(),
        category: ToolCategory::Audio,
        name: "speech.create".to_string(),
        display_name: "Text to Speech".to_string(),
        version: VERSION.to_string(),
        description: "Synthesizes speech audio from input text through the cloudrouter \
                      text-to-speech gateway, returning an audio asset URL."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Text to synthesize."
                },
                "model": {
                    "type": "string",
                    "description": "Audio model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "voice": {
                    "type": "string",
                    "description": "Voice identifier used for speech generation.",
                    "default": "alloy"
                },
                "speed": {
                    "type": "number",
                    "description": "Speech speed multiplier when supported."
                },
                "response_format": {
                    "type": "string",
                    "description": "Requested audio response format (mp3, wav, ...)."
                }
            },
            "required": ["input"]
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "kind": { "type": "string", "const": "audio" },
                "source": { "type": "string", "const": "provider_asset" },
                "url": { "type": "string", "description": "Audio asset delivery URL." }
            },
            "required": ["kind", "source", "url"]
        }),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Audio.policy_category("generate")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

fn transcriptions_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::TRANSCRIPTIONS_CREATE.to_string(),
        category: ToolCategory::Audio,
        name: "transcriptions.create".to_string(),
        display_name: "Audio Transcription".to_string(),
        version: VERSION.to_string(),
        description: "Transcribes audio (URL or file reference) into text through the \
                      cloudrouter speech-to-text gateway."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file": {
                    "type": "object",
                    "description": "Audio source: provider file id or URL reference."
                },
                "model": {
                    "type": "string",
                    "description": "Transcription model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "language": {
                    "type": "string",
                    "description": "ISO-639-1 language hint when supported."
                },
                "prompt": {
                    "type": "string",
                    "description": "Optional transcription prompt."
                },
                "response_format": {
                    "type": "string",
                    "description": "Requested response format."
                }
            },
            "required": ["file", "model"]
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "text": { "type": "string", "description": "Transcribed text." },
                "language": { "type": "string" },
                "duration": { "type": "number" }
            },
            "required": ["text"]
        }),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Audio.policy_category("transcribe")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

fn translations_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::TRANSLATIONS_CREATE.to_string(),
        category: ToolCategory::Audio,
        name: "translations.create".to_string(),
        display_name: "Audio Translation".to_string(),
        version: VERSION.to_string(),
        description: "Translates audio speech into English text through the cloudrouter \
                      audio translation gateway."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "file": {
                    "type": "object",
                    "description": "Audio source: provider file id or URL reference."
                },
                "model": {
                    "type": "string",
                    "description": "Translation model id or Cloud Router catalog key.",
                    "default": "default"
                },
                "prompt": {
                    "type": "string",
                    "description": "Optional translation prompt."
                },
                "response_format": {
                    "type": "string",
                    "description": "Requested response format."
                }
            },
            "required": ["file", "model"]
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "text": { "type": "string", "description": "Translated text." },
                "language": { "type": "string" },
                "duration": { "type": "number" }
            },
            "required": ["text"]
        }),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Audio.policy_category("translate")],
        timeout_ms: 60_000,
        availability: ToolAvailability::Available,
    }
}

fn voices_list_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VOICES_LIST.to_string(),
        category: ToolCategory::Audio,
        name: "voices.list".to_string(),
        display_name: "List Voices".to_string(),
        version: VERSION.to_string(),
        description: "Lists voices available for speech synthesis on the cloudrouter \
                      audio gateway."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "limit": { "type": "integer", "description": "Maximum voices to return." }
            }
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "voices": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "name": { "type": "string" },
                            "status": { "type": "string" }
                        }
                    }
                }
            },
            "required": ["voices"]
        }),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Audio.policy_category("list")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn voices_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VOICES_CREATE.to_string(),
        category: ToolCategory::Audio,
        name: "voices.create".to_string(),
        display_name: "Create Voice".to_string(),
        version: VERSION.to_string(),
        description: "Registers a custom voice on the cloudrouter audio gateway for use \
                      in speech synthesis."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Voice name."
                },
                "description": {
                    "type": "string",
                    "description": "Voice description."
                }
            },
            "required": ["name"]
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Created voice id." },
                "name": { "type": "string" },
                "status": { "type": "string" }
            },
            "required": ["id"]
        }),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Audio.policy_category("voice")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn voice_consents_create_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VOICE_CONSENTS_CREATE.to_string(),
        category: ToolCategory::Audio,
        name: "voice_consents.create".to_string(),
        display_name: "Create Voice Consent".to_string(),
        version: VERSION.to_string(),
        description: "Records a voice authorization/consent on the cloudrouter audio \
                      gateway (compliance prerequisite for speech synthesis)."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Consent record name."
                },
                "consentDocument": {
                    "type": "string",
                    "description": "Consent document reference or text."
                }
            },
            "required": ["name"]
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Created consent id." },
                "name": { "type": "string" },
                "status": { "type": "string" }
            },
            "required": ["id"]
        }),
        side_effect_level: "side_effectful".to_string(),
        policy_categories: vec![ToolCategory::Audio.policy_category("consent")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

fn voice_consents_list_definition() -> MediaToolDefinition {
    MediaToolDefinition {
        tool_id: tool_ids::VOICE_CONSENTS_LIST.to_string(),
        category: ToolCategory::Audio,
        name: "voice_consents.list".to_string(),
        display_name: "List Voice Consents".to_string(),
        version: VERSION.to_string(),
        description: "Lists voice authorization/consent records on the cloudrouter \
                      audio gateway."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "limit": { "type": "integer", "description": "Maximum consents to return." }
            }
        }),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "consents": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "name": { "type": "string" },
                            "status": { "type": "string" }
                        }
                    }
                }
            },
            "required": ["consents"]
        }),
        side_effect_level: "read_only".to_string(),
        policy_categories: vec![ToolCategory::Audio.policy_category("consent")],
        timeout_ms: 30_000,
        availability: ToolAvailability::Available,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_audio_tool_has_stable_ids_and_required_fields() {
        for definition in audio_tool_definitions() {
            assert!(
                definition.tool_id.starts_with("audio."),
                "{}",
                definition.tool_id
            );
            assert_eq!(definition.category, ToolCategory::Audio);
            assert!(!definition.name.is_empty());
            assert!(!definition.display_name.is_empty());
            assert!(!definition.description.is_empty());
            assert_eq!(definition.input_schema["type"], "object");
            assert_eq!(definition.output_schema["type"], "object");
            assert!(!definition.policy_categories.is_empty());
        }
    }

    #[test]
    fn speech_tool_requires_input_and_is_generative() {
        let speech = audio_tool_definitions()
            .into_iter()
            .find(|definition| definition.tool_id == tool_ids::SPEECH_CREATE)
            .expect("speech tool present");
        assert_eq!(speech.side_effect_level, "side_effectful");
        assert_eq!(speech.availability, ToolAvailability::Available);
        assert_eq!(
            speech.input_schema["required"],
            serde_json::json!(["input"])
        );
        assert_eq!(speech.output_schema["properties"]["kind"]["const"], "audio");
    }

    #[test]
    fn voice_and_consent_tools_are_registered() {
        let definitions = audio_tool_definitions();
        for tool_id in [
            tool_ids::VOICES_CREATE,
            tool_ids::VOICE_CONSENTS_CREATE,
            tool_ids::VOICE_CONSENTS_LIST,
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
            .find(|definition| definition.tool_id == tool_ids::VOICES_CREATE)
            .expect("voices.create present");
        assert_eq!(create.side_effect_level, "side_effectful");
        assert_eq!(create.input_schema["required"], serde_json::json!(["name"]));
    }
}
