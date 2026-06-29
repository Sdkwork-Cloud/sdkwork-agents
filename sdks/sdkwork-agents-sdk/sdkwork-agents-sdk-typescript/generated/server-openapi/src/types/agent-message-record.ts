import type { Int64String } from './int64-string';

export interface AgentMessageRecord {
  messageId: string;
  sessionId: string;
  agentId: string;
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  contentType?: string;
  status: 'sent' | 'delivered' | 'read' | 'failed' | 'cancelled';
  sequence: Int64String;
  inputTokens?: Int64String;
  outputTokens?: Int64String;
  modelId?: string;
  providerId?: string;
  parentMessageId?: string;
  createdAt: string;
  updatedAt: string;
}
