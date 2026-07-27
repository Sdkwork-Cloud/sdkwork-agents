import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const bundlePath = path.join(
  root,
  "apps/sdkwork-agents-mini-program/src/runtime/agents-app.js",
);
const runtimeEnvPath = path.join(
  root,
  "apps/sdkwork-agents-mini-program/src/runtime/runtime-env.js",
);
const buildManifestPath = path.join(
  root,
  "apps/sdkwork-agents-mini-program/src/runtime/build-manifest.json",
);

assert.ok(fs.existsSync(bundlePath), "mini-program runtime bundle must exist; run pnpm --filter @sdkwork/agents-mini-program build");
assert.ok(fs.existsSync(runtimeEnvPath), "selected mini-program runtime env must exist");
assert.ok(fs.existsSync(buildManifestPath), "mini-program build manifest must exist");

const bundle = fs.readFileSync(bundlePath, "utf8");
assert.ok(bundle.length > 10_000, "runtime bundle looks truncated");

for (const marker of [
  "bootstrapAgentsMiniProgram",
  "getAgentsMpSdkClient",
  "createAgentsAppSdkClientConfig",
]) {
  assert.match(bundle, new RegExp(marker), `runtime bundle must export ${marker}`);
}

const runtimeEnv = fs.readFileSync(runtimeEnvPath, "utf8");
const buildManifest = JSON.parse(fs.readFileSync(buildManifestPath, "utf8"));
assert.match(runtimeEnv, /SDKWORK_PROFILE_ID/u);
assert.equal(buildManifest.profileId, `${buildManifest.deploymentProfile}.${buildManifest.environment}`);
assert.equal(buildManifest.runtimeTarget, "mini-program");
assert.equal(buildManifest.platform, "MP_WEIXIN");

const appSource = fs.readFileSync(
  path.join(root, "apps/sdkwork-agents-mini-program/src/app.js"),
  "utf8",
);
assert.match(appSource, /require\("\.\/runtime\/runtime-env"\)/u);
assert.doesNotMatch(appSource, /agentsAppApiBaseUrl:\s*"http:\/\/127\.0\.0\.1/u);

for (const forbiddenMarker of [
  "generated/server-openapi",
  "domain-transport-sdk",
  "domain-transport-typescript",
]) {
  assert.doesNotMatch(
    bundle,
    new RegExp(forbiddenMarker.replaceAll("/", "[\\\\/]"), "u"),
    `runtime bundle must not expose generated SDK transport source path ${forbiddenMarker}`,
  );
}

console.log("mini-program runtime contract passed.");
