import type { AgentProviderBindingRecord } from './agent-provider-binding-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated agent provider binding list response following SdkWorkApiResponse envelope. */
export interface AgentProviderBindingListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkPageData & Record<string, unknown>;
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
