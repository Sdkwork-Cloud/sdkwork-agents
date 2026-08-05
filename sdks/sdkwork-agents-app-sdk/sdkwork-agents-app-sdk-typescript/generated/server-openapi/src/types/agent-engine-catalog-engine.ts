import type { AgentEngineAccessModeCatalogEntry } from './agent-engine-access-mode-catalog-entry';
import type { AgentEngineModelCatalogEntry } from './agent-engine-model-catalog-entry';

export interface AgentEngineCatalogEngine {
  engineKey: string;
  engineKind: 'code' | 'work' | 'simple' | 'unknown';
  tier: string;
  agentId: string;
  bindingId: string;
  models: AgentEngineModelCatalogEntry[];
  defaultAccessModeId: string;
  accessModes: AgentEngineAccessModeCatalogEntry[];
  available: boolean;
  unavailableReason?: string;
}
