export type MessageRole = 'user' | 'model';

export interface ChatMessage {
  id: string;
  role: MessageRole;
  text: string;
  images?: string[];
  mediaResources?: import('@sdkwork/agents-pc-core/sdk').AgentsDriveMediaResource[];
  feedback?: 'up' | 'down';
  feedbackVersion?: string;
}

export interface ChatSession {
  id: string;
  title: string;
  messages: ChatMessage[];
  updatedAt: number;
  version: string;
  projectId?: string;
  pinned?: boolean;
  userStateVersion?: string;
}
