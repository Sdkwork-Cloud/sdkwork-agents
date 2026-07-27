import type { AgentSessionCheckpointStatus } from './agent-session-checkpoint-status';
import type { Int64String } from './int64-string';

export interface AgentSessionCheckpointRecord {
  checkpointId: string;
  tenantId: Int64String;
  organizationId: Int64String;
  sessionId: string;
  turnId?: string | null;
  runtimeBindingId?: string | null;
  checkpointKind: string;
  providerCheckpointRef?: string | null;
  driveSpaceId?: string | null;
  driveNodeId?: string | null;
  resumable: boolean;
  status: AgentSessionCheckpointStatus;
  createdBy: Int64String;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  restoredAt?: string | null;
  invalidatedAt?: string | null;
  retentionUntil?: string | null;
}
