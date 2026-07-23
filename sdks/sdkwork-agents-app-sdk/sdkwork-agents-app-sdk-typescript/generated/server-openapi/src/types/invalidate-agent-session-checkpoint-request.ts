import type { Int64String } from './int64-string';

export interface InvalidateAgentSessionCheckpointRequest {
  reason?: string;
  expectedVersion: Int64String;
  requestedAt: string;
}
