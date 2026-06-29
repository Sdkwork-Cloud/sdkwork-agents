export interface AppSendAgentChatMessageRequest {
  content: string;
  contentType?: string;
  metadataJson?: string;
  modelId?: string;
  requestedAt: string;
}
