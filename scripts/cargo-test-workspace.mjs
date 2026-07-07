#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const result = spawnSync('cargo', ['test', '--workspace', '--all-features', '--quiet'], {
  cwd: repoRoot,
  stdio: 'inherit',
  shell: false,
});
if (result.error) {
  console.error(`failed to run cargo test workspace: ${result.error.message}`);
}
process.exit(result.status ?? 1);
