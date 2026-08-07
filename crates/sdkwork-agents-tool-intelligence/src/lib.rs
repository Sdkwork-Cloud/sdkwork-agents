//! Intelligence tool sub-crate for SDKWork Agents.
//!
//! Owns the `intelligence` category of the extensible media tool family:
//! model discovery (`model.list`), embeddings (`embedding.create`), and
//! content moderation (`moderation.create`), all backed by the cloudrouter
//! open-api surface with caller auth-token account-pool routing.

mod definitions;
mod invoke;
mod provider;

pub use definitions::{intelligence_tool_definitions, tool_ids};
pub use provider::{IntelligenceMediaToolProvider, INTELLIGENCE_PROVIDER_ID};
