import type { AgentChatTurnStatus } from './agent-chat-turn-status';
import type { Int64String } from './int64-string';

export interface AgentChatTurnRecord {
  id: Int64String;
  turnId: string;
  tenantId: Int64String;
  organizationId: Int64String;
  sessionId: string;
  agentId: string;
  ownerUserId: Int64String;
  clientRequestId?: string | null;
  idempotencyKey: string;
  requestMessageId: string;
  responseMessageId?: string | null;
  status: AgentChatTurnStatus;
  requestedModelId?: string | null;
  providerBindingId?: string | null;
  modelId?: string | null;
  providerId?: string | null;
  inputTokens: Int64String;
  outputTokens: Int64String;
  finishReason?: string | null;
  errorCode?: string | null;
  errorDetail?: string | null;
  traceId?: string | null;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  startedAt?: string;
  completedAt?: string;
  cancelRequestedAt?: string;
  cancelledAt?: string;
}
