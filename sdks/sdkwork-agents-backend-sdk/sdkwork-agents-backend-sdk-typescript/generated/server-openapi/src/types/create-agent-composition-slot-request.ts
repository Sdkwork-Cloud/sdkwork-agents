import type { AgentCompositionSlotKind } from './agent-composition-slot-kind';
import type { AgentCompositionTargetModule } from './agent-composition-target-module';

export interface CreateAgentCompositionSlotRequest {
  slotId: string;
  slotKind: AgentCompositionSlotKind;
  targetModule: AgentCompositionTargetModule;
  targetRef: string;
  targetVersionRef?: string | null;
  priority?: number;
  enabled?: boolean;
  policyJson?: string;
  requestedAt: string;
}
