import type { AgentModelProviderId } from './agent-model-provider-id';

export interface AppliedAgentModelSelectionRecord {
  /** Saved custom configuration identity. Omitted for catalog models. */
  configurationId?: string;
  profileId: string;
  engineId: AgentModelProviderId;
  agentId: string;
  providerScope: string;
  modelId: string;
}
