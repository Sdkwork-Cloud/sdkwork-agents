import type { AgentTurnInputQueueDriveRef } from './agent-turn-input-queue-drive-ref';
import type { AgentTurnMode } from './agent-turn-mode';
import type { Int64String } from './int64-string';

export interface UpdateAgentTurnInputQueueEntryRequest {
  content: string;
  displayText?: string;
  contentType?: string;
  attachmentNames?: string[];
  driveRefs?: AgentTurnInputQueueDriveRef[];
  turnMode: AgentTurnMode;
  runtimeBindingId?: string | null;
  requestedModelId?: string | null;
  accessModeId?: string | null;
  expectedVersion: Int64String;
  requestedAt: string;
}
