import type { Int64String } from './int64-string';

export interface FailAgentTurnInputQueueEntryRequest {
  expectedVersion: Int64String;
  fencingToken: Int64String;
  claimToken: string;
  errorCode: string;
  errorDetail?: string | null;
  requestedAt: string;
}
