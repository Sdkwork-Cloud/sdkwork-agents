import type { AgentProjectDriveAccessMode } from './agent-project-drive-access-mode';
import type { AgentProjectVisibility } from './agent-project-visibility';
import type { Int64String } from './int64-string';

export interface UpdateAgentProjectRequest {
  expectedVersion?: Int64String;
  name?: string;
  description?: string | null;
  visibility?: AgentProjectVisibility;
  driveAccessMode?: AgentProjectDriveAccessMode;
  defaultAgentId?: string | null;
  defaultModelId?: string | null;
}
