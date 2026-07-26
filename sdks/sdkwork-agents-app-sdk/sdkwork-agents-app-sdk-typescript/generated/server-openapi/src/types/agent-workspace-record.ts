import type { AgentWorkspaceStatus } from './agent-workspace-status';
import type { Int64String } from './int64-string';

export interface AgentWorkspaceRecord {
  id: Int64String;
  workspaceId: string;
  tenantId: Int64String;
  organizationId: Int64String;
  ownerUserId: Int64String;
  name: string;
  description?: string | null;
  isDefault: boolean;
  status: AgentWorkspaceStatus;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  archivedAt?: string;
}
