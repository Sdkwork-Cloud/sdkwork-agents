import type { Int64String } from './int64-string';

export interface AgentTaskRecord {
  taskId: string;
  agentId: string;
  tenantId: Int64String;
  organizationId: Int64String;
  ownerUserId: Int64String;
  sessionId: string;
  title: string | null;
  prompt: string;
  scheduleKind: 'one_time' | 'cron';
  cronExpression: string | null;
  /** IANA time zone identifier used for cron evaluation. */
  timezone: string;
  scheduledAt: string | null;
  startsAt: string | null;
  endsAt: string | null;
  nextFireAt: string | null;
  misfirePolicy: 'skip' | 'fire_once' | 'catch_up';
  overlapPolicy: 'skip' | 'queue';
  maxConcurrentRuns: number;
  maxCatchUpRuns: number;
  maxAttempts: number;
  retryInitialDelaySeconds: number;
  retryMaxDelaySeconds: number;
  timeoutSeconds: number;
  priority: number;
  status: 'active' | 'paused' | 'completed' | 'cancelled';
  generation: Int64String;
  externalRef: string | null;
  metadataJson: string;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  completedAt: string | null;
  pausedAt: string | null;
  cancelledAt: string | null;
}
