//! Video media tool sub-crate for SDKWork Agents.
//!
//! Owns the `video` category of the extensible media tool family: generation,
//! retrieval, listing, edits, extensions, remix, and characters, all backed by
//! the cloudrouter open-api surface with caller auth-token account-pool
//! routing. Generation is asynchronous: submit tools return `taskId`, the
//! retrieve tool polls the task until the asset URL is available.

mod definitions;
mod invoke;
mod provider;

pub use definitions::{tool_ids, video_tool_definitions};
pub use provider::{VideoMediaToolProvider, VIDEO_PROVIDER_ID};
