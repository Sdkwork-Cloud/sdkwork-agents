#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  alignBootstrapCompositionImports,
  buildDependencyCompositionManifest,
  ensureAppRootComponentSpecPointer,
  ensureCoreCompositionScaffold,
  ensureMissingAppCorePackage,
  extractApplicationCode,
  findCorePackages,
  listClientAppRoots,
  readJson,
  syncCoreSdkDependencies,
  toPosix,
  writeJson,
} from '../../sdkwork-specs/tools/lib/dependency-composition.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function collectPnpmPackages(appRoot) {
  const packages = [];
  const packagesDir = path.join(appRoot, 'packages');
  if (!fs.existsSync(packagesDir)) return packages;
  for (const entry of fs.readdirSync(packagesDir, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const packageJsonPath = path.join(packagesDir, entry.name, 'package.json');
    if (!fs.existsSync(packageJsonPath)) continue;
    const packageJson = readJson(packageJsonPath);
    if (packageJson.name) packages.push(packageJson.name);
  }
  return packages;
}

function alignClientAppRoot(clientRoot) {
  const changes = [];
  const applicationCode = extractApplicationCode(clientRoot.appRootName);
  if (!applicationCode) return changes;

  changes.push(
    ...ensureMissingAppCorePackage(clientRoot.appRoot, applicationCode, clientRoot.architecture, {}).map(
      (item) => `${toPosix(path.relative(repoRoot, clientRoot.appRoot))}/${item}`,
    ),
  );

  const cores = findCorePackages(
    clientRoot.appRoot,
    clientRoot.appRootName,
    applicationCode,
    clientRoot.architecture,
  );

  const manifestPath = path.join(clientRoot.appRoot, 'specs/dependency.composition.json');
  const manifest = buildDependencyCompositionManifest({
    applicationCode,
    architecture: clientRoot.architecture,
    cores,
    buildToolPackages: collectPnpmPackages(clientRoot.appRoot),
  });

  const existing = fs.existsSync(manifestPath) ? readJson(manifestPath) : null;
  if (!existing || JSON.stringify(existing) !== JSON.stringify(manifest)) {
    writeJson(manifestPath, manifest);
    changes.push(toPosix(path.relative(repoRoot, manifestPath)));
  }

  changes.push(
    ...ensureAppRootComponentSpecPointer(clientRoot.appRoot, {}).map((item) =>
      toPosix(path.relative(repoRoot, path.join(clientRoot.appRoot, item))),
    ),
  );

  for (const core of cores) {
    changes.push(
      ...syncCoreSdkDependencies(core, {}).map((item) =>
        toPosix(path.relative(repoRoot, path.join(core.packageDir, item))),
      ),
    );

    for (const change of ensureCoreCompositionScaffold(core, {})) {
      changes.push(`${toPosix(path.relative(repoRoot, core.packageDir))}: ${change}`);
    }

    if (fs.existsSync(core.componentSpecPath)) {
      const coreSpec = readJson(core.componentSpecPath);
      coreSpec.contracts = coreSpec.contracts ?? {};
      const appManifestPointer = toPosix(path.relative(path.dirname(core.componentSpecPath), manifestPath));
      if (coreSpec.contracts.dependencyComposition !== appManifestPointer) {
        coreSpec.contracts.dependencyComposition = appManifestPointer;
        writeJson(core.componentSpecPath, coreSpec);
        changes.push(toPosix(path.relative(repoRoot, core.componentSpecPath)));
      }
    }
  }

  changes.push(
    ...alignBootstrapCompositionImports(clientRoot.appRoot, cores, {}).map(
      (item) => `${toPosix(path.relative(repoRoot, clientRoot.appRoot))}/${item}`,
    ),
  );

  return changes;
}

const allChanges = [];
for (const clientRoot of listClientAppRoots(repoRoot)) {
  allChanges.push(...alignClientAppRoot(clientRoot));
}

if (allChanges.length === 0) {
  console.log('align-client-dependency-composition: no changes');
} else {
  console.log(`align-client-dependency-composition: ${allChanges.length} change(s)`);
  for (const change of allChanges) console.log(`- ${change}`);
}
