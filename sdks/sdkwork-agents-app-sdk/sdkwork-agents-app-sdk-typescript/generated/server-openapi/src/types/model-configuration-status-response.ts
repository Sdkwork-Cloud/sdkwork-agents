import type { ModelConfigurationStatusRecord } from './model-configuration-status-record';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Stored profile plus provider-native config read-back response. */
export interface ModelConfigurationStatusResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: ModelConfigurationStatusRecord; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
