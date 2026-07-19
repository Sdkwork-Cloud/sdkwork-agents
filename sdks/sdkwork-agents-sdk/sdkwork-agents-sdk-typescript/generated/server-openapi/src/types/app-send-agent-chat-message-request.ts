import type { MediaResource } from './media-resource';

export interface AppSendAgentChatMessageRequest {
  content: string;
  contentType?: string;
  metadataJson?: string;
  mediaResources?: MediaResource[];
  modelId?: string;
  idempotencyKey: string;
  clientRequestId?: string;
  requestedAt: string;
}
