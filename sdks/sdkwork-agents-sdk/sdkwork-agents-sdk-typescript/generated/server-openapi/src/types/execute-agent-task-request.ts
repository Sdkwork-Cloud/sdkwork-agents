import type { Int64String } from './int64-string';

export interface ExecuteAgentTaskRequest {
  idempotencyKey: string;
  expectedVersion?: Int64String;
  requestedAt: string;
}
