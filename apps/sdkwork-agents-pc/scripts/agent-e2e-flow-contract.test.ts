import assert from "node:assert/strict";
import { pathToFileURL } from "node:url";

import type { SdkworkAgentsAppClient } from "@sdkwork/agents-pc-core/sdk/agentsAppSdkClient";
import type * as AgentServiceModule from "../packages/sdkwork-agents-pc-agents/src/services/AgentService.ts";

type AgentServiceExports = typeof AgentServiceModule;

async function loadAgentServiceModule(): Promise<AgentServiceExports> {
  const moduleUrl = pathToFileURL(
    "./packages/sdkwork-agents-pc-agents/src/services/AgentService.ts",
  ).href;
  return import(moduleUrl) as Promise<AgentServiceExports>;
}

async function loadAgentChatService() {
  const moduleUrl = pathToFileURL(
    "./packages/sdkwork-agents-pc-agents/src/services/AgentChatService.ts",
  ).href;
  return import(moduleUrl) as Promise<
    typeof import("../packages/sdkwork-agents-pc-agents/src/services/AgentChatService.ts")
  >;
}

const agentId = "agent.e2e.flow";
const compositionSlots: Array<Record<string, unknown>> = [];

const fakeClient = {
  ai: {
    agents: {
      compositionSlots: {
        async list() {
          return {
            items: compositionSlots,
            pageInfo: { page: 1, pageSize: 100, totalItems: String(compositionSlots.length), totalPages: 1 },
          };
        },
        async create(_agentId: string, body: { data?: Record<string, unknown> }) {
          const slot = { slotId: "slot.e2e", version: "1", ...(body.data ?? {}) };
          compositionSlots.push(slot);
          return slot;
        },
        async delete() {
          return { accepted: true };
        },
      },
      async create(body: Record<string, unknown>) {
        return {
          agentId,
          tenantId: "100001",
          organizationId: "0",
          ownerUserId: "100",
          code: agentId,
          displayName: String(body.displayName ?? "E2E Flow Agent"),
          description: "E2E contract agent",
          manifest: { description: "E2E" },
          managementProfile: body.managementProfile ?? {
            model: "model.openai.gpt-4o",
            systemPrompt: "You are a test agent.",
            type: "independent",
          },
          status: "active",
          visibility: "private",
          version: "1",
          createdAt: "2026-06-01T00:00:00Z",
          updatedAt: "2026-06-01T00:00:00Z",
        };
      },
      sessions: {
        async create(_id: string, body: { title?: string }) {
          return {
            sessionId: "session.e2e.1",
            title: body.title ?? "E2E Chat",
          };
        },
      },
      messages: {
        async list() {
          return {
            items: [],
            pageInfo: { page: 1, pageSize: 100, totalItems: "0", totalPages: 1 },
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
          messageId: "msg.e2e.assistant",
          role: "assistant",
          content: `reply:${body.content}`,
          createdAt: "2026-06-01T00:00:01Z",
        },
      };
    },
  },
} as unknown as SdkworkAgentsAppClient;

const { createSdkworkAgentService } = await loadAgentServiceModule();
const { AgentChatService } = await loadAgentChatService();

const agentService = createSdkworkAgentService(() => fakeClient);
const chatService = new AgentChatService(() => fakeClient);

const created = await agentService.createAgent({
  name: "E2E Flow Agent",
  model: "model.openai.gpt-4o",
  systemPrompt: "You are a test agent.",
});
assert.equal(created.id, agentId);

const sessionId = await chatService.createSession(agentId, "E2E session");
assert.equal(sessionId, "session.e2e.1");

const reply = await chatService.sendMessage(agentId, sessionId, "hello e2e");
assert.equal(reply.role, "assistant");
assert.equal(reply.content, "reply:hello e2e");

console.log("sdkwork agents pc e2e flow contract passed (create agent → chat).");
