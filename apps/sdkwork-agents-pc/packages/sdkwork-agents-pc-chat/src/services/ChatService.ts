import type { ChatMessage } from '../types';
import type { AgentsDriveMediaResource } from '@sdkwork/agents-pc-core/sdk/driveUploadService';
import { createSdkworkChatRequestContext } from '@sdkwork/agents-pc-core/session';

export interface ChatServiceOptions {
  sessionId: string;
  model: string;
  messages: ChatMessage[];
  signal?: AbortSignal;
  onMessageUpdate: (text: string) => void;
  onComplete?: (message?: { id: string }) => void;
  onError?: (error: string) => void;
}

const DEFAULT_CHAT_AGENT_ID = 'agent.chat.default';
let chatAgentPort: ChatAgentPort | null = null;

export interface ChatAgentConfig {
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
  createAgent(agent: ChatAgentConfig): Promise<unknown>;
  updateAgent(agentId: string, patch: { model: string }): Promise<unknown>;
  resolveOrCreateSession(agentId: string, sessionId: string, title: string): Promise<string>;
  listSessions(agentId: string): Promise<Array<{
    id: string;
    title: string;
    updatedAt: string;
    version: string;
    projectId?: string;
  }>>;
  updateSession(
    agentId: string,
    sessionId: string,
    patch: { title?: string; projectId?: string; clearProject?: boolean; expectedVersion?: string },
  ): Promise<{ id: string; title: string; updatedAt: string; version: string; projectId?: string }>;
  deleteSession(agentId: string, sessionId: string): Promise<void>;
  listSessionUserStates(agentId: string, pinnedOnly?: boolean): Promise<Array<{
    sessionId: string;
    pinned: boolean;
    version: string;
  }>>;
  updateSessionUserState(
    agentId: string,
    sessionId: string,
    patch: { pinned: boolean; expectedVersion?: string },
  ): Promise<{ sessionId: string; pinned: boolean; version: string }>;
  listMessageFeedback(agentId: string, sessionId: string): Promise<Array<{
    messageId: string;
    rating?: 'up' | 'down';
    version: string;
  }>>;
  updateMessageFeedback(
    agentId: string,
    sessionId: string,
    messageId: string,
    patch: { rating?: 'up' | 'down'; clearFeedback?: boolean; expectedVersion?: string },
  ): Promise<{ messageId: string; rating?: 'up' | 'down'; version: string }>;
  listMessages(
    agentId: string,
    sessionId: string,
  ): Promise<Array<{
    id: string;
    role: 'user' | 'assistant' | 'system' | 'tool';
    content: string;
    mediaResources?: AgentsDriveMediaResource[];
  }>>;
  resolveMediaPreviewUrl(driveUri: string): Promise<string>;
  sendMessage(
    agentId: string,
    sessionId: string,
    content: string,
    model: string,
    media?: AgentsDriveMediaResource[],
  ): Promise<{ id: string; content: string }>;
}

export function configureChatAgentPort(port: ChatAgentPort): void {
  chatAgentPort = port;
}

function requireChatAgentPort(): ChatAgentPort {
  if (!chatAgentPort) {
    throw new Error('Chat agent port is not configured.');
  }
  return chatAgentPort;
}

export type ChatAgentPermissionScopeReader = () => string[];

function readChatAgentPermissionScopeFromSession(): string[] {
  return createSdkworkChatRequestContext()?.permissionScope ?? [];
}

let chatAgentPermissionScopeReader: ChatAgentPermissionScopeReader =
  readChatAgentPermissionScopeFromSession;

/** Overrides where the caller's IAM permission scope is read from. */
export function configureChatAgentPermissionScopeReader(
  reader: ChatAgentPermissionScopeReader,
): void {
  chatAgentPermissionScopeReader = reader;
}

/**
 * Mirrors the backend `IamGatedPolicyProvider` grant rules: updating an agent
 * record (`agents.update`) requires `ai.agents.manage`, with `ai.*` and `*`
 * wildcards also granting it. Chat-only callers must not attempt the model
 * sync PATCH at all, since the API contract rejects it with 403.
 */
export function callerScopeGrantsAgentManage(scopes: string[]): boolean {
  return scopes.some(
    (scope) => scope === 'ai.agents.manage' || scope === 'ai.*' || scope === '*',
  );
}

function defaultAgent(model: string): ChatAgentConfig {
  return {
    id: DEFAULT_CHAT_AGENT_ID,
    name: 'SDKWork Agents',
    description: 'SDKWork Agents PC built-in conversational assistant.',
    type: 'normal',
    model,
    systemPrompt: 'You are SDKWork Agents. Provide accurate, concise, secure, and useful answers.',
    welcomeMessage: 'How can I help?',
  };
}

async function ensureChatAgent(model: string): Promise<void> {
  const port = requireChatAgentPort();
  const current = await port.getAgent(DEFAULT_CHAT_AGENT_ID);
  if (!current) {
    await port.createAgent(defaultAgent(model));
    return;
  }
  if (model && current.model !== model) {
    // The model is passed per message through sendMessage, so syncing the
    // stored default is only worthwhile for callers the backend allows to
    // update the agent. Without manage scope the PATCH would always 403.
    if (!callerScopeGrantsAgentManage(chatAgentPermissionScopeReader())) {
      return;
    }
    try {
      await port.updateAgent(DEFAULT_CHAT_AGENT_ID, { model });
    } catch (error) {
      // Best-effort: a failed model sync (e.g. a stale scope claim) must not
      // block session loading or chat.
      console.warn('Failed to sync the default chat agent model', error);
    }
  }
}

