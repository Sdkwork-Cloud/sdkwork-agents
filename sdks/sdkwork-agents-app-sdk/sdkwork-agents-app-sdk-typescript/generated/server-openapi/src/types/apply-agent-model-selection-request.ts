import type { AgentModelProviderId } from './agent-model-provider-id';

export interface ApplyAgentModelSelectionRequest {
  /** Saved custom configuration identity. Omit for catalog models. */
  configurationId?: string;
  engineId: AgentModelProviderId;
  /** Optional configuration subject (user-created agent id). Defaults to the canonical engine agent. */
  agentId?: string;
  modelId: string;
}
