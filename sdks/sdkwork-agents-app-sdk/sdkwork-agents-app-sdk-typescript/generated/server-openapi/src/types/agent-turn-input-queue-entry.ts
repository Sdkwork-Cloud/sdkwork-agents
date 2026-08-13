import type { AgentTurnInputQueueDriveRef } from './agent-turn-input-queue-drive-ref';
import type { AgentTurnInputQueueStatus } from './agent-turn-input-queue-status';
import type { AgentTurnMode } from './agent-turn-mode';
import type { Int64String } from './int64-string';

export interface AgentTurnInputQueueEntry {
  queueEntryId: string;
  sessionId: string;
  agentId: string;
  content: string;
  displayText: string;
  contentType: string;
  attachmentNames: string[];
  driveRefs: AgentTurnInputQueueDriveRef[];
  turnMode: AgentTurnMode;
  /** Agent system prompt injected ahead of the turn history. */
  systemPrompt?: string;
  runtimeBindingId?: string | null;
  requestedModelId?: string | null;
  accessModeId?: string | null;
  idempotencyKey: string;
  payloadHash: string;
  clientRequestId: string;
  position: Int64String;
  status: AgentTurnInputQueueStatus;
  claimOwner?: string | null;
  claimExpiresAt?: string | null;
  fencingToken: Int64String;
  errorCode?: string | null;
  /** Sanitized error detail without credentials or provider secrets. */
  errorDetail?: string | null;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  claimedAt?: string | null;
  failedAt?: string | null;
}
