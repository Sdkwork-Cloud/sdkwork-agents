import type { Int64String } from './int64-string';

export interface SendAgentChatMessageRequest {
  tenantId: Int64String;
  content: string;
  contentType?: string;
  metadataJson?: string;
  modelId?: string;
  requestedAt: string;
}
