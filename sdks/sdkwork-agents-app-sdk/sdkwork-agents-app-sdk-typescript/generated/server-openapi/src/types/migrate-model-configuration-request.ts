import type { AgentModelProviderId } from './agent-model-provider-id';

export interface MigrateModelConfigurationRequest {
  engineId: AgentModelProviderId;
  profileId: string;
  fromConfigurationVersion: string;
  toConfigurationVersion: string;
}
