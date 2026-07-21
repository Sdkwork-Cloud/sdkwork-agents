import assert from 'node:assert/strict';
import { gzipSync } from 'node:zlib';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const distRoot = path.join(appRoot, 'dist');
const html = readFileSync(path.join(distRoot, 'index.html'), 'utf8');
const initialScripts = [...html.matchAll(/<(?:script|link)[^>]+(?:src|href)="([^"]+\.js)"/gu)]
  .map((match) => match[1])
  .filter((value, index, values) => values.indexOf(value) === index);

assert.ok(initialScripts.length > 0, 'PC production build must expose an initial JavaScript entry.');
const initialBytes = initialScripts.reduce((total, assetPath) => {
  const absolutePath = path.join(distRoot, assetPath.replace(/^\//u, ''));
  return total + gzipSync(readFileSync(absolutePath)).byteLength;
}, 0);

// The lazy Token Plan entry adds only its navigation/runtime handshake to the shell.
const maxInitialGzipBytes = 262 * 1024;
assert.ok(
  initialBytes <= maxInitialGzipBytes,
  `PC initial JavaScript is ${Math.ceil(initialBytes / 1024)} KiB gzip; budget is ${maxInitialGzipBytes / 1024} KiB.`,
);

const forbiddenInitialChunks = ['MarkdownRendererImpl', 'monaco', 'pdf-export', 'editor'];
for (const forbidden of forbiddenInitialChunks) {
  assert.equal(
    initialScripts.some((assetPath) => assetPath.toLowerCase().includes(forbidden.toLowerCase())),
    false,
    `${forbidden} must remain outside the initial route dependency closure.`,
  );
}

console.log(`PC bundle budget passed: ${Math.ceil(initialBytes / 1024)} KiB initial gzip across ${initialScripts.length} chunk(s).`);
