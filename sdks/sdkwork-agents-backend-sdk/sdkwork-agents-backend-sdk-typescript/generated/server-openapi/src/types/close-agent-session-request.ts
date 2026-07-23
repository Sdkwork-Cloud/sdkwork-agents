import type { Int64String } from './int64-string';

export interface CloseAgentSessionRequest {
  expectedVersion: Int64String;
  requestedAt: string;
}
