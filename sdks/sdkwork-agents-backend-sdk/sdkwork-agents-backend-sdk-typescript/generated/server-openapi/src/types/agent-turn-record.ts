import type { AgentTurnMode } from './agent-turn-mode';
import type { AgentTurnStatus } from './agent-turn-status';
import type { Int64String } from './int64-string';

export interface AgentTurnRecord {
  turnId: string;
  tenantId: Int64String;
  organizationId: Int64String;
  sessionId: string;
  agentId: string;
  ownerUserId: Int64String;
  runtimeBindingId?: string | null;
  clientRequestId?: string | null;
  idempotencyKey: string;
  payloadHash: string;
  requestItemId: string;
  responseItemId?: string | null;
  turnMode: AgentTurnMode;
  status: AgentTurnStatus;
  requestedModelId?: string | null;
  providerBindingId?: string | null;
  modelId?: string | null;
  providerId?: string | null;
  inputTokens: Int64String;
  outputTokens: Int64String;
  cachedTokens: Int64String;
  finishReason?: string | null;
  errorCode?: string | null;
  /** Sanitized error detail that never contains credentials or provider secrets. */
  errorDetail?: string | null;
  traceId?: string | null;
  attemptCount: number;
  maxAttempts: number;
  nextRetryAt?: string | null;
  availableAt: string;
  leaseOwner?: string | null;
  leaseExpiresAt?: string | null;
  fencingToken: Int64String;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  startedAt?: string | null;
  completedAt?: string | null;
  cancelRequestedAt?: string | null;
  cancelledAt?: string | null;
  retentionUntil?: string | null;
}
