import type { AgentSessionEntrySurface } from './agent-session-entry-surface';
import type { AgentSessionKind } from './agent-session-kind';

export interface CreateAgentSessionRequest {
  agentId?: string;
  projectId?: string;
  sessionKind: AgentSessionKind;
  entrySurface: AgentSessionEntrySurface;
  sourceModule?: string;
  sourceContextKind?: string;
  sourceContextId?: string;
  parentSessionId?: string;
  forkedFromTurnId?: string;
  title?: string;
  idempotencyKey: string;
  payloadHash: string;
  requestedAt: string;
}
