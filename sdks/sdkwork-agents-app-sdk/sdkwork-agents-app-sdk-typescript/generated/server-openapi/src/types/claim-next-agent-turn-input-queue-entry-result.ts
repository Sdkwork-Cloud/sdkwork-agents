import type { AgentTurnInputQueueEntry } from './agent-turn-input-queue-entry';

export interface ClaimNextAgentTurnInputQueueEntryResult {
  outcome: 'claimed' | 'busy' | 'blocked' | 'active_turn' | 'empty';
  entry?: AgentTurnInputQueueEntry | null;
  claimToken?: string | null;
}
