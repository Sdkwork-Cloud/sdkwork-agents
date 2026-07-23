import type { Int64String } from './int64-string';

export interface AnswerAgentInteractionRequest {
  answer: string;
  selectedOptionValue?: string;
  rejected: boolean;
  claimToken: string;
  fencingToken: Int64String;
  expectedVersion: Int64String;
  requestedAt: string;
}
