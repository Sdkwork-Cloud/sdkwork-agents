import type { AgentItemDriveRefRecord } from './agent-item-drive-ref-record';
import type { AgentSessionItemKind } from './agent-session-item-kind';
import type { AgentSessionItemStatus } from './agent-session-item-status';
import type { Int64String } from './int64-string';

export interface AgentSessionItemRecord {
  tenantId: Int64String;
  organizationId: Int64String;
  sessionId: string;
  itemId: string;
  kind: AgentSessionItemKind;
  content?: string | null;
  contentType?: string;
  status: AgentSessionItemStatus;
  sequence: Int64String;
  inputTokens: Int64String;
  outputTokens: Int64String;
  modelId?: string | null;
  providerId?: string | null;
  toolName?: string | null;
  toolCallId?: string | null;
  toolArguments?: Record<string, unknown>;
  toolResult?: Record<string, unknown>;
  /** Full raw provider protocol payload (provider thread item JSON) preserved without loss. */
  providerPayload?: Record<string, unknown>;
  parentItemId?: string | null;
  turnId?: string | null;
  driveRefs: AgentItemDriveRefRecord[];
  createdBy: Int64String;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  completedAt?: string | null;
  redactedAt?: string | null;
  redactedBy?: Int64String | null;
  retentionUntil?: string | null;
}
