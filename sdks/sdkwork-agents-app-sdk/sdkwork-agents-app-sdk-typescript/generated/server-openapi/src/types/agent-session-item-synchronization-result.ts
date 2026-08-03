import type { Int64String } from './int64-string';

export type AgentSessionItemSynchronizationStatus =
  | 'engine-unavailable'
  | 'imported'
  | 'no-active-binding'
  | 'not-provider-session';

export interface AgentSessionItemSynchronizationResult {
  status: AgentSessionItemSynchronizationStatus;
  importedItemCount: Int64String;
}
