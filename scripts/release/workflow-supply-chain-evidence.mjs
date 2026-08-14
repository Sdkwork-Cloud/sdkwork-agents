#!/usr/bin/env node

import { createHash, createPrivateKey, createPublicKey, sign as signBytes } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const modulePath = fileURLToPath(import.meta.url);
const repoRoot = path.resolve(path.dirname(modulePath), '..', '..');

function required(value, label) {
  const text = String(value ?? '').trim();
  if (!text) throw new Error(`${label} is required`);
  return text;
}

function releasePaths() {
  const packageId = required(process.env.SDKWORK_PACKAGE_ID, 'SDKWORK_PACKAGE_ID');
  const relativeArtifact = required(
    process.env.SDKWORK_PACKAGE_ARTIFACT_PATH,
    'SDKWORK_PACKAGE_ARTIFACT_PATH',
  );
  if (path.isAbsolute(relativeArtifact) || relativeArtifact.split(/[\\/]/u).includes('..')) {
    throw new Error('artifact path must be repository-relative');
  }
  const artifactPath = path.resolve(repoRoot, relativeArtifact);
  if (!fs.existsSync(artifactPath) || !fs.statSync(artifactPath).isFile()) {
    throw new Error(`artifact does not exist: ${artifactPath}`);
  }
  const evidenceRoot = path.join(repoRoot, 'dist', 'release-evidence', packageId);
  fs.mkdirSync(evidenceRoot, { recursive: true });
  return {
    packageId,
    relativeArtifact: relativeArtifact.replaceAll('\\', '/'),
    artifactPath,
    signaturePath: path.join(evidenceRoot, `${path.basename(artifactPath)}.sig.json`),
    sbomPath: path.join(evidenceRoot, `${path.basename(artifactPath)}.cdx.json`),
    provenancePath: path.join(evidenceRoot, `${path.basename(artifactPath)}.intoto.jsonl`),
  };
}

function sha256(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

function signingKey() {
  const inline = String(process.env.SDKWORK_RELEASE_SIGNING_PRIVATE_KEY ?? '').trim();
  const keyFile = String(process.env.SDKWORK_RELEASE_SIGNING_KEY_FILE ?? '').trim();
  if ((!inline && !keyFile) || (inline && keyFile)) {
    throw new Error('configure exactly one real release signing key source');
  }
  if (keyFile && !fs.existsSync(keyFile)) throw new Error(`signing key does not exist: ${keyFile}`);
  return createPrivateKey({
    key: inline || fs.readFileSync(keyFile),
    passphrase: String(process.env.SDKWORK_RELEASE_SIGNING_PRIVATE_KEY_PASSWORD ?? '') || undefined,
  });
}

function signArtifact() {
  const paths = releasePaths();
  const bytes = fs.readFileSync(paths.artifactPath);
  const key = signingKey();
  const algorithm = ['ed25519', 'ed448'].includes(key.asymmetricKeyType) ? null : 'sha256';
  const publicKey = createPublicKey(key).export({ format: 'der', type: 'spki' });
  const envelope = {
    schemaVersion: 1,
    artifact: paths.relativeArtifact,
    digest: `sha256:${sha256(bytes)}`,
    algorithm: key.asymmetricKeyType,
    publicKeyFingerprint: `sha256:${sha256(publicKey)}`,
    signatureBase64: signBytes(algorithm, bytes, key).toString('base64'),
  };
  fs.writeFileSync(paths.signaturePath, `${JSON.stringify(envelope, null, 2)}\n`, { mode: 0o600 });
}

function attestArtifact() {
  const paths = releasePaths();
  const bytes = fs.readFileSync(paths.artifactPath);
  const digest = sha256(bytes);
  const version = required(process.env.SDKWORK_PACKAGE_VERSION, 'SDKWORK_PACKAGE_VERSION');
  const sourceCommit = execFileSync('git', ['rev-parse', 'HEAD'], {
    cwd: repoRoot,
    encoding: 'utf8',
  }).trim();
  const sbom = {
    bomFormat: 'CycloneDX',
    specVersion: '1.5',
    version: 1,
    metadata: {
      component: {
        type: 'application',
        name: paths.packageId,
        version,
        hashes: [{ alg: 'SHA-256', content: digest }],
      },
    },
    components: [{
      type: 'file',
      name: path.basename(paths.artifactPath),
      version,
      hashes: [{ alg: 'SHA-256', content: digest }],
      properties: [{ name: 'sdkwork:sizeBytes', value: String(bytes.length) }],
    }],
  };
  const provenance = {
    _type: 'https://in-toto.io/Statement/v1',
    subject: [{ name: paths.relativeArtifact, digest: { sha256: digest } }],
    predicateType: 'https://slsa.dev/provenance/v1',
    predicate: {
      buildDefinition: {
        buildType: 'https://sdkwork.com/buildtypes/github-workflow/v1',
        externalParameters: { packageId: paths.packageId },
        resolvedDependencies: [{
          uri: 'git+https://github.com/sdkwork-ai/sdkwork-agents',
          digest: { gitCommit: sourceCommit },
        }],
      },
      runDetails: {
        builder: { id: 'https://github.com/sdkwork-ai/sdkwork-github-workflow' },
        metadata: { invocationId: String(process.env.GITHUB_RUN_ID ?? 'local-validation') },
      },
    },
  };
  fs.writeFileSync(paths.sbomPath, `${JSON.stringify(sbom, null, 2)}\n`);
  fs.writeFileSync(paths.provenancePath, `${JSON.stringify(provenance)}\n`);
}

const command = process.argv[2];
try {
  if (command === 'sign') signArtifact();
  else if (command === 'attest') attestArtifact();
  else throw new Error('command must be sign or attest');
} catch (error) {
  console.error(`[sdkwork-agents-supply-chain] ${error.message}`);
  process.exitCode = 1;
}
