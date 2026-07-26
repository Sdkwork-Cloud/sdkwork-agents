import type { AgentProjectDriveAccessMode } from './agent-project-drive-access-mode';
import type { AgentProjectStatus } from './agent-project-status';
import type { AgentProjectVisibility } from './agent-project-visibility';
import type { Int64String } from './int64-string';

export interface AgentProjectRecord {
  id: Int64String;
  projectId: string;
  workspaceId: string;
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
  importSourceKind?: string | null;
  importSourceRef?: string | null;
  driveSpaceId?: string | null;
  driveRootEntryId?: string | null;
  driveLogicalPath?: string | null;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  archivedAt?: string;
}