function canonicalSessionId(sessionId: string): string {
  const normalized = sessionId.trim().toLowerCase().replace(/[^a-z0-9_-]/gu, '-');
  return sessionId.startsWith('session.') ? sessionId : `session.${normalized}`;
}

async function resolveSession(model: string, localSessionId: string): Promise<string> {
  await ensureChatAgent(model);
  return requireChatAgentPort().resolveOrCreateSession(
    DEFAULT_CHAT_AGENT_ID,
    canonicalSessionId(localSessionId),
    'SDKWork Agents',
  );
}

export class ChatService {
  static async loadSessions(model: string): Promise<Array<{
    id: string;
    title: string;
    updatedAt: number;
    version: string;
    projectId?: string;
    messages: ChatMessage[];
  }>> {
    await ensureChatAgent(model);
    const port = requireChatAgentPort();
    const [sessions, userStates] = await Promise.all([
      port.listSessions(DEFAULT_CHAT_AGENT_ID),
      port.listSessionUserStates(DEFAULT_CHAT_AGENT_ID, true),
    ]);
    const userStateBySessionId = new Map(
      userStates.map((state) => [state.sessionId, state]),
    );
    return Promise.all(
      sessions.map(async (session) => {
        const userState = userStateBySessionId.get(session.id);
        const [messages, feedbackItems] = await Promise.all([
          port.listMessages(DEFAULT_CHAT_AGENT_ID, session.id),
          port.listMessageFeedback(DEFAULT_CHAT_AGENT_ID, session.id),
        ]);
        const feedbackByMessageId = new Map(
          feedbackItems.map((feedback) => [feedback.messageId, feedback]),
        );
        return {
          id: session.id,
          title: session.title,
          updatedAt: Date.parse(session.updatedAt) || 0,
          version: session.version,
          projectId: session.projectId,
          pinned: userState?.pinned ?? false,
          userStateVersion: userState?.version,
          messages: await Promise.all(messages.map(async (message) => {
            const feedback = feedbackByMessageId.get(message.id);
            const mediaResources = await Promise.all(
              (message.mediaResources ?? []).map(async (resource) => {
                try {
                  const url = await port.resolveMediaPreviewUrl(resource.uri);
                  return { ...resource, url };
                } catch {
                  return resource;
                }
              }),
            );
            return {
              id: message.id,
              role: message.role === 'assistant' ? 'model' : 'user',
              text: message.content,
              images: mediaResources
                .filter((resource) => resource.kind === 'image' && resource.url)
                .map((resource) => resource.url as string),
              mediaResources,
              feedback: feedback?.rating,
              feedbackVersion: feedback?.version,
            };
          })),
        };
      }),
    );
  }

  static async setSessionPinned(sessionId: string, pinned: boolean, version?: string) {
    return requireChatAgentPort().updateSessionUserState(
      DEFAULT_CHAT_AGENT_ID,
      canonicalSessionId(sessionId),
      {
        pinned,
        ...(version ? { expectedVersion: version } : {}),
      },
    );
  }

  static async setMessageFeedback(
    sessionId: string,
    messageId: string,
    rating: 'up' | 'down' | undefined,
    version?: string,
  ) {
    return requireChatAgentPort().updateMessageFeedback(
      DEFAULT_CHAT_AGENT_ID,
      canonicalSessionId(sessionId),
      messageId,
      rating
        ? { rating, ...(version ? { expectedVersion: version } : {}) }
        : { clearFeedback: true, ...(version ? { expectedVersion: version } : {}) },
    );
  }

  static async renameSession(sessionId: string, title: string, version: string) {
    return requireChatAgentPort().updateSession(DEFAULT_CHAT_AGENT_ID, canonicalSessionId(sessionId), {
      title,
      ...(version ? { expectedVersion: version } : {}),
    });
  }

  static async moveSession(sessionId: string, projectId: string, version: string) {
    return requireChatAgentPort().updateSession(DEFAULT_CHAT_AGENT_ID, canonicalSessionId(sessionId), {
      projectId,
      ...(version ? { expectedVersion: version } : {}),
    });
  }

  static async deleteSession(sessionId: string): Promise<void> {
    await requireChatAgentPort().deleteSession(DEFAULT_CHAT_AGENT_ID, canonicalSessionId(sessionId));
  }

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
      const sessionId = await resolveSession(options.model, options.sessionId);
      if (options.signal?.aborted) {
        options.onError?.('AbortError');
        return;
      }
      const response = await requireChatAgentPort().sendMessage(
        DEFAULT_CHAT_AGENT_ID,
        sessionId,
        latest.text
          || latest.mediaResources?.map((item) => item.fileName ?? item.id).join(', ')
          || 'Attachment',
        options.model,
        latest.mediaResources,
      );
      if (options.signal?.aborted) {
        options.onError?.('AbortError');
        return;
      }
      options.onMessageUpdate(response.content);
      options.onComplete?.({ id: response.id });
    } catch (error) {
      console.error('Agents chat request failed', error);
      options.onError?.('Agents chat request failed.');
    }
  }
}
