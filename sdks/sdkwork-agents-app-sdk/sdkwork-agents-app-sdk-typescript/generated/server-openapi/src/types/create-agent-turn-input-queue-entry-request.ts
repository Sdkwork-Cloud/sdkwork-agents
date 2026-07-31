import type { AgentTurnInputQueueDriveRef } from './agent-turn-input-queue-drive-ref';
import type { AgentTurnMode } from './agent-turn-mode';

export interface CreateAgentTurnInputQueueEntryRequest {
  queueEntryId?: string;
  content: string;
  displayText?: string;
  contentType?: string;
  attachmentNames?: string[];
  driveRefs?: AgentTurnInputQueueDriveRef[];
  turnMode: AgentTurnMode;
  runtimeBindingId?: string | null;
  requestedModelId?: string | null;
  accessModeId?: string | null;
  requestedAt: string;
}
