export interface CreateAgentSessionRuntimeBindingRequest {
  runtimeBindingId?: string;
  runtimeLocationId?: string;
  hostMode: string;
  transportKind: string;
  providerBindingId: string;
  modelId: string;
  providerId: string;
  nativeSessionId?: string;
  nativeSessionTreeId?: string;
  nativeParentSessionId?: string;
  nativeForkedFromSessionId?: string;
  requestedAt: string;
}
