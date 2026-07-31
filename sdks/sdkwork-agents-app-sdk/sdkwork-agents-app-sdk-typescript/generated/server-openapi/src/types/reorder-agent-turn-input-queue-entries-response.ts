import type { AgentTurnInputQueueEntry } from './agent-turn-input-queue-entry';

export interface ReorderAgentTurnInputQueueEntriesResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & { items: AgentTurnInputQueueEntry[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
