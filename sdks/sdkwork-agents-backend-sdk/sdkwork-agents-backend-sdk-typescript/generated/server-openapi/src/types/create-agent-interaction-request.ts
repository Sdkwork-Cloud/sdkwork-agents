import type { CreateAgentInteractionData } from './create-agent-interaction-data';

export interface CreateAgentInteractionRequest {
  data: CreateAgentInteractionData;
  requestedAt: string;
}
