import type { AgentCompositionSlotKind } from './agent-composition-slot-kind';
import type { AgentCompositionTargetModule } from './agent-composition-target-module';
import type { AgentStatus } from './agent-status';
import type { Int64String } from './int64-string';

export interface AgentCompositionSlotRecord {
  id: Int64String;
  tenantId: Int64String;
  organizationId: Int64String;
  agentId: string;
  slotId: string;
  slotKind: AgentCompositionSlotKind;
  targetModule: AgentCompositionTargetModule;
  targetRef: string;
  targetVersionRef?: string | null;
  priority: string;
  enabled: boolean;
  policyJson: string;
  status: AgentStatus;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  deletedAt?: string;
}
