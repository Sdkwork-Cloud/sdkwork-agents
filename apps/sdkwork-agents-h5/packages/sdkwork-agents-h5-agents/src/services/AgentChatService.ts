import { sendAgentChatMessageSync } from "@sdkwork/agents-h5-core/sdk/agentsAppSdkClient";
import {
  getAgentsAppSdkClientWithSession,
  type SdkworkAgentsAppClient,
} from "@sdkwork/agents-h5-core/sdk/agentsAppSdkClient";
import { extractOffsetPageInfo, type OffsetPageInfo } from "@sdkwork/agents-h5-core/sdk/pagination";

import { extractArray, extractResourceRecord, isRecord } from "./sdkEnvelope";

export interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "system" | "tool";
  content: string;
  createdAt: string;
}

export interface ChatMessageListPage {
  items: ChatMessage[];
  pageInfo: OffsetPageInfo;
}

/** Matches backend `CHAT_CONTEXT_MESSAGE_LIMIT` for one interactive chat page. */
const CHAT_MESSAGE_PAGE_SIZE = 50;

function pickString(record: Record<string, unknown> | undefined, keys: string[]): string | undefined {
  if (!record) {
    return undefined;
  }
  for (const key of keys) {
    const value = record[key];
    if (typeof value === "string" && value.trim()) {
      return value.trim();
    }
  }
  return undefined;
}

function toChatMessage(record: Record<string, unknown>): ChatMessage {
  const messageId = pickString(record, ["messageId", "message_id", "id"]);
  if (!messageId) {
    throw new Error("Chat message did not include messageId.");
  }

  return {
    id: messageId,
    role: (pickString(record, ["role"]) as ChatMessage["role"]) ?? "assistant",
    content: pickString(record, ["content"]) ?? "",
    createdAt: pickString(record, ["createdAt", "created_at"]) ?? new Date().toISOString(),
  };
}

function toSessionId(record: Record<string, unknown>): string {
  return pickString(record, ["sessionId", "session_id", "id"]) ?? "";
}

function readSessionStatus(record: Record<string, unknown>): string | undefined {
  return pickString(record, ["status"]);
}

export class AgentChatService {
  constructor(
    private readonly getClient: () => SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession,
  ) {}

  async createSession(agentId: string, title?: string): Promise<string> {
    const response = await this.getClient().ai.agents.sessions.create(agentId, {
      title: title?.trim() || "Chat",
      requestedAt: new Date().toISOString(),
    });
    const session = extractResourceRecord(response);
    const sessionId = toSessionId(session);
    if (!sessionId) {
      throw new Error("Chat session create did not return sessionId.");
    }
    return sessionId;
  }

  async listSessions(agentId: string, page = 1, pageSize = 10): Promise<string[]> {
    const response = await this.getClient().ai.agents.sessions.list(agentId, { page, pageSize });
    return extractArray(response)
      .map((item) => (isRecord(item) ? toSessionId(item) : ""))
      .filter((sessionId) => sessionId.length > 0);
  }

  /** Reuse the latest active session when present; otherwise create one. */
  async resolveOrCreateSession(agentId: string, title?: string): Promise<string> {
    const response = await this.getClient().ai.agents.sessions.list(agentId, {
      page: 1,
      pageSize: 10,
    });
    const sessions = extractArray(response).filter(isRecord);
    const reusable = sessions.find((session) => {
      const status = readSessionStatus(session);
      return !status || status === "active";
    });
    if (reusable) {
      const sessionId = toSessionId(reusable);
      if (sessionId) {
        return sessionId;
      }
    }
    return this.createSession(agentId, title);
  }

  /** Load one server page for interactive chat history (`PAGINATION_SPEC.md` §8). */
  async listMessagesPage(
    agentId: string,
    sessionId: string,
    page = 1,
  ): Promise<ChatMessageListPage> {
    const response = await this.getClient().ai.agents.messages.list(agentId, sessionId, {
      page,
      pageSize: CHAT_MESSAGE_PAGE_SIZE,
    });
    return {
      items: this.normalizeMessages(response),
      pageInfo: extractOffsetPageInfo(response),
    };
  }

  /** Load the newest transcript window (last offset page when history spans multiple pages). */
  async loadRecentMessages(agentId: string, sessionId: string): Promise<ChatMessageListPage> {
    const probe = await this.listMessagesPage(agentId, sessionId, 1);
    const targetPage = probe.pageInfo.totalPages > 0 ? probe.pageInfo.totalPages : 1;
    if (targetPage === 1) {
      return probe;
    }
    return this.listMessagesPage(agentId, sessionId, targetPage);
  }

  async listMessages(agentId: string, sessionId: string): Promise<ChatMessage[]> {
    const page = await this.loadRecentMessages(agentId, sessionId);
    return page.items;
  }

  async sendMessage(
    agentId: string,
    sessionId: string,
    content: string,
    modelId?: string,
  ): Promise<ChatMessage> {
    const body = {
      content: content.trim(),
      contentType: "text/plain",
      requestedAt: new Date().toISOString(),
      ...(modelId ? { modelId } : {}),
    };
    const response = await sendAgentChatMessageSync(
      this.getClient(),
      agentId,
      sessionId,
      body,
    );
    const completion = extractResourceRecord(response);
    const assistantRecord = isRecord(completion.assistantMessage)
      ? completion.assistantMessage
      : isRecord(completion.assistant_message)
        ? completion.assistant_message
        : undefined;
    if (!assistantRecord) {
      throw new Error("Chat completion did not return assistantMessage.");
    }
    return toChatMessage(assistantRecord);
  }

  private normalizeMessages(response: unknown): ChatMessage[] {
    return extractArray(response)
      .map((item) => (isRecord(item) ? toChatMessage(item) : undefined))
      .filter((item): item is ChatMessage => Boolean(item))
      .sort((left, right) => left.createdAt.localeCompare(right.createdAt));
  }
}

export const agentChatService = new AgentChatService();
