import type { AgentProjectDriveAccessMode } from './agent-project-drive-access-mode';
import type { AgentProjectStatus } from './agent-project-status';
import type { AgentProjectVisibility } from './agent-project-visibility';
import type { Int64String } from './int64-string';

export interface AgentProjectRecord {
  id: Int64String;
  projectId: string;
  tenantId: Int64String;
  organizationId: Int64String;
  ownerUserId: Int64String;
  name: string;
  description?: string | null;
  visibility: AgentProjectVisibility;
  status: AgentProjectStatus;
  driveAccessMode: AgentProjectDriveAccessMode;
  defaultAgentId?: string | null;
  defaultModelId?: string | null;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  archivedAt?: string;
}
