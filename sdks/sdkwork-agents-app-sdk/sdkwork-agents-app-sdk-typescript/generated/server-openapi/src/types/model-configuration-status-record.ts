import type { AgentModelProviderId } from './agent-model-provider-id';

export interface ModelConfigurationStatusRecord {
  profileId: string;
  engineId: AgentModelProviderId;
  agentId: string;
  providerScope: string;
  status: 'draft' | 'active' | 'deprecated' | 'archived';
  /** Provider-level read-back state of the native config surface. */
  materialization: 'unsupported' | 'not_materialized' | 'materialized' | 'diverged';
  /** Drift state derived by comparing the native config surface with the stored profile. */
  derivedState: 'materialized' | 'diverged' | 'not_materialized' | 'unsupported';
  expectedBaseUrl?: string | null;
  expectedDefaultModel?: string | null;
  effectiveBaseUrl?: string | null;
  effectiveDefaultModel?: string | null;
  credentialConfigured: boolean;
  issues: string[];
}
