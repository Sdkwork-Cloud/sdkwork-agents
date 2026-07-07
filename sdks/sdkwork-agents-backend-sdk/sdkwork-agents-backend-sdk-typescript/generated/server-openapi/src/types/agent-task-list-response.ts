import type { AgentTaskRecord } from './agent-task-record';
import type { SdkWorkPageData } from './sdk-work-page-data';

export interface AgentTaskListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md §15.3. */
  code: 0;
  data: unknown & SdkWorkPageData & Record<string, unknown>;
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
