#!/usr/bin/env node
import { execFileSync, spawn } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { parseArgs } from 'node:util';
import { fileURLToPath } from 'node:url';
import { createTopologyRuntime, loadTopologySpec } from '@sdkwork/app-topology';
import { mergeRepoDevBootstrapAccessTokenEnv } from '../../sdkwork-iam/scripts/dev/create-dev-bootstrap-access-token-env.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const topology = createTopologyRuntime(
  loadTopologySpec(path.join(repoRoot, 'specs/topology.spec.json')),
  repoRoot,
);
const developmentProfileId = topology.defaults.developmentProfileId;
const selectedProfile = resolveSelectedProfile();
const runtimeEnv = mergeRepoDevBootstrapAccessTokenEnv({
  repoRoot,
  env: topology.mergeRuntimeEnv(
    topology.loadProfile(selectedProfile.profileId),
    process.env,
    topology.applyProfileEnv(selectedProfile.profileId),
  ),
});
runtimeEnv.SDKWORK_ENVIRONMENT = selectedProfile.environment;
runtimeEnv.SDKWORK_APP_ROOT = repoRoot;
runtimeEnv.SDKWORK_IAM_APP_ROOT = process.env.SDKWORK_IAM_APP_ROOT
  ?? runtimeEnv.SDKWORK_IAM_APP_ROOT
  ?? path.resolve(repoRoot, '../sdkwork-iam');
const selectedProcesses = topology.listOrchestrationProcesses(selectedProfile.profileId);
const applicationIngressProcess = selectedProcesses.find(
  (processDefinition) => processDefinition.id === 'application.public-ingress',
);
const pcRendererProcess = selectedProcesses.find(
  (processDefinition) => processDefinition.id === 'pc-renderer',
);
const healthSurfaces = topology.listHealthSurfaces(selectedProfile.profileId);
const gatewayBind = runtimeEnv.SDKWORK_AGENTS_APPLICATION_PUBLIC_INGRESS_BIND ?? '0.0.0.0:8095';
const applicationPublicHttpUrl = runtimeEnv.SDKWORK_AGENTS_APPLICATION_PUBLIC_HTTP_URL ?? 'http://127.0.0.1:8095';
const gatewayHealthUrl = runtimeEnv.SDKWORK_AGENTS_DEV_GATEWAY_HEALTH_URL
  ?? `${applicationPublicHttpUrl.replace(/\/+$/u, '')}/healthz`;
const webHost = process.env.SDKWORK_AGENTS_PC_DEV_HOST
  ?? runtimeEnv.SDKWORK_AGENTS_PC_DEV_HOST
  ?? '0.0.0.0';
const webPort = process.env.SDKWORK_AGENTS_PC_DEV_PORT
  ?? runtimeEnv.SDKWORK_AGENTS_PC_DEV_PORT
  ?? '5195';
const gatewayProfileMarker = runtimeEnv.SDKWORK_AGENTS_DEV_GATEWAY_PROFILE_MARKER
  ?? path.join(repoRoot, '.sdkwork', 'tmp', 'agents-dev-gateway-profile.json');
const requiredSourceFiles = [
  'Cargo.toml',
  'crates/sdkwork-agents-standalone-gateway/src/main.rs',
  'apps/sdkwork-agents-pc/package.json',
  'apps/sdkwork-agents-pc/vite.config.ts',
];

function resolveSelectedProfile() {
  const { values } = parseArgs({
    args: process.argv.slice(2),
    options: {
      'deployment-profile': { type: 'string' },
      environment: { type: 'string' },
    },
    strict: true,
  });
  const defaultProfile = topology.parseProfileId(developmentProfileId);
  const deploymentProfile = topology.assertDeploymentProfile(
    values['deployment-profile'] ?? defaultProfile.deploymentProfile,
  );
  const environment = topology.assertEnvironment(values.environment ?? defaultProfile.environment);
  const profileId = `${deploymentProfile}.${environment}`;
  topology.assertProfileId(profileId);
  return { deploymentProfile, environment, profileId };
}

