export interface AppCreateAgentSessionRequest {
  sessionId?: string;
  projectId?: string;
  title?: string;
  providerBindingId?: string;
  modelId?: string;
  metadataJson?: string;
  requestedAt: string;
}
