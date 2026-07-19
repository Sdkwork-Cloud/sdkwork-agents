import type { Int64String } from './int64-string';

export interface AppUpdateAgentSessionRequest {
  expectedVersion?: Int64String;
  title?: string;
  projectId?: string;
  clearProject?: boolean;
}
