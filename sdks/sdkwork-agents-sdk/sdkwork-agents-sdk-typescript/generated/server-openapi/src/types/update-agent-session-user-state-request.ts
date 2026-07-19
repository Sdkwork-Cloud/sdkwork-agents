import type { Int64String } from './int64-string';

/** At least one state mutation field other than expectedVersion is required. */
export interface UpdateAgentSessionUserStateRequest {
  expectedVersion?: Int64String;
  pinned?: boolean;
  hidden?: boolean;
  markOpened?: boolean;
  lastReadMessageSequence?: Int64String;
  customTitle?: string;
  clearCustomTitle?: boolean;
}
