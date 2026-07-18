import type { ChatMessage } from '../sdk/types';
import type { AgentsDriveMediaResource } from '@sdkwork/agents-pc-core/sdk';

export interface ChatServiceOptions {
  model: string;
  messages: ChatMessage[];
  signal?: AbortSignal;
  onMessageUpdate: (text: string) => void;
  onComplete?: () => void;
  onError?: (error: string) => void;
}

const CHATBOX_AGENT_ID = 'agent.chatbox.default';
let chatboxSessionId: string | null = null;
let chatAgentPort: ChatAgentPort | null = null;

export interface ChatboxAgentConfig {
  id: string;
  name: string;
  description: string;
  type: 'normal';
  model: string;
  systemPrompt: string;
  welcomeMessage: string;
}

export interface ChatAgentPort {
  getAgent(agentId: string): Promise<{ model?: string } | null>;
  createAgent(agent: ChatboxAgentConfig): Promise<unknown>;
  updateAgent(agentId: string, patch: { model: string }): Promise<unknown>;
  resolveOrCreateSession(agentId: string, title: string): Promise<string>;
  sendMessage(
    agentId: string,
    sessionId: string,
    content: string,
    model: string,
    media?: AgentsDriveMediaResource[],
  ): Promise<{ content: string }>;
}

export function configureChatAgentPort(port: ChatAgentPort): void {
  chatAgentPort = port;
  chatboxSessionId = null;
}

function requireChatAgentPort(): ChatAgentPort {
  if (!chatAgentPort) {
    throw new Error('Chat agent port is not configured.');
  }
  return chatAgentPort;
}

function defaultAgent(model: string): ChatboxAgentConfig {
  return {
    id: CHATBOX_AGENT_ID,
    name: 'SDKWork Chatbox',
    description: 'SDKWork Agents PC built-in conversational assistant.',
    type: 'normal',
    model,
    systemPrompt: 'You are SDKWork Chatbox. Provide accurate, concise, secure, and useful answers.',
    welcomeMessage: 'How can I help?',
  };
}

async function ensureChatboxAgent(model: string): Promise<void> {
  const port = requireChatAgentPort();
  const current = await port.getAgent(CHATBOX_AGENT_ID);
  if (!current) {
    await port.createAgent(defaultAgent(model));
    return;
  }
  if (model && current.model !== model) {
    await port.updateAgent(CHATBOX_AGENT_ID, { model });
  }
}

async function resolveSession(model: string): Promise<string> {
  await ensureChatboxAgent(model);
  if (!chatboxSessionId) {
    chatboxSessionId = await requireChatAgentPort().resolveOrCreateSession(
      CHATBOX_AGENT_ID,
      'SDKWork Chatbox',
    );
  }
  return chatboxSessionId;
}

export class ChatService {
  static async streamChat(options: ChatServiceOptions): Promise<void> {
    if (options.signal?.aborted) {
      options.onError?.('AbortError');
      return;
    }

    const latest = options.messages.at(-1);
    if (!latest || latest.role !== 'user') {
      options.onError?.('A user message is required.');
      return;
    }

    try {
      const sessionId = await resolveSession(options.model);
      if (options.signal?.aborted) {
        options.onError?.('AbortError');
        return;
      }
      const response = await requireChatAgentPort().sendMessage(
        CHATBOX_AGENT_ID,
        sessionId,
        latest.text || latest.mediaResources?.map((item) => item.fileName).join(', ') || 'Attachment',
        options.model,
        latest.mediaResources,
      );
      if (options.signal?.aborted) {
        options.onError?.('AbortError');
        return;
      }
      options.onMessageUpdate(response.content);
      options.onComplete?.();
    } catch (error) {
      console.error('Agents chat request failed', error);
      options.onError?.('Agents chat request failed.');
    }
  }
}
