export interface SessionProviderIdentity {
  runtimeBindingId: string | null;
  providerBindingId: string | null;
  providerId: string | null;
  modelId: string | null;
  providerSessionId: string | null;
  providerSessionTreeId: string | null;
  providerParentSessionId: string | null;
  providerForkedFromSessionId: string | null;
}
