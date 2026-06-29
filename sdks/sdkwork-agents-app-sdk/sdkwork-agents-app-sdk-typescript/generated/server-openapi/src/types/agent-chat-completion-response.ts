import type { AgentMessageRecord } from './agent-message-record';
import type { AgentSessionRecord } from './agent-session-record';

export interface AgentChatCompletionResponse {
  data: Record<string, unknown>;
}
