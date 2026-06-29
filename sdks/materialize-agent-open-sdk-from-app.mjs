import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { resolveAgentsSdkFamily } from './_shared/agents-sdk-families.mjs';
import { countAgentOpenApiOperations } from './_shared/agent-sdk-ownership.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const appFamily = resolveAgentsSdkFamily('app');
const openFamily = resolveAgentsSdkFamily('open');

const openAuthorityPath = path.join(
  root,
  'sdks',
  openFamily.familyDir,
  'openapi',
  `${openFamily.authority}.openapi.yaml`,
);
const openAuthority = fs.readFileSync(openAuthorityPath, 'utf8');
const expectedOpenOperations = countAgentOpenApiOperations(openAuthority);

const APP_ONLY_PATH_MARKERS = [
  '/restore',
  '/code_engines',
  '/mcp_servers',
];

const sourceRoot = path.join(
  root,
  'sdks',
  appFamily.familyDir,
  appFamily.languagePackageDir,
  'generated',
  'server-openapi',
);
const targetRoot = path.join(
  root,
  'sdks',
  openFamily.familyDir,
  openFamily.languagePackageDir,
  'generated',
  'server-openapi',
);

if (!fs.existsSync(sourceRoot)) {
  throw new Error(
    `App SDK output missing at ${path.relative(root, sourceRoot)}. Run workspace-agent-sdkgen --family app --mode apply first.`,
  );
}

copyDirectory(sourceRoot, targetRoot, sourceRoot, (relativePath, content) => {
  let transformed = content;
  if (shouldTransform(relativePath)) {
    transformed = transformed
      .replaceAll(appFamily.apiPrefix, openFamily.apiPrefix)
      .replaceAll(appFamily.packageName, openFamily.packageName)
      .replaceAll(appFamily.sdkName, openFamily.sdkName)
      .replaceAll(appFamily.authority, openFamily.authority)
      .replaceAll('APP_API_PREFIX', 'AGENT_API_PREFIX')
      .replaceAll('appApiPath', 'agentApiPath')
      .replaceAll('@sdkwork/agents-app-sdk', '@sdkwork/agents-sdk');
  }

  const posix = relativePath.replace(/\\/g, '/');
  if (posix === 'src/api/ai.ts') {
    transformed = stripAppOnlyOpenSdkSurface(transformed);
  }
  if (posix === 'README.md') {
    transformed = stripAppOnlyOpenSdkReadmeExamples(transformed);
  }

  return transformed;
});

writeDerivationEvidence(targetRoot, appFamily, openFamily, expectedOpenOperations);
assertOpenSdkSurface(targetRoot, expectedOpenOperations);
console.log('Agent open SDK transport derived from app SDK.');

function stripAppOnlyOpenSdkSurface(source) {
  let output = source;

  output = removeClassBlock(output, 'AiAgentsMcpServersApi');
  output = removeClassBlock(output, 'AiAgentsCodeEnginesApi');

  output = output.replace(
    /\n  public readonly codeEngines: AiAgentsCodeEnginesApi;\n/g,
    '\n',
  );
  output = output.replace(
    /\n  public readonly mcpServers: AiAgentsMcpServersApi;\n/g,
    '\n',
  );
  output = output.replace(
    /\n    this\.codeEngines = new AiAgentsCodeEnginesApi\(client\);\n/g,
    '\n',
  );
  output = output.replace(
    /\n    this\.mcpServers = new AiAgentsMcpServersApi\(client\);\n/g,
    '\n',
  );

  output = output.replace(
    /\n\/\*\* Restore one soft-deleted managed agent \*\/\n  async restore[\s\S]*?\n  \}\n/,
    '\n',
  );

  output = output.replace(
    /,\s*CodeEngineCatalogListResponse,\s*CreateAgentCompositionSlotRequest/,
    ', CreateAgentCompositionSlotRequest',
  );
  output = output.replace(
    /,\s*McpServerMarketplaceListResponse,\s*RestoreAgentRequest/,
    ', RestoreAgentRequest',
  );
  output = output.replace(/,\s*RestoreAgentRequest/, '');

  return output;
}

