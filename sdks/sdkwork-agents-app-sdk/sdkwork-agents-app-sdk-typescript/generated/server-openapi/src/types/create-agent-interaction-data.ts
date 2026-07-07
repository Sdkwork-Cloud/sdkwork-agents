import type { Int64String } from './int64-string';

export interface CreateAgentInteractionData {
  tenantId?: Int64String;
  organizationId: Int64String;
  interactionId?: string;
  engineKey: string;
  kind: 'approval' | 'user_question';
  prompt: string;
  optionsJson?: string;
}
