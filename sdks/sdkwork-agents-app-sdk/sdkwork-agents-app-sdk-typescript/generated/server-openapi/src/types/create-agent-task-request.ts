export interface CreateAgentTaskRequest {
  taskId?: string | null;
  sessionId: string;
  title: string;
  prompt: string;
  scheduleKind: 'one_time' | 'cron';
  cronExpression?: string | null;
  timezone: string;
  scheduledAt?: string | null;
  startsAt?: string | null;
  endsAt?: string | null;
  misfirePolicy?: 'skip' | 'fire_once' | 'catch_up';
  overlapPolicy?: 'skip' | 'queue';
  maxConcurrentRuns?: number;
  maxCatchUpRuns?: number;
  maxAttempts?: number;
  retryInitialDelaySeconds?: number;
  retryMaxDelaySeconds?: number;
  timeoutSeconds?: number;
  priority?: number;
  externalRef?: string | null;
  metadataJson?: string | null;
  requestedAt: string;
}
