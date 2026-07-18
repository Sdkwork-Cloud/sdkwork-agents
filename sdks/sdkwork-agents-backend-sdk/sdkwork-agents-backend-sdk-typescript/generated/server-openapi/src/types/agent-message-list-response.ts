import type { AgentMessageRecord } from './agent-message-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated agent message list response following SdkWorkApiResponse envelope. */
export interface AgentMessageListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkPageData & Record<string, unknown>;
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
