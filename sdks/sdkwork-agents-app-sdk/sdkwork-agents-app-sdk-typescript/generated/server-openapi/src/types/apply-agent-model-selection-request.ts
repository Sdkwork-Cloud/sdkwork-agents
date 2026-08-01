import type { AgentModelProviderId } from './agent-model-provider-id';

export interface ApplyAgentModelSelectionRequest {
  /** Saved custom configuration identity. Omit for catalog models. */
  configurationId?: string;
  engineId: AgentModelProviderId;
  modelId: string;
}
