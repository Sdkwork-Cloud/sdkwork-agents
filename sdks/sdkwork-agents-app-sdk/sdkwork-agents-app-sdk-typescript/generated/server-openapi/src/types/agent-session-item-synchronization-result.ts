import type { Int64String } from './int64-string';

/** Outcome of the best-effort provider transcript synchronization. `imported` means the provider transcript was loaded and reconciled; the other statuses describe why the synchronization did not run, in which case the persisted item window read through agents.sessionItems.list is authoritative. */
export interface AgentSessionItemSynchronizationResult {
  status: 'imported' | 'not-provider-session' | 'no-active-binding' | 'engine-unavailable';
  importedItemCount: Int64String;
}
