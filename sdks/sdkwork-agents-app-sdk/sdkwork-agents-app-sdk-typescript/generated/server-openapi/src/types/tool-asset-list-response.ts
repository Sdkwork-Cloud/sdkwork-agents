import type { ToolAssetView } from './tool-asset-view';

/** Generated media asset list following SdkWorkApiResponse envelope; the list is returned as a bare array under data. */
export interface ToolAssetListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & ToolAssetView[];
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
