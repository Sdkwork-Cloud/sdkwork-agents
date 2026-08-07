# sdkwork-agents-tool-contract

Extensible media tool definition contract for SDKWork Agents.

This crate owns the category taxonomy, tool definitions, call/result shapes,
and the provider trait that every media tool sub-crate implements. It is
provider-neutral: it never touches the sdkwork-cloudrouter SDK or any HTTP
transport — category crates (audio/video/music/sound-effect/image) supply the
invocation behaviour through the `MediaToolProvider` trait, and the cloudrouter
adapter crate supplies the shared open-api client.

Categories are additive: a new category only requires a new sub-crate that
implements `MediaToolProvider` and registers in the application-level registry.
