import type { Int64String } from './int64-string';

export interface ProjectSessionSynchronizationIssue {
  code: 'non_root_session' | 'duplicate_provider_session' | 'invalid_provider_session_identity' | 'invalid_runtime_binding_identity' | 'invalid_synchronization_timestamp' | 'runtime_identity_reconciliation_failed' | 'session_reconciliation_failed' | 'inventory_item_limit_exceeded' | 'synchronization_time_budget_exceeded';
  count: Int64String;
  disposition: 'skipped' | 'failed';
}
