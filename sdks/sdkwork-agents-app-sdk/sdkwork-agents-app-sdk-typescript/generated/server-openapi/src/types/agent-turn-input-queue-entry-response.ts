import type { AgentTurnInputQueueEntry } from './agent-turn-input-queue-entry';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

export interface AgentTurnInputQueueEntryResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentTurnInputQueueEntry; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
