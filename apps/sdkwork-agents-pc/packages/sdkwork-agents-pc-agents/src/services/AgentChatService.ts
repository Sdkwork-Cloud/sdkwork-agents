import { sendAgentChatMessageSync } from "@sdkwork/agents-pc-core/sdk/agentsAppSdkClient";
import {
  getAgentsAppSdkClientWithSession,
  type SdkworkAgentsAppClient,
} from "@sdkwork/agents-pc-core/sdk/agentsAppSdkClient";
import { extractOffsetPageInfo, type OffsetPageInfo } from "@sdkwork/agents-pc-core/sdk/pagination";
import type { AgentsDriveMediaResource } from "@sdkwork/agents-pc-core/sdk/driveUploadService";
import { uuid } from "@sdkwork/utils";

import { extractArray, extractResourceRecord, isRecord } from "./sdkEnvelope";

export interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "system" | "tool";
  content: string;
  createdAt: string;
  mediaResources?: AgentsDriveMediaResource[];
}

export interface ChatMessageListPage {
  items: ChatMessage[];
  pageInfo: OffsetPageInfo;
}

export interface ChatSessionSummary {
  id: string;
  title: string;
  updatedAt: string;
  version: string;
  projectId?: string;
}

export interface ChatSessionUserState {
  sessionId: string;
  pinned: boolean;
  version: string;
}

export interface ChatMessageFeedback {
  messageId: string;
  rating?: "up" | "down";
  version: string;
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
    mediaResources: Array.isArray(record.mediaResources)
      ? record.mediaResources
          .filter(isRecord)
          .map((resource): AgentsDriveMediaResource | undefined => {
            const id = pickString(resource, ["id"]);
            const kind = pickString(resource, ["kind"]);
            const source = pickString(resource, ["source"]);
            const uri = pickString(resource, ["uri"]);
            if (!id || !kind || source !== "drive" || !uri) {
              return undefined;
            }
            return {
              id,
              kind: kind as AgentsDriveMediaResource["kind"],
              source: "drive",
              uri,
              url: pickString(resource, ["url"]),
              fileName: pickString(resource, ["fileName"]),
              mimeType: pickString(resource, ["mimeType"]),
              sizeBytes: pickString(resource, ["sizeBytes"]),
              metadata: isRecord(resource.metadata) ? resource.metadata : {},
            };
          })
          .filter((resource): resource is AgentsDriveMediaResource => Boolean(resource))
      : [],
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

  async createSession(agentId: string, title?: string, sessionId?: string): Promise<string> {
    const response = await this.getClient().ai.agents.sessions.create(agentId, {
      ...(sessionId ? { sessionId } : {}),
      title: title?.trim() || "Chat",
      requestedAt: new Date().toISOString(),
    });
    const session = extractResourceRecord(response);
    const createdSessionId = toSessionId(session);
    if (!createdSessionId) {
      throw new Error("Chat session create did not return sessionId.");
    }
    return createdSessionId;
  }

  async listSessions(agentId: string, page = 1, pageSize = 10): Promise<string[]> {
    const response = await this.getClient().ai.agents.sessions.list(agentId, { page, pageSize });
    return extractArray(response)
      .map((item) => (isRecord(item) ? toSessionId(item) : ""))
      .filter((sessionId) => sessionId.length > 0);
  }

  async listSessionSummaries(
    agentId: string,
    page = 1,
    pageSize = 50,
  ): Promise<ChatSessionSummary[]> {
    const response = await this.getClient().ai.agents.sessions.list(agentId, { page, pageSize });
    return extractArray(response)
      .filter(isRecord)
      .map((record) => ({
        id: toSessionId(record),
        title: pickString(record, ["title"]) ?? "New chat",
        updatedAt:
          pickString(record, ["updatedAt", "updated_at"])
          ?? new Date(0).toISOString(),
        version: pickString(record, ["version"]) ?? "0",
        projectId: pickString(record, ["projectId", "project_id"]),
      }))
      .filter((session) => session.id.length > 0);
  }

  async updateSession(
    agentId: string,
    sessionId: string,
    patch: { title?: string; projectId?: string; clearProject?: boolean; expectedVersion?: string },
  ): Promise<ChatSessionSummary> {
    const response = await this.getClient().ai.agents.sessions.update(agentId, sessionId, patch);
    const record = extractResourceRecord(response);
    return {
      id: toSessionId(record),
      title: pickString(record, ["title"]) ?? "New chat",
      updatedAt: pickString(record, ["updatedAt", "updated_at"]) ?? new Date().toISOString(),
      version: pickString(record, ["version"]) ?? patch.expectedVersion ?? "0",
      projectId: pickString(record, ["projectId", "project_id"]),
    };
  }

