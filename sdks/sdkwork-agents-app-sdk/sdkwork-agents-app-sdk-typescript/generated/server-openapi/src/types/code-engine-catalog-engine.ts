import type { CodeEngineAccessModeCatalogEntry } from './code-engine-access-mode-catalog-entry';
import type { CodeEngineModelCatalogEntry } from './code-engine-model-catalog-entry';

export interface CodeEngineCatalogEngine {
  engineKey: string;
  tier: string;
  agentId: string;
  bindingId: string;
  models: CodeEngineModelCatalogEntry[];
  defaultAccessModeId: string;
  accessModes: CodeEngineAccessModeCatalogEntry[];
}
