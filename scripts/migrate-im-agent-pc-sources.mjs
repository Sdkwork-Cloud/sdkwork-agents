#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const imChatRoot = path.resolve(repoRoot, "../sdkwork-im/apps/sdkwork-im-pc/packages/sdkwork-im-pc-chat/src");
const agentsPkgRoot = path.resolve(repoRoot, "apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-agents/src");

const files = [
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
  "components/ModalWrapper.tsx",
  "components/Toast.tsx",
  "components/MessageInput.tsx",
  "components/EmojiPicker.tsx",
  "services/AgentService.ts",
  "services/DefaultAvatarService.ts",
];

const replacements = [
  ["@sdkwork/im-pc-commons", "@sdkwork/agents-pc-commons"],
  ["@sdkwork/im-pc-core/sdk/agentAppSdkClient", "@sdkwork/agents-pc-core/sdk/agentsAppSdkClient"],
  ["@sdkwork/agent-app-sdk", "@sdkwork/agents-app-sdk"],
  [
    "import { knowledgeSelectionService, type KnowledgeBase } from '@sdkwork/knowledgebase-pc-knowledge';",
    "import { knowledgeSelectionService, type KnowledgeBase } from '../services/KnowledgeSelectionService';",
  ],
  [
    "import { voiceService, VoiceConfig } from '@sdkwork/im-pc-voice';",
    "import { voiceService, VoiceConfig } from '../services/VoiceService';",
  ],
  ["owner: { name: config.author ?? 'sdkwork-im-pc' }", "owner: { name: config.author ?? 'sdkwork-agents-pc' }"],
];

function applyReplacements(content) {
  let next = content;
  for (const [from, to] of replacements) {
    next = next.split(from).join(to);
  }
  return next;
}

for (const file of files) {
  const source = path.join(imChatRoot, file);
  const target = path.join(agentsPkgRoot, file);
  if (!fs.existsSync(source)) {
    console.warn(`skip missing source: ${file}`);
    continue;
  }
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, applyReplacements(fs.readFileSync(source, "utf8")), "utf8");
}

console.log(`Synced ${files.length} agent PC sources from sdkwork-im.`);
