import type { AgentProjectDriveAccessMode } from './agent-project-drive-access-mode';
import type { AgentProjectVisibility } from './agent-project-visibility';

export interface CreateAgentProjectRequest {
  projectId?: string;
  workspaceId?: string;
  name: string;
  description?: string;
  visibility?: AgentProjectVisibility;
  driveAccessMode?: AgentProjectDriveAccessMode;
  defaultAgentId?: string;
  defaultModelId?: string;
}
