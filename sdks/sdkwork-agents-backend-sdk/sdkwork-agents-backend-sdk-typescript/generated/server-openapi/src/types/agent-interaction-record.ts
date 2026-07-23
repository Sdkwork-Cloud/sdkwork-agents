import type { AgentInteractionKind } from './agent-interaction-kind';
import type { AgentInteractionOption } from './agent-interaction-option';
import type { AgentInteractionResolution } from './agent-interaction-resolution';
import type { AgentInteractionStatus } from './agent-interaction-status';
import type { Int64String } from './int64-string';

export interface AgentInteractionRecord {
  interactionId: string;
  tenantId: Int64String;
  organizationId: Int64String;
  sessionId: string;
  turnId?: string | null;
  runtimeBindingId?: string | null;
  providerInteractionId?: string | null;
  kind: AgentInteractionKind;
  status: AgentInteractionStatus;
  prompt: string;
  options: AgentInteractionOption[];
  resolution?: AgentInteractionResolution | null;
  claimOwner?: string | null;
  claimExpiresAt?: string;
  fencingToken: Int64String;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  resolvedAt?: string;
  retentionUntil?: string;
}
