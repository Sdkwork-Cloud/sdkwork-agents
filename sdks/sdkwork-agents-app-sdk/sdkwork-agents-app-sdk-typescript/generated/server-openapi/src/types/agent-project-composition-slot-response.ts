import type { AgentProjectCompositionSlotRecord } from './agent-project-composition-slot-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single project composition slot response. */
export interface AgentProjectCompositionSlotResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentProjectCompositionSlotRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
