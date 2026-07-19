import type { Int64String } from './int64-string';

export interface AgentResourceUserStateRecord {
  id: Int64String;
  tenantId: Int64String;
  organizationId: Int64String;
  userId: Int64String;
  resourceType: 'session' | 'project';
  resourceId: string;
  pinnedAt?: string;
  hiddenAt?: string;
  lastOpenedAt?: string;
  lastReadMessageSequence?: string;
  customTitle?: string;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
}
