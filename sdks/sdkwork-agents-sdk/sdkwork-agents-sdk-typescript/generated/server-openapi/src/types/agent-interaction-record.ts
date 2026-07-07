import type { Int64String } from './int64-string';

export interface AgentInteractionRecord {
  interactionId: string;
  sessionId: string;
  agentId: string;
  tenantId: Int64String;
  organizationId: Int64String;
  engineKey: string;
  kind: 'approval' | 'user_question';
  status: string;
  prompt: string;
  optionsJson: string;
  resolutionJson: string;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  resolvedAt?: string;
}
