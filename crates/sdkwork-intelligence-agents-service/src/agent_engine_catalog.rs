//! Canonical agent-engine catalog assembled through `sdkwork-agents-runtime-facade`.
//!
//! The catalog is exposed as a single composite resource under
//! `SdkWorkResourceData<AgentEngineCatalog>` per `API_SPEC.md` §16.

use sdkwork_agents_runtime_facade::AgentEngineCatalog;

use crate::runtime_facade_bridge::shared_agent_engine_host;

pub fn list_agent_engine_catalog() -> AgentEngineCatalog {
    shared_agent_engine_host()
        .as_deref()
        .map(AgentsAgentEngineHostExt::catalog)
        .unwrap_or_else(empty_catalog)
}

fn empty_catalog() -> AgentEngineCatalog {
    AgentEngineCatalog {
        engines: Vec::new(),
    }
}

trait AgentsAgentEngineHostExt {
    fn catalog(&self) -> AgentEngineCatalog;
}

impl AgentsAgentEngineHostExt for sdkwork_agents_runtime_facade::AgentsAgentEngineHost {
    fn catalog(&self) -> AgentEngineCatalog {
        sdkwork_agents_runtime_facade::AgentsAgentEngineHost::catalog(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_serializes_with_engines_key() {
        let catalog = list_agent_engine_catalog();
        let json = serde_json::to_value(&catalog).expect("serialize catalog");
        assert!(json.get("engines").is_some());
    }
}
