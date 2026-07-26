import type { Int64String } from './int64-string';

export interface UpdateAgentWorkspaceRequest {
  expectedVersion: Int64String;
  name?: string;
  description?: string | null;
}
