import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { countAgentOpenApiOperations } from '../sdks/_shared/agent-sdk-ownership.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

const openOpenApi = fs.readFileSync(
  path.join(
    root,
    'crates/sdkwork-intelligence-agents-service/specs/openapi/agents-open-api.openapi.yaml',
  ),
  'utf8',
);
const openSdkAi = fs.readFileSync(
  path.join(
    root,
    'sdks/sdkwork-agents-sdk/sdkwork-agents-sdk-typescript/generated/server-openapi/src/api/ai.ts',
  ),
  'utf8',
);
const serviceHttp = fs.readFileSync(
  path.join(root, 'crates/sdkwork-intelligence-agents-service/src/http.rs'),
  'utf8',
);

assert.equal(countAgentOpenApiOperations(openOpenApi), 27);
assert.doesNotMatch(openOpenApi, /\/agent\/v3\/api\/ai\/agents\/\{agentId\}\/restore:/);
assert.doesNotMatch(openOpenApi, /\/agent\/v3\/api\/ai\/code_engines:/);
assert.doesNotMatch(openOpenApi, /\/agent\/v3\/api\/ai\/mcp_servers:/);

for (const forbidden of ['/restore`', '/code_engines', '/mcp_servers']) {
  assert.doesNotMatch(
    openSdkAi,
    new RegExp(forbidden.replace('/', '\\/')),
    `open SDK ai.ts must not reference app-only path ${forbidden}`,
  );
}
assert.doesNotMatch(openSdkAi, /\basync restore\(/);
assert.doesNotMatch(openSdkAi, /AiAgentsCodeEnginesApi|AiAgentsMcpServersApi/);

assert.doesNotMatch(
  serviceHttp.match(/pub fn build_open_routes[\s\S]*?^}/m)?.[0] ?? serviceHttp,
  /\/agent\/v3\/api\/ai\/agents\/\{agentId\}\/restore/,
);

console.log('agents open sdk surface contract passed.');
