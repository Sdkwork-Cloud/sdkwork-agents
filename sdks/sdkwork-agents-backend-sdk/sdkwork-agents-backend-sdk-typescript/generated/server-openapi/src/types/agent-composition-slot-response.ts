import type { AgentCompositionSlotRecord } from './agent-composition-slot-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single agent composition slot response following SdkWorkApiResponse envelope. */
export interface AgentCompositionSlotResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md §15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & Record<string, unknown>;
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
