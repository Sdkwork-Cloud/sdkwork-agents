import type { Int64String } from './int64-string';

export interface CreateAgentSessionRequest {
  tenantId?: Int64String;
  organizationId?: Int64String;
  ownerUserId?: Int64String;
  sessionId?: string;
  title?: string;
  providerBindingId?: string;
  modelId?: string;
  metadataJson?: string;
  requestedAt: string;
}
