import type { AppliedAgentModelSelectionRecord } from './applied-agent-model-selection-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single applied Agent model selection response. */
export interface AppliedAgentModelSelectionResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AppliedAgentModelSelectionRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
