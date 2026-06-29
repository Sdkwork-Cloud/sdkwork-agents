//! Canonical code-engine catalog projection via `sdkwork-agents-runtime-facade`.
//!
//! The catalog is exposed as a single composite resource under
//! `SdkWorkResourceData<CodeEngineCatalog>` per `API_SPEC.md` §16.

use sdkwork_agents_runtime_facade::CodeEngineCatalog;

use crate::runtime_facade_bridge::shared_code_engine_host;

pub fn list_code_engine_catalog() -> CodeEngineCatalog {
    shared_code_engine_host()
        .map(AgentsCodeEngineHostExt::catalog)
        .unwrap_or_else(empty_catalog)
}

fn empty_catalog() -> CodeEngineCatalog {
    CodeEngineCatalog { engines: Vec::new() }
}

trait AgentsCodeEngineHostExt {
    fn catalog(&self) -> CodeEngineCatalog;
}

impl AgentsCodeEngineHostExt for sdkwork_agents_runtime_facade::AgentsCodeEngineHost {
    fn catalog(&self) -> CodeEngineCatalog {
        sdkwork_agents_runtime_facade::AgentsCodeEngineHost::catalog(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_serializes_with_engines_key() {
        let catalog = list_code_engine_catalog();
        let json = serde_json::to_value(&catalog).expect("serialize catalog");
        assert!(json.get("engines").is_some());
    }
}
