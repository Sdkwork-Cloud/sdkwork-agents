import type { Int64String } from './int64-string';

export interface RestoreAgentSessionCheckpointRequest {
  expectedVersion: Int64String;
  requestedAt: string;
}
