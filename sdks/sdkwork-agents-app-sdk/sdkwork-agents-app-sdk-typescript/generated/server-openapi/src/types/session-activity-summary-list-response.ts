import type { PageInfo } from './page-info';
import type { SdkWorkPageData } from './sdk-work-page-data';
import type { SessionActivitySummary } from './session-activity-summary';

/** Bounded current-state Session activity snapshot page. */
export interface SessionActivitySummaryListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: SessionActivitySummary[]; pageInfo: PageInfo & { mode: 'cursor'; }; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
