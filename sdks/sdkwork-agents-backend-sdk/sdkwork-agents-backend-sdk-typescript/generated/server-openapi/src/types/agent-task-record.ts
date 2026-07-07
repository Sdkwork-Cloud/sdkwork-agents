import type { Int64String } from './int64-string';

export interface AgentTaskRecord {
  taskId: string;
  agentId: string;
  title: string;
  prompt: string;
  status: string;
  externalRef?: string;
  metadataJson?: string;
  version: Int64String;
  createdAt: string;
  updatedAt: string;
  startedAt?: string;
  completedAt?: string;
  cancelledAt?: string;
}
