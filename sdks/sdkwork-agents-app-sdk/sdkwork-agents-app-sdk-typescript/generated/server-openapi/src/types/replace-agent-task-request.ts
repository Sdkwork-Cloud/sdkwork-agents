import type { Int64String } from './int64-string';

export interface ReplaceAgentTaskRequest {
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
  expectedVersion: Int64String;
  requestedAt: string;
}
