import type { Int64String } from './int64-string';

export interface ApproveAgentInteractionRequest {
  approved: boolean;
  reason?: string;
  claimToken: string;
  fencingToken: Int64String;
  expectedVersion: Int64String;
  requestedAt: string;
}
