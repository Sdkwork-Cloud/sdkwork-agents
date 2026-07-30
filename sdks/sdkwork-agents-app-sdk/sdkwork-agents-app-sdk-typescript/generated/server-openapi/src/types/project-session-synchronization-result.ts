import type { Int64String } from './int64-string';
import type { ProjectSessionSynchronizationIssue } from './project-session-synchronization-issue';

export interface ProjectSessionSynchronizationResult {
  projectId: string;
  synchronizedSessionCount: Int64String;
  skippedSessionCount: Int64String;
  failedSessionCount: Int64String;
  issues: ProjectSessionSynchronizationIssue[];
}
