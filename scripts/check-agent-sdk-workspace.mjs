import { spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { AGENTS_SDK_FAMILIES } from '../sdks/_shared/agents-sdk-families.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

const commands = [
  ['node', ['sdks/test/verify-agent-sdk-ownership-boundaries.test.mjs']],
  ...AGENTS_SDK_FAMILIES.map((family) => [
    'node',
    [`sdks/${family.familyDir}/bin/verify-sdk.mjs`]
  ])
];

for (const [command, args] of commands) {
  run(command, args);
}

console.log('Agent SDK workspace verification passed.');

function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: 'utf8',
    stdio: 'inherit'
  });
  if (result.status !== 0) {
    throw new Error(`Command failed: ${command} ${args.join(' ')}`);
  }
}
