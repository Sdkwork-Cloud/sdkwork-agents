import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import {
  resolveAgentsSdkFamily,
  resolveAgentsSdkLanguageTargets
} from '../../_shared/agents-sdk-families.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const family = resolveAgentsSdkFamily('app');

verify(family);
console.log(`${family.familyDir} SDK boundary check passed.`);

function verify(candidate) {
  const familyRoot = path.join(root, 'sdks', candidate.familyDir);
  const authority = path.join(familyRoot, 'openapi', `${candidate.authority}.openapi.yaml`);
  const sdkgen = path.join(familyRoot, 'openapi', `${candidate.authority}.sdkgen.yaml`);
  const languageTargets = resolveAgentsSdkLanguageTargets(candidate);

  for (const filePath of [authority, sdkgen]) {
    if (!fs.existsSync(filePath)) {
      throw new Error(`missing OpenAPI file: ${filePath}`);
    }
  }
  for (const target of languageTargets) {
    const output = path.join(familyRoot, target.workspace, 'generated', 'server-openapi');
    if (!fs.existsSync(output) || !fs.statSync(output).isDirectory()) {
      throw new Error(`missing generated ${target.language} output directory: ${output}`);
    }
  }

  const authorityText = fs.readFileSync(authority, 'utf8');
  const sdkgenText = fs.readFileSync(sdkgen, 'utf8');
  for (const required of [
    candidate.apiPrefix,
    candidate.title,
    'operationId: agents.list',
    'operationId: agents.providerBindings.create',
    'operationId: agents.compositionSlots.list',
    'Access-Token'
  ]) {
    if (!authorityText.includes(required) || !sdkgenText.includes(required)) {
      throw new Error(`${candidate.familyDir} OpenAPI boundary missing ${required}`);
    }
  }
  if (sdkgenText.includes("$ref: '#/components/responses/Problem'")) {
    throw new Error(`${candidate.familyDir} sdkgen input must inline explicit problem responses`);
  }

  verifyFlutterOutput(familyRoot, candidate);
}

function verifyFlutterOutput(familyRoot, candidate) {
  const flutterTarget = resolveAgentsSdkLanguageTargets(candidate).find(
    (target) => target.language === 'flutter'
  );
  if (!flutterTarget) return;
  const output = path.join(familyRoot, flutterTarget.workspace, 'generated', 'server-openapi');
  const sdkMetadata = JSON.parse(fs.readFileSync(path.join(output, 'sdkwork-sdk.json'), 'utf8'));
  if (sdkMetadata.language !== 'flutter' || sdkMetadata.sdkType !== candidate.sdkType) {
    throw new Error(`${candidate.familyDir} Flutter generator metadata is inconsistent`);
  }
  const pubspec = fs.readFileSync(path.join(output, 'pubspec.yaml'), 'utf8');
  if (!pubspec.includes(`name: ${flutterTarget.packageName}`)) {
    throw new Error(`${candidate.familyDir} Flutter package name is inconsistent`);
  }
  const libraryEntry = fs.readFileSync(
    path.join(output, flutterTarget.entrypoint),
    'utf8',
  );
  for (const requiredExport of [
    "export 'app_client.dart';",
    "export 'src/models.dart';",
    "export 'src/api/api.dart';",
  ]) {
    if (!libraryEntry.includes(requiredExport)) {
      throw new Error(
        `${candidate.familyDir} Flutter package root is missing ${requiredExport}`,
      );
    }
  }
  const models = fs.readFileSync(path.join(output, 'lib', 'src', 'models.dart'), 'utf8');
  for (const requiredModel of [
    'AgentSessionRecord',
    'AgentSessionItemRecord',
    'AgentTurnRecord',
    'AgentInteractionRecord',
    'AgentSessionCheckpointRecord',
    'AgentSessionRuntimeBindingRecord'
  ]) {
    if (!models.includes(`class ${requiredModel}`)) {
      throw new Error(`${candidate.familyDir} Flutter SDK is missing ${requiredModel}`);
    }
  }
  const api = fs.readFileSync(path.join(output, 'lib', 'src', 'api', 'ai.dart'), 'utf8');
  for (const requiredMethod of [
    'agentsSessionsCreate',
    'agentsSessionsRetrieve',
    'agentsTurnsList',
    'agentsTurnsStream',
    'agentsSessionItemsList',
    'agentsSessionItemsRetrieve',
    'agentsItemFeedbackUpdate',
    'agentsInteractionsClaim',
  ]) {
    if (!api.includes(requiredMethod)) {
      throw new Error(`${candidate.familyDir} Flutter SDK is missing ${requiredMethod}`);
    }
  }
  for (const requiredPath of [
    '/sessions',
    '/items',
    '/turns',
    '/interactions',
    '/checkpoints',
    '/runtime_bindings'
  ]) {
    if (!api.includes(requiredPath)) {
      throw new Error(`${candidate.familyDir} Flutter SDK is missing ${requiredPath}`);
    }
  }
  for (const forbidden of ['chatTurns', 'conversationId', 'chatMessageId']) {
    if (api.includes(forbidden) || models.includes(forbidden)) {
      throw new Error(`${candidate.familyDir} Flutter SDK contains legacy ${forbidden}`);
    }
  }
}
