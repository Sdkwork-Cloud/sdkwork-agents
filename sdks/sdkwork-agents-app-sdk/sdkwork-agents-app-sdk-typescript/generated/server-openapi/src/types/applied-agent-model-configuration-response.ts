import type { AppliedAgentModelConfigurationRecord } from './applied-agent-model-configuration-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single applied Agent model configuration response. */
export interface AppliedAgentModelConfigurationResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AppliedAgentModelConfigurationRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
