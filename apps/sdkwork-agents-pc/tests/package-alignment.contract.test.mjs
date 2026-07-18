import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const appRoot = fileURLToPath(new URL('..', import.meta.url));
const packagesRoot = path.join(appRoot, 'packages');
const retiredProductToken = ['chat', 'box'].join('');
const retiredPackageStem = ['sdkwork', retiredProductToken].join('-');
const retiredNpmScope = `@sdkwork/${retiredProductToken}`;

const EXPECTED_PACKAGES = [
  'sdkwork-agents-pc-agents',
  'sdkwork-agents-pc-assets',
  'sdkwork-agents-pc-canvas',
  'sdkwork-agents-pc-chat',
  'sdkwork-agents-pc-commons',
  'sdkwork-agents-pc-core',
  'sdkwork-agents-pc-creative',
  'sdkwork-agents-pc-desktop',
  'sdkwork-agents-pc-inspiration',
  'sdkwork-agents-pc-presentation',
];

function sourceFiles(root) {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    if (entry.name === 'node_modules') {
      return [];
    }
    const absolutePath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      return sourceFiles(absolutePath);
    }
    return /\.(?:json|ts|tsx)$/.test(entry.name) ? [absolutePath] : [];
  });
}

test('all PC packages use the canonical agents package family', () => {
  const packageDirectories = readdirSync(packagesRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort();

  assert.deepEqual(packageDirectories, EXPECTED_PACKAGES);

  for (const directory of packageDirectories) {
    const manifest = JSON.parse(readFileSync(path.join(packagesRoot, directory, 'package.json'), 'utf8'));
    const componentSpec = JSON.parse(readFileSync(path.join(packagesRoot, directory, 'specs', 'component.spec.json'), 'utf8'));
    const expectedPackageName = `@sdkwork/${directory.slice('sdkwork-'.length)}`;

    assert.equal(manifest.name, expectedPackageName, `${directory} npm name`);
    assert.equal(manifest.sdkwork.surface, 'app', `${directory} surface`);
    assert.match(manifest.sdkwork.architecture, /^pc-(?:react|desktop)$/, `${directory} architecture`);
    assert.equal(componentSpec.component.name, directory, `${directory} component name`);
    assert.equal(componentSpec.component.root, `apps/sdkwork-agents-pc/packages/${directory}`, `${directory} component root`);
  }
});

test('canonical PC packages contain no legacy package identity or source deep imports', () => {
  for (const file of sourceFiles(packagesRoot)) {
    const source = readFileSync(file, 'utf8');
    assert.equal(source.toLowerCase().includes(retiredPackageStem), false, file);
    assert.equal(source.toLowerCase().includes(retiredNpmScope), false, file);
    assert.doesNotMatch(source, /@\/packages\//, file);
  }
});
