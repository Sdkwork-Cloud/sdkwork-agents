#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const bind = process.env.SDKWORK_AGENTS_APPLICATION_PUBLIC_INGRESS_BIND ?? '127.0.0.1:8095';
process.env.SDKWORK_AGENTS_APPLICATION_PUBLIC_INGRESS_BIND = bind;
process.env.SDKWORK_AGENT_SERVER_BIND = bind;

const child = spawnSync(
  'cargo',
  ['run', '-p', 'sdkwork-agents-standalone-gateway', '--quiet'],
  { cwd: repoRoot, stdio: 'inherit', shell: process.platform === 'win32' },
);
process.exit(child.status ?? 1);
