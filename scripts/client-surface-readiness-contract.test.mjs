import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function mustExist(relativePath) {
  const absolute = path.join(root, relativePath);
  assert.ok(fs.existsSync(absolute), `${relativePath} must exist`);
  return fs.readFileSync(absolute, "utf8");
}

function mustExport(relativePath, exportName) {
  const content = mustExist(relativePath);
  assert.match(
    content,
    new RegExp(`export\\s+(function|const|class|type)\\s+${exportName}\\b|export\\s*\\{[^}]*\\b${exportName}\\b`),
    `${relativePath} must export ${exportName}`,
  );
}

const pcAgents = "apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-agents/src";
const h5Agents = "apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-agents/src";
const pcApp = "apps/sdkwork-agents-pc/src";
const h5App = "apps/sdkwork-agents-h5/src";
const pcCoreSdk = "apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-core/src/sdk";
const h5CoreSdk = "apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-core/src/sdk";

mustExport(`${pcApp}/components/AuthGate.tsx`, "AuthGate");
mustExport(`${h5App}/components/AuthGate.tsx`, "AuthGate");
mustExport(`${pcAgents}/pages/AgentChatView.tsx`, "AgentChatView");
mustExport(`${h5Agents}/pages/AgentChatView.tsx`, "AgentChatView");
mustExport(`${pcAgents}/services/AgentChatService.ts`, "AgentChatService");
mustExport(`${h5Agents}/services/AgentChatService.ts`, "AgentChatService");
mustExport(`${pcCoreSdk}/runtimeEnv.ts`, "readRuntimeEnv");
mustExport(`${h5CoreSdk}/runtimeEnv.ts`, "readRuntimeEnv");

const h5AgentsPkg = JSON.parse(
  mustExist("apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-agents/package.json"),
);
assert.equal(
  h5AgentsPkg.dependencies?.["@sdkwork/agents-app-sdk"],
  undefined,
  "H5 agents capability package must not depend on @sdkwork/agents-app-sdk directly",
);

const mpAgentsPage = mustExist("apps/sdkwork-agents-mini-program/src/pages/agents/index.js");
assert.doesNotMatch(
  mustExist("apps/sdkwork-agents-mini-program/src/pages/agents/index.wxml"),
  /<web-view/u,
  "agents index page must be native (WebView moved to agents-h5)",
);
assert.match(mpAgentsPage, /getAgentsMpSdkClient/u, "agents index must load agents via runtime SDK");
assert.match(
  mustExist("apps/sdkwork-agents-mini-program/src/pages/agents/index.wxml"),
  /agents-h5/u,
  "agents index wxml must link to H5 fallback page",
);

for (const [label, relativePath] of [
  ["PC", `${pcApp}/App.tsx`],
  ["H5", `${h5App}/App.tsx`],
]) {
  const appSource = mustExist(relativePath);
  assert.match(appSource, /AuthGate/u, `${label} App.tsx must wrap routes with AuthGate`);
  assert.match(appSource, /AgentChatView/u, `${label} App.tsx must wire AgentChatView`);
  assert.match(appSource, /CHAT_ROUTE/u, `${label} App.tsx must wire chat route constant`);
}

console.log("client surface readiness contract passed.");
