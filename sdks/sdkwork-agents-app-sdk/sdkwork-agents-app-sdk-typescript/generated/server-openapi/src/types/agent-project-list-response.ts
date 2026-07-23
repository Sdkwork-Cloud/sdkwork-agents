import type { AgentProjectRecord } from './agent-project-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated project response following SdkWorkApiResponse envelope. */
export interface AgentProjectListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: AgentProjectRecord[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
