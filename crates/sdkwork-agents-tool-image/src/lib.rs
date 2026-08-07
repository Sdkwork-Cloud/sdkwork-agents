//! Image media tool sub-crate for SDKWork Agents.
//!
//! Owns the `image` category of the extensible media tool family: generation,
//! edits, and variations, all backed by the cloudrouter open-api surface with
//! caller auth-token account-pool routing. Vendor wire fields are normalized
//! into `MediaResource` shape before leaving the provider.

mod definitions;
mod invoke;
mod provider;

pub use definitions::{image_tool_definitions, tool_ids};
pub use provider::{ImageMediaToolProvider, IMAGE_PROVIDER_ID};
