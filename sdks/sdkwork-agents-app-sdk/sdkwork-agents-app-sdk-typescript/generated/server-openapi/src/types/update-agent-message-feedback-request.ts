import type { Int64String } from './int64-string';

/** Supply rating to create or update feedback, or clearFeedback to remove it. */
export interface UpdateAgentMessageFeedbackRequest {
  expectedVersion?: Int64String;
  rating?: 'up' | 'down';
  clearFeedback?: boolean;
  reasonCode?: string;
  comment?: string;
}
