import type { Int64String } from './int64-string';

export interface ProjectSessionSynchronizationResult {
  projectId: string;
  synchronizedSessionCount: Int64String;
}
