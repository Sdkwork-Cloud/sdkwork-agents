import type { AgentTurnExecutionResponse } from './agent-turn-execution-response';

/** Typed server-sent event data for one agent turn execution. */
export interface AgentTurnStreamEvent {
  eventType: 'delta' | 'completion';
  index?: number;
  delta?: string;
  response?: AgentTurnExecutionResponse;
}
