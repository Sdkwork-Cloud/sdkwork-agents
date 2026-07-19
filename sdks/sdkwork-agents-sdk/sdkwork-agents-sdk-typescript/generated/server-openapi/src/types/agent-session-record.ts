import type { Int64String } from './int64-string';

export interface AgentSessionRecord {
  sessionId: string;
  agentId: string;
  tenantId: Int64String;
  organizationId?: Int64String;
  ownerUserId?: Int64String;
  projectId?: string | null;
  title?: string;
  status: 'active' | 'idle' | 'closed' | 'archived';
  providerBindingId?: string;
  modelId?: string;
  messageCount: Int64String;
  lastMessageSequence: Int64String;
  totalInputTokens?: Int64String;
  totalOutputTokens?: Int64String;
  metadataJson?: string;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  lastMessageAt?: string;
  closedAt?: string;
  archivedAt?: string;
}
