import type { ChatMessage } from '../sdk';

export interface ChatboxPcSession {
  id: string;
  messages: ChatMessage[];
  title: string;
  updatedAt: number;
}
