import type { AgentInteractionKind } from './agent-interaction-kind';
import type { AgentInteractionOption } from './agent-interaction-option';
import type { AgentInteractionRequest } from './agent-interaction-request';

export interface CreateAgentInteractionRequest {
  interactionId?: string;
  turnId?: string;
  runtimeBindingId?: string;
  providerInteractionId?: string;
  kind: AgentInteractionKind;
  prompt: string;
  options?: AgentInteractionOption[];
  request?: AgentInteractionRequest;
  retentionUntil?: string;
  requestedAt: string;
}
