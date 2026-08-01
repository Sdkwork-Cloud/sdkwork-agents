export interface AgentTaskRunAttemptRecord {
  attemptId: string;
  runId: string;
  attemptNo: number;
  status: 'claimed' | 'running' | 'succeeded' | 'failed' | 'lease_expired' | 'cancelled';
  failureClass: string | null;
  errorCode: string | null;
  createdAt: string;
  updatedAt: string;
  startedAt: string | null;
  heartbeatAt: string | null;
  finishedAt: string | null;
}
