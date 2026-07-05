import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const bundlePath = path.join(
  root,
  "apps/sdkwork-agents-mini-program/src/runtime/agents-app.js",
);

assert.ok(fs.existsSync(bundlePath), "mini-program runtime bundle must exist; run pnpm --filter @sdkwork/agents-mini-program build");

const bundle = fs.readFileSync(bundlePath, "utf8");
assert.ok(bundle.length > 10_000, "runtime bundle looks truncated");

for (const marker of [
  "bootstrapAgentsMiniProgram",
  "getAgentsMpSdkClient",
  "createAgentsAppSdkClientConfig",
]) {
  assert.match(bundle, new RegExp(marker), `runtime bundle must export ${marker}`);
}

console.log("mini-program runtime contract passed.");
