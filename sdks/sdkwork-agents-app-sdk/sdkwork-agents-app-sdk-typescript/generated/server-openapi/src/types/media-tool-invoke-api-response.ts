import type { MediaToolInvokeResponse } from './media-tool-invoke-response';

/** Media tool invocation result following SdkWorkApiResponse envelope. */
export interface MediaToolInvokeApiResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & MediaToolInvokeResponse;
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
