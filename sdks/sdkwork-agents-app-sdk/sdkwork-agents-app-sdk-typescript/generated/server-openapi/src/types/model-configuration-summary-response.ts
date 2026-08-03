import type { ModelConfigurationSummaryRecord } from './model-configuration-summary-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single applied model configuration profile response. */
export interface ModelConfigurationSummaryResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: ModelConfigurationSummaryRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
