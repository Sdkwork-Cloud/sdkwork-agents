#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const agentsRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const agentsPcScripts = path.join(agentsRoot, "apps/sdkwork-agents-pc/scripts");
const imPcScripts = path.resolve(agentsRoot, "../sdkwork-im/apps/sdkwork-im-pc/scripts");

const pathReplacements = [
  ["'../packages/sdkwork-agents-pc-agents/", "'../../../sdkwork-agents/apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-agents/"],
  ['"../packages/sdkwork-agents-pc-agents/', '"../../../sdkwork-agents/apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-agents/'],
  ["'./packages/sdkwork-agents-pc-agents/", "'../../../sdkwork-agents/apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-agents/"],
  ['"./packages/sdkwork-agents-pc-agents/', '"../../../sdkwork-agents/apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-agents/'],
  ["'../packages/sdkwork-agents-pc-core/", "'../../../sdkwork-agents/apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-core/"],
  ['"../packages/sdkwork-agents-pc-core/', '"../../../sdkwork-agents/apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-core/'],
];

let synced = 0;
for (const file of fs.readdirSync(agentsPcScripts)) {
  if (!file.startsWith("agent-") || !file.endsWith(".test.ts")) {
    continue;
  }
  const sourcePath = path.join(agentsPcScripts, file);
  const targetPath = path.join(imPcScripts, file);
  let content = fs.readFileSync(sourcePath, "utf8");
  for (const [from, to] of pathReplacements) {
    content = content.split(from).join(to);
  }
  fs.writeFileSync(targetPath, content, "utf8");
  synced += 1;
}

console.log(`Synced ${synced} agent contract tests from sdkwork-agents-pc to sdkwork-im-pc.`);
