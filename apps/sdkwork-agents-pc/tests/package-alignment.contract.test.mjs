import assert from 'node:assert/strict';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const appRoot = fileURLToPath(new URL('..', import.meta.url));
const repositoryRoot = path.resolve(appRoot, '..', '..');
const packagesRoot = path.join(appRoot, 'packages');
const retiredProductToken = ['chat', 'box'].join('');
const retiredPackageStem = ['sdkwork', retiredProductToken].join('-');
const retiredNpmScope = `@sdkwork/${retiredProductToken}`;
const retiredThirdPartyBrand = ['Chat', 'GPT'].join('');
const migrationArchive = path.join(path.dirname(appRoot), `${path.basename(appRoot)}2.zip`);
const TEXT_FILE_PATTERN = /\.(?:cjs|css|html|js|json|jsx|lock|md|mjs|scss|toml|ts|tsx|txt|xml|yaml|yml)$/;
const EXCLUDED_DIRECTORIES = new Set(['.git', 'dist', 'node_modules', 'target']);

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
  'sdkwork-agents-pc-membership',
  'sdkwork-agents-pc-presentation',
];

function activeFiles(root) {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    if (entry.isDirectory() && EXCLUDED_DIRECTORIES.has(entry.name)) {
      return [];
    }
    const absolutePath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      return activeFiles(absolutePath);
    }
    return [absolutePath];
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
  for (const file of activeFiles(packagesRoot).filter((entry) => TEXT_FILE_PATTERN.test(entry))) {
    const source = readFileSync(file, 'utf8');
    assert.equal(source.toLowerCase().includes(retiredPackageStem), false, file);
    assert.equal(source.toLowerCase().includes(retiredNpmScope), false, file);
    assert.doesNotMatch(source, /@\/packages\//, file);
  }
});

test('active repository tree contains no retired product identity or migration archive', () => {
  assert.equal(existsSync(migrationArchive), false, migrationArchive);

  for (const file of activeFiles(repositoryRoot)) {
    const relativePath = path.relative(repositoryRoot, file);
    assert.equal(relativePath.toLowerCase().includes(retiredPackageStem), false, relativePath);

    if (TEXT_FILE_PATTERN.test(file)) {
      const source = readFileSync(file, 'utf8').toLowerCase();
      assert.equal(source.includes(retiredPackageStem), false, relativePath);
      assert.equal(source.includes(retiredNpmScope), false, relativePath);
    }
  }
});

test('PC-authored copy uses the SDKWork Agents localized product identity', () => {
  const chatPackageRoot = path.join(packagesRoot, 'sdkwork-agents-pc-chat');
  const commonsPackageRoot = path.join(packagesRoot, 'sdkwork-agents-pc-commons');
  const chatInputSource = readFileSync(
    path.join(chatPackageRoot, 'src', 'components', 'ChatInput.tsx'),
    'utf8',
  );
  const catalogs = ['en-US', 'zh-CN'].map((locale) =>
    JSON.parse(
      readFileSync(
        path.join(commonsPackageRoot, 'src', 'i18n', locale, 'agents', 'workbench', 'chat.json'),
        'utf8',
      ),
    ),
  );

  for (const file of activeFiles(appRoot).filter((entry) => TEXT_FILE_PATTERN.test(entry))) {
    const source = readFileSync(file, 'utf8');
    assert.equal(source.includes(retiredThirdPartyBrand), false, path.relative(appRoot, file));
  }
  assert.match(chatInputSource, /\{t\(['"]disclaimer['"]\)\}/);
  for (const catalog of catalogs) {
    assert.match(catalog.disclaimer, /^SDKWork Agents\b/);
    assert.match(catalog.projectInstructionsDescription, /\bSDKWork Agents\b/);
  }
});
