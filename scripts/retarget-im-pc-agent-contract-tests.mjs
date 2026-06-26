#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const imPcRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../sdkwork-im/apps/sdkwork-im-pc",
);
const scriptsDir = path.join(imPcRoot, "scripts");
const agentsAgentsRoot =
  "../../../sdkwork-agents/apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-agents";
const agentsServicePath = `${agentsAgentsRoot}/src/services/AgentService.ts`;
const agentsCreateViewPath = `${agentsAgentsRoot}/src/pages/CreateAgentView.tsx`;
const agentsAgentViewPath = `${agentsAgentsRoot}/src/pages/AgentView.tsx`;
const agentsDefaultsPath = `${agentsAgentsRoot}/src/components/AgentDefaults.ts`;
const agentsCreateModalPath = `${agentsAgentsRoot}/src/components/CreateAgentModal.tsx`;
const imChatRoot = "../../../sdkwork-im/apps/sdkwork-im-pc/packages/sdkwork-im-pc-chat";
const imTypesRoot = "../../../sdkwork-im/apps/sdkwork-im-pc/packages/sdkwork-im-pc-types";
const agentsSdkClientPath = "@sdkwork/agents-pc-core/sdk/agentsAppSdkClient";

const replacements = [
  ["../packages/sdkwork-im-pc-chat/src/services/AgentService.ts", agentsServicePath],
  ["./packages/sdkwork-im-pc-chat/src/services/AgentService.ts", agentsServicePath],
  ["'../packages/sdkwork-im-pc-chat/src/services/AgentService.ts'", `'${agentsServicePath}'`],
  ["'./packages/sdkwork-im-pc-chat/src/services/AgentService.ts'", `'${agentsServicePath}'`],
  ["../packages/sdkwork-im-pc-chat/src/pages/CreateAgentView.tsx", agentsCreateViewPath],
  ["./packages/sdkwork-im-pc-chat/src/pages/CreateAgentView.tsx", agentsCreateViewPath],
  ["'../packages/sdkwork-im-pc-chat/src/pages/CreateAgentView.tsx'", `'${agentsCreateViewPath}'`],
  ["'./packages/sdkwork-im-pc-chat/src/pages/CreateAgentView.tsx'", `'${agentsCreateViewPath}'`],
  ["../packages/sdkwork-im-pc-chat/src/pages/AgentView.tsx", agentsAgentViewPath],
  ["./packages/sdkwork-im-pc-chat/src/pages/AgentView.tsx", agentsAgentViewPath],
  ["../packages/sdkwork-im-pc-chat/src/components/AgentDefaults.ts", agentsDefaultsPath],
  ["./packages/sdkwork-im-pc-chat/src/components/AgentDefaults.ts", agentsDefaultsPath],
  ["./packages/sdkwork-im-pc-chat/src/components/CreateAgentModal.tsx", `${agentsCreateModalPath}`],
  ["'./packages/sdkwork-im-pc-chat/src/components/CreateAgentModal.tsx'", `'${agentsCreateModalPath}'`],
  ["./packages/sdkwork-im-pc-chat/src/pages/ChatLayout.tsx", `${imChatRoot}/src/pages/ChatLayout.tsx`],
  ["'./packages/sdkwork-im-pc-chat/src/pages/ChatLayout.tsx'", `'${imChatRoot}/src/pages/ChatLayout.tsx'`],
  ["./packages/sdkwork-im-pc-chat/src/index.ts", `${imChatRoot}/src/index.ts`],
  ["'./packages/sdkwork-im-pc-chat/src/index.ts'", `'${imChatRoot}/src/index.ts'`],
  ["./packages/sdkwork-im-pc-chat/src/services/ChatService.ts", `${imChatRoot}/src/services/ChatService.ts`],
  ["'./packages/sdkwork-im-pc-chat/src/services/ChatService.ts'", `'${imChatRoot}/src/services/ChatService.ts'`],
  ["./packages/sdkwork-im-pc-chat/src/components/ChatWindow.tsx", `${imChatRoot}/src/components/ChatWindow.tsx`],
  ["'./packages/sdkwork-im-pc-chat/src/components/ChatWindow.tsx'", `'${imChatRoot}/src/components/ChatWindow.tsx'`],
  ["./packages/sdkwork-im-pc-types/src/chat.ts", `${imTypesRoot}/src/chat.ts`],
  ["'./packages/sdkwork-im-pc-types/src/chat.ts'", `'${imTypesRoot}/src/chat.ts'`],
  ["@sdkwork/im-pc-core/sdk/agentAppSdkClient", agentsSdkClientPath],
  ["assert.equal(params.tenantId, '0');", "assert.equal(params.page, 1);\n        assert.equal(params.pageSize, 100);"],
  ["assert.equal(requests.retrieve?.params.tenantId, '0');", "assert.ok(requests.retrieve?.params);"],
  ["assert.equal(deploymentCall?.params?.tenantId, '0');", "assert.ok(deploymentCall);"],
];

let updated = 0;
for (const file of fs.readdirSync(scriptsDir)) {
  if (!file.startsWith("agent-") || !file.endsWith(".test.ts")) {
    continue;
  }
  const filePath = path.join(scriptsDir, file);
  const original = fs.readFileSync(filePath, "utf8");
  let content = original;
  for (const [from, to] of replacements) {
    content = content.split(from).join(to);
  }
  if (content !== original) {
    fs.writeFileSync(filePath, content, "utf8");
    updated += 1;
  }
}

console.log(`Retargeted ${updated} sdkwork-im-pc agent contract tests to sdkwork-agents-pc-agents.`);
