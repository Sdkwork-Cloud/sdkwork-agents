import assert from "node:assert/strict";
import { pathToFileURL } from "node:url";

import type { SdkworkAgentsAppClient } from "@sdkwork/agents-pc-core/sdk/agentsAppSdkClient";

async function loadAgentChatService() {
  const moduleUrl = pathToFileURL(
    "./packages/sdkwork-agents-pc-agents/src/services/AgentChatService.ts",
  ).href;
  return import(moduleUrl) as Promise<typeof import("../packages/sdkwork-agents-pc-agents/src/services/AgentChatService.ts")>;
}

const fakeClient = {
  ai: {
    agents: {
      sessions: {
        async list() {
          return {
            items: [{ sessionId: "session.test.1", status: "active" }],
            pageInfo: { page: 1, pageSize: 10, hasMore: false },
          };
        },
        async create(_agentId: string, body: { title?: string }) {
          return {
            sessionId: "session.test.1",
            title: body.title ?? "Chat",
          };
        },
      },
      messages: {
        async list(_agentId: string, _sessionId: string) {
          return {
            items: [
              {
                messageId: "msg-1",
                role: "user",
                content: "hello",
                createdAt: "2026-06-01T00:00:00Z",
              },
            ],
            pageInfo: { page: 1, pageSize: 20, totalItems: "1", totalPages: 1 },
          };
        },
      },
    },
  },
  http: {
    async post(path: string, body: { content: string }) {
      assert.match(path, /\/messages\?stream=false$/u);
      return {
        assistantMessage: {
          messageId: "msg-2",
          role: "assistant",
          content: `echo:${body.content}`,
          createdAt: "2026-06-01T00:00:01Z",
        },
      };
    },
  },
} as unknown as SdkworkAgentsAppClient;

const { AgentChatService } = await loadAgentChatService();
const chat = new AgentChatService(() => fakeClient);

const sessionId = await chat.resolveOrCreateSession("agent.test", "Contract chat");
assert.equal(sessionId, "session.test.1");

const messages = await chat.listMessages("agent.test", sessionId);
assert.equal(messages.length, 1);
assert.equal(messages[0]?.role, "user");

const reply = await chat.sendMessage("agent.test", sessionId, "ping");
assert.equal(reply.role, "assistant");
assert.equal(reply.content, "echo:ping");

console.log("sdkwork agents pc agent chat service contract passed.");
