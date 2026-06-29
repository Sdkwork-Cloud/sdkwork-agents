import type { CodeEngineModelCatalogEntry } from './code-engine-model-catalog-entry';

export interface CodeEngineCatalogEngine {
  engineKey: string;
  agentId: string;
  bindingId: string;
  models: CodeEngineModelCatalogEntry[];
}
