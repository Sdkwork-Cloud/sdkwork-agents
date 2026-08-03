import type { AgentModelProviderId } from './agent-model-provider-id';
import type { SdkWorkResourceData } from './sdk-work-resource-data';

/** Single migrated model configuration profile response. */
export interface MigratedModelConfigurationResponse {
  /** Numeric success result code. MUST be 0 on HTTP 2xx JSON bodies. See API_SPEC.md 搂15.3. */
  code: 0;
  data: unknown & SdkWorkResourceData & { item: { profileId: string; engineId: AgentModelProviderId; agentId: string; configurationVersion: string; status: 'draft' | 'active' | 'deprecated' | 'archived'; migrationPlanId: string; }; };
  /** Server-owned request correlation id. Clients MUST NOT supply this value. */
  traceId: string;
}
