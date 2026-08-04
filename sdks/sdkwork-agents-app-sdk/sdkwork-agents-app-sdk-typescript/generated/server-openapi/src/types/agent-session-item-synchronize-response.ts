import type { AgentSessionItemSynchronizationResult } from './agent-session-item-synchronization-result';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Provider Session history synchronization outcome following the SDKWork resource envelope. */
export interface AgentSessionItemSynchronizeResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentSessionItemSynchronizationResult; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
