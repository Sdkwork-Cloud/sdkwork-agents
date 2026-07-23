import type { AgentTurnMode } from './agent-turn-mode';

export interface CreateAgentTurnRequest {
  turnId?: string;
  content: string;
  contentType?: string;
  turnMode: AgentTurnMode;
  runtimeBindingId?: string;
  requestedModelId?: string;
  idempotencyKey: string;
  payloadHash: string;
  clientRequestId?: string;
  driveRefs?: ({ resourceRole: 'attachment' | 'image' | 'audio' | 'artifact'; driveSpaceId: string; driveNodeId: string; })[];
  requestedAt: string;
}
