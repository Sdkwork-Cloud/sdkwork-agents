#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const imPcRoot = path.resolve(repoRoot, "../sdkwork-im/apps/sdkwork-im-pc");
const agentsPcRoot = path.resolve(repoRoot, "apps/sdkwork-agents-pc");
const scriptsDir = path.join(agentsPcRoot, "scripts");
const tests = [
  "agent-service-management-profile-contract.test.ts",
  "agent-service-scope-consistency-contract.test.ts",
];

fs.mkdirSync(scriptsDir, { recursive: true });

const replacements = [
  ["@sdkwork/im-pc-core/sdk/agentAppSdkClient", "@sdkwork/agents-pc-core/sdk/agentsAppSdkClient"],
  ["../packages/sdkwork-im-pc-chat/src/services/AgentService.ts", "../packages/sdkwork-agents-pc-agents/src/services/AgentService.ts"],
  ["./packages/sdkwork-im-pc-chat/src/services/AgentService.ts", "./packages/sdkwork-agents-pc-agents/src/services/AgentService.ts"],
];

for (const testFile of tests) {
  const source = path.join(imPcRoot, "scripts", testFile);
  const target = path.join(scriptsDir, testFile);
  let content = fs.readFileSync(source, "utf8");
  for (const [from, to] of replacements) {
    content = content.split(from).join(to);
  }
  fs.writeFileSync(target, content, "utf8");
}

const runnerPath = path.join(scriptsDir, "agent-contracts.test.mjs");
fs.writeFileSync(
  runnerPath,
  `#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptsDir = path.dirname(fileURLToPath(import.meta.url));
const files = ${JSON.stringify(tests)};

for (const file of files) {
  const result = spawnSync(process.execPath, ['--import', 'tsx', path.join(scriptsDir, file)], {
    stdio: 'inherit',
    cwd: path.resolve(scriptsDir, '..'),
  });
  if (result.status !== 0) process.exit(result.status ?? 1);
}
`,
  "utf8",
);

console.log(`Migrated ${tests.length} agent contract tests.`);
