import type { Int64String } from './int64-string';

export interface AgentTurnInputQueueReorderEntry {
  queueEntryId: string;
  expectedVersion: Int64String;
}
