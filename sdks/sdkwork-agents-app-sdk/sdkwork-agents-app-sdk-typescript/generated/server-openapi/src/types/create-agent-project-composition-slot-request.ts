import type { AgentCompositionSlotKind } from './agent-composition-slot-kind';
import type { AgentCompositionTargetModule } from './agent-composition-target-module';

export interface CreateAgentProjectCompositionSlotRequest {
  slotId: string;
  slotKind: AgentCompositionSlotKind;
  targetModule: AgentCompositionTargetModule;
  targetRef: string;
  targetVersionRef?: string;
  priority?: number;
  enabled?: boolean;
  policyJson?: string;
}
