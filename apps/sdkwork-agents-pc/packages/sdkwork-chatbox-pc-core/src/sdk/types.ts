export type MessageRole = 'user' | 'model';

export interface ChatMessage {
  id: string;
  role: MessageRole;
  text: string;
  images?: string[];
  mediaResources?: import('@sdkwork/agents-pc-core/sdk').AgentsDriveMediaResource[];
}

export interface ChatSession {
  id: string;
  title: string;
  messages: ChatMessage[];
  updatedAt: number;
}
