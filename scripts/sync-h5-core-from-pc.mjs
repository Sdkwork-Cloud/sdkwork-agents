#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const pcCore = path.join(repoRoot, "apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-core/src");
const h5Core = path.join(repoRoot, "apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-core/src");

function copyAdapted(relativePath, replacements) {
  let content = fs.readFileSync(path.join(pcCore, relativePath), "utf8");
  for (const [from, to] of replacements) {
    content = content.split(from).join(to);
  }
  const target = path.join(h5Core, relativePath);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, content, "utf8");
}

copyAdapted("sdk/agentsAppSdkClient.ts", [
  ["SDKWORK_AGENTS_PC_", "SDKWORK_AGENTS_H5_"],
  ['platform: "pc"', 'platform: "h5"'],
]);
copyAdapted("session/session.ts", [
  ["sdkwork-agents-pc", "sdkwork-agents-h5"],
  ["SDKWORK_AGENTS_PC_", "SDKWORK_AGENTS_H5_"],
]);
copyAdapted("session/secureSessionStorage.ts", []);

console.log("Synced H5 core SDK/session files from PC.");
