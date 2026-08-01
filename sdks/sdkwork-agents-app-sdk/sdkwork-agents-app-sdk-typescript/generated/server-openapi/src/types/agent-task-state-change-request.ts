import type { Int64String } from './int64-string';

export interface AgentTaskStateChangeRequest {
  expectedVersion: Int64String;
  requestedAt: string;
}
