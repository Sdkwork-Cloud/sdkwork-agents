import type { AgentMessageRecord } from './agent-message-record';
import type { AgentSessionRecord } from './agent-session-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Agent chat completion response following SdkWorkApiResponse envelope with composite item payload. */
export interface AgentChatCompletionResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & Record<string, unknown>;
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
