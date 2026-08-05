import type { Int64String } from './int64-string';
import type { ProjectSessionSynchronizationIssue } from './project-session-synchronization-issue';

export interface ProjectSessionSynchronizationResult {
  /** completed when the result was served from the refresh cache, accepted when a cold synchronization was enqueued on a background worker, pending when a synchronization is already running. */
  status: 'completed' | 'accepted' | 'pending';
  projectId: string;
  synchronizedSessionCount: Int64String;
  skippedSessionCount: Int64String;
  failedSessionCount: Int64String;
  issues: ProjectSessionSynchronizationIssue[];
}
