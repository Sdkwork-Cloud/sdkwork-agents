import { completeAgentTurn } from "@sdkwork/agents-h5-core/sdk/agentsAppSdkClient";
import {
  getAgentsAppSdkClientWithSession,
  type AgentSessionItemRecord,
  type AgentSessionRecord,
  type SdkworkAgentsAppClient,
} from "@sdkwork/agents-h5-core/sdk/agentsAppSdkClient";
import { sha256Hash, uuid } from "@sdkwork/utils";
import { MAX_LIST_PAGE_SIZE, toOffsetPageInfo, type OffsetPageInfo } from "@sdkwork/agents-h5-core/sdk/pagination";

import { resolveChatRuntimeModel } from "./RuntimeCatalogService";

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

/** Matches the bounded server context window for one interactive session page. */
const SESSION_ITEM_PAGE_SIZE = 50;

function toChatMessage(record: AgentSessionItemRecord): ChatMessage {
  if (!record.itemId) {
    throw new Error("Agent session item did not include itemId.");
  }

  const role: ChatMessage["role"] = record.kind === "user_input"
    ? "user"
    : record.kind === "system_instruction"
        || record.kind === "status_notice"
        || record.kind === "error_notice"
      ? "system"
      : record.kind === "tool_call" || record.kind === "tool_result"
        ? "tool"
        : "assistant";

  return {
    id: record.itemId,
    role,
    content: record.content ?? "",
    createdAt: record.createdAt,
  };
}

function toSessionId(record: AgentSessionRecord): string {
  return record.sessionId;
}

function findAssistantOutput(
  items: AgentSessionItemRecord[],
): AgentSessionItemRecord | undefined {
  for (let index = items.length - 1; index >= 0; index -= 1) {
    const item = items[index];
    if (item.kind === "assistant_output") {
      return item;
    }
  }
  return undefined;
}

export class AgentChatService {
  constructor(
    private readonly getClient: () => SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession,
  ) {}

  async createSession(agentId: string, title?: string): Promise<string> {
    const idempotencyKey = uuid();
    const normalizedTitle = title?.trim() || "Agent session";
    const session = await this.getClient().ai.agents.sessions.create(agentId, {
      sessionKind: "assistant",
      entrySurface: "h5",
      title: normalizedTitle,
      idempotencyKey,
      payloadHash: `sha256:${sha256Hash(JSON.stringify({
        sessionKind: "assistant",
        entrySurface: "h5",
        title: normalizedTitle,
      }))}`,
      requestedAt: new Date().toISOString(),
    });
    const sessionId = toSessionId(session);
    if (!sessionId) {
      throw new Error("Chat session create did not return sessionId.");
    }
    return sessionId;
  }

  async listSessions(agentId: string, pageSize = 10): Promise<string[]> {
    const response = await this.getClient().ai.agents.sessions.list(agentId, { pageSize });
    return (response.items as AgentSessionRecord[])
      .map(toSessionId)
      .filter((sessionId) => sessionId.length > 0);
  }

  /** Reuse the latest active session when present; otherwise create one. */
  async resolveOrCreateSession(agentId: string, title?: string): Promise<string> {
    const response = await this.getClient().ai.agents.sessions.list(agentId, {
      pageSize: 10,
    });
    const sessions = response.items as AgentSessionRecord[];
    const reusable = sessions.find((session) => {
      return session.status === "active";
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
    cursor?: string,
  ): Promise<ChatMessageListPage> {
    const response = await this.getClient().ai.agents.sessionItems.list(agentId, sessionId, {
      cursor,
      pageSize: SESSION_ITEM_PAGE_SIZE,
    });
    return {
      items: this.normalizeMessages(response.items as AgentSessionItemRecord[]),
      pageInfo: toOffsetPageInfo(response.pageInfo),
    };
  }

  /** Load the newest transcript window (last offset page when history spans multiple pages). */
  async loadRecentMessages(agentId: string, sessionId: string): Promise<ChatMessageListPage> {
    let page = await this.listMessagesPage(agentId, sessionId);
    for (
      let guard = 0;
      guard < MAX_LIST_PAGE_SIZE && page.pageInfo.hasMore && page.pageInfo.nextCursor;
      guard += 1
    ) {
      page = await this.listMessagesPage(agentId, sessionId, page.pageInfo.nextCursor);
    }
    return page;
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
    const requestId = uuid();
    // Resolve the runtime model id for the turn request. No local provider
    // binding or API key is required: chat turns route through the cloudrouter
    // account-pool gateway using the caller's auth token (sessions that
    // already carry a runtime binding keep the local binding chain).
    const runtimeModel = await resolveChatRuntimeModel(modelId, this.getClient());
    const contentType = "text/plain";
    const payloadHash = `sha256:${sha256Hash(JSON.stringify({
      content: content.trim(),
      contentType,
      requestedModelId: runtimeModel.id,
    }))}`;
    const body = {
      content: content.trim(),
      contentType,
      turnMode: "interactive" as const,
      requestedAt: new Date().toISOString(),
      idempotencyKey: requestId,
      payloadHash,
      clientRequestId: requestId,
      requestedModelId: runtimeModel.id,
    };
    const completion = await completeAgentTurn(
      this.getClient(),
      agentId,
      sessionId,
      body,
    );
    const assistantRecord = findAssistantOutput(completion.items);
    if (!assistantRecord) {
      throw new Error("Agent turn did not return an assistant_output item.");
    }
    return toChatMessage(assistantRecord);
  }

  private normalizeMessages(items: AgentSessionItemRecord[]): ChatMessage[] {
    return items
      .map(toChatMessage)
      .sort((left, right) => left.createdAt.localeCompare(right.createdAt));
  }
}

export let agentChatService = createSdkworkAgentChatService();

let activeChatClientGetter: (() => SdkworkAgentsAppClient) | undefined;

/**
 * Host injection hook for the chat transport (mirrors `configureAgentService`).
 *
 * Host apps such as sdkwork-im-h5 construct their own agents app SDK client
 * (their token manager + gateway root) and inject it here so agent sessions
 * never fall back to the standalone agents-h5 session storage / base URL.
 */
export function configureAgentChatService(
  getClient?: () => SdkworkAgentsAppClient,
): AgentChatService {
  activeChatClientGetter = getClient;
  agentChatService = createSdkworkAgentChatService(getClient);
  return agentChatService;
}

export function createSdkworkAgentChatService(
  getClient?: () => SdkworkAgentsAppClient,
): AgentChatService {
  return new AgentChatService(getClient);
}
