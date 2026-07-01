import {
  getAgentsAppSdkClientWithSession,
  type SdkworkAgentsAppClient,
} from "@sdkwork/agents-h5-core/sdk/agentsAppSdkClient";

import { extractArray, extractResourceRecord, isRecord } from "./sdkEnvelope";

export interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "system" | "tool";
  content: string;
  createdAt: string;
}

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
  return {
    id: pickString(record, ["messageId", "message_id", "id"]) ?? crypto.randomUUID(),
    role: (pickString(record, ["role"]) as ChatMessage["role"]) ?? "assistant",
    content: pickString(record, ["content"]) ?? "",
    createdAt: pickString(record, ["createdAt", "created_at"]) ?? new Date().toISOString(),
  };
}

function toSessionId(record: Record<string, unknown>): string {
  return pickString(record, ["sessionId", "session_id", "id"]) ?? "";
}

export class AgentChatService {
  constructor(private readonly client: SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession()) {}

  async createSession(agentId: string, title?: string): Promise<string> {
    const response = await this.client.ai.agents.sessions.create(agentId, {
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

  async listMessages(agentId: string, sessionId: string): Promise<ChatMessage[]> {
    const response = await this.client.ai.agents.messages.list(agentId, sessionId, {
      page: 1,
      pageSize: 100,
    });
    return extractArray(response)
      .map((item) => (isRecord(item) ? toChatMessage(item) : undefined))
      .filter((item): item is ChatMessage => Boolean(item))
      .sort((left, right) => left.createdAt.localeCompare(right.createdAt));
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
    const path = `/ai/agents/${encodeURIComponent(agentId)}/sessions/${encodeURIComponent(sessionId)}/messages?stream=false`;
    const response = await this.client.http.post<unknown>(
      path,
      body,
      undefined,
      undefined,
      "application/json",
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
}

export const agentChatService = new AgentChatService();
