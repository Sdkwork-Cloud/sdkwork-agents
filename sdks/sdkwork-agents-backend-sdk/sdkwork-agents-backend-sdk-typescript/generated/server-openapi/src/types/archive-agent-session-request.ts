import type { Int64String } from './int64-string';

export interface ArchiveAgentSessionRequest {
  tenantId: Int64String;
  expectedVersion?: Int64String;
  requestedAt: string;
}
