import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

test('platform integration contract: kernel bridge uses web-framework served router', () => {
  const source = fs.readFileSync(
    path.join(repoRoot, 'crates/sdkwork-agents-kernel-bridge/src/lib.rs'),
    'utf8',
  );
  assert.match(source, /build_served_combined_router/);
  assert.match(source, /app::build_app/);
});

test('platform integration contract: utils used in contract crate', () => {
  const source = fs.readFileSync(
    path.join(repoRoot, 'crates/sdkwork-agents-contract/src/lib.rs'),
    'utf8',
  );
  assert.match(source, /sdkwork_utils_rust::parse_bool/);
});
