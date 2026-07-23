import type { AgentResourceUserStateRecord } from './agent-resource-user-state-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single per-user resource state following SdkWorkApiResponse envelope. */
export interface AgentResourceUserStateResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentResourceUserStateRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
