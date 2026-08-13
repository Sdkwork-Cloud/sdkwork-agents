import type { AgentModelProviderId } from './agent-model-provider-id';

export interface ApplyAgentModelConfigurationRequest {
  configurationId: string;
  engineId: AgentModelProviderId;
  /** Optional configuration subject (user-created agent id). Defaults to the canonical engine agent. */
  agentId?: string;
  vendorCode: string;
  baseUrl: string;
  /** Plaintext credential accepted only for immediate Secret Provider storage. */
  apiKey?: string;
  defaultModelId: string;
  supportedModelIds: string[];
  /** Defaults to every supported Agent provider when omitted or empty. */
  supportedProviderIds?: AgentModelProviderId[];
  inputContextTokens?: string;
  outputContextTokens?: string;
  toolCallRounds?: string;
  supportsMultimodal?: boolean;
}
