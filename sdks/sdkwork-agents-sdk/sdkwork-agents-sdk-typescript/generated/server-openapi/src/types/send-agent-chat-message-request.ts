import type { Int64String } from './int64-string';
import type { MediaResource } from './media-resource';

export interface SendAgentChatMessageRequest {
  tenantId: Int64String;
  content: string;
  contentType?: string;
  metadataJson?: string;
  mediaResources?: MediaResource[];
  modelId?: string;
  requestedAt: string;
}
