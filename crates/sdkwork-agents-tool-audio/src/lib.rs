//! Audio media tool sub-crate for SDKWork Agents.
//!
//! Owns the `audio` category of the extensible media tool family: speech
//! synthesis, transcription, translation, and voice listing, all backed by the
//! cloudrouter open-api surface with caller auth-token account-pool routing.
//!
//! The provider implements both the media tool contract
//! (`MediaToolProvider`) and the kernel `ToolProvider` SPI.

mod definitions;
mod invoke;
mod provider;

pub use definitions::{audio_tool_definitions, tool_ids};
pub use provider::{AudioMediaToolProvider, AUDIO_PROVIDER_ID};
