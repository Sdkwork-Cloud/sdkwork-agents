import type { Int64String } from './int64-string';

export interface AppCloseAgentSessionRequest {
  expectedVersion?: Int64String;
  requestedAt: string;
}
