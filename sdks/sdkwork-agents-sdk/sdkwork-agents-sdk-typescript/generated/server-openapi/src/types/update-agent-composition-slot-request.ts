import type { AgentCompositionSlotKind } from './agent-composition-slot-kind';
import type { AgentCompositionTargetModule } from './agent-composition-target-module';
import type { Int64String } from './int64-string';

export interface UpdateAgentCompositionSlotRequest {
  expectedVersion?: Int64String;
  slotKind?: AgentCompositionSlotKind;
  targetModule?: AgentCompositionTargetModule;
  targetRef?: string;
  targetVersionRef?: string | null;
  priority?: number;
  enabled?: boolean;
  policyJson?: string;
  requestedAt: string;
}
