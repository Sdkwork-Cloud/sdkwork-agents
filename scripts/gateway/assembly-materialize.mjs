#!/usr/bin/env node
import path from 'node:path';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { materializeGatewayAssembly } from '../../../sdkwork-specs/tools/materialize-gateway-assembly.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const result = materializeGatewayAssembly(root);
if (!result.ok) {
  console.error('api:assembly:materialize failed: ' + result.message);
  process.exit(1);
}

const componentSpecPath = path.join(
  root,
  'crates',
  `sdkwork-${result.applicationCode}-gateway-assembly`,
  'specs',
  'component.spec.json',
);
const componentSpec = JSON.parse(readFileSync(componentSpecPath, 'utf8'));
componentSpec.component.root = `crates/sdkwork-${result.applicationCode}-gateway-assembly`;
writeFileSync(componentSpecPath, `${JSON.stringify(componentSpec, null, 2)}\n`, 'utf8');

console.log('api:assembly:materialize wrote ' + result.crateDir + ' (' + result.routeCrates + ' route crates)');
