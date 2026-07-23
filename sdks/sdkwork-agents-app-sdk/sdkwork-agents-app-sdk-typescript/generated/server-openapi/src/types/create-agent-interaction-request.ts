import type { AgentInteractionKind } from './agent-interaction-kind';
import type { AgentInteractionOption } from './agent-interaction-option';

export interface CreateAgentInteractionRequest {
  interactionId?: string;
  turnId?: string;
  runtimeBindingId?: string;
  providerInteractionId?: string;
  kind: AgentInteractionKind;
  prompt: string;
  options?: AgentInteractionOption[];
  retentionUntil?: string;
  requestedAt: string;
}
