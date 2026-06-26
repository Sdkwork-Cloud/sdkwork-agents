import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { resolveAgentsSdkFamily } from './_shared/agents-sdk-families.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const appFamily = resolveAgentsSdkFamily('app');
const openFamily = resolveAgentsSdkFamily('open');

const sourceRoot = path.join(
  root,
  'sdks',
  appFamily.familyDir,
  appFamily.languagePackageDir,
  'generated',
  'server-openapi'
);
const targetRoot = path.join(
  root,
  'sdks',
  openFamily.familyDir,
  openFamily.languagePackageDir,
  'generated',
  'server-openapi'
);

if (!fs.existsSync(sourceRoot)) {
  throw new Error(
    `App SDK output missing at ${path.relative(root, sourceRoot)}. Run workspace-agent-sdkgen --family app --mode apply first.`
  );
}

copyDirectory(sourceRoot, targetRoot, sourceRoot, (relativePath, content) => {
  if (!shouldTransform(relativePath)) {
    return content;
  }

  return content
    .replaceAll(appFamily.apiPrefix, openFamily.apiPrefix)
    .replaceAll(appFamily.packageName, openFamily.packageName)
    .replaceAll(appFamily.sdkName, openFamily.sdkName)
    .replaceAll(appFamily.authority, openFamily.authority)
    .replaceAll('APP_API_PREFIX', 'AGENT_API_PREFIX')
    .replaceAll('appApiPath', 'agentApiPath')
    .replaceAll('SdkworkAppClient', 'SdkworkAgentClient')
    .replaceAll('@sdkwork/agents-app-sdk', '@sdkwork/agents-sdk');
});

writeDerivationEvidence(targetRoot, appFamily, openFamily);
console.log('Agent open SDK transport derived from app SDK.');

function shouldTransform(relativePath) {
  const posix = relativePath.replace(/\\/g, '/');
  return (
    posix.endsWith('.ts') ||
    posix.endsWith('.json') ||
    posix.endsWith('.md') ||
    posix.endsWith('package.json')
  );
}

function copyDirectory(sourceDir, targetDir, sourceRoot, transform) {
  fs.rmSync(targetDir, { recursive: true, force: true });
  fs.mkdirSync(targetDir, { recursive: true });

  for (const entry of fs.readdirSync(sourceDir, { withFileTypes: true })) {
    const sourcePath = path.join(sourceDir, entry.name);
    const targetPath = path.join(targetDir, entry.name);
    if (entry.isDirectory()) {
      copyDirectory(sourcePath, targetPath, sourceRoot, transform);
      continue;
    }
    if (!entry.isFile()) {
      continue;
    }

    const raw = fs.readFileSync(sourcePath, 'utf8');
    const relativePath = path.relative(sourceRoot, sourcePath);
    const transformed = transform(relativePath, raw);
    fs.writeFileSync(targetPath, transformed, 'utf8');
  }
}

function writeDerivationEvidence(targetRoot, appFamily, openFamily) {
  const evidenceDir = path.join(targetRoot, '.sdkwork');
  fs.mkdirSync(evidenceDir, { recursive: true });
  fs.writeFileSync(
    path.join(evidenceDir, 'sdkwork-open-sdk-derivation.json'),
    `${JSON.stringify(
      {
        schemaVersion: 1,
        generator: 'materialize-agent-open-sdk-from-app',
        derivedFrom: {
          familyDir: appFamily.familyDir,
          authority: appFamily.authority,
          apiPrefix: appFamily.apiPrefix,
          packageName: appFamily.packageName
        },
        target: {
          familyDir: openFamily.familyDir,
          authority: openFamily.authority,
          apiPrefix: openFamily.apiPrefix,
          packageName: openFamily.packageName
        },
        managedBy: 'sdks/materialize-agent-open-sdk-from-app.mjs'
      },
      null,
      2
    )}\n`,
    'utf8'
  );
}
