//! File media tool sub-crate for SDKWork Agents.
//!
//! Owns the `file` category of the extensible media tool family: gateway file
//! upload, listing, retrieval, deletion, and content, backed by the
//! cloudrouter open-api surface with caller auth-token account-pool routing.
//! `file.upload` closes the input chain for the media tools that consume file
//! references (transcription, image edit, video generation).

mod definitions;
mod invoke;
mod provider;

pub use definitions::{file_tool_definitions, tool_ids};
pub use provider::{FileMediaToolProvider, FILE_PROVIDER_ID};
