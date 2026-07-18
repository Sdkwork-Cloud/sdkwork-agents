import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function readEnv(relativePath) {
  return Object.fromEntries(
    read(relativePath)
      .split(/\r?\n/u)
      .map((line) => line.trim())
      .filter((line) => line && !line.startsWith('#'))
      .map((line) => {
        const separator = line.indexOf('=');
        return [line.slice(0, separator), line.slice(separator + 1)];
      }),
  );
}

test('root dev runner starts the API gateway before the PC browser renderer', () => {
  const runner = read('scripts/agents-dev.mjs');

  assert.match(runner, /sdkwork-agents-standalone-gateway/u);
  assert.match(runner, /\/healthz/u);
  assert.match(runner, /@sdkwork\/agents-pc/u);
  assert.match(runner, /waitForGateway/u);
  assert.match(runner, /--host/u);
  assert.match(runner, /--strictPort/u);
});

test('PC development server exposes LAN access and same-origin API proxying', () => {
  const vite = read('apps/sdkwork-agents-pc/vite.config.ts');
  const runtimeConfig = read('apps/sdkwork-agents-pc/src/bootstrap/runtimeConfig.ts');
  const sdkClient = read(
    'apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-core/src/sdk/agentsAppSdkClient.ts',
  );

  assert.match(vite, /host:\s*['"]0\.0\.0\.0['"]/u);
  assert.match(vite, /port:\s*5195/u);
  assert.match(vite, /['"]\/app\/v3\/api['"]:\s*['"]http:\/\/127\.0\.0\.1:8095['"]/u);
  assert.match(runtimeConfig, /window\.location\.origin/u);
  assert.match(sdkClient, /window\.location\.origin/u);
});

test('gateway startup output distinguishes health links from authenticated API origins', () => {
  const accessUrls = read(
    'crates/sdkwork-agents-standalone-gateway/src/access_urls.rs',
  );

  assert.match(accessUrls, /health URL/u);
  assert.match(accessUrls, /\/healthz/u);
  assert.match(accessUrls, /API origin \(authentication required\)/u);
});

test('all topology profiles project one environment into embedded Web Framework routers', () => {
  for (const profile of [
    'standalone.development',
    'standalone.production',
    'cloud.development',
    'cloud.production',
  ]) {
    const env = readEnv(`configs/topology/${profile}.env`);
    assert.equal(
      env.SDKWORK_ENVIRONMENT,
      env.SDKWORK_AGENTS_ENVIRONMENT,
      `${profile} must project the application environment to embedded IAM and dependency routers`,
    );
    if (env.SDKWORK_AGENTS_ENVIRONMENT === 'production') {
      assert.equal(env.SDKWORK_CORS_ALLOWED_ORIGINS, 'https://agents.sdkwork.com');
    }
  }
});
