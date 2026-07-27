import type { Int64String } from './int64-string';

export interface SessionActivityFreshness {
  activityAt: string;
  source: 'session' | 'turn' | 'interaction' | 'runtime_binding' | 'user_state';
  observedAt: string | null;
  /** Expiry of the effective non-terminal evidence when an authoritative lease exists. */
  freshUntil: string | null;
  sessionVersion: Int64String;
  latestTurnVersion: Int64String | null;
  latestInteractionId: string | null;
  /** Per-identity version of the latest Interaction, including resolved-state tombstones. */
  latestInteractionVersion: Int64String | null;
  latestRuntimeBindingId: string | null;
  /** Per-identity version of the latest RuntimeBinding, including deactivation tombstones. */
  latestRuntimeBindingVersion: Int64String | null;
  pendingInteractionVersion: Int64String | null;
  currentRuntimeBindingVersion: Int64String | null;
  userStateVersion: Int64String | null;
}
