import type { AgentInteractionAction } from './agent-interaction-action';
import type { AgentInteractionCorrelation } from './agent-interaction-correlation';
import type { AgentInteractionRequestCategory } from './agent-interaction-request-category';
import type { AgentInteractionRequestData } from './agent-interaction-request-data';
import type { AgentInteractionRequestKind } from './agent-interaction-request-kind';

export interface AgentInteractionRequest {
  schemaVersion: 1;
  category: AgentInteractionRequestCategory;
  kind: AgentInteractionRequestKind;
  allowedActions: AgentInteractionAction[];
  data: AgentInteractionRequestData;
  correlation?: AgentInteractionCorrelation;
}
