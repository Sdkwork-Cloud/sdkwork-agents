import type { ChatMessage } from './types';

export interface ChatPcSession {
  id: string;
  messages: ChatMessage[];
  title: string;
  updatedAt: number;
}
