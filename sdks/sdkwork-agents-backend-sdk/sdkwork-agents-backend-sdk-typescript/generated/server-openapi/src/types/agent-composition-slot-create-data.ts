import type { AgentCompositionSlotKind } from './agent-composition-slot-kind';
import type { AgentCompositionTargetModule } from './agent-composition-target-module';
import type { Int64String } from './int64-string';

export interface AgentCompositionSlotCreateData {
  tenantId: Int64String;
  organizationId: Int64String;
  slotId: string;
  slotKind: AgentCompositionSlotKind;
  targetModule: AgentCompositionTargetModule;
  targetRef: string;
  targetVersionRef?: string | null;
  priority?: number;
  enabled?: boolean;
  policyJson?: string;
}
