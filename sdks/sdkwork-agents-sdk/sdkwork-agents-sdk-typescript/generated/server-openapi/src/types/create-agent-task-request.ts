export interface CreateAgentTaskRequest {
  title: string;
  prompt: string;
  externalRef?: string;
  metadataJson?: string;
  requestedAt: string;
}
