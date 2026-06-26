#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const failures = [];

function readText(relativePath) {
  const absolutePath = path.join(repoRoot, relativePath);
  if (!fs.existsSync(absolutePath)) {
    failures.push(`${relativePath} must exist`);
    return '';
  }
  return fs.readFileSync(absolutePath, 'utf8');
}

function readJson(relativePath) {
  const text = readText(relativePath).replace(/^\uFEFF/, '');
  return JSON.parse(text);
}

function assert(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}

function assertDirectory(relativePath) {
  assert(fs.existsSync(path.join(repoRoot, relativePath)), `${relativePath}/ must exist`);
}

const requiredDirectories = [
  'apis',
  'apps',
  'crates',
  'sdks',
  'database',
  'deployments',
  'configs',
  'scripts',
  'docs',
  'tests',
  '.sdkwork',
  'specs',
];

for (const directory of requiredDirectories) {
  assertDirectory(directory);
}

assert(fs.existsSync(path.join(repoRoot, 'sdkwork.app.config.json')), 'sdkwork.app.config.json must exist');
assert(fs.existsSync(path.join(repoRoot, 'sdkwork.workflow.json')), 'sdkwork.workflow.json must exist');
assert(fs.existsSync(path.join(repoRoot, 'package.json')), 'package.json must exist');
assert(
  fs.existsSync(path.join(repoRoot, '.github/workflows/package.yml')),
  '.github/workflows/package.yml must exist per GITHUB_WORKFLOW_SPEC.md',
);

const packageJson = readJson('package.json');
for (const script of ['dev', 'build', 'test', 'check', 'verify', 'clean']) {
  assert(packageJson.scripts?.[script], `package.json must expose pnpm ${script}`);
}
assert(packageJson.scripts?.['check:architecture-alignment'], 'package.json must expose check:architecture-alignment');
assert(packageJson.scripts?.['topology:validate'], 'package.json must expose topology:validate');
assert(packageJson.scripts?.['db:validate'], 'package.json must expose db:validate');
assert(packageJson.dependencies?.['@sdkwork/app-topology'], 'package.json must declare @sdkwork/app-topology');

const cargoToml = readText('Cargo.toml');
assert(cargoToml.includes('sdkwork-web-core'), 'Cargo.toml must declare sdkwork-web-core');
assert(cargoToml.includes('sdkwork-web-axum'), 'Cargo.toml must declare sdkwork-web-axum');
assert(cargoToml.includes('sdkwork-iam-web-adapter'), 'Cargo.toml must declare sdkwork-iam-web-adapter');
assert(cargoToml.includes('sdkwork-database-config'), 'Cargo.toml must declare sdkwork-database-config');
assert(cargoToml.includes('sdkwork-database-sqlx'), 'Cargo.toml must declare sdkwork-database-sqlx');
assert(cargoToml.includes('sdkwork-utils-rust'), 'Cargo.toml must declare sdkwork-utils-rust');
assert(cargoToml.includes('sdkwork-agent-server'), 'Cargo.toml must declare sdkwork-kernel sdkwork-agent-server dependency');
assert(cargoToml.includes('sdkwork-intelligence-agents-service'), 'Cargo.toml must declare sdkwork-intelligence-agents-service');
assert(cargoToml.includes('sdkwork-routes-agents-http-shared'), 'Cargo.toml must declare sdkwork-routes-agents-http-shared');
assert(!cargoToml.includes('sdkwork-agent-business'), 'Cargo.toml must not reference retired sdkwork-agent-business');
assert(!cargoToml.includes('sdkwork-discovery'), 'sdkwork-discovery is deferred until RPC services exist');

const contractSource = readText('crates/sdkwork-agents-contract/src/lib.rs');
assert(
  contractSource.includes('sdkwork_utils_rust::parse_bool'),
  'sdkwork-agents-contract must use sdkwork-utils-rust parse_bool',
);

const bridgeSource = readText('crates/sdkwork-agents-kernel-bridge/src/lib.rs');
const agentHttpStateSource = readText('crates/sdkwork-agents-kernel-bridge/src/agent_http_state.rs');
assert(
  bridgeSource.includes('build_served_combined_router'),
  'kernel-bridge must compose kernel web-framework served router',
);
assert(
  bridgeSource.includes('app::build_app'),
  'kernel-bridge must compose kernel operational router',
);
assert(
  bridgeSource.includes('build_agent_http_state'),
  'kernel-bridge must bootstrap agent HTTP state through dedicated module',
);
assert(
  agentHttpStateSource.includes('connect_from_agents_managed_store_env'),
  'agent_http_state must wire postgres managed-store repository in production path',
);
assert(
  agentHttpStateSource.includes('agents_use_dev_inline_auth_resolver'),
  'agent_http_state must gate dev inline auth through contract helpers',
);

