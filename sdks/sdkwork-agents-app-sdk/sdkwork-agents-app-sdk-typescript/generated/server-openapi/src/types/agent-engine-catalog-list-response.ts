import type { AgentEngineCatalog } from './agent-engine-catalog';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Agent engine catalog response following SdkWorkApiResponse envelope; the catalog is returned as a single composite resource under data.item. */
export interface AgentEngineCatalogListResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: AgentEngineCatalog; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
