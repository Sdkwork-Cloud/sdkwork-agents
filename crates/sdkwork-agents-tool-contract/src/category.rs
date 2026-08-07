//! Media tool category taxonomy for the SDKWork Agents tool family.
//!
//! The category is the stable top-level classification of a media tool
//! sub-crate. Categories are additive: a new category requires a new sub-crate
//! implementing `MediaToolProvider` and registration in the application-level
//! registry; nothing in this taxonomy changes.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Stable media tool category ids.
///
/// Serialized as the kebab-case form (`audio`, `video`, `music`,
/// `sound-effect`, `image`) so ids survive renames and stay machine-readable
/// in descriptors and audit records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolCategory {
    Audio,
    Video,
    Music,
    SoundEffect,
    Image,
    /// File management on the gateway (upload/list/retrieve/delete/content);
    /// the input foundation for transcription, image edit, and image/video
    /// generation tools.
    File,
    /// Model discovery, embeddings, and content moderation.
    Intelligence,
}

impl ToolCategory {
    /// Stable machine-readable category id used in `provider_id`,
    /// `policy_categories`, and audit records.
    pub fn as_str(self) -> &'static str {
        match self {
            ToolCategory::Audio => "audio",
            ToolCategory::Video => "video",
            ToolCategory::Music => "music",
            ToolCategory::SoundEffect => "sound-effect",
            ToolCategory::Image => "image",
            ToolCategory::File => "file",
            ToolCategory::Intelligence => "intelligence",
        }
    }

    /// The full provider id for the cloudrouter-backed category provider.
    ///
    /// Media categories live under `cloudrouter.media.*`; the intelligence
    /// category (model discovery/embeddings/moderations) is not a media
    /// surface and uses `cloudrouter.intelligence`.
    pub fn provider_id(self) -> String {
        match self {
            ToolCategory::Intelligence => "cloudrouter.intelligence".to_string(),
            _ => format!("cloudrouter.media.{}", self.as_str()),
        }
    }

    /// Default policy category driving authorization, e.g. `media.audio.generate`.
    pub fn policy_category(self, operation: &str) -> String {
        format!("media.{}.{}", self.as_str(), operation)
    }
}

impl fmt::Display for ToolCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ToolCategory {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "audio" => Ok(ToolCategory::Audio),
            "video" => Ok(ToolCategory::Video),
            "music" => Ok(ToolCategory::Music),
            "sound-effect" | "sound_effect" | "sfx" => Ok(ToolCategory::SoundEffect),
            "image" | "images" | "picture" => Ok(ToolCategory::Image),
            "file" | "files" => Ok(ToolCategory::File),
            "intelligence" | "model" | "models" => Ok(ToolCategory::Intelligence),
            other => Err(format!("unknown tool category: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_ids_are_stable() {
        assert_eq!(ToolCategory::Audio.as_str(), "audio");
        assert_eq!(ToolCategory::Video.as_str(), "video");
        assert_eq!(ToolCategory::Music.as_str(), "music");
        assert_eq!(ToolCategory::SoundEffect.as_str(), "sound-effect");
        assert_eq!(ToolCategory::Image.as_str(), "image");
        assert_eq!(ToolCategory::File.as_str(), "file");
        assert_eq!(ToolCategory::Intelligence.as_str(), "intelligence");
    }

    #[test]
    fn provider_ids_and_policy_categories_derive_from_category() {
        assert_eq!(ToolCategory::Audio.provider_id(), "cloudrouter.media.audio");
        assert_eq!(
            ToolCategory::SoundEffect.provider_id(),
            "cloudrouter.media.sound-effect"
        );
        assert_eq!(ToolCategory::File.provider_id(), "cloudrouter.media.file");
        assert_eq!(
            ToolCategory::Intelligence.provider_id(),
            "cloudrouter.intelligence"
        );
        assert_eq!(
            ToolCategory::Image.policy_category("generate"),
            "media.image.generate"
        );
        assert_eq!(
            ToolCategory::Music.policy_category("create"),
            "media.music.create"
        );
        assert_eq!(
            ToolCategory::Intelligence.policy_category("moderate"),
            "media.intelligence.moderate"
        );
    }

    #[test]
    fn category_parses_known_ids_and_aliases() {
        assert_eq!("audio".parse(), Ok(ToolCategory::Audio));
        assert_eq!("sound-effect".parse(), Ok(ToolCategory::SoundEffect));
        assert_eq!("sfx".parse(), Ok(ToolCategory::SoundEffect));
        assert_eq!("image".parse(), Ok(ToolCategory::Image));
        assert_eq!("file".parse(), Ok(ToolCategory::File));
        assert_eq!("intelligence".parse(), Ok(ToolCategory::Intelligence));
        assert_eq!("AUDIO".parse(), Ok(ToolCategory::Audio));
        assert!("unknown".parse::<ToolCategory>().is_err());
    }

    #[test]
    fn category_round_trips_display() {
        for category in [
            ToolCategory::Audio,
            ToolCategory::Video,
            ToolCategory::Music,
            ToolCategory::SoundEffect,
            ToolCategory::Image,
            ToolCategory::File,
            ToolCategory::Intelligence,
        ] {
            assert_eq!(category.to_string().parse(), Ok(category));
        }
    }
}
