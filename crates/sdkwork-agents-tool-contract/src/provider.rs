//! The media tool provider trait implemented by every category sub-crate.

use crate::call::{MediaToolCall, MediaToolResult};
use crate::category::ToolCategory;
use crate::definition::MediaToolDefinition;
use crate::error::MediaToolError;

/// A provider of media tools for one category.
///
/// Category crates implement this trait and expose their definitions as
/// kernel-compatible `ToolDescriptor` projections; the application-level
/// registry aggregates providers and routes calls by `tool_id`.
///
/// The provider receives the caller auth token out-of-band (never inside
/// `arguments`), matching the cloudrouter account-pool routing model: the
/// caller's login token selects the tenant account group upstream.
pub trait MediaToolProvider: std::fmt::Debug + Send + Sync {
    /// The category owned by this provider.
    fn category(&self) -> ToolCategory;

    /// Static tool definitions for the category.
    fn definitions(&self) -> Vec<MediaToolDefinition>;

    /// Executes one tool call.
    ///
    /// `auth_token` is the caller's cloudrouter auth token (login identity);
    /// synchronous tools return `succeeded`/`failed` results, async task tools
    /// return `pending` with a `taskId` in the output for the poll tool.
    fn invoke(
        &self,
        call: &MediaToolCall,
        auth_token: Option<&str>,
    ) -> Result<MediaToolResult, MediaToolError>;
}

/// Resolves the auth token for a tool call at invocation time.
///
/// The application layer implements this against session-scoped state (the
/// token captured from the turn or request context); providers never store
/// caller tokens.
pub trait MediaAuthTokenResolver: std::fmt::Debug + Send + Sync {
    fn resolve(&self, session_id: Option<&str>) -> Option<String>;
}

/// Static auth token resolver for tests and embedded flows.
#[derive(Debug, Clone, Default)]
pub struct StaticMediaAuthTokenResolver(pub Option<String>);

impl MediaAuthTokenResolver for StaticMediaAuthTokenResolver {
    fn resolve(&self, _session_id: Option<&str>) -> Option<String> {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_resolver_returns_configured_token() {
        let resolver = StaticMediaAuthTokenResolver(Some("token".to_string()));
        assert_eq!(resolver.resolve(None).as_deref(), Some("token"));
        let empty = StaticMediaAuthTokenResolver(None);
        assert_eq!(empty.resolve(Some("session.x")), None);
    }
}
