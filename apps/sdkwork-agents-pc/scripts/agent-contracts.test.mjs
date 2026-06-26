#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptsDir = path.dirname(fileURLToPath(import.meta.url));
const files = ["agent-service-management-profile-contract.test.ts","agent-service-scope-consistency-contract.test.ts"];

for (const file of files) {
  const result = spawnSync(process.execPath, ['--import', 'tsx', path.join(scriptsDir, file)], {
    stdio: 'inherit',
    cwd: path.resolve(scriptsDir, '..'),
  });
  if (result.status !== 0) process.exit(result.status ?? 1);
}
