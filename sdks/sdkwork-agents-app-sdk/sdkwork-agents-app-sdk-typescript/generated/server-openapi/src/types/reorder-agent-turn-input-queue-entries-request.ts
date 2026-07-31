import type { AgentTurnInputQueueReorderEntry } from './agent-turn-input-queue-reorder-entry';

export interface ReorderAgentTurnInputQueueEntriesRequest {
  orderedEntries: AgentTurnInputQueueReorderEntry[];
  requestedAt: string;
}
