#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const imChatRoot = path.resolve(repoRoot, "../sdkwork-im/apps/sdkwork-im-pc/packages/sdkwork-im-pc-chat/src");

const agentOnlyFiles = [
  "pages/AgentView.tsx",
  "pages/CreateAgentView.tsx",
  "components/AgentDefaults.ts",
  "components/CreateAgentModal.tsx",
  "components/SelectKnowledgeModal.tsx",
  "components/SelectSkillsModal.tsx",
  "components/SelectToolsModal.tsx",
  "components/SelectModelPopover.tsx",
  "components/SelectVoiceModal.tsx",
  "components/EditBasicInfoModal.tsx",
  "services/AgentService.ts",
];

for (const file of agentOnlyFiles) {
  const target = path.join(imChatRoot, file);
  if (fs.existsSync(target)) {
    fs.unlinkSync(target);
    console.log(`removed ${file}`);
  }
}

console.log("Removed duplicated agent module sources from sdkwork-im-pc-chat.");
