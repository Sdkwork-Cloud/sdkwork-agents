#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const pcAgentsRoot = path.resolve(repoRoot, "apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-agents");
const h5AgentsRoot = path.resolve(repoRoot, "apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-agents");

function copyDir(src, dest) {
  fs.mkdirSync(dest, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyDir(srcPath, destPath);
      continue;
    }
    if (!/\.(ts|tsx|json)$/.test(entry.name)) {
      continue;
    }
    let content = fs.readFileSync(srcPath, "utf8");
    content = content
      .split("@sdkwork/agents-pc-commons").join("@sdkwork/agents-h5-commons")
      .split("@sdkwork/agents-pc-core").join("@sdkwork/agents-h5-core")
      .split("sdkwork-agents-pc").join("sdkwork-agents-h5")
      .split("VITE_SDKWORK_AGENTS_PC_").join("VITE_SDKWORK_AGENTS_H5_");
    fs.writeFileSync(destPath, content, "utf8");
  }
}

if (fs.existsSync(h5AgentsRoot)) {
  fs.rmSync(h5AgentsRoot, { recursive: true, force: true });
}
copyDir(pcAgentsRoot, h5AgentsRoot);

const h5PackageJson = {
  name: "@sdkwork/agents-h5-agents",
  private: true,
  version: "0.1.0",
  type: "module",
  exports: {
    ".": "./src/index.ts",
  },
  dependencies: {
    "@sdkwork/agents-app-sdk": "workspace:*",
    "@sdkwork/agents-h5-commons": "workspace:*",
    "@sdkwork/agents-h5-core": "workspace:*",
    "@tiptap/extension-placeholder": "catalog:",
    "@tiptap/pm": "catalog:",
    "@tiptap/react": "catalog:",
    "@tiptap/starter-kit": "catalog:",
    "emoji-picker-react": "catalog:",
    "lucide-react": "catalog:",
    "motion": "catalog:",
    "react": "catalog:",
  },
};

fs.writeFileSync(path.join(h5AgentsRoot, "package.json"), `${JSON.stringify(h5PackageJson, null, 2)}\n`, "utf8");
console.log("Materialized sdkwork-agents-h5-agents from PC agents package.");
