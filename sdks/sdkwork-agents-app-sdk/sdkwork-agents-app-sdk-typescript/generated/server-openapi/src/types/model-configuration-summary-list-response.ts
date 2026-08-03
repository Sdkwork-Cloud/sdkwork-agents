import type { ModelConfigurationSummaryRecord } from './model-configuration-summary-record';
import type { PageInfo } from './page-info';
import type { SdkWorkPageData } from './sdk-work-page-data';

/** Applied model configuration profiles list response. */
export interface ModelConfigurationSummaryListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkPageData & { items: ModelConfigurationSummaryRecord[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
