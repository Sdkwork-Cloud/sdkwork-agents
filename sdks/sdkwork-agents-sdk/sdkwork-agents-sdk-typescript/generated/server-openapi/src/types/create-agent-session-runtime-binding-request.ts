export interface CreateAgentSessionRuntimeBindingRequest {
  runtimeBindingId?: string;
  runtimeLocationId?: string;
  hostMode: string;
  transportKind: string;
  providerBindingId: string;
  modelId: string;
  providerId: string;
  providerSessionId?: string;
  providerSessionTreeId?: string;
  providerParentSessionId?: string;
  providerForkedFromSessionId?: string;
  requestedAt: string;
}
