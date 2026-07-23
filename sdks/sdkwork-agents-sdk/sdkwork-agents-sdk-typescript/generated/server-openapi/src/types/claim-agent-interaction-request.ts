import type { Int64String } from './int64-string';

export interface ClaimAgentInteractionRequest {
  claimOwner: string;
  leaseSeconds?: number;
  expectedVersion: Int64String;
  requestedAt: string;
}
