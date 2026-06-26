import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { AGENTS_SDK_FAMILIES } from './_shared/agents-sdk-families.mjs';
import {
  annotateAgentOpenApiOwnership,
  syncAgentSdkOwnershipWorkspace
} from './_shared/agent-sdk-ownership.mjs';
import {
  ensureTrailingNewline,
  materializeInternalOpenApiSdkgen
} from './_shared/materialize-internal-openapi.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

const LEGACY_OPENAPI_BASENAMES = {
  open: ['sdkwork-agent-open-api.openapi.yaml', 'sdkwork-agent-open-api.sdkgen.yaml'],
  app: ['sdkwork-agent-app-api.openapi.yaml', 'sdkwork-agent-app-api.sdkgen.yaml'],
  backend: [
    'sdkwork-agent-backend-api.openapi.yaml',
    'sdkwork-agent-backend-api.sdkgen.yaml'
  ]
};

for (const family of AGENTS_SDK_FAMILIES) {
  const sourcePath = path.join(root, family.sourceOpenApi);
  if (!fs.existsSync(sourcePath)) {
    throw new Error(`Missing source OpenAPI: ${family.sourceOpenApi}`);
  }

  const source = fs.readFileSync(sourcePath, 'utf8');
  const authority = ensureTrailingNewline(annotateAgentOpenApiOwnership(source, family));
  const sdkgen = materializeInternalOpenApiSdkgen(authority, family.authority);

  const openapiDir = path.join(root, 'sdks', family.familyDir, 'openapi');
  fs.mkdirSync(openapiDir, { recursive: true });

  writeTextIfChanged(
    path.join(openapiDir, `${family.authority}.openapi.yaml`),
    authority
  );
  writeTextIfChanged(
    path.join(openapiDir, `${family.authority}.sdkgen.yaml`),
    sdkgen
  );

  for (const legacyBasename of LEGACY_OPENAPI_BASENAMES[family.key] ?? []) {
    const legacyPath = path.join(openapiDir, legacyBasename);
    if (fs.existsSync(legacyPath)) {
      fs.unlinkSync(legacyPath);
    }
  }
}

syncAgentSdkOwnershipWorkspace(root, AGENTS_SDK_FAMILIES);
console.log('Agent v3 OpenAPI SDK boundaries materialized.');

function writeTextIfChanged(filePath, content) {
  const normalized = ensureTrailingNewline(content);
  if (fs.existsSync(filePath) && fs.readFileSync(filePath, 'utf8') === normalized) {
    return;
  }
  fs.writeFileSync(filePath, normalized, 'utf8');
}
