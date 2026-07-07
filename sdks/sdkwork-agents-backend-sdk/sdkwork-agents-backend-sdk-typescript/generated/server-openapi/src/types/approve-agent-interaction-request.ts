import type { Int64String } from './int64-string';

export interface ApproveAgentInteractionRequest {
  tenantId: Int64String;
  approved: boolean;
  reason?: string;
  expectedVersion: Int64String;
  requestedAt: string;
}
