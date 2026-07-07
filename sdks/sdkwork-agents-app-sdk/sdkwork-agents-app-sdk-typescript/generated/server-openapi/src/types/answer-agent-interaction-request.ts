import type { Int64String } from './int64-string';

export interface AnswerAgentInteractionRequest {
  tenantId?: Int64String;
  answer: string;
  optionLabel?: string;
  rejected: boolean;
  expectedVersion: Int64String;
  requestedAt: string;
}
