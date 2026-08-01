import type { Int64String } from './int64-string';

export interface ReconcileAgentTaskRunRequest {
  outcome: 'succeeded' | 'failed' | 'cancelled';
  errorCode?: string | null;
  expectedVersion: Int64String;
  requestedAt: string;
}
