import type { ClaimNextAgentTurnInputQueueEntryResult } from './claim-next-agent-turn-input-queue-entry-result';

export interface ClaimNextAgentTurnInputQueueEntryResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & ClaimNextAgentTurnInputQueueEntryResult;
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
