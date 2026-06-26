#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const agentsRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const agentsPcRoot = path.join(agentsRoot, "apps/sdkwork-agents-pc");
const imScriptsDir = path.resolve(agentsRoot, "../sdkwork-im/apps/sdkwork-im-pc/scripts");
const imScriptsNodeModules = path.join(imScriptsDir, "node_modules");
const requireFromAgentsPc = createRequire(path.join(agentsPcRoot, "package.json"));
const typescriptRoot = path.dirname(requireFromAgentsPc.resolve("typescript/package.json"));
const typescriptLink = path.join(imScriptsNodeModules, "typescript");

if (!fs.existsSync(imScriptsNodeModules)) {
  fs.mkdirSync(imScriptsNodeModules, { recursive: true });
}
if (!fs.existsSync(typescriptLink)) {
  fs.symlinkSync(typescriptRoot, typescriptLink, "junction");
}

const tests = fs
  .readdirSync(imScriptsDir)
  .filter((file) => file.startsWith("agent-") && file.endsWith(".test.ts"))
  .sort();

let failed = 0;
for (const file of tests) {
  process.stdout.write(`Running ${file}... `);
  const result = spawnSync(
    "pnpm",
    ["exec", "tsx", "--tsconfig", "tsconfig.app.json", path.join(imScriptsDir, file)],
    {
      cwd: agentsPcRoot,
      shell: true,
      stdio: "pipe",
      encoding: "utf8",
    },
  );
  if (result.status === 0) {
    process.stdout.write("passed\n");
    continue;
  }
  failed += 1;
  process.stdout.write("failed\n");
  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }
}

if (failed > 0) {
  console.error(`${failed} IM agent contract test(s) failed.`);
  process.exit(1);
}

console.log(`All ${tests.length} IM agent contract tests passed.`);
