import type { AgentTurnExecutionResponse } from './agent-turn-execution-response';
import type { AgentTurnRuntimeEvent } from './agent-turn-runtime-event';

/** Typed server-sent event data for one agent turn execution. */
export interface AgentTurnStreamEvent {
  eventType: 'event' | 'delta' | 'completion';
  event?: AgentTurnRuntimeEvent;
  index?: number;
  delta?: string;
  response?: AgentTurnExecutionResponse;
}
