import type { AgentWorkspaceRecord } from './agent-workspace-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single Workspace response following SdkWorkApiResponse envelope. */
export interface AgentWorkspaceResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentWorkspaceRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
