import type { AgentInteractionRecord } from './agent-interaction-record';
import type { AgentResourceUserStateRecord } from './agent-resource-user-state-record';
import type { AgentSessionRecord } from './agent-session-record';
import type { AgentSessionRuntimeBindingRecord } from './agent-session-runtime-binding-record';
import type { AgentTurnRecord } from './agent-turn-record';
import type { SessionActivityFreshness } from './session-activity-freshness';
import type { SessionPresentationPhase } from './session-presentation-phase';
import type { SessionProviderActivityObservation } from './session-provider-activity-observation';
import type { SessionProviderIdentity } from './session-provider-identity';

export interface SessionActivitySummary {
  session: AgentSessionRecord;
  latestTurn: AgentTurnRecord | null;
  pendingInteraction: AgentInteractionRecord | null;
  currentRuntimeBinding: AgentSessionRuntimeBindingRecord | null;
  /** Latest RuntimeBinding by activity time, including failed or deactivated tombstone state. */
  latestRuntimeBinding: AgentSessionRuntimeBindingRecord | null;
  userState: AgentResourceUserStateRecord | null;
  providerIdentity: SessionProviderIdentity;
  freshness: SessionActivityFreshness;
  providerActivity: SessionProviderActivityObservation | null;
  presentationPhase: SessionPresentationPhase;
}
