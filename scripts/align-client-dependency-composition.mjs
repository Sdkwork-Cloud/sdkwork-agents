#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const alignScript = path.resolve(repoRoot, '../sdkwork-specs/tools/align-app-composition.mjs');
const dryRun = process.argv.includes('--dry-run');
const args = ['--root', repoRoot];
if (dryRun) args.push('--dry-run');

const result = spawnSync(process.execPath, [alignScript, ...args], { stdio: 'inherit' });
process.exit(result.status ?? 1);
