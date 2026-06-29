export interface AppCreateAgentSessionRequest {
  sessionId?: string;
  title?: string;
  providerBindingId?: string;
  modelId?: string;
  metadataJson?: string;
  requestedAt: string;
}