function removeClassBlock(source, className) {
  const pattern = new RegExp(
    `export class ${className}[\\s\\S]*?^}\\n`,
    'm',
  );
  return source.replace(pattern, '');
}

function stripAppOnlyOpenSdkReadmeExamples(source) {
  return source
    .replace(/client\.ai\.agents\.codeEngines\.list\(\)/g, 'client.ai.agents.list()')
    .replace(/List canonical code-engine catalog/g, 'List managed agents');
}

function shouldTransform(relativePath) {
  const posix = relativePath.replace(/\\/g, '/');
  return (
    posix.endsWith('.ts') ||
    posix.endsWith('.json') ||
    posix.endsWith('.md') ||
    posix.endsWith('package.json')
  );
}

function copyDirectory(sourceDir, targetDir, sourceRootDir, transform) {
  fs.rmSync(targetDir, { recursive: true, force: true });
  fs.mkdirSync(targetDir, { recursive: true });

  for (const entry of fs.readdirSync(sourceDir, { withFileTypes: true })) {
    const sourcePath = path.join(sourceDir, entry.name);
    const targetPath = path.join(targetDir, entry.name);
    if (entry.isDirectory()) {
      copyDirectory(sourcePath, targetPath, sourceRootDir, transform);
      continue;
    }
    if (!entry.isFile()) {
      continue;
    }

    const raw = fs.readFileSync(sourcePath, 'utf8');
    const relativePath = path.relative(sourceRootDir, sourcePath);
    const transformed = transform(relativePath, raw);
    fs.writeFileSync(targetPath, transformed, 'utf8');
  }
}

function assertOpenSdkSurface(targetRoot, expectedOpenOperations) {
  const aiSource = fs.readFileSync(path.join(targetRoot, 'src/api/ai.ts'), 'utf8');
  for (const marker of APP_ONLY_PATH_MARKERS) {
    if (aiSource.includes(`\`${marker}\``) || aiSource.includes(`'${marker}'`)) {
      throw new Error(`Derived open SDK must not expose app-only path ${marker}`);
    }
  }
  if (/\basync restore\(/.test(aiSource)) {
    throw new Error('Derived open SDK must not expose agents.restore');
  }
  if (/AiAgentsCodeEnginesApi|AiAgentsMcpServersApi/.test(aiSource)) {
    throw new Error('Derived open SDK must not expose app-only catalog APIs');
  }

  const actualOperations = (aiSource.match(/async \w+\(/g) ?? []).length;
  if (actualOperations < expectedOpenOperations) {
    throw new Error(
      `Derived open SDK operation surface too small: expected at least ${expectedOpenOperations}, found ${actualOperations}`,
    );
  }
}

function writeDerivationEvidence(targetRoot, appFamily, openFamily, expectedOpenOperations) {
  const evidenceDir = path.join(targetRoot, '.sdkwork');
  fs.mkdirSync(evidenceDir, { recursive: true });
  fs.writeFileSync(
    path.join(evidenceDir, 'sdkwork-open-sdk-derivation.json'),
    `${JSON.stringify(
      {
        schemaVersion: 2,
        generator: 'materialize-agent-open-sdk-from-app',
        derivedFrom: {
          familyDir: appFamily.familyDir,
          authority: appFamily.authority,
          apiPrefix: appFamily.apiPrefix,
          packageName: appFamily.packageName,
        },
        target: {
          familyDir: openFamily.familyDir,
          authority: openFamily.authority,
          apiPrefix: openFamily.apiPrefix,
          packageName: openFamily.packageName,
        },
        expectedOpenOperations,
        strippedAppOnlyPaths: APP_ONLY_PATH_MARKERS,
        managedBy: 'sdks/materialize-agent-open-sdk-from-app.mjs',
      },
      null,
      2,
    )}\n`,
    'utf8',
  );
}
