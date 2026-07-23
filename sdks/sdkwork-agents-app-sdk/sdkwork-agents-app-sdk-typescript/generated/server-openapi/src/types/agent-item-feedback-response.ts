import type { AgentItemFeedbackRecord } from './agent-item-feedback-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single assistant-output item feedback following SdkWorkApiResponse envelope. */
export interface AgentItemFeedbackResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentItemFeedbackRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
