//! Music media tool sub-crate for SDKWork Agents.
//!
//! Owns the `music` category of the extensible media tool family: Suno-
//! compatible music generation and task polling, backed by the cloudrouter
//! open-api surface with caller auth-token account-pool routing. Generation
//! is asynchronous: the create tool returns `taskId`, the list tool polls the
//! task until audio track URLs are available.

mod definitions;
mod invoke;
mod provider;

pub use definitions::{music_tool_definitions, tool_ids};
pub use provider::{MusicMediaToolProvider, MUSIC_PROVIDER_ID};
