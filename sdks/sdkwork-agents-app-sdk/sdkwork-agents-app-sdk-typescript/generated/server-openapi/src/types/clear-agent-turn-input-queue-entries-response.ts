import type { Int64String } from './int64-string';

export interface ClearAgentTurnInputQueueEntriesResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & { clearedCount: Int64String; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
