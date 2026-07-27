import type { Int64String } from './int64-string';

export interface UpdateAgentSessionRuntimeBindingRequest {
  runtimeLocationId?: string;
  clearRuntimeLocation?: boolean;
  hostMode?: string;
  transportKind?: string;
  providerBindingId?: string;
  modelId?: string;
  providerId?: string;
  providerSessionId?: string;
  providerSessionTreeId?: string;
  providerParentSessionId?: string;
  providerForkedFromSessionId?: string;
  expectedVersion: Int64String;
  requestedAt: string;
}
