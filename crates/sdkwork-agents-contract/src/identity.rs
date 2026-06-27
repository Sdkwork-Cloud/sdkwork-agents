//! Canonical SDKWork Agents identity defaults for manifests, bootstrap, tests, and seeds.

/// IAM tenant application runtime `app_id` and JWT `app_id` claim for sdkwork-agents.
pub const AGENTS_APP_ID: &str = "sdkwork-agents";

/// Default development tenant identifier.
pub const AGENTS_DEFAULT_TENANT_ID: &str = "100001";

/// Default development organization identifier (root org).
pub const AGENTS_DEFAULT_ORGANIZATION_ID: &str = "0";

/// Numeric tenant id for database rows and service fixtures.
pub const AGENTS_DEFAULT_TENANT_ID_I64: i64 = 100_001;

/// Numeric organization id for database rows and service fixtures.
pub const AGENTS_DEFAULT_ORGANIZATION_ID_I64: i64 = 0;
