import type { Int64String } from './int64-string';

export interface ChangeAgentSessionRuntimeBindingStatusRequest {
  reason?: string;
  expectedVersion: Int64String;
  requestedAt: string;
}