function assertLocallyRunnableProfile() {
  if (selectedProfile.deploymentProfile !== 'standalone' || selectedProfile.environment !== 'development') {
    throw new Error(
      `pnpm dev validates ${selectedProfile.profileId} but only runs standalone.development locally. Use the selected profile's deployment orchestration for ${selectedProfile.profileId}.`,
    );
  }
  if (!applicationIngressProcess?.crate) {
    throw new Error(`${selectedProfile.profileId} must declare application.public-ingress in topology orchestration`);
  }
  if (!pcRendererProcess?.package || !pcRendererProcess.script) {
    throw new Error(`${selectedProfile.profileId} must declare pc-renderer in topology orchestration`);
  }
  if (!healthSurfaces.includes('application.public-ingress')) {
    throw new Error(`${selectedProfile.profileId} must health-check application.public-ingress`);
  }
}

function ensureBuildCriticalSources() {
  const missing = requiredSourceFiles.filter((relativePath) => !existsSync(path.join(repoRoot, relativePath)));
  if (missing.length === 0) return;
  try {
    execFileSync('git', ['checkout', 'HEAD', '--', ...missing], { cwd: repoRoot, stdio: 'ignore' });
  } catch {
    // The actionable error below names the unrecovered sources and recovery command.
  }
  const unrecovered = missing.filter((relativePath) => !existsSync(path.join(repoRoot, relativePath)));
  if (unrecovered.length > 0) {
    throw new Error(
      `Missing build-critical source files: ${unrecovered.join(', ')}. Recover with: git checkout HEAD -- ${unrecovered.join(' ')}`,
    );
  }
}

function describeDatabaseProfile() {
  if (runtimeEnv.SDKWORK_AGENTS_DATABASE_ENGINE) return runtimeEnv.SDKWORK_AGENTS_DATABASE_ENGINE;
  return runtimeEnv.SDKWORK_AGENTS_DATABASE_URL?.startsWith('sqlite:') ? 'sqlite' : 'operator-configured';
}

function reportResolvedProfile() {
  console.log(
    `[sdkwork-agents-dev] resolved deploymentProfile=${selectedProfile.deploymentProfile} environment=${selectedProfile.environment} runtimeTarget=${runtimeEnv.SDKWORK_AGENTS_RUNTIME_TARGET ?? 'unknown'} databaseProfile=${describeDatabaseProfile()} profileId=${selectedProfile.profileId}`,
  );
}

const children = new Set();
let shuttingDown = false;
let shutdownPromise;
let startupFailure;
let ownsGateway = false;

function start(command, args, options = {}) {
  const child = spawn(command, args, {
    cwd: repoRoot,
    env: { ...runtimeEnv, ...options.env },
    shell: process.platform === 'win32',
    stdio: 'inherit',
  });
  children.add(child);
  child.once('exit', (code, signal) => {
    children.delete(child);
    if (!shuttingDown && code !== 0) {
      startupFailure = new Error(`${command} exited with code ${code}`);
    }
    if (!shuttingDown && code !== null) {
      void shutdown(code, `${command} exited with code ${code}`);
    } else if (!shuttingDown && signal) {
      void shutdown(1, `${command} exited from signal ${signal}`);
    }
  });
  return child;
}

async function gatewayIsHealthy() {
  try {
    const response = await fetch(gatewayHealthUrl, { signal: AbortSignal.timeout(1_000) });
    return response.ok;
  } catch {
    return false;
  }
}

function gatewayProfileMatchesRuntime() {
  if (!existsSync(gatewayProfileMarker)) return false;
  try {
    const marker = JSON.parse(readFileSync(gatewayProfileMarker, 'utf8'));
    return marker.profileId === runtimeEnv.SDKWORK_AGENTS_PROFILE_ID
      && marker.environment === runtimeEnv.SDKWORK_ENVIRONMENT
      && marker.corsAllowedOrigins === (runtimeEnv.SDKWORK_CORS_ALLOWED_ORIGINS ?? '');
  } catch {
    return false;
  }
}

