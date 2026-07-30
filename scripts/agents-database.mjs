#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { createTopologyRuntime, loadTopologySpec } from '@sdkwork/app-topology';
import { ensureRustToolchain } from './rust-toolchain.mjs';

const supportedCommands = new Set([
  'plan',
  'init',
  'migrate',
  'seed',
  'status',
  'drift',
  'drift-check',
  'bootstrap',
]);
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function run() {
  const command = process.argv[2];
  if (!supportedCommands.has(command)) {
    throw new Error(`Unsupported Agents database command: ${command ?? '<missing>'}.`);
  }

  const topology = createTopologyRuntime(
    loadTopologySpec(path.join(repoRoot, 'specs/topology.spec.json')),
    repoRoot,
  );
  const env = ensureRustToolchain({
    cwd: repoRoot,
    env: topology.resolveIamDevEnv(process.env, { stdout: process.stdout }),
  });
  const cargo = process.platform === 'win32' ? 'cargo.exe' : 'cargo';
  const result = spawnSync(
    cargo,
    [
      'run',
      '--manifest-path',
      path.resolve(repoRoot, '../sdkwork-database/Cargo.toml'),
      '-p',
      'sdkwork-database-cli',
      '--',
      '--app-root',
      repoRoot,
      command,
    ],
    {
      cwd: repoRoot,
      env,
      stdio: 'inherit',
      windowsHide: true,
    },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`sdkwork-database-cli ${command} exited with code ${result.status ?? 1}.`);
  }
}

try {
  run();
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
