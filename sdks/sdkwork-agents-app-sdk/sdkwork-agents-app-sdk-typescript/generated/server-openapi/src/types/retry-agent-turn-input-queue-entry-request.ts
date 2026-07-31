import type { Int64String } from './int64-string';

export interface RetryAgentTurnInputQueueEntryRequest {
  expectedVersion: Int64String;
  requestedAt: string;
}
