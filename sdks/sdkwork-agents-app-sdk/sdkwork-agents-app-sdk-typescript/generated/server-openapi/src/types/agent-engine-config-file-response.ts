import type { AgentEngineConfigFileView } from './agent-engine-config-file-view';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Provider-native configuration file content following the SdkWorkApiResponse envelope. */
export interface AgentEngineConfigFileResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentEngineConfigFileView; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