  async deleteSession(agentId: string, sessionId: string): Promise<void> {
    await this.getClient().ai.agents.sessions.delete(agentId, sessionId);
  }

  async listSessionUserStates(
    agentId: string,
    pinnedOnly = false,
  ): Promise<ChatSessionUserState[]> {
    const response = await this.getClient().ai.agents.sessionUserStates.list(agentId, {
      page: 1,
      pageSize: 200,
      pinnedOnly,
    });
    return extractArray(response)
      .filter(isRecord)
      .map((record) => ({
        sessionId: pickString(record, ["resourceId", "resource_id"]) ?? "",
        pinned: Boolean(pickString(record, ["pinnedAt", "pinned_at"])),
        version: pickString(record, ["version"]) ?? "0",
      }))
      .filter((state) => state.sessionId.length > 0);
  }

  async updateSessionUserState(
    agentId: string,
    sessionId: string,
    patch: { pinned: boolean; expectedVersion?: string },
  ): Promise<ChatSessionUserState> {
    const response = await this.getClient().ai.agents.sessionUserStates.update(
      agentId,
      sessionId,
      patch,
    );
    const record = extractResourceRecord(response);
    return {
      sessionId: pickString(record, ["resourceId", "resource_id"]) ?? sessionId,
      pinned: Boolean(pickString(record, ["pinnedAt", "pinned_at"])),
      version: pickString(record, ["version"]) ?? patch.expectedVersion ?? "0",
    };
  }

  async listMessageFeedback(
    agentId: string,
    sessionId: string,
  ): Promise<ChatMessageFeedback[]> {
    const response = await this.getClient().ai.agents.messageFeedback.list(agentId, sessionId, {
      page: 1,
      pageSize: CHAT_MESSAGE_PAGE_SIZE,
    });
    return extractArray(response)
      .filter(isRecord)
      .map((record) => ({
        messageId: pickString(record, ["messageId", "message_id"]) ?? "",
        rating: pickString(record, ["rating"]) as ChatMessageFeedback["rating"],
        version: pickString(record, ["version"]) ?? "0",
      }))
      .filter((feedback) => feedback.messageId.length > 0);
  }

  async updateMessageFeedback(
    agentId: string,
    sessionId: string,
    messageId: string,
    patch: {
      rating?: "up" | "down";
      clearFeedback?: boolean;
      expectedVersion?: string;
    },
  ): Promise<ChatMessageFeedback> {
    const response = await this.getClient().ai.agents.messageFeedback.update(
      agentId,
      sessionId,
      messageId,
      patch,
    );
    const record = extractResourceRecord(response);
    return {
      messageId: pickString(record, ["messageId", "message_id"]) ?? messageId,
      rating: pickString(record, ["deletedAt", "deleted_at"])
        ? undefined
        : pickString(record, ["rating"]) as ChatMessageFeedback["rating"],
      version: pickString(record, ["version"]) ?? patch.expectedVersion ?? "0",
    };
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

  async resolveOrCreateNamedSession(
    agentId: string,
    sessionId: string,
    title?: string,
  ): Promise<string> {
    const sessionIds = await this.listSessions(agentId, 1, 200);
    if (sessionIds.includes(sessionId)) {
      return sessionId;
    }
    return this.createSession(agentId, title, sessionId);
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
    media?: AgentsDriveMediaResource | AgentsDriveMediaResource[],
  ): Promise<ChatMessage> {
    const mediaResources = media ? (Array.isArray(media) ? media : [media]) : [];
    const requestId = uuid();
    const body = {
      content: content.trim(),
      contentType: mediaResources[0]?.mimeType ?? "text/plain",
      ...(mediaResources.length > 0 ? {
        mediaResources: mediaResources.map((item) => ({
            id: item.id,
            kind: item.kind,
            source: item.source,
            uri: item.uri,
            fileName: item.fileName,
            mimeType: item.mimeType,
            sizeBytes: item.sizeBytes,
            metadata: item.metadata,
          })),
      } : {}),
      requestedAt: new Date().toISOString(),
      idempotencyKey: requestId,
      clientRequestId: requestId,
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
