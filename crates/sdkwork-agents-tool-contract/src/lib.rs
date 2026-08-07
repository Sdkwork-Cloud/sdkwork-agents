//! Extensible media tool definition contract for SDKWork Agents.
//!
//! This crate owns the category taxonomy ([`ToolCategory`]), static tool
//! definitions ([`MediaToolDefinition`]), call/result shapes
//! ([`MediaToolCall`], [`MediaToolResult`]), normalized media output
//! ([`MediaResource`]), typed errors ([`MediaToolError`]), and the provider
//! trait ([`MediaToolProvider`]) implemented by every category sub-crate
//! (audio/video/music/sound-effect/image).
//!
//! The contract is provider-neutral: it never touches the cloudrouter SDK or
//! any HTTP transport. Category crates supply invocation behaviour; the
//! cloudrouter adapter crate supplies the shared open-api client; the
//! application registry aggregates providers.

mod call;
mod category;
mod definition;
mod error;
mod kernel;
mod provider;

pub use call::{MediaToolCall, MediaToolResult};
pub use category::ToolCategory;
pub use definition::{MediaResource, MediaToolDefinition, ToolAvailability};
pub use error::MediaToolError;
pub use kernel::{
    media_tool_call, project_invoke_result, project_kernel_error, project_tool_descriptor,
    project_tool_result, side_effect_level, JSON_SCHEMA_DIALECT,
};
pub use provider::{MediaAuthTokenResolver, MediaToolProvider, StaticMediaAuthTokenResolver};
