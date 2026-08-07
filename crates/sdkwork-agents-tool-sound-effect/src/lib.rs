//! Sound-effect media tool sub-crate for SDKWork Agents.
//!
//! Owns the `sound-effect` category of the extensible media tool family. The
//! tool definition and taxonomy are in place; invocation reports
//! capability-missing until the cloudrouter gateway opens a sound-effect
//! endpoint. The provider implements both the media tool contract and the
//! kernel `ToolProvider` SPI.

mod definitions;
mod provider;

pub use definitions::{sound_effect_tool_definitions, tool_ids};
pub use provider::{SoundEffectMediaToolProvider, SOUND_EFFECT_PROVIDER_ID};
