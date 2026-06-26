use serde::{Deserialize, Serialize};
use sdkwork_agents_runtime_facade::{
    bootstrap_canonical_code_engine_catalog, CodeEngineCatalog, CodeEngineCatalogEngine,
    CodeEngineModelCatalogEntry,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeEngineCatalogResponseDto {
    pub engines: Vec<CodeEngineCatalogEngineDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeEngineCatalogEngineDto {
    pub engine_key: String,
    pub agent_id: String,
    pub binding_id: String,
    pub models: Vec<CodeEngineModelCatalogEntryDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeEngineModelCatalogEntryDto {
    pub engine_key: String,
    pub model_id: String,
    pub label: String,
    pub description: String,
    pub provider_id: String,
    pub binding_id: String,
    pub default_for_engine: bool,
}

pub fn load_code_engine_catalog() -> Result<CodeEngineCatalogResponseDto, String> {
    bootstrap_canonical_code_engine_catalog().map(to_catalog_response_dto)
}

fn to_catalog_response_dto(catalog: CodeEngineCatalog) -> CodeEngineCatalogResponseDto {
    CodeEngineCatalogResponseDto {
        engines: catalog
            .engines
            .into_iter()
            .map(to_catalog_engine_dto)
            .collect(),
    }
}

fn to_catalog_engine_dto(engine: CodeEngineCatalogEngine) -> CodeEngineCatalogEngineDto {
    CodeEngineCatalogEngineDto {
        engine_key: engine.engine_key,
        agent_id: engine.agent_id,
        binding_id: engine.binding_id,
        models: engine.models.into_iter().map(to_catalog_model_dto).collect(),
    }
}

fn to_catalog_model_dto(entry: CodeEngineModelCatalogEntry) -> CodeEngineModelCatalogEntryDto {
    CodeEngineModelCatalogEntryDto {
        engine_key: entry.engine_key,
        model_id: entry.model_id,
        label: entry.label,
        description: entry.description,
        provider_id: entry.provider_id,
        binding_id: entry.binding_id,
        default_for_engine: entry.default_for_engine,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_canonical_code_engine_catalog() {
        let catalog = load_code_engine_catalog().expect("catalog");
        assert_eq!(catalog.engines.len(), 4);
    }
}
