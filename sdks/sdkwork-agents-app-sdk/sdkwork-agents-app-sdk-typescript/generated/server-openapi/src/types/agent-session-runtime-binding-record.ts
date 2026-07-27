import type { AgentSessionRuntimeBindingStatus } from './agent-session-runtime-binding-status';
import type { Int64String } from './int64-string';

export interface AgentSessionRuntimeBindingRecord {
  runtimeBindingId: string;
  tenantId: Int64String;
  organizationId: Int64String;
  sessionId: string;
  runtimeLocationId?: string | null;
  hostMode: string;
  transportKind: string;
  providerBindingId: string;
  modelId: string;
  providerId: string;
  providerSessionId?: string | null;
  providerSessionTreeId?: string | null;
  providerParentSessionId?: string | null;
  providerForkedFromSessionId?: string | null;
  status: AgentSessionRuntimeBindingStatus;
  isCurrent: boolean;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  activatedAt?: string | null;
  deactivatedAt?: string | null;
}
