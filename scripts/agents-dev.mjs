#!/usr/bin/env node
import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const gatewayBind = process.env.SDKWORK_AGENTS_APPLICATION_PUBLIC_INGRESS_BIND ?? '0.0.0.0:8095';
const gatewayHealthUrl = process.env.SDKWORK_AGENTS_DEV_GATEWAY_HEALTH_URL ?? 'http://127.0.0.1:8095/healthz';
const webHost = process.env.SDKWORK_AGENTS_PC_DEV_HOST ?? '0.0.0.0';
const webPort = process.env.SDKWORK_AGENTS_PC_DEV_PORT ?? '5195';
const requiredSourceFiles = [
  'Cargo.toml',
  'crates/sdkwork-agents-standalone-gateway/src/main.rs',
  'apps/sdkwork-agents-pc/package.json',
  'apps/sdkwork-agents-pc/vite.config.ts',
];

for (const relativePath of requiredSourceFiles) {
  if (!existsSync(path.join(repoRoot, relativePath))) {
    throw new Error(`Missing build-critical source file: ${relativePath}. Recover with: git checkout HEAD -- ${relativePath}`);
  }
}

const children = new Set();
let shuttingDown = false;
let startupFailure;

function start(command, args, options = {}) {
  const child = spawn(command, args, {
    cwd: repoRoot,
    env: { ...process.env, ...options.env },
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
      shutdown(code, `${command} exited with code ${code}`);
    } else if (!shuttingDown && signal) {
      shutdown(1, `${command} exited from signal ${signal}`);
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

async function waitForGateway(timeoutMs = 60_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (startupFailure) throw startupFailure;
    if (await gatewayIsHealthy()) return;
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`Agents gateway did not become healthy within ${timeoutMs}ms: ${gatewayHealthUrl}`);
}

function stopChild(child) {
  if (child.exitCode !== null || child.signalCode !== null) return;
  if (process.platform === 'win32') {
    const killer = spawn('taskkill.exe', ['/pid', String(child.pid), '/t', '/f'], {
      shell: false,
      stdio: 'ignore',
    });
    killer.unref();
    return;
  }
  child.kill('SIGTERM');
}

function shutdown(exitCode, reason) {
  if (shuttingDown) return;
  shuttingDown = true;
  if (reason) console.error(`[sdkwork-agents-dev] ${reason}`);
  for (const child of children) stopChild(child);
  setTimeout(() => process.exit(exitCode), 250).unref();
}

process.on('SIGINT', () => shutdown(0));
process.on('SIGTERM', () => shutdown(0));

try {
  if (await gatewayIsHealthy()) {
    console.log(`[sdkwork-agents-dev] reusing healthy API gateway: ${gatewayHealthUrl}`);
  } else {
    start('cargo', ['run', '-p', 'sdkwork-agents-standalone-gateway', '--quiet'], {
      env: {
        SDKWORK_AGENTS_APPLICATION_PUBLIC_INGRESS_BIND: gatewayBind,
        SDKWORK_AGENT_SERVER_BIND: gatewayBind,
      },
    });
    await waitForGateway();
  }

  console.log(`[sdkwork-agents-dev] API health: ${gatewayHealthUrl}`);
  start(
    'pnpm',
    ['--filter', '@sdkwork/agents-pc', 'dev', '--', '--host', webHost, '--port', webPort, '--strictPort'],
  );
} catch (error) {
  shutdown(1, error instanceof Error ? error.message : String(error));
}
