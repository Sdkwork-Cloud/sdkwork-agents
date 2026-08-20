//! Generations MCP tool for sdkwork-agents.
//!
//! Wraps the sdkwork-generations service to expose image/video/music/sfx/voice
//! generation capabilities as MCP tools callable by AI agents. Backed by the
//! cloudrouter open-api surface via the shared media transport crate, with
//! caller auth-token account-pool routing.

mod definitions;
mod invoke;
mod provider;

pub use definitions::{generations_tool_definitions, tool_ids};
pub use provider::{GenerationsToolProvider, GENERATIONS_PROVIDER_ID};
