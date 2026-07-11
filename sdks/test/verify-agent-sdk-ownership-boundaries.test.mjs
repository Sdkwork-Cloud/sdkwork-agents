import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  AGENTS_SDK_FAMILIES,
  AGENTS_SDK_OWNER,
  AGENTS_SDK_OWNERSHIP_STANDARD_VERSION
} from '../_shared/agents-sdk-families.mjs';
import {
  annotateAgentOpenApiOwnership,
  countAgentOpenApiOperations
} from '../_shared/agent-sdk-ownership.mjs';
import {
  ensureTrailingNewline,
  materializeInternalOpenApiSdkgen
} from '../_shared/materialize-internal-openapi.mjs';
import { SDKWORK_SDKGEN_STANDARD } from '../_shared/sdkgen-standard.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');

for (const family of AGENTS_SDK_FAMILIES) {
  verifyFamily(family);
}

console.log('Agent SDK ownership boundary verification passed.');

function verifyFamily(family) {
  const familyRoot = path.join(repoRoot, 'sdks', family.familyDir);
  const sourceOpenApiPath = path.join(repoRoot, family.sourceOpenApi);
  const authorityPath = path.join(familyRoot, 'openapi', `${family.authority}.openapi.yaml`);
  const sdkgenPath = path.join(familyRoot, 'openapi', `${family.authority}.sdkgen.yaml`);
  const assemblyPath = path.join(familyRoot, '.sdkwork-assembly.json');
  const sdkManifestPath = path.join(familyRoot, 'sdk-manifest.json');
  const componentSpecPath = path.join(familyRoot, 'specs', 'component.spec.json');
  const generatedOutputPath = path.join(
    familyRoot,
    family.languagePackageDir,
    SDKWORK_SDKGEN_STANDARD.generatedOutput
  );

  for (const filePath of [
    sourceOpenApiPath,
    authorityPath,
    sdkgenPath,
    assemblyPath,
    sdkManifestPath,
    componentSpecPath
  ]) {
    assertFileExists(filePath, `${family.familyDir} required ownership file is missing`);
  }
  assertDirectoryExists(
    generatedOutputPath,
    `${family.familyDir} generated TypeScript output directory is missing`
  );

  const expectedAuthority = ensureTrailingNewline(
    annotateAgentOpenApiOwnership(readText(sourceOpenApiPath), family)
  );
  const actualAuthority = readText(authorityPath);
  assert.equal(
    actualAuthority,
    expectedAuthority,
    `${family.familyDir} authority OpenAPI drifted; rerun node sdks/materialize-agent-v3-openapi-boundaries.mjs`
  );

  const expectedSdkgen = materializeInternalOpenApiSdkgen(expectedAuthority, family.authority);
  const actualSdkgen = readText(sdkgenPath);
  assert.equal(
    actualSdkgen,
    expectedSdkgen,
    `${family.familyDir} sdkgen OpenAPI drifted; rerun node sdks/materialize-agent-v3-openapi-boundaries.mjs`
  );

  const ownerOnlyOperationCount = countAgentOpenApiOperations(actualSdkgen);
  const assembly = readJson(assemblyPath);
  const sdkManifest = readJson(sdkManifestPath);
  const componentSpec = readJson(componentSpecPath);

  assert.equal(assembly.workspace, family.familyDir, `${family.familyDir} assembly workspace`);
  assert.equal(assembly.apiAuthority, family.authority, `${family.familyDir} assembly authority`);
  assert.equal(
    assembly.authoritySpec,
    `openapi/${family.authority}.openapi.yaml`,
    `${family.familyDir} assembly authoritySpec`
  );
  assert.equal(
    assembly.generationInputSpec,
    `openapi/${family.authority}.sdkgen.yaml`,
    `${family.familyDir} assembly generationInputSpec`
  );
  assert.equal(
    assembly.discoverySurface?.apiPrefix,
    family.apiPrefix,
    `${family.familyDir} assembly apiPrefix`
  );
  assert.equal(assembly.sdkOwner, AGENTS_SDK_OWNER, `${family.familyDir} assembly sdkOwner`);
  assert.equal(
    assembly.metadata?.managedBy,
    'sdks/_shared/agent-sdk-ownership.mjs',
    `${family.familyDir} assembly managedBy`
  );
  assert.equal(
    assembly.metadata?.standardVersion,
    AGENTS_SDK_OWNERSHIP_STANDARD_VERSION,
    `${family.familyDir} assembly standardVersion`
  );
  assert.equal(
    assembly.metadata?.ownerOnlyOperationCount,
    ownerOnlyOperationCount,
    `${family.familyDir} assembly ownerOnlyOperationCount`
  );
  assert.deepEqual(
    assembly.sdkDependencies,
    family.sdkDependencies,
    `${family.familyDir} assembly sdkDependencies`
  );

  const typescriptLanguage = assembly.languages?.find((entry) => entry.language === 'typescript');
  assert.ok(typescriptLanguage, `${family.familyDir} assembly must declare TypeScript language`);
  assert.equal(
    typescriptLanguage.workspace,
    family.languagePackageDir,
    `${family.familyDir} TypeScript workspace`
  );
  assert.equal(
    typescriptLanguage.generatedPath,
    `${family.languagePackageDir}/${SDKWORK_SDKGEN_STANDARD.generatedOutput}`,
    `${family.familyDir} TypeScript generatedPath`
  );
  assert.equal(
    typescriptLanguage.manifestPath,
    `${family.languagePackageDir}/${SDKWORK_SDKGEN_STANDARD.generatedOutput}/package.json`,
    `${family.familyDir} TypeScript manifestPath`
  );
  assert.equal(typescriptLanguage.name, family.packageName, `${family.familyDir} package name`);

  assert.equal(sdkManifest.schemaVersion, 1, `${family.familyDir} sdk manifest schemaVersion`);
  assert.equal(sdkManifest.sdkName, family.sdkName, `${family.familyDir} sdkName`);
  assert.equal(sdkManifest.packageName, family.packageName, `${family.familyDir} packageName`);
  assert.equal(sdkManifest.sdkOwner, AGENTS_SDK_OWNER, `${family.familyDir} sdkOwner`);
  assert.equal(sdkManifest.apiAuthority, family.authority, `${family.familyDir} apiAuthority`);
  assert.equal(sdkManifest.sdkFamily, family.familyDir, `${family.familyDir} sdkFamily`);
  assert.equal(sdkManifest.sdkType, family.sdkType, `${family.familyDir} sdkType`);
  assert.equal(sdkManifest.sdkSurface, family.sdkSurface, `${family.familyDir} sdkSurface`);
  assert.equal(sdkManifest.language, 'typescript', `${family.familyDir} language`);
  assert.equal(sdkManifest.apiPrefix, family.apiPrefix, `${family.familyDir} apiPrefix`);
  assert.equal(
    sdkManifest.generationInputSpec,
    `openapi/${family.authority}.sdkgen.yaml`,
    `${family.familyDir} generationInputSpec`
  );
  assert.equal(
    sdkManifest.generatedOutput,
    `${family.languagePackageDir}/${SDKWORK_SDKGEN_STANDARD.generatedOutput}`,
    `${family.familyDir} generatedOutput`
  );
  assert.equal(
    sdkManifest.standardProfile,
    SDKWORK_SDKGEN_STANDARD.standardProfile,
    `${family.familyDir} standardProfile`
  );
  assert.equal(
    sdkManifest.ownerOnlyOperationCount,
    ownerOnlyOperationCount,
    `${family.familyDir} sdk manifest ownerOnlyOperationCount`
  );
  assert.deepEqual(
    sdkManifest.sdkDependencies,
    family.sdkDependencies,
    `${family.familyDir} sdk manifest dependencies`
  );

  assert.equal(componentSpec.kind, 'sdkwork.component.spec', `${family.familyDir} component kind`);
  assert.equal(componentSpec.component?.name, family.familyDir, `${family.familyDir} component name`);
  assert.equal(componentSpec.component?.type, 'sdk-family', `${family.familyDir} component type`);
  assert.equal(
    componentSpec.component?.root,
    `sdks/${family.familyDir}`,
    `${family.familyDir} component root`
  );
  assert.equal(componentSpec.component?.generated, true, `${family.familyDir} generated flag`);
  assert.deepEqual(
    componentSpec.component?.manifests,
    ['.sdkwork-assembly.json'],
    `${family.familyDir} component manifests`
  );
  assert.equal(componentSpec.sdk?.family, family.familyDir, `${family.familyDir} sdk.family`);
  assert.equal(componentSpec.sdk?.authority, family.authority, `${family.familyDir} sdk.authority`);
  assert.equal(componentSpec.sdk?.sdkOwner, AGENTS_SDK_OWNER, `${family.familyDir} sdkOwner`);
  assert.equal(componentSpec.sdk?.apiPrefix, family.apiPrefix, `${family.familyDir} sdk apiPrefix`);
  assert.equal(
    componentSpec.sdk?.packageName,
    family.packageName,
    `${family.familyDir} component packageName`
  );
  assert.deepEqual(
    componentSpec.contracts?.sdkDependencies,
    family.sdkDependencies,
    `${family.familyDir} component dependencies`
  );
  assert.deepEqual(
    componentSpec.contracts?.dependencyApiExports,
    [],
    `${family.familyDir} dependencyApiExports must be explicit empty array`
  );
  assert.deepEqual(
    componentSpec.contracts?.dependencyApiSurfaces,
    [],
    `${family.familyDir} dependencyApiSurfaces must be explicit empty array`
  );

  const verificationCommands = componentSpec.verification?.commands ?? [];
  for (const requiredCommand of [
    `node sdks/${family.familyDir}/bin/verify-sdk.mjs`,
    'node sdks/test/verify-agent-sdk-ownership-boundaries.test.mjs',
    'node scripts/check-agent-sdk-workspace.mjs'
  ]) {
    assert.ok(
      verificationCommands.includes(requiredCommand),
      `${family.familyDir} component spec must include ${requiredCommand}`
    );
  }
}

function readText(filePath) {
  return fs.readFileSync(filePath, 'utf8');
}

function readJson(filePath) {
  return JSON.parse(readText(filePath));
}

function assertFileExists(filePath, message) {
  assert.ok(fs.existsSync(filePath) && fs.statSync(filePath).isFile(), `${message}: ${filePath}`);
}

function assertDirectoryExists(filePath, message) {
  assert.ok(
    fs.existsSync(filePath) && fs.statSync(filePath).isDirectory(),
    `${message}: ${filePath}`
  );
}
