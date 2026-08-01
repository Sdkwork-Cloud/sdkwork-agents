import type { Int64String } from './int64-string';

export interface AgentTaskRunRecord {
  runId: string;
  taskId: string;
  sessionId: string;
  agentId: string;
  ownerUserId: Int64String;
  triggerKind: 'scheduled' | 'manual' | 'business_retry';
  scheduleGeneration: Int64String;
  scheduledFor: string;
  retryOfRunId: string | null;
  priority: number;
  status: 'pending' | 'claimed' | 'running' | 'succeeded' | 'failed' | 'cancelled' | 'reconciling' | 'dead_letter';
  turnId: string | null;
  attemptCount: number;
  maxAttempts: number;
  availableAt: string;
  timeoutAt: string | null;
  failureClass: string | null;
  errorCode: string | null;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  claimedAt: string | null;
  startedAt: string | null;
  finishedAt: string | null;
  cancelledAt: string | null;
}
