import type { AgentCompositionSlotKind } from './agent-composition-slot-kind';
import type { AgentCompositionTargetModule } from './agent-composition-target-module';
import type { Int64String } from './int64-string';

export interface UpdateAgentProjectCompositionSlotRequest {
  expectedVersion: Int64String;
  slotKind?: AgentCompositionSlotKind;
  targetModule?: AgentCompositionTargetModule;
  targetRef?: string;
  targetVersionRef?: string;
  clearTargetVersionRef?: boolean;
  priority?: number;
  enabled?: boolean;
  policyJson?: string;
}
