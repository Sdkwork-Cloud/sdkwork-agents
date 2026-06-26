import type { AgentCompositionSlotRecord } from './agent-composition-slot-record';

export interface AgentCompositionSlotListResponse {
  data: Record<string, unknown>;
  requestId?: string;
}
