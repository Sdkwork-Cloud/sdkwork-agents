//! SDKWork Agents runtime environment helpers.

mod runtime_env;
mod identity;

pub use identity::{
    AGENTS_APP_ID, AGENTS_APPLICATION_REGISTRY_KEY, AGENTS_DEFAULT_ORGANIZATION_ID,
    AGENTS_DEFAULT_ORGANIZATION_ID_I64, AGENTS_DEFAULT_TENANT_ID, AGENTS_DEFAULT_TENANT_ID_I64,
};
pub use runtime_env::env_test_lock;
pub fn agents_environment_name() -> String {
    std::env::var("SDKWORK_AGENTS_ENVIRONMENT")
        .or_else(|_| std::env::var("SDKWORK_AGENTS_CONFIG_PROFILE"))
        .unwrap_or_else(|_| "development".to_string())
        .to_ascii_lowercase()
}

/// Returns true for production-like profiles that must never use dev inline auth.
pub fn agents_is_production_like_environment() -> bool {
    matches!(
        agents_environment_name().as_str(),
        "production" | "prod" | "staging" | "stage" | "test"
    )
}

/// `SDKWORK_AGENTS_DEV_AUTH_BYPASS` enables inline dev credentials only outside production profiles.
pub fn agents_dev_auth_bypass_enabled() -> bool {
    std::env::var("SDKWORK_AGENTS_DEV_AUTH_BYPASS")
        .ok()
        .and_then(|value| sdkwork_utils_rust::parse_bool(value.trim()))
        .unwrap_or(false)
}

/// Whether HTTP surfaces may use inline dev auth resolver.
pub fn agents_use_dev_inline_auth_resolver() -> bool {
    !agents_is_production_like_environment() && agents_dev_auth_bypass_enabled()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bool_from_utils_for_dev_bypass() {
        std::env::set_var("SDKWORK_AGENTS_DEV_AUTH_BYPASS", "true");
        assert!(agents_dev_auth_bypass_enabled());
        std::env::remove_var("SDKWORK_AGENTS_DEV_AUTH_BYPASS");
    }
}
