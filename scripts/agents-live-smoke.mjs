#!/usr/bin/env node

import assert from "node:assert/strict";

const baseUrl = (process.env.SDKWORK_AGENTS_SMOKE_BASE_URL ?? "http://127.0.0.1:8095").replace(
  /\/+$/u,
  "",
);

async function fetchText(path) {
  const response = await fetch(`${baseUrl}${path}`);
  const body = await response.text();
  return { response, body };
}

async function main() {
  const health = await fetchText("/health");
  assert.equal(health.response.status, 200, `/health must return 200 (got ${health.response.status})`);

  const metrics = await fetchText("/metrics/agents");
  assert.equal(
    metrics.response.status,
    200,
    `/metrics/agents must return 200 (got ${metrics.response.status})`,
  );
  assert.match(metrics.body, /sdkwork_agents_/u, "/metrics/agents must expose sdkwork_agents_* metrics");

  console.log(`agents live smoke passed against ${baseUrl}`);
}

main().catch((error) => {
  console.error(
    [
      "agents live smoke failed.",
      `baseUrl=${baseUrl}`,
      "Start gateway: pnpm dev",
      "Override: SDKWORK_AGENTS_SMOKE_BASE_URL=http://host:port node scripts/agents-live-smoke.mjs",
      error instanceof Error ? error.message : String(error),
    ].join("\n"),
  );
  process.exit(1);
});
