import type { AgentResourceUserStateRecord } from './agent-resource-user-state-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Paginated per-user resource state following SdkWorkApiResponse envelope. */
export interface AgentResourceUserStateListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: AgentResourceUserStateRecord[]; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
