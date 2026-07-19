import type { AgentCompositionSlotKind } from './agent-composition-slot-kind';
import type { AgentCompositionTargetModule } from './agent-composition-target-module';
import type { Int64String } from './int64-string';

export interface AgentProjectCompositionSlotRecord {
  id: Int64String;
  tenantId: Int64String;
  organizationId: Int64String;
  projectId: string;
  slotId: string;
  slotKind: AgentCompositionSlotKind;
  targetModule: AgentCompositionTargetModule;
  targetRef: string;
  targetVersionRef?: string | null;
  priority: number;
  enabled: boolean;
  policyJson: string;
  createdBy: Int64String;
  updatedBy: Int64String;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
}
