import type { Int64String } from './int64-string';

export interface CancelAgentTaskRequest {
  expectedVersion?: Int64String;
  requestedAt: string;
}
