import type { Int64String } from './int64-string';

export interface CancelAgentChatTurnRequest {
  expectedVersion?: Int64String;
  requestedAt: string;
}
