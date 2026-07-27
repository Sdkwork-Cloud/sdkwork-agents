export interface SessionProviderActivityObservation {
  providerSessionId: string;
  state: 'idle' | 'working' | 'waiting' | 'failed' | null;
  freshness: 'fresh' | 'stale' | 'unsupported' | 'unavailable';
  evidenceKind: 'provider_status' | 'provider_event' | 'provider_lock' | 'provider_process' | null;
  interactionHint: 'approval_required' | 'user_input_required' | null;
  observedAt: string | null;
  freshUntil: string | null;
}
