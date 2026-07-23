import type { AgentItemFeedbackRecord } from './agent-item-feedback-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated assistant-output item feedback following SdkWorkApiResponse envelope. */
export interface AgentItemFeedbackListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: AgentItemFeedbackRecord[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