function writeGatewayProfileMarker() {
  mkdirSync(path.dirname(gatewayProfileMarker), { recursive: true });
  writeFileSync(
    gatewayProfileMarker,
    `${JSON.stringify({
      corsAllowedOrigins: runtimeEnv.SDKWORK_CORS_ALLOWED_ORIGINS ?? '',
      environment: runtimeEnv.SDKWORK_ENVIRONMENT,
      profileId: runtimeEnv.SDKWORK_AGENTS_PROFILE_ID,
    })}\n`,
    'utf8',
  );
}

async function waitForGateway(timeoutMs = 60_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (startupFailure) throw startupFailure;
    if (await gatewayIsHealthy()) return;
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`Agents gateway did not become healthy within ${timeoutMs}ms: ${gatewayHealthUrl}`);
}

function waitForChildExit(child, timeoutMs = 5_000) {
  if (child.exitCode !== null || child.signalCode !== null) return Promise.resolve();
  return new Promise((resolve) => {
    const timeout = setTimeout(resolve, timeoutMs);
    child.once('exit', () => {
      clearTimeout(timeout);
      resolve();
    });
  });
}

async function stopChild(child) {
  if (child.exitCode !== null || child.signalCode !== null) return;
  if (process.platform === 'win32') {
    await new Promise((resolve) => {
      const killer = spawn('taskkill.exe', ['/pid', String(child.pid), '/t', '/f'], {
        shell: false,
        stdio: 'ignore',
      });
      killer.once('error', resolve);
      killer.once('exit', resolve);
    });
    await waitForChildExit(child);
    return;
  }
  child.kill('SIGTERM');
  await waitForChildExit(child);
}

function shutdown(exitCode, reason) {
  if (shutdownPromise) return shutdownPromise;
  shuttingDown = true;
  if (reason) console.error(`[sdkwork-agents-dev] ${reason}`);
  shutdownPromise = (async () => {
    await Promise.allSettled([...children].map((child) => stopChild(child)));
    if (ownsGateway) rmSync(gatewayProfileMarker, { force: true });
    process.exit(exitCode);
  })();
  return shutdownPromise;
}

process.on('SIGINT', () => void shutdown(0));
process.on('SIGTERM', () => void shutdown(0));

try {
  reportResolvedProfile();
  assertLocallyRunnableProfile();
  ensureBuildCriticalSources();
  const healthyGateway = await gatewayIsHealthy();
  if (healthyGateway && gatewayProfileMatchesRuntime()) {
    console.log(`[sdkwork-agents-dev] reusing healthy API gateway: ${gatewayHealthUrl}`);
  } else {
    if (healthyGateway) {
      throw new Error(
        `A healthy API gateway is already listening at ${gatewayHealthUrl}, but its profile does not match ${selectedProfile.profileId}. Stop that gateway before running pnpm dev.`,
      );
    }
    ownsGateway = true;
    start('cargo', ['run', '-p', applicationIngressProcess.crate, '--quiet'], {
      env: {
        SDKWORK_AGENTS_APPLICATION_PUBLIC_INGRESS_BIND: gatewayBind,
        SDKWORK_AGENT_SERVER_BIND: gatewayBind,
      },
    });
    await waitForGateway();
    writeGatewayProfileMarker();
  }

  console.log(`[sdkwork-agents-dev] API health: ${gatewayHealthUrl}`);
  start(
    'pnpm',
    ['--filter', pcRendererProcess.package, 'exec', pcRendererProcess.script, '--host', webHost, '--port', webPort, '--strictPort'],
  );
} catch (error) {
  await shutdown(1, error instanceof Error ? error.message : String(error));
}
