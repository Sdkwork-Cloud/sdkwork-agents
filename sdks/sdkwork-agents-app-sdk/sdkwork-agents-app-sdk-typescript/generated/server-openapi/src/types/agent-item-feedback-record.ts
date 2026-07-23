import type { Int64String } from './int64-string';

export interface AgentItemFeedbackRecord {
  id: Int64String;
  tenantId: Int64String;
  organizationId: Int64String;
  itemId: string;
  userId: Int64String;
  rating: 'up' | 'down';
  reasonCode?: string;
  comment?: string;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  deletedAt?: string;
}
