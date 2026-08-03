import type { AgentModelProviderId } from './agent-model-provider-id';

export interface ModelConfigurationSummaryRecord {
  profileId: string;
  engineId: AgentModelProviderId;
  agentId: string;
  providerScope: string;
  configurationVersion: string;
  status: 'draft' | 'active' | 'deprecated' | 'archived';
  baseUrl: string;
  defaultModelId: string;
  supportedModelIds: string[];
  apiKeyConfigured: boolean;
}