assert(fs.existsSync(path.join(repoRoot, 'crates/sdkwork-agents-database-host/src/lib.rs')), 'sdkwork-agents-database-host crate must exist');
assert(fs.existsSync(path.join(repoRoot, 'crates/sdkwork-agents-gateway-assembly/src/lib.rs')), 'sdkwork-agents-gateway-assembly crate must exist');
assert(fs.existsSync(path.join(repoRoot, 'crates/sdkwork-agents-integration-tests/Cargo.toml')), 'integration-tests crate must exist');
assert(fs.existsSync(path.join(repoRoot, 'deployments/docker/Dockerfile')), 'deployments/docker/Dockerfile must exist');
assert(fs.existsSync(path.join(repoRoot, '.env.example')), '.env.example must exist');

const apiServerCargo = readText('crates/sdkwork-agents-api-server/Cargo.toml');
assert(
  apiServerCargo.includes('sdkwork-agents-standalone-gateway'),
  'api-server must expose sdkwork-agents-standalone-gateway binary alias',
);

const topologySpec = readJson('specs/topology.spec.json');
assert(
  topologySpec.scripts?.dev === 'scripts/agents-dev.mjs',
  'topology.spec.json dev script must reference scripts/agents-dev.mjs',
);
assert(
  !JSON.stringify(topologySpec).includes('llm-dev.mjs'),
  'topology.spec.json must not retain llm stale script references',
);
assert(
  topologySpec.components?.cloudGateway?.configGlob?.includes('agents'),
  'topology.spec.json cloud gateway configGlob must reference agents profiles',
);

for (const profileFile of Object.values(topologySpec.profileFiles ?? {})) {
  assert(fs.existsSync(path.join(repoRoot, profileFile)), `${profileFile} must exist for topology profile`);
}

const boundaries = readText('crates/sdkwork-agents-kernel-bridge/src/boundaries.rs');
assert(boundaries.includes('agents-domain-service'), 'boundaries must declare agents-domain-service capability');
assert(!boundaries.includes('business'), 'boundaries must not retain business capability tokens');

for (const sdkManifestPath of [
  'sdks/sdkwork-agents-sdk/sdk-manifest.json',
  'sdks/sdkwork-agents-app-sdk/sdk-manifest.json',
  'sdks/sdkwork-agents-backend-sdk/sdk-manifest.json',
]) {
  const sdkManifest = readJson(sdkManifestPath);
  assert(sdkManifest.sdkOwner === 'sdkwork-agents', `${sdkManifestPath} sdkOwner must be sdkwork-agents`);
}

const kernelRoot = path.resolve(repoRoot, '..', 'sdkwork-kernel');
const retiredKernelPaths = [
  'sdkwork-agent-business',
  'apis/agent-business',
  'crates/sdkwork-routes-agent-http-shared',
  'crates/sdkwork-routes-agent-open-api',
  'crates/sdkwork-routes-agent-app-api',
  'crates/sdkwork-routes-agent-backend-api',
  'sdks/sdkwork-agent-sdk',
  'sdks/sdkwork-agent-app-sdk',
  'sdks/sdkwork-agent-backend-sdk',
];
for (const retiredPath of retiredKernelPaths) {
  assert(
    !fs.existsSync(path.join(kernelRoot, retiredPath)),
    `sdkwork-kernel must not retain retired path ${retiredPath}; managed agents domain belongs in sdkwork-agents`,
  );
}

const componentSpec = readJson('specs/component.spec.json');
const sdkDeps = componentSpec.contracts?.sdkDependencies ?? [];
const depIds = sdkDeps.map((entry) => entry.workspace);
assert(depIds.includes('sdkwork-web-framework'), 'component.spec.json must declare sdkwork-web-framework');
assert(depIds.includes('sdkwork-database'), 'component.spec.json must declare sdkwork-database');
assert(depIds.includes('sdkwork-utils'), 'component.spec.json must declare sdkwork-utils');
assert(depIds.includes('sdkwork-kernel'), 'component.spec.json must declare sdkwork-kernel');
const driveDep = sdkDeps.find((entry) => entry.workspace === 'sdkwork-drive');
assert(driveDep, 'component.spec.json must declare sdkwork-drive for upload integration');
assert(!depIds.includes('sdkwork-discovery'), 'component.spec.json must not require sdkwork-discovery yet');

const appManifest = readJson('sdkwork.app.config.json');
assert(
  appManifest.metadata?.topologySpec === 'specs/topology.spec.json',
  'sdkwork.app.config.json metadata.topologySpec must reference specs/topology.spec.json',
);
assert(
  appManifest.metadata?.kernelDependency?.includes('sdkwork-kernel'),
  'sdkwork.app.config.json must declare kernel dependency',
);
assert(
  appManifest.app?.appType === 'APP_API',
  'sdkwork.app.config.json appType must be APP_API for rust HTTP service',
);

if (failures.length > 0) {
  console.error('sdkwork-agents architecture alignment failures:');
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log('sdkwork-agents architecture alignment passed');
